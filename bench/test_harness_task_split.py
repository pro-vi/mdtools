"""H's selection/staging intent, exercised only on synthetic task packages.

Historical tests traversing holdout descriptions/fingerprints are not recovered.
"""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

import pytest

from bench import harness
from bench.test_harness_run_artifacts import python_command, synthetic_task


def test_nested_multiple_inputs_preserve_paths_and_hide_expected(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    for reference in ("one/shared/input.md", "two/shared/input.md", "support/value.txt"):
        destination = fixtures / reference
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(reference.encode())
    task = replace(task, input_files=["input.md", "one/shared/input.md", "two/shared/input.md"],
                   support_files=["support/value.txt"], description="Perform the declared file edit.")
    script = (
        "from pathlib import Path; "
        "assert {str(p) for p in Path('.').rglob('*') if p.is_file()} == "
        "{'input.md','one/shared/input.md','two/shared/input.md','support/value.txt'}; "
        "assert Path('one/shared/input.md').read_text() == 'one/shared/input.md'; "
        "assert Path('two/shared/input.md').read_text() == 'two/shared/input.md'; "
        "Path('input.md').write_bytes(b'after\\n')"
    )
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=root / "receipt")
    assert result.grade.kind == "pass"
    assert "after" not in harness.build_prompt(task)
    assert str(expected) not in harness.build_prompt(task)
    assert "before" not in harness.build_prompt(task)


@pytest.mark.parametrize("reference", ["../outside.md", "/absolute.md", "nested/../input.md", "./input.md", "nested//input.md", "nested\\input.md", "missing.md"])
def test_invalid_input_rejects_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, reference: str) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, input_files=[reference])
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError):
        harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []
    assert not (root / "receipt").exists()


@pytest.mark.parametrize("target", ["input", "parent", "expected", "root"])
def test_symlink_components_reject_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, target: str) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    if target == "input":
        (fixtures / "alias.md").symlink_to(fixtures / "input.md")
        task = replace(task, input_files=["alias.md"])
    elif target == "parent":
        (fixtures / "alias").symlink_to(fixtures, target_is_directory=True)
        task = replace(task, input_files=["alias/input.md"])
    elif target == "expected":
        (expected / "alias.md").symlink_to(expected / "answer.md")
        task = replace(task, expected_output="alias.md")
    else:
        (root / "alias").symlink_to(fixtures, target_is_directory=True)
        fixtures = root / "alias"
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError, match="symlink"):
        harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []


@pytest.mark.parametrize("inputs,support", [(["input.md", "input.md"], None), (["input.md"], ["input.md"]), (["input.md", "input.md/child"], None)])
def test_staging_collision_rejects_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, inputs: list[str], support: list[str] | None) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, input_files=inputs, support_files=support)
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError, match="collision"):
        harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []


def test_missing_expected_and_overlapping_roots_reject_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    for changed_task, expected_root in ((replace(task, expected_output="missing.md"), expected), (task, fixtures)):
        with pytest.raises(ValueError):
            harness.run_agent(changed_task, fixture_root=fixtures, expected_root=expected_root,
                command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []


def test_invalid_expected_text_rejects_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, scorer=replace(task.scorer, kind="normalized_text"))
    (expected / "answer.md").write_bytes(b"\xff")
    launches = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: launches.append(args))
    with pytest.raises(ValueError, match="invalid_expected_utf8"):
        harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert launches == []
    assert not (root / "receipt").exists()
