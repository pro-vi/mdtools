"""Offline fixed-task path selectively recovered from H (c933520).

Retains task fields, fresh staging, subprocess execution, final-file capture and
artifact writing. Provider/Pi/multifile branches, dual scorers, quarantine, old
resume/report policy and eager configuration imports are deliberately removed.
"""

from __future__ import annotations

import argparse
from contextlib import suppress
from dataclasses import asdict, dataclass
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import shutil
import signal
import subprocess
import sys
import tempfile
from typing import Sequence

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from bench.command_policy import build_runner_command
from bench.manifest import ExperimentSpec, canonical_json, sha256_file, sha256_text
from bench.neutral_scorer import Comparison, StructuralDiffPolicy, answer_instructions, grade_submission, validate_expected, validate_policy


@dataclass(frozen=True)
class BenchTask:
    id: str
    description: str
    input_files: list[str]
    expected_output: str
    expected_artifact: str
    difficulty: str
    scorer: StructuralDiffPolicy
    expected_stdout: str | None = None
    support_files: list[str] | None = None


@dataclass(frozen=True)
class OfflineResult:
    """Temporary U1 receipt; U4 will supply shared validated attempt records."""
    schema: str
    backend: str
    task_id: str
    execution: str
    comparison: Comparison | None
    error: str | None
    exit_code: int | None
    experiment_id: str
    artifacts: dict[str, str]


def _relative_path(reference: str) -> Path:
    if not isinstance(reference, str) or not reference or "\\" in reference or "\0" in reference:
        raise ValueError("artifact reference must be a nonempty relative POSIX path")
    parts = reference.split("/")
    if any(part in ("", ".", "..") for part in parts) or PurePosixPath(reference).is_absolute():
        raise ValueError("artifact traversal or noncanonical path")
    return Path(*parts)


def _assert_no_symlinks(path: Path) -> None:
    for component in (*reversed(path.parents), path):
        if component.is_symlink():
            raise ValueError("symlink component in artifact path")


def _root(path: Path) -> Path:
    path = Path(os.path.abspath(path))
    _assert_no_symlinks(path)
    return path


def _read_source(root: Path, reference: str) -> bytes:
    path = root / _relative_path(reference)
    _assert_no_symlinks(path)
    if not path.is_file():
        raise ValueError("missing or nonregular source object")
    return path.read_bytes()


def _overlaps(left: Path, right: Path) -> bool:
    return left == right or left in right.parents or right in left.parents


def build_prompt(task: BenchTask) -> str:
    """Task text and relative references only; never preload input or expected bytes."""
    return (f"TASK: {task.description}\nINPUT FILES: {json.dumps(task.input_files)}\n"
            f"SUPPORT FILES: {json.dumps(task.support_files or [])}\n"
            f"OUTPUT: {answer_instructions(task.scorer, artifact=task.expected_artifact)}\n")


def _validate_task(task: BenchTask) -> None:
    if not isinstance(task.id, str) or not task.id or not isinstance(task.description, str):
        raise ValueError("task requires a string identity and description")
    if not isinstance(task.input_files, list) or not task.input_files or any(not isinstance(ref, str) for ref in task.input_files):
        raise ValueError("task input_files must be a nonempty list of paths")
    if task.support_files is not None and (not isinstance(task.support_files, list) or any(not isinstance(ref, str) for ref in task.support_files)):
        raise ValueError("task support_files must be a list of paths or null")
    if not isinstance(task.scorer, StructuralDiffPolicy):
        raise ValueError("task scorer must be a StructuralDiffPolicy")
    if not isinstance(task.difficulty, str):
        raise ValueError("task difficulty must be a string")


def _write_private(path: Path, content: bytes) -> None:
    # pathlib's parents=True applies mode only to the last directory. Create
    # each missing component explicitly so nested captured artifacts stay private.
    for directory in (*reversed(path.parent.parents), path.parent):
        if not directory.exists():
            directory.mkdir(mode=0o700)
    with path.open("xb") as handle:
        os.chmod(path, 0o600)
        handle.write(content)


