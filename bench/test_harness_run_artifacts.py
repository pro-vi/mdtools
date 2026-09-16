"""Synthetic adaptation of H's artifact-persistence tests (c933520)."""

from __future__ import annotations

from dataclasses import replace
import hashlib
import json
from pathlib import Path
import sys
import os
import subprocess

import pytest

from bench.harness import BenchTask, StructuralDiffPolicy, offline_exercise, run_agent


def synthetic_task(root: Path) -> tuple[BenchTask, Path, Path]:
    fixtures, expected = root / "fixtures", root / "expected"
    fixtures.mkdir()
    expected.mkdir()
    (fixtures / "input.md").write_bytes(b"before\n")
    (expected / "answer.md").write_bytes(b"after\n")
    return (BenchTask("synthetic", "Write after.", ["input.md"], "answer.md", "file_contents",
        "synthetic", StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False)),
        fixtures, expected)


def python_command(script: str) -> list[str]:
    return [str(Path(sys.executable).resolve()), "-I", "-c", script]


def test_subprocess_mutation_is_captured_graded_and_persisted(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    receipt = root / "receipt"
    result = offline_exercise(receipt)
    assert result.execution == "completed"
    assert result.comparison.kind == "pass"
    assert (receipt / "artifacts/final/nested/input.md").read_bytes() == b"# After\n"
    assert (receipt / "artifacts/stdout.bin").read_bytes() == b"finished\n"
    stored = json.loads((receipt / "result.json").read_text())
    assert stored["schema"] == "mdtools.cli-eval.offline/0"
    assert stored["backend"] == "synthetic"
    assert stored["comparison"]["kind"] == "pass"
    assert "correct" not in stored
    assert json.loads((receipt / "report.json").read_text())["live_study_evidence"] is False
    for name, digest in stored["artifacts"].items():
        artifact = receipt / name
        assert hashlib.sha256(artifact.read_bytes()).hexdigest() == digest
        assert artifact.stat().st_mode & 0o777 == 0o600
    assert receipt.stat().st_mode & 0o777 == 0o700
    assert all(directory.stat().st_mode & 0o777 == 0o700 for directory in receipt.rglob("*") if directory.is_dir())


def test_wrong_final_file_fails_without_consulting_stdout(tmp_path: Path) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("print('after')"), results_dir=tmp_path.resolve() / "receipt")
    assert result.execution == "completed"
    assert result.comparison.kind == "fail"
    assert (fixtures / "input.md").read_bytes() == b"before\n"


@pytest.mark.parametrize("script,execution", [
    ("from pathlib import Path; Path('input.md').write_bytes(b'after\\n'); raise SystemExit(7)", "infrastructure_error"),
    ("from pathlib import Path; import time; Path('input.md').write_bytes(b'after\\n'); time.sleep(3)", "timed_out"),
])
def test_incomplete_execution_retains_file_without_comparison(tmp_path: Path, script: str, execution: str) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=tmp_path.resolve() / "receipt", timeout_seconds=0.5)
    assert result.execution == execution
    assert result.comparison is None
    assert "artifacts/final/input.md" in result.artifacts


@pytest.mark.parametrize("script", [
    "from pathlib import Path; Path('input.md').unlink()",
    "from pathlib import Path; Path('input.md').unlink(); Path('input.md').symlink_to('/dev/null')",
])
def test_capture_error_is_not_a_semantic_failure(tmp_path: Path, script: str) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=tmp_path.resolve() / "receipt")
    assert result.execution == "completed"
    assert result.comparison is None
    assert result.error == "capture_unavailable"


def test_existing_result_is_not_overwritten(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    result_dir = root / "receipt"
    offline_exercise(result_dir)
    before = (result_dir / "result.json").read_bytes()
    with pytest.raises(FileExistsError):
        offline_exercise(result_dir)
    assert (result_dir / "result.json").read_bytes() == before


def test_declared_normalization_retains_newlines_and_nonascii_whitespace() -> None:
    from bench.neutral_scorer import score_task

    raw = StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False)
    normalized = replace(raw, kind="normalized_text", normalize_line_endings=True, ignore_trailing_whitespace=True)
    assert score_task(normalized, b"hello \r\n", b"hello\n").kind == "pass"
    assert score_task(raw, b"hello\r\n", b"hello\n").kind == "fail"
    assert score_task(normalized, b"hello", b"hello\n").kind == "fail"
    assert score_task(normalized, "hello\u00a0\n".encode(), b"hello\n").kind == "fail"


def test_recovered_independent_heading_parser_uses_locked_dependency() -> None:
    from bench.neutral_scorer import neutral_heading_tree

    assert neutral_heading_tree("# One\r\n\r\n## *Two*\n") == [(1, "One"), (2, "Two")]
    assert neutral_heading_tree("```markdown\n# Not a heading\n```\n# Real\n") == [(1, "Real")]


def test_invalid_actual_text_is_a_semantic_failure(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, scorer=replace(task.scorer, kind="normalized_text"))
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("from pathlib import Path; Path('input.md').write_bytes(b'\\xff')"), results_dir=root / "receipt")
    assert result.execution == "completed"
    assert result.comparison.kind == "fail"
    assert result.comparison.reason == "invalid_utf8"


def test_raw_bytes_accepts_binary_expected_and_actual(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    (expected / "answer.md").write_bytes(b"\xff")
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("from pathlib import Path; Path('input.md').write_bytes(b'\\xff')"), results_dir=root / "receipt")
    assert result.execution == "completed"
    assert result.comparison.kind == "pass"


def test_timeout_stops_owned_descendant_and_preserves_unrelated_process(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    # Parent exits immediately; its descendant inherits the output pipes. The
    # controller must still reach its deadline and stop that descendant group.
    script = (
        "import os, subprocess, sys; "
        "child = subprocess.Popen([sys.executable, '-I', '-c', 'import time; time.sleep(30)']); "
        "print(child.pid, flush=True); os._exit(0)"
    )
    with subprocess.Popen(python_command("import time; time.sleep(30)"), start_new_session=True) as sentinel:
        try:
            result = run_agent(task, fixture_root=fixtures, expected_root=expected,
                command=python_command(script), results_dir=root / "receipt", timeout_seconds=0.5)
            assert result.execution == "timed_out"
            assert result.comparison is None
            child_pid = int((root / "receipt/artifacts/stdout.bin").read_text().strip())
            # kill(0) can see a short-lived zombie; ps identifies a live writer.
            observed = subprocess.run(["/bin/ps", "-o", "stat=", "-p", str(child_pid)], capture_output=True, check=False)
            assert not observed.stdout.strip() or observed.stdout.strip().startswith(b"Z")
            assert sentinel.poll() is None
        finally:
            sentinel.kill()
            sentinel.wait()
