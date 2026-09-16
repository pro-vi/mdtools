"""Synthetic adaptation of H's artifact-persistence tests (c933520)."""

from __future__ import annotations

from dataclasses import replace
import hashlib
import json
from pathlib import Path
import sys
import os
import subprocess
import signal
import time

import pytest

from bench import harness
from bench.harness import BenchTask, StructuralDiffPolicy, offline_exercise, run_agent
from bench.manifest import sha256_text
from bench.trial_records import (AttemptKey, AttemptStart, ExecutionOutcome, Grade, Usage,
    RecordIntegrityError, derive_trial_disposition)
from bench.test_trial_records import synthetic_cli_events


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
    assert result.execution.kind == "completed"
    assert result.grade.kind == "pass"
    assert (receipt / "artifacts/final/nested/input.md").read_bytes() == b"# After\n"
    assert (receipt / "artifacts/stdout.bin").read_bytes() == b"finished\n"
    stored = json.loads((receipt / "result.json").read_text())
    assert stored["schema"] == "mdtools.cli-eval/1"
    assert stored["backend"] == "synthetic"
    assert stored["grade"]["kind"] == "pass"
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
    assert result.execution.kind == "completed"
    assert result.grade.kind == "fail"
    assert (fixtures / "input.md").read_bytes() == b"before\n"


@pytest.mark.parametrize("script,execution", [
    ("from pathlib import Path; Path('input.md').write_bytes(b'after\\n'); raise SystemExit(7)", "infrastructure_error"),
    ("from pathlib import Path; import time; Path('input.md').write_bytes(b'after\\n'); time.sleep(3)", "timed_out"),
])
def test_incomplete_execution_retains_file_without_comparison(tmp_path: Path, script: str, execution: str) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=tmp_path.resolve() / "receipt", timeout_seconds=0.5)
    assert result.execution.kind == execution
    assert result.grade.kind in ("not_run", "unavailable")
    assert "artifacts/final/input.md" in result.artifacts


@pytest.mark.parametrize("script", [
    "from pathlib import Path; Path('input.md').unlink()",
    "from pathlib import Path; Path('input.md').unlink(); Path('input.md').symlink_to('/dev/null')",
])
def test_capture_error_is_not_a_semantic_failure(tmp_path: Path, script: str) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=tmp_path.resolve() / "receipt")
    assert result.execution.kind == "completed"
    assert result.grade.kind in ("not_run", "unavailable")
    assert result.grade.reason == "capture_unavailable"


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
    assert result.execution.kind == "completed"
    assert result.grade.kind == "fail"
    assert result.grade.reason == "invalid_utf8"


def test_raw_bytes_accepts_binary_expected_and_actual(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    (expected / "answer.md").write_bytes(b"\xff")
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("from pathlib import Path; Path('input.md').write_bytes(b'\\xff')"), results_dir=root / "receipt")
    assert result.execution.kind == "completed"
    assert result.grade.kind == "pass"


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
            assert result.execution.kind == "timed_out"
            assert result.grade.kind in ("not_run", "unavailable")
            child_pid = int((root / "receipt/artifacts/stdout.bin").read_text().strip())
            # kill(0) can see a short-lived zombie; ps identifies a live writer.
            observed = subprocess.run(["/bin/ps", "-o", "stat=", "-p", str(child_pid)], capture_output=True, check=False)
            assert not observed.stdout.strip() or observed.stdout.strip().startswith(b"Z")
            assert sentinel.poll() is None
        finally:
            sentinel.kill()
            sentinel.wait()


def test_start_persisted_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    original = harness.subprocess.Popen
    starts = []
    def spy(*args: object, **kwargs: object):
        store = harness.AttemptStore(tmp_path / "receipt")
        start, final = store.load()
        assert final is None
        assert (tmp_path / "receipt/events.jsonl").exists()
        assert start.reservation_usd is None and start.backend == "synthetic"
        starts.append(start)
        return original(*args, **kwargs)
    monkeypatch.setattr(harness.subprocess, "Popen", spy)
    offline_exercise(tmp_path / "receipt")
    assert len(starts) == 1