def write_run_artifacts(results_dir: Path, *, spec: ExperimentSpec, result: OfflineResult) -> None:
    """Publish U1 summary last; this is not U4's recoverable attempt finalization."""
    _write_private(results_dir / "experiment.json", (canonical_json(asdict(spec)) + "\n").encode())
    _write_private(results_dir / "report.json", (canonical_json({
        "backend": "synthetic", "task_id": result.task_id,
        "execution": result.execution, "comparison": asdict(result.comparison) if result.comparison else None,
        "live_study_evidence": False,
    }) + "\n").encode())
    _write_private(results_dir / "result.json", (canonical_json(asdict(result)) + "\n").encode())


def _stop_owned_group(process: subprocess.Popen[bytes]) -> None:
    """Stop only the session/group created for this synthetic subprocess."""
    with suppress(ProcessLookupError):  # An already-empty group needs no signal.
        os.killpg(process.pid, signal.SIGKILL)


def run_agent(task: BenchTask, *, fixture_root: Path, expected_root: Path,
              command: Sequence[str], results_dir: Path, runner: str = "synthetic",
              timeout_seconds: float = 10) -> OfflineResult:
    """Run trusted synthetic argv and capture only the declared final artifacts.

References are relative to the supplied roots. The first input is the declared
file result for file kinds. Entire synthetic stdout is the explicit final text
for output kinds; no intermediate-answer recovery exists. Other input/support
files retain their full relative paths. Expected roots must be separate.
    """
    argv = build_runner_command(command, runner=runner)
    _validate_task(task)
    validate_policy(task.scorer, artifact=task.expected_artifact)
    if task.expected_stdout is not None and not isinstance(task.expected_stdout, str):
        raise ValueError("expected_stdout must be a string or null")
    expected_stdout = task.expected_stdout.encode("utf-8") if task.expected_stdout is not None else None
    if isinstance(timeout_seconds, bool) or not isinstance(timeout_seconds, (int, float)) or not math.isfinite(timeout_seconds) or timeout_seconds <= 0:
        raise ValueError("timeout must be a positive finite number")
    fixture_root, expected_root, results_dir = map(_root, (fixture_root, expected_root, results_dir))
    if any((_overlaps(fixture_root, expected_root), _overlaps(results_dir, fixture_root), _overlaps(results_dir, expected_root))):
        raise ValueError("fixture, expected and result roots must be disjoint")
    references = [*task.input_files, *(task.support_files or [])]
    relative_paths = [_relative_path(reference) for reference in references]
    if len(set(relative_paths)) != len(relative_paths) or any(
        left in right.parents for left in relative_paths for right in relative_paths if left != right
    ):
        raise ValueError("staging collision")
    sources = {reference: _read_source(fixture_root, reference) for reference in references}
    expected = _read_source(expected_root, task.expected_output)
    validate_expected(task.scorer, expected, artifact=task.expected_artifact, expected_stdout=expected_stdout)
    expected_stat = (expected_root / task.expected_output).stat()
    if any((fixture_root / reference).stat().st_ino == expected_stat.st_ino and
           (fixture_root / reference).stat().st_dev == expected_stat.st_dev for reference in references):
        raise ValueError("expected source aliases a worker input")
    prompt = build_prompt(task)
    # U4 extends this temporary synthetic identity with the remaining fields.
    spec = ExperimentSpec("synthetic", sha256_text(canonical_json(asdict(task))),
        sha256_text(prompt), {name: hashlib.sha256(content).hexdigest() for name, content in sources.items()},
        hashlib.sha256(expected).hexdigest(), tuple(argv), sha256_file(argv[0]),
        sha256_file(Path(__file__).with_name("requirements.lock")),
        sha256_file(__file__), sha256_file(Path(__file__).with_name("neutral_scorer.py")))
    # Existing output is never overwritten, including an incomplete prior exercise.
    results_dir.mkdir(parents=True, mode=0o700, exist_ok=False)
    artifacts: dict[str, str] = {}
    execution, error, exit_code, comparison = "completed", None, None, None
    with tempfile.TemporaryDirectory(prefix="mdtools_offline_") as temporary:
        workspace = Path(temporary).resolve()
        worker = workspace / "fixtures"
        worker.mkdir(mode=0o700)
        scratch = workspace / "scratch"
        scratch.mkdir(mode=0o700)
        for reference, content in sources.items():
            _write_private(worker / reference, content)
        child_env = {"PATH": "/usr/bin:/bin", "TMPDIR": str(scratch),
                     "LANG": "C.UTF-8", "LC_ALL": "C", "PYTHONNOUSERSITE": "1"}
        try:
            with subprocess.Popen(argv, cwd=worker, env=child_env, stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True) as process:
                try:
                    stdout, stderr = process.communicate(prompt.encode(), timeout=timeout_seconds)
                    exit_code = process.returncode
                    if exit_code != 0:
                        execution, error = "infrastructure_error", "synthetic_process_exit"
                except subprocess.TimeoutExpired:
                    execution, error = "timed_out", "synthetic_deadline"
                    _stop_owned_group(process)
                    stdout, stderr = process.communicate()
                    exit_code = process.returncode
                except KeyboardInterrupt:
                    execution, error = "interrupted", "synthetic_interruption"
                    _stop_owned_group(process)
                    stdout, stderr = process.communicate()
                    exit_code = process.returncode
                finally:
                    # Also stop background members after an ordinary parent exit.
                    _stop_owned_group(process)
        except OSError:
            stdout, stderr = b"", b""
            execution, error = "infrastructure_error", "synthetic_spawn_failed"
        for name, content in (("stdout.bin", stdout), ("stderr.bin", stderr)):
            _write_private(results_dir / "artifacts" / name, content)
            artifacts[f"artifacts/{name}"] = hashlib.sha256(content).hexdigest()
        actual = None
        if task.expected_artifact in ("file_contents", "stdout_and_file"):
            try:
                actual = _read_source(worker, task.input_files[0])
            except ValueError:
                if execution == "completed":
                    error = "capture_unavailable"
            else:
                final_reference = "artifacts/final/" + task.input_files[0]
                _write_private(results_dir / final_reference, actual)
                artifacts[final_reference] = hashlib.sha256(actual).hexdigest()
        # The trusted synthetic runner's entire stdout is its explicit final
        # submission. It has no provider/tool-event decoder or answer fallback.
        # U4 will feed untouched terminal receipt text into the same grade API.
        if task.expected_artifact != "file_contents":
            final_reference = "artifacts/final_submission.bin"
            _write_private(results_dir / final_reference, stdout)
            artifacts[final_reference] = hashlib.sha256(stdout).hexdigest()
        if execution == "completed" and error is None:
            try:
                comparison = grade_submission(task.scorer, artifact=task.expected_artifact,
                    final_text=stdout, actual_file=actual, expected=expected, expected_stdout=expected_stdout)
            except (ValueError, RuntimeError):
                error = "grader_unavailable"
    result = OfflineResult("mdtools.cli-eval.offline/0", "synthetic", task.id, execution,
                           comparison, error, exit_code, spec.identity, artifacts)
    write_run_artifacts(results_dir, spec=spec, result=result)
    return result


