"""Synthetic adaptation of H's runner admission/isolation tests (c933520).

Native file tools, provider parsers, Pi helpers and old BASH_ENV guards omitted.
"""

from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
import subprocess
import sys

import pytest

from bench import harness
from bench.command_policy import build_runner_command
from bench.test_harness_run_artifacts import python_command, synthetic_task


@pytest.mark.parametrize("runner", ["claude_cli", "claude-cli", "pi-json", "oai-loop", "unknown"])
def test_live_or_unknown_runner_never_launches(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, runner: str) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError, match="unsupported runner"):
        harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), runner=runner, results_dir=root / "receipt")
    assert launches == []


@pytest.mark.parametrize("artifact", ["stdout_text", "stdout_and_file", "json_envelope", "multi_file_contents_any", "unknown"])
def test_unsupported_artifact_never_launches(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, artifact: str) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError, match="unsupported artifact"):
        harness.run_agent(replace(task, expected_artifact=artifact), fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []


def test_unsupported_policy_dimensions_never_launch(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    for policy in (replace(task.scorer, kind="structural"), replace(task.scorer, compare_heading_tree=True),
                   replace(task.scorer, json_required_keys=[]), replace(task.scorer, normalize_line_endings=1)):
        with pytest.raises(ValueError):
            harness.run_agent(replace(task, scorer=policy), fixture_root=fixtures, expected_root=expected,
                command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []


@pytest.mark.parametrize("command", [[], "python3 -c pass", ["python3"], ["/definitely/missing"], 7])
def test_argv_admission_requires_explicit_executable(command: object) -> None:
    with pytest.raises(ValueError):
        build_runner_command(command, runner="synthetic")


def test_spaces_and_quotes_are_preserved_in_argv(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    payload = "a b ' c \" d"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=[*python_command("import sys; print(sys.argv[1])"), payload], results_dir=root / "receipt")
    assert result.execution == "completed"
    assert (root / "receipt/artifacts/stdout.bin").read_text().strip() == payload


def test_child_environment_does_not_inherit_user_configuration(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    monkeypatch.setenv("U1_SYNTHETIC_CONFIG_CANARY", "synthetic-not-a-secret")
    script = "import os; assert 'U1_SYNTHETIC_CONFIG_CANARY' not in os.environ; assert 'HOME' not in os.environ; print('clean')"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=root / "receipt")
    assert result.execution == "completed"
    assert (root / "receipt/artifacts/stdout.bin").read_bytes() == b"clean\n"


def test_import_and_default_cli_need_no_provider_configuration(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    isolated_cwd = root / "isolated_cwd"
    isolated_cwd.mkdir()
    repo = Path(__file__).resolve().parent.parent
    python = sys.executable  # Preserve the fresh virtual environment for import proof.
    env = {"PATH": "/usr/bin:/bin", "LANG": "C", "TMPDIR": str(root)}
    # -I drops cwd/site user configuration. Explicit source path is the only import input.
    imported = subprocess.run([python, "-I", "-c",
        f"import sys; sys.path.insert(0, {str(repo)!r}); import bench.harness; "
        "assert not any(n.startswith(('bench.pi_', 'bench.oai_', 'bench.multifile')) for n in sys.modules); print('offline-import')"],
        cwd=isolated_cwd, env=env, capture_output=True, check=True)
    assert imported.stdout == b"offline-import\n"
    completed = subprocess.run([python, "-I", str(repo / "bench/harness.py")], cwd=isolated_cwd,
        env=env, capture_output=True, check=True)
    receipt = json.loads(completed.stdout)
    assert receipt["backend"] == "synthetic"
    assert receipt["comparison"]["kind"] == "pass"
    assert Path(receipt["results_dir"]).is_relative_to(root)


@pytest.mark.parametrize("updates", [{"id": 1}, {"input_files": "input.md"}, {"input_files": []},
    {"input_files": [1]}, {"support_files": "input.md"}, {"support_files": [1]}, {"scorer": {}}, {"difficulty": 1}])
def test_malformed_task_shapes_never_launch(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, updates: dict[str, object]) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError):
        harness.run_agent(replace(task, **updates), fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []


def test_symlink_executable_rejects_before_spawn(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    executable = root / "python"
    executable.symlink_to(Path(sys.executable).resolve())
    with pytest.raises(ValueError, match="symlink"):
        build_runner_command([str(executable)], runner="synthetic")