def test_completed_attempt_finalizes_once(tmp_path: Path) -> None:
    result = offline_exercise(tmp_path / "receipt")
    store = harness.AttemptStore(tmp_path / "receipt")
    before = (tmp_path / "receipt/result.json").read_bytes()
    repeated = store.finalize(execution=result.execution, grade=result.grade, usage=result.usage,
        artifacts=dict(result.artifacts), evidence_complete=result.evidence_complete, exit_code=result.exit_code)
    assert repeated == result
    assert (tmp_path / "receipt/result.json").read_bytes() == before


def test_conflicting_finalization_is_rejected(tmp_path: Path) -> None:
    result = offline_exercise(tmp_path / "receipt")
    store = harness.AttemptStore(tmp_path / "receipt")
    with pytest.raises(RecordIntegrityError, match="conflicting immutable"):
        store.finalize(execution=result.execution, grade=Grade("fail", "different"), usage=result.usage,
            artifacts=dict(result.artifacts), evidence_complete=True, exit_code=result.exit_code)
    assert store.load()[1] == result


@pytest.mark.parametrize("window", ["start", "events", "manifest", "publish"])
def test_crash_windows_preserve_unfinished_or_complete_state(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, window: str) -> None:
    directory = tmp_path / "attempt-000"
    store = harness.AttemptStore(directory)
    key = AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, 0)
    start = AttemptStart(key, "synthetic")
    store.start(start)
    if window == "start":
        assert store.load() == (start, None)
        return
    store.append_event({"type": "system", "subtype": "init", "provenance": "synthetic"})
    if window == "events":
        assert store.load() == (start, None)
        assert directory.joinpath("events.jsonl").read_bytes().endswith(b"\n")
        return
    original_link = harness.os.link
    def interrupted_link(source: Path, target: Path) -> None:
        if window == "publish":
            original_link(source, target)
        raise OSError("synthetic publication crash")
    with monkeypatch.context() as context:
        context.setattr(harness.os, "link", interrupted_link)
        with pytest.raises(OSError, match="synthetic publication"):
            store.finalize(execution=ExecutionOutcome("interrupted", "operator_interrupt"), grade=Grade("not_run", "operator_interrupt"),
                usage=Usage(), artifacts={}, evidence_complete=False)
    assert (directory / "artifact_manifest.json").exists()
    observed_start, observed_result = store.load()
    assert observed_start == start
    if window == "manifest":
        assert observed_result is None
    else:
        assert observed_result.execution.kind == "interrupted"
    # Recovery uses the same one finalizer API, not a second overwrite path.
    recovered = store.finalize(execution=ExecutionOutcome("interrupted", "operator_interrupt"), grade=Grade("not_run", "operator_interrupt"),
        usage=Usage(), artifacts={}, evidence_complete=False)
    assert store.load()[1] == recovered


@pytest.mark.parametrize("damage", ["missing", "changed", "symlink", "manifest"])
def test_missing_or_changed_evidence_never_loads_as_complete(tmp_path: Path, damage: str) -> None:
    result = offline_exercise(tmp_path / "receipt")
    artifact = tmp_path / "receipt/artifacts/stdout.bin"
    if damage == "missing":
        artifact.unlink()
    elif damage == "changed":
        artifact.write_bytes(b"changed")
    elif damage == "symlink":
        artifact.unlink()
        artifact.symlink_to("/dev/null")
    else:
        (tmp_path / "receipt/artifact_manifest.json").write_bytes(b"{}")
    with pytest.raises(ValueError):
        harness.AttemptStore(tmp_path / "receipt").load()


@pytest.mark.parametrize("field", ["repetition", "evidence_complete"])
def test_artifact_manifest_rejects_bool_as_index_and_numeric_boolean(tmp_path: Path, field: str) -> None:
    offline_exercise(tmp_path / "receipt")
    manifest_path = tmp_path / "receipt/artifact_manifest.json"
    manifest = json.loads(manifest_path.read_bytes())
    if field == "repetition":
        manifest["key"]["repetition"] = False
    else:
        manifest["evidence_complete"] = 1
    changed = json.dumps(manifest).encode()
    manifest_path.write_bytes(changed)
    result_path = tmp_path / "receipt/result.json"
    result = json.loads(result_path.read_bytes())
    result["artifact_manifest_sha256"] = hashlib.sha256(changed).hexdigest()
    result_path.write_bytes(json.dumps(result).encode())
    with pytest.raises(RecordIntegrityError):
        harness.AttemptStore(tmp_path / "receipt").load()