def offline_exercise(results_dir: Path) -> OfflineResult:
    """One actual subprocess mutation; no task registry, provider or user settings."""
    with tempfile.TemporaryDirectory(prefix="mdtools_synthetic_") as temporary:
        root = Path(temporary).resolve()
        fixtures, expected = root / "inputs", root / "expected"
        _write_private(fixtures / "nested/input.md", b"# Before\n")
        _write_private(expected / "answer.md", b"# After\n")
        task = BenchTask("synthetic-u1", "Change the heading to After.", ["nested/input.md"],
            "answer.md", "file_contents", "synthetic",
            StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False))
        script = "from pathlib import Path; Path('nested/input.md').write_bytes(b'# After\\n'); print('finished')"
        return run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=[str(Path(sys.executable).resolve()), "-I", "-c", script], results_dir=results_dir)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Offline synthetic validation only; real runners are unavailable.")
    parser.add_argument("--offline-exercise", action="store_true", help="Explicitly select the default synthetic exercise")
    parser.add_argument("--results-dir", type=Path, help="New controller-owned result directory")
    args = parser.parse_args(argv)
    if args.results_dir is None:
        parent = Path(tempfile.mkdtemp(prefix="mdtools_offline_receipt_")).resolve()
        results_dir = parent / "result"
    else:
        results_dir = args.results_dir
    result = offline_exercise(results_dir)
    print(canonical_json({"backend": result.backend, "execution": result.execution,
        "comparison": asdict(result.comparison) if result.comparison else None, "results_dir": str(results_dir)}))
    return 0 if result.execution == "completed" and result.comparison and result.comparison.kind == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