def test_permission_fault_survives_resume_after_actual_controller_crash(tmp_path: Path) -> None:
    directory = tmp_path / "attempt-000"
    repo = str(Path(__file__).resolve().parent.parent)
    events = synthetic_cli_events(denied=True)[:3]
    script = (f"import sys,os; sys.path.insert(0,{repo!r}); from bench.harness import AttemptStore; "
        "from bench.trial_records import AttemptKey,AttemptStart; from pathlib import Path; "
        f"s=AttemptStore(Path({str(directory)!r})); s.start(AttemptStart(AttemptKey({'a' * 64!r},'synthetic','no-md',0,0),'synthetic')); "
        f"[s.append_event(e) for e in {events!r}]; os._exit(17)")
    crashed = subprocess.run([sys.executable, "-I", "-c", script], capture_output=True, check=False)
    assert crashed.returncode == 17
    store = harness.AttemptStore(directory)
    start, final = store.load()
    assert final is None
    assert derive_trial_disposition(start.key.trial, starts=[start]).kind == "running"
    with pytest.raises(RecordIntegrityError, match="permission_denied"):
        store.assert_admission()
    # A later successful receipt is retained, but cannot erase the early fault.
    store.append_event(synthetic_cli_events()[-1])
    assert store.persisted_fault() == "permission_denied"


def test_denial_stops_owned_process_before_later_effect(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    task = replace(task, expected_artifact="stdout_text")
    denied = b"".join((json.dumps(e) + "\n").encode() for e in synthetic_cli_events(denied=True)[:3])
    marker = tmp_path / "later-effect"
    script = f"import sys,time; from pathlib import Path; sys.stdout.buffer.write({denied!r}); sys.stdout.buffer.flush(); time.sleep(0.4); Path({str(marker)!r}).write_text('forbidden later effect')"
    calls = []
    monkeypatch.setattr(harness, "grade_submission", lambda *a, **kw: calls.append(kw))
    result = run_agent(task, fixture_root=fixtures, expected_root=expected, command=python_command(script),
        results_dir=tmp_path / "receipt", event_format="claude_stream", timeout_seconds=3)
    assert result.permission_fault == "permission_denied" and result.grade.kind == "not_run"
    assert not result.evidence_complete and calls == [] and not marker.exists()
    assert harness.AttemptStore(tmp_path / "receipt").persisted_fault() == "permission_denied"


def test_capture_persistence_failure_never_publishes_or_grades(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    calls = []
    original = harness._write_private
    def failing_capture(path: Path, content: bytes) -> None:
        if path.name == "stdout.bin":
            raise OSError("synthetic capture failure")
        original(path, content)
    monkeypatch.setattr(harness, "_write_private", failing_capture)
    monkeypatch.setattr(harness, "grade_submission", lambda *a, **kw: calls.append(kw))
    with pytest.raises(OSError, match="capture failure"):
        run_agent(task, fixture_root=fixtures, expected_root=expected, command=python_command("print('after')"), results_dir=tmp_path / "receipt")
    assert harness.AttemptStore(tmp_path / "receipt").load()[1] is None
    assert calls == []


def test_timeout_keeps_partial_evidence_and_rejects_late_result(tmp_path: Path) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    result = run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("from pathlib import Path; import time; Path('input.md').write_bytes(b'after\\n'); print('partial',flush=True); time.sleep(3)"),
        results_dir=tmp_path / "receipt", timeout_seconds=0.2)
    assert result.execution.kind == "timed_out" and result.grade.kind == "not_run" and not result.evidence_complete
    store = harness.AttemptStore(tmp_path / "receipt")
    with pytest.raises(RecordIntegrityError, match="closed attempt"):
        store.append_event(synthetic_cli_events()[-1])
    with pytest.raises(RecordIntegrityError):
        store.finalize(execution=ExecutionOutcome("completed"), grade=Grade("pass", "late"), usage=result.usage,
            artifacts=dict(result.artifacts), evidence_complete=False)
    assert store.load()[1] == result


def test_interrupt_preserves_attempt_and_stops_next_launch(tmp_path: Path) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    repo = str(Path(__file__).resolve().parent.parent)
    child = python_command("import time; print('started',flush=True); time.sleep(30)")
    script = (f"import sys; sys.path.insert(0,{repo!r}); from pathlib import Path; from bench.harness import *; "
        f"task={task!r}; result=run_agent(task, fixture_root=Path({str(fixtures)!r}), expected_root=Path({str(expected)!r}), command={child!r}, results_dir=Path({str(tmp_path / 'receipt')!r})); "
        "print(result.execution.kind)")
    with subprocess.Popen([sys.executable, "-I", "-c", script], stdout=subprocess.PIPE, stderr=subprocess.PIPE) as controller:
        deadline = time.monotonic() + 5
        while not (tmp_path / "receipt/started.json").exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        # Give the real capture loop time to enter its signal-handling boundary.
        time.sleep(0.15)
        controller.send_signal(signal.SIGINT)
        stdout, stderr = controller.communicate(timeout=5)
    assert controller.returncode == 0, stderr
    assert stdout.strip() == b"interrupted"
    start, result = harness.AttemptStore(tmp_path / "receipt").load()
    assert result.grade.kind == "not_run" and not result.evidence_complete
    assert derive_trial_disposition(start.key.trial, starts=[start], results=[result]).next_ordinal is None


def test_retry_retains_first_attempt_and_cost_with_fresh_original_inputs(tmp_path: Path) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    events = synthetic_cli_events()
    events[-1].update(subtype="error_during_execution", terminal_reason="transport_error", is_error=True)
    stream = b"".join((json.dumps(e) + "\n").encode() for e in events)
    command = python_command("from pathlib import Path; import sys; assert Path('input.md').read_bytes()==b'before\\n'; "
        f"Path('input.md').write_bytes(b'after\\n'); sys.stdout.buffer.write({stream!r})")
    first = run_agent(task, fixture_root=fixtures, expected_root=expected, command=command,
        results_dir=tmp_path / "attempt-000", event_format="claude_stream")
    first_start, reloaded = harness.AttemptStore(tmp_path / "attempt-000").load()
    assert first == reloaded
    second = run_agent(task, fixture_root=fixtures, expected_root=expected, command=command,
        results_dir=tmp_path / "attempt-001", event_format="claude_stream", attempt_key=replace(first.key, ordinal=1),
        prior_attempts=[(first_start, first)])
    second_start, _ = harness.AttemptStore(tmp_path / "attempt-001").load()
    selected = derive_trial_disposition(first.key.trial, starts=[first_start, second_start], results=[first, second])
    assert selected.kind == "operational_failed" and selected.selected == second.key
    assert first.usage.estimated_usd + second.usage.estimated_usd == 0.02
    assert (fixtures / "input.md").read_bytes() == b"before\n"
    assert (tmp_path / "attempt-000/artifacts/final/input.md").read_bytes() == b"after\n"
    assert (tmp_path / "attempt-001/artifacts/final/input.md").read_bytes() == b"after\n"
    with pytest.raises(RecordIntegrityError, match="forbids"):
        run_agent(task, fixture_root=fixtures, expected_root=expected, command=command,
            results_dir=tmp_path / "attempt-002", event_format="claude_stream", attempt_key=replace(first.key, ordinal=2),
            prior_attempts=[(first_start, first), (second_start, second)])
    assert not (tmp_path / "attempt-002").exists()


@pytest.mark.parametrize("execution", [ExecutionOutcome("timed_out", "deadline"), ExecutionOutcome("interrupted", "operator_interrupt")])
def test_finalizer_retains_denial_when_controller_outcome_already_closed(tmp_path: Path, execution: ExecutionOutcome) -> None:
    store = harness.AttemptStore(tmp_path / "attempt-000")
    start = AttemptStart(AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, 0), "synthetic")
    store.start(start)
    for event in synthetic_cli_events(denied=True)[:3]:
        store.append_event(event)
    result = store.finalize(execution=execution, grade=Grade("not_run", execution.reason), usage=Usage(),
        artifacts={}, evidence_complete=False)
    assert result.execution == execution and result.permission_fault == "permission_denied"
    assert store.load()[1] == result
    with pytest.raises(RecordIntegrityError, match="permission_denied"):
        store.assert_admission()


def test_second_finalizer_cannot_write_locked_attempt(tmp_path: Path) -> None:
    store = harness.AttemptStore(tmp_path / "attempt-000")
    store.start(AttemptStart(AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, 0), "synthetic"))
    with store._locked_file():
        with pytest.raises(BlockingIOError):
            harness.AttemptStore(store.directory).finalize(execution=ExecutionOutcome("interrupted", "operator_interrupt"),
                grade=Grade("not_run", "operator_interrupt"), usage=Usage(), artifacts={}, evidence_complete=False)
    assert store.load()[1] is None


@pytest.mark.parametrize("boundary", ["feed", "raw_callback", "read", "finish", "cleanup"])
def test_interruption_at_capture_boundaries_stops_and_reaps_owned_process(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, boundary: str) -> None:
    cleanup_calls = []
    store = harness.AttemptStore(tmp_path / "attempt-000")
    store.start(AttemptStart(AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, 0), "synthetic"))
    def persist(raw: dict[str, object]) -> None:
        store.append_event(raw)
        if boundary == "raw_callback":
            raise KeyboardInterrupt
    decoder = harness.ClaudeStreamDecoder(on_raw=persist)
    if boundary == "cleanup":
        decoder = None
    if boundary == "feed":
        monkeypatch.setattr(decoder, "feed", lambda chunk: (_ for _ in ()).throw(KeyboardInterrupt()))
    if boundary == "finish":
        monkeypatch.setattr(decoder, "finish", lambda **kwargs: (_ for _ in ()).throw(KeyboardInterrupt()))
    script = "import sys,time; print('{}',flush=True); " + ("time.sleep(0.08); print('{}',flush=True); time.sleep(20)" if boundary not in ("finish", "cleanup") else "pass")
    with subprocess.Popen(python_command("import time; time.sleep(20)"), start_new_session=True) as sentinel:
        try:
            with subprocess.Popen(python_command(script), stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, start_new_session=True) as process:
                original_read = harness.os.read
                reads = []
                if boundary == "read":
                    def interrupted_read(descriptor: int, count: int) -> bytes:
                        if descriptor == process.stdout.fileno():
                            reads.append(descriptor)
                            if len(reads) == 2:
                                raise KeyboardInterrupt
                        return original_read(descriptor, count)
                    monkeypatch.setattr(harness.os, "read", interrupted_read)
                def stop_owned() -> None:
                    cleanup_calls.append(process.pid)
                    if boundary == "cleanup" and len(cleanup_calls) == 1:
                        raise KeyboardInterrupt
                    harness._stop_owned_group(process)
                try:
                    try:
                        captured = harness.capture_process(process, prompt=b"", timeout_seconds=3, decoder=decoder, stop_owned=stop_owned)
                    except KeyboardInterrupt:
                        pytest.fail("capture interruption bypassed owned lifecycle cleanup")
                    assert captured.execution == ExecutionOutcome("interrupted", "operator_interrupt")
                    assert captured.stdout.startswith(b"{}\n")
                    assert cleanup_calls and process.poll() is not None
                    assert sentinel.poll() is None
                    if boundary == "raw_callback":
                        assert (store.directory / "events.jsonl").read_bytes() == b"{}\n"
                finally:
                    harness._stop_owned_group(process)
                    process.wait(timeout=5)
        finally:
            sentinel.kill()
            sentinel.wait()


def test_interruption_cleanup_failure_is_reported_after_owned_parent_reaped(monkeypatch: pytest.MonkeyPatch) -> None:
    decoder = harness.ClaudeStreamDecoder()
    monkeypatch.setattr(decoder, "feed", lambda chunk: (_ for _ in ()).throw(KeyboardInterrupt()))
    calls = []
    with subprocess.Popen(python_command("print('{}',flush=True); import time; time.sleep(20)"), stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True) as process:
        def failed_owned_cleanup() -> None:
            calls.append(process.pid)
            raise OSError("synthetic owned cleanup failure")
        try:
            try:
                with pytest.raises(OSError, match="owned cleanup failure"):
                    harness.capture_process(process, prompt=b"", timeout_seconds=3, decoder=decoder, stop_owned=failed_owned_cleanup)
            except KeyboardInterrupt:
                pytest.fail("capture interruption bypassed supplied cleanup")
            assert calls and process.poll() is not None
        finally:
            harness._stop_owned_group(process)
            process.wait(timeout=5)


@pytest.mark.parametrize("damage", ["final_text", "usage", "model"])
@pytest.mark.parametrize("phase", ["finalize", "reload"])
def test_stream_evidence_cannot_claim_unrelated_terminal_fields(tmp_path: Path, damage: str, phase: str) -> None:
    store = harness.AttemptStore(tmp_path / "attempt-000")
    store.start(AttemptStart(AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, 0), "synthetic"))
    decoder = harness.ClaudeStreamDecoder(on_raw=store.append_event)
    events = synthetic_cli_events()
    for event in events:
        decoder.accept(event)
    receipt = decoder.finish().receipt
    payloads = {"artifacts/stdout.bin": b"synthetic transport capture", "artifacts/stderr.bin": b"",
                "artifacts/final_submission.bin": receipt.final_text.encode()}
    if damage == "final_text" and phase == "finalize":
        payloads["artifacts/final_submission.bin"] = b"unrelated answer"
    artifacts = {}
    for name, content in payloads.items():
        harness._write_private(store.directory / name, content)
        artifacts[name] = hashlib.sha256(content).hexdigest()
    usage = replace(receipt.usage, input_tokens=999) if damage == "usage" and phase == "finalize" else receipt.usage
    model = "unrelated-model" if damage == "model" and phase == "finalize" else receipt.observed_model
    if phase == "finalize":
        with pytest.raises(RecordIntegrityError, match="terminal|receipt"):
            store.finalize(execution=receipt.execution, grade=Grade("pass", "synthetic-grade"), usage=usage,
                           artifacts=artifacts, evidence_complete=True, observed_model=model)
        return
    store.finalize(execution=receipt.execution, grade=Grade("pass", "synthetic-grade"), usage=usage,
                   artifacts=artifacts, evidence_complete=True, observed_model=model)
    result_path = store.directory / "result.json"
    stored = json.loads(result_path.read_bytes())
    if damage == "usage":
        stored["usage"]["input_tokens"] = 999
    elif damage == "model":
        stored["observed_model"] = "unrelated-model"
    else:
        name = "artifacts/final_submission.bin"
        (store.directory / name).write_bytes(b"unrelated answer")
        digest = hashlib.sha256(b"unrelated answer").hexdigest()
        stored["artifacts"][name] = digest
        manifest_path = store.directory / "artifact_manifest.json"
        manifest = json.loads(manifest_path.read_bytes())
        manifest["artifacts"][name] = digest
        manifest_bytes = json.dumps(manifest).encode()
        manifest_path.write_bytes(manifest_bytes)
        stored["artifact_manifest_sha256"] = hashlib.sha256(manifest_bytes).hexdigest()
    result_path.write_bytes(json.dumps(stored).encode())
    with pytest.raises(RecordIntegrityError, match="terminal|receipt"):
        store.load()


def test_stream_receipt_binding_preserves_independent_controller_elapsed(tmp_path: Path) -> None:
    store = harness.AttemptStore(tmp_path / "attempt-000")
    store.start(AttemptStart(AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, 0), "synthetic"))
    decoder = harness.ClaudeStreamDecoder(on_raw=store.append_event)
    for event in synthetic_cli_events():
        decoder.accept(event)
    receipt = decoder.finish().receipt
    payloads = {"artifacts/stdout.bin": b"synthetic capture", "artifacts/stderr.bin": b"",
                "artifacts/final_submission.bin": receipt.final_text.encode()}
    artifacts = {}
    for name, content in payloads.items():
        harness._write_private(store.directory / name, content)
        artifacts[name] = hashlib.sha256(content).hexdigest()
    usage = replace(receipt.usage, elapsed_seconds=12.34)
    final = store.finalize(execution=receipt.execution, grade=Grade("pass", "synthetic-grade"), usage=usage,
        artifacts=artifacts, evidence_complete=True, observed_model=receipt.observed_model)
    assert final.usage.elapsed_seconds == 12.34
    assert store.load()[1] == final
