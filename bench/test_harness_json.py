"""Adapt H JSON/scoring tests without treatment diagnostics or holdout reads."""

from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
import subprocess
import shutil
import os
import copy

import pytest

from bench import harness, neutral_scorer as scorer
from bench.test_harness_run_artifacts import python_command, synthetic_task
from bench.test_neutral_scorer import family_cases, policy
from bench.test_trial_records import synthetic_cli_events
from bench.trial_records import (RecordIntegrityError, ToolCall, ToolResult, RunReceipt,
    ExecutionOutcome, Grade, Usage, event_from_dict, record_dict)


@pytest.mark.parametrize("actual,expected,passes", [
    (b'{"pending":true}', b'{"pending":1}', False),
    (b'{"pending":1.0}', b'{"pending":1}', True),
    (b'{"value":1.0000000000000001}', b'{"value":1.0}', False),
    (b'{"pending":1,"pending":1}', b'{"pending":1}', False),
    (b'{"pending":NaN}', b'{"pending":1}', False),
    (b'{"pending":Infinity}', b'{"pending":1}', False),
    (b'{"pending":1e999}', b'{"pending":1}', False),
    (b'{}', b'{"pending":1}', False),
    (b'{"a":1,"b":[]}', b'{"b":[],"a":1}', True),
    (b'[]', b'[]', True),
    (b'[1,2]', b'[2,1]', False),
    (b'{"a":{"n":true}}', b'{"a":{"n":1}}', False),
    (b'```json\n{"pending":1}\n```', b'{"pending":1}', False),
    (b'correct: {"pending":1}', b'{"pending":1}', False),
    (b'', b'{"pending":1}', False),
    (b'\xff', b'{"pending":1}', False),
])
def test_json_types_and_required_fields(actual: bytes, expected: bytes, passes: bool) -> None:
    comparison = scorer.grade_json(policy("structural", json_canonical=True), actual, expected)
    assert (comparison.kind == "pass") is passes


def test_json_required_key_projection_preserves_array_order_and_facts() -> None:
    declared = policy("structural", json_canonical=True, json_required_keys=["loc", "text"])
    expected = b'[{"loc":"unchanged-original-locator","text":"first","extra":1},{"loc":"second","text":"last"}]'
    good = b'[{"text":"first","loc":"unchanged-original-locator"},{"text":"last","loc":"second","extra":99}]'
    assert scorer.grade_json(declared, good, expected).kind == "pass"
    assert scorer.grade_json(declared, b'[{"text":"first"}]', expected).reason == "invalid_submission:missing_required_keys"
    assert scorer.grade_json(declared, good.replace(b"unchanged-original-locator", b"new-address"), expected).kind == "fail"
    assert scorer.grade_json(declared, b'[]', b'[]').kind == "pass"
    with pytest.raises(ValueError, match="invalid_expected_json:missing_required_keys"):
        scorer.grade_json(declared, b'[]', b'[{"text":"no locator"}]')


def test_json_validation_detection_and_non_detection() -> None:
    empty = policy("structural", json_canonical=True, json_required_keys=[])
    assert scorer.grade_json(empty, b'{"a":1.0}', b'{"a":1}').kind == "pass"
    assert scorer.grade_json(empty, b'{"a":2}', b'{"a":1}').kind == "fail"
    empty_key = replace(empty, json_required_keys=[""])
    assert scorer.grade_json(empty_key, b'{"":1.0,"other":2}', b'{"":1}').kind == "pass"
    assert scorer.grade_json(empty_key, b'{"other":1}', b'{"":1}').kind == "fail"
    assert scorer.grade_json(empty, b'{"a":1e999}', b'{"a":10e998}').kind == "pass"


def test_semantic_heading_projection_and_required_empty_list() -> None:
    declared = policy("structural", compare_heading_tree=True)
    expected = b'{"entries":[{"heading":{"level":2,"text":"Repeat","span":1}},{"heading":{"level":2,"text":"Repeat"}}]}'
    answer = b'[{"level":2,"text":"Repeat"},{"text":"Repeat","level":2}]'
    assert scorer.grade_json(declared, answer, expected).kind == "pass"
    for bad in (b'[]', b'[{"level":true,"text":"Repeat"}]', b'[{"level":2}]', expected, b'{}'):
        assert scorer.grade_json(declared, bad, expected).kind == "fail"
    assert scorer.grade_json(declared, b'[]', b'{"entries":[]}').kind == "pass"
    assert scorer.grade_json(declared, b'{}', b'{"entries":[]}').kind == "fail"
    with pytest.raises(ValueError, match="missing_heading_entries"):
        scorer.grade_json(declared, b'[]', b'{}')


def test_frontmatter_payload_presence_and_format_are_required() -> None:
    declared = policy("structural", compare_frontmatter_json=True)
    expected = b'{"present":true,"frontmatter":{"format":"Yaml","data":{"n":1}}}'
    actual = b'{"present":true,"format":"Yaml","value":{"n":1}}'
    assert scorer.grade_json(declared, actual, expected).kind == "pass"
    for expected_format in (b"Yaml", b"yaml", b"YAML"):
        for actual_format in (b"Yaml", b"yaml", b"YAML"):
            assert scorer.grade_json(declared, actual.replace(b"Yaml", actual_format),
                expected.replace(b"Yaml", expected_format)).kind == "pass"
    assert scorer.grade_json(declared, actual.replace(b"Yaml", b"Custom"), expected.replace(b"Yaml", b"custom")).kind == "fail"
    assert scorer.grade_json(declared, actual.replace(b"Yaml", b" YAML "), expected).kind == "fail"
    for bad in (actual.replace(b"Yaml", b"Toml"), actual.replace(b"Yaml", b"toml"), actual.replace(b'"n":1', b'"n":true'), b'{"present":true,"value":{"n":1}}', b'{"present":1,"format":"Yaml","value":{"n":1}}'):
        assert scorer.grade_json(declared, bad, expected).kind == "fail"
    absent = b'{"present":false,"format":null,"value":null}'
    assert scorer.grade_json(declared, absent, b'{"present":false,"frontmatter":null}').kind == "pass"
    assert scorer.grade_json(declared, absent.replace(b'"value":null', b'"value":{}'), b'{"present":false}').kind == "fail"


def test_link_destinations_keep_collection_order_and_required_fields() -> None:
    declared = policy("structural", compare_link_destinations=True)
    expected = b'{"links":[{"kind":"link","destination":"a","span":1},{"kind":"image","destination":"b"}]}'
    good = b'[{"kind":"link","destination":"a"},{"kind":"image","destination":"b"}]'
    assert scorer.grade_json(declared, good, expected).kind == "pass"
    assert scorer.grade_json(declared, b'[]', b'{"links":[]}').kind == "pass"
    for bad in (b'{}', b'[{"destination":"a"}]', good.replace(b'"a"', b'"wrong"'), expected):
        assert scorer.grade_json(declared, bad, expected).kind == "fail"
    with pytest.raises(ValueError, match="missing_links"):
        scorer.grade_json(declared, b'[]', b'{}')


@pytest.mark.parametrize("artifact,declared,expected,expected_stdout", [
    ("json_envelope", policy("structural", json_canonical=True), b'{"n":1,"n":1}', None),
    ("json_envelope", policy("structural", compare_heading_tree=True), b'{"entries":[{"heading":{"level":true,"text":"x"}}]}', None),
    ("json_envelope", policy("structural", json_canonical=True), b'{"n":NaN}', None),
    ("stdout_text", policy("raw_bytes"), b'\xff', None),
    ("stdout_and_file", policy("raw_bytes"), b'file', None),
    ("stdout_and_file", policy("raw_bytes"), b'file', b'\xff'),
])
def test_invalid_expectation_rejects_before_spawn(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, artifact: str, declared: scorer.StructuralDiffPolicy, expected: bytes, expected_stdout: bytes | None) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected_root = synthetic_task(root)
    task = replace(task, expected_artifact=artifact, scorer=declared, expected_stdout=expected_stdout.decode("latin1") if expected_stdout is not None else None)
    (expected_root / "answer.md").write_bytes(expected)
    calls = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: calls.append(args))
    # Python strings encode as valid UTF-8, so the invalid stdout bytes boundary
    # itself is tested directly; the controller never accepts arbitrary bytes.
    if expected_stdout == b'\xff':
        with pytest.raises(ValueError, match="invalid_expected_stdout_utf8"):
            scorer.validate_expected(declared, expected, artifact=artifact, expected_stdout=expected_stdout)
        return
    with pytest.raises(ValueError):
        harness.run_agent(task, fixture_root=fixtures, expected_root=expected_root,
            command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert calls == []
    assert not (root / "receipt").exists()


@pytest.mark.parametrize("artifact,declared,expected,final,file_bytes,expected_stdout", [
    ("json_envelope", policy("structural", json_canonical=True), b'{"count":1}', b'{"count":1}', None, None),
    ("json_envelope", policy("structural", compare_heading_tree=True), b'{"entries":[{"heading":{"level":1,"text":"H"}}]}', b'[{"level":1,"text":"H"}]', None, None),
    ("json_envelope", policy("structural", compare_frontmatter_json=True), b'{"present":true,"frontmatter":{"format":"Yaml","data":{"a":1}}}', b'{"present":true,"format":"Yaml","value":{"a":1}}', None, None),
    ("json_envelope", policy("structural", compare_link_destinations=True), b'{"links":[{"kind":"link","destination":"a"}]}', b'[{"kind":"link","destination":"a"}]', None, None),
    ("stdout_text", policy(), b'text\n', b'text\n', None, None),
    ("stdout_and_file", policy("raw_bytes"), b'after\n', b'text\n', b'after\n', "text\n"),
    ("file_contents", policy(compare_heading_tree=True, compare_block_order=True, compare_block_text=True), b'# After\n', b'ignored', b'# After\n', None),
])
def test_synthetic_subprocess_admitted_families(tmp_path: Path, artifact: str, declared: scorer.StructuralDiffPolicy, expected: bytes, final: bytes, file_bytes: bytes | None, expected_stdout: str | None) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected_root = synthetic_task(root)
    task = replace(task, expected_artifact=artifact, scorer=declared, expected_stdout=expected_stdout, input_files=["input.md", "second.md"])
    (fixtures / "second.md").write_bytes(b"second independent input\n")
    (expected_root / "answer.md").write_bytes(expected)
    script = "from pathlib import Path; import sys; assert Path('second.md').read_bytes() == b'second independent input\\n'; "
    if file_bytes is not None:
        script += f"Path('input.md').write_bytes({file_bytes!r}); "
    script += f"sys.stdout.buffer.write({final!r})"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected_root,
        command=python_command(script), results_dir=root / "receipt")
    assert result.execution.kind == "completed"
    assert result.grade.kind == "pass"
    if artifact != "file_contents":
        assert (root / "receipt/artifacts/final_submission.bin").read_bytes() == final


def test_intermediate_output_cannot_replace_submission(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, expected_artifact="json_envelope", scorer=policy("structural", json_canonical=True))
    (expected / "answer.md").write_bytes(b'{"pending":1}')
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("import sys; sys.stderr.write('{\"pending\":1}'); sys.stdout.write('{\"pending\":0}')"), results_dir=root / "receipt")
    assert result.execution.kind == "completed"
    assert result.grade.kind == "fail"
    assert result.grade.reason == "json_mismatch"
    assert (root / "receipt/artifacts/stderr.bin").read_bytes() == b'{"pending":1}'
    assert (root / "receipt/artifacts/final_submission.bin").read_bytes() == b'{"pending":0}'


@pytest.mark.parametrize("text,file_bytes,passes", [(b"text\n", b"after\n", True), (b"wrong\n", b"after\n", False), (b"text\n", b"wrong\n", False), (b"", b"after\n", False)])
def test_stdout_and_file_composition(tmp_path: Path, text: bytes, file_bytes: bytes, passes: bool) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, expected_artifact="stdout_and_file", expected_stdout="text\n")
    script = f"from pathlib import Path; import sys; Path('input.md').write_bytes({file_bytes!r}); sys.stdout.buffer.write({text!r})"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=root / "receipt")
    assert (result.grade.kind == "pass") is passes


def public_task(task_id: str) -> harness.BenchTask:
    """Explicit allowlisted jq projection: never load holdout rows into Python."""
    if task_id not in ("T1", "T2", "T10", "T21"):
        raise ValueError("public packaging fixture is not allowlisted")
    repo = Path(__file__).resolve().parent.parent
    jq = shutil.which("jq")
    if jq is None:
        raise RuntimeError("public-fixture projection requires jq")
    projected = subprocess.run([jq, "--arg", "id", task_id, '.[] | select(.id == $id) | {id,description,input_files,expected_output,expected_artifact,difficulty,scorer,expected_stdout,support_files}', str(repo / "bench/tasks/tasks.json")], check=True, capture_output=True)
    record = json.loads(projected.stdout)
    record["scorer"] = scorer.StructuralDiffPolicy(**record["scorer"])
    return harness.BenchTask(**record)


@pytest.mark.parametrize("task_id", ["T1", "T2", "T10", "T21"])
def test_public_packaging_preserves_frozen_bytes(tmp_path: Path, task_id: str) -> None:
    root, repo = tmp_path.resolve(), Path(__file__).resolve().parent.parent
    task = public_task(task_id)
    fixtures, expected_root = root / "inputs", root / "expected"
    for reference in [*task.input_files, *(task.support_files or [])]:
        target = fixtures / reference
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((repo / reference).read_bytes())
    expected = (repo / task.expected_output).read_bytes()
    target = expected_root / task.expected_output
    target.parent.mkdir(parents=True)
    target.write_bytes(expected)
    historical = subprocess.run(["git", "show", f"c933520:{task.expected_output}"], cwd=repo, check=True, capture_output=True).stdout
    assert historical == expected
    if task.expected_artifact == "json_envelope":
        answer = json.dumps(scorer.expected_answer(task.scorer, expected), ensure_ascii=False).encode()
        script = f"import sys; sys.stdout.buffer.write({answer!r})"
    else:
        script = f"from pathlib import Path; Path({task.input_files[0]!r}).write_bytes({expected!r})"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected_root,
        command=python_command(script), results_dir=root / "receipt")
    assert result.grade.kind == "pass"
    assert (repo / task.expected_output).read_bytes() == expected


@pytest.mark.parametrize("adapter,affected", [("grade_stdout_text", "stdout_text"), ("grade_stdout_and_file", "stdout_and_file")])
@pytest.mark.parametrize("fault", ["exception", "wrong_grade"])
def test_text_adapter_fault_preserves_other_families_stored_evidence(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, adapter: str, affected: str, fault: str) -> None:
    root = tmp_path.resolve()
    cases = family_cases()
    def exercise(directory: Path, family: str, case: dict[str, object]) -> tuple[harness.AttemptResult, dict[str, bytes], str]:
        directory.mkdir()
        task, fixtures, expected = synthetic_task(directory)
        task = replace(task, description="Compute the requested result.", expected_artifact=family, scorer=case["policy"], expected_stdout=case.get("expected_stdout", b"").decode() if family == "stdout_and_file" else None)
        (expected / "answer.md").write_bytes(case["expected"])
        script = "from pathlib import Path; import sys; "
        if case["actual_file"] is not None:
            script += f"Path('input.md').write_bytes({case['actual_file']!r}); "
        script += f"sys.stdout.buffer.write({case['final_text']!r})"
        result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command(script), results_dir=directory / "receipt")
        evidence = {name: (directory / "receipt" / name).read_bytes() for name in result.artifacts}
        return result, evidence, harness.build_prompt(task)
    before = {family: exercise(root / (family + "_before"), family, case) for family, case in cases.items()}
    def broken(*args: object) -> scorer.Grade:
        if fault == "exception":
            raise RuntimeError("synthetic adapter defect")
        return scorer.Grade("fail", "injected_wrong_grade")
    monkeypatch.setattr(scorer, adapter, broken)
    for family, case in cases.items():
        result, evidence, prompt = exercise(root / (family + "_after"), family, case)
        original_result, original_evidence, original_prompt = before[family]
        assert evidence == original_evidence
        assert result.artifacts == original_result.artifacts
        assert prompt == original_prompt
        assert result.key.experiment_id == original_result.key.experiment_id
        if family == affected:
            if fault == "exception":
                assert result.grade.kind in ("not_run", "unavailable")
                assert result.grade.reason == "grader_unavailable"
                assert result.execution.kind == "completed"
            else:
                assert result.grade.kind == "fail"
        else:
            assert result.key == original_result.key
            assert result.grade == original_result.grade
            assert result.execution == original_result.execution


def test_grader_source_digest_changes_synthetic_identity(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    command = python_command("from pathlib import Path; Path('input.md').write_bytes(b'after\\n')")
    before = harness.run_agent(task, fixture_root=fixtures, expected_root=expected, command=command, results_dir=root / "before")
    original_hash = harness.sha256_file
    def changed_grader(path: str | Path) -> str:
        return "0" * 64 if Path(path).name == "neutral_scorer.py" else original_hash(path)
    monkeypatch.setattr(harness, "sha256_file", changed_grader)
    after = harness.run_agent(task, fixture_root=fixtures, expected_root=expected, command=command, results_dir=root / "after")
    assert before.grade == after.grade
    assert before.artifacts == after.artifacts
    assert before.key.experiment_id != after.key.experiment_id
    before_spec = json.loads((root / "before/experiment.json").read_bytes())
    after_spec = json.loads((root / "after/experiment.json").read_bytes())
    assert before_spec["task_sha256"] == after_spec["task_sha256"]
    assert before_spec["task_sha256"] == harness.sha256_text(harness.canonical_json(harness.asdict(task)))
    assert before_spec["harness_sha256"] == after_spec["harness_sha256"]
    assert before_spec["grader_sha256"] != after_spec["grader_sha256"]


@pytest.mark.parametrize("artifact,declared", [
    ("json_envelope", policy("structural", compare_heading_tree=True, compare_link_destinations=True)),
    ("json_envelope", policy("structural", json_canonical=True, json_required_keys=["a", "a"])),
    ("file_contents", policy(compare_frontmatter_json=True)),
    ("stdout_text", policy(compare_block_text=True)),
])
def test_denied_unsupported_policy_before_launch(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, artifact: str, declared: scorer.StructuralDiffPolicy) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    calls = []
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: calls.append(args))
    with pytest.raises(ValueError):
        harness.run_agent(replace(task, expected_artifact=artifact, scorer=declared), fixture_root=fixtures,
            expected_root=expected, command=python_command("raise SystemExit(99)"), results_dir=root / "receipt")
    assert calls == []
    assert not (root / "receipt").exists()


@pytest.mark.parametrize("artifact", ["json_envelope", "stdout_text", "stdout_and_file"])
def test_missing_submission_is_semantic_failure(tmp_path: Path, artifact: str) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    declared = policy("structural", json_canonical=True) if artifact == "json_envelope" else policy("raw_bytes")
    task = replace(task, scorer=declared, expected_artifact=artifact, expected_stdout="text\n" if artifact == "stdout_and_file" else None)
    (expected / "answer.md").write_bytes(b'{"n":1}' if artifact == "json_envelope" else b"after\n")
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command("from pathlib import Path; Path('input.md').write_bytes(b'after\\n')"), results_dir=root / "receipt")
    assert result.execution.kind == "completed"
    assert result.grade.kind == "fail"
    assert result.execution.reason is None


def emit_events(events: list[dict[str, object]]) -> list[str]:
    encoded = b"".join((json.dumps(event, ensure_ascii=False) + "\n").encode("utf-8") for event in events)
    return python_command(f"import sys; sys.stdout.buffer.write({encoded!r}); sys.stdout.buffer.flush()")


def run_event_fixture(root: Path, events: list[dict[str, object]]) -> harness.AttemptResult:
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, expected_artifact="stdout_text")
    (expected / "answer.md").write_bytes(events[-1].get("result", "after").encode("utf-8"))
    return harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=emit_events(events), results_dir=root / "receipt", event_format="claude_stream")


@pytest.mark.parametrize("denied", [False, True])
def test_permission_denial_precedes_grading(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, denied: bool) -> None:
    calls = []
    original = harness.grade_submission
    def spy(*args: object, **kwargs: object) -> Grade:
        calls.append(kwargs["final_text"])
        return original(*args, **kwargs)
    monkeypatch.setattr(harness, "grade_submission", spy)
    result = run_event_fixture(tmp_path.resolve(), synthetic_cli_events(denied=denied, final_text=" intact\n中文 "))
    if denied:
        assert result.execution == ExecutionOutcome("infrastructure_error", "permission_denied")
        assert result.permission_fault == "permission_denied" and result.grade.kind == "not_run"
        assert calls == []
        assert result.usage.estimated_usd == 0.01
        with pytest.raises(RecordIntegrityError, match="permission_denied"):
            harness.AttemptStore(tmp_path / "receipt").assert_admission()
    else:
        assert result.grade.kind == "pass" and calls == [" intact\n中文 ".encode()]
    assert harness.AttemptStore(tmp_path / "receipt").load()[1] == result


@pytest.mark.parametrize("tool_id", ["call:A", "01", "1", "toolu_local_permission_probe", " Call:中 ", " ", "a", "A"])
@pytest.mark.parametrize("denied", [False, True])
def test_tool_use_ids_are_opaque(tool_id: str, denied: bool) -> None:
    for _ in range(2):  # Reuse in another attempt is independent.
        decoder = harness.ClaudeStreamDecoder()
        events = synthetic_cli_events(tool_id=tool_id, denied=denied)
        for event in events:
            decoder.accept(event)
            decoder.accept(copy.deepcopy(event))
        parsed = decoder.finish()
        assert len([e for e in parsed.events if isinstance(e, ToolCall)]) == 1
        assert len([e for e in parsed.events if isinstance(e, ToolResult)]) == 1
        assert len([e for e in parsed.events if isinstance(e, RunReceipt)]) == 1
        assert parsed.events[0].tool_use_id == tool_id
        assert parsed.events[1].tool_use_id == tool_id
        for normalized in parsed.events:
            assert event_from_dict(record_dict(normalized)) == normalized
        assert parsed.receipt.usage.input_tokens == 3


@pytest.mark.parametrize("target", ["call", "result", "receipt", "reference", "denial_reference", "denial_input"])
def test_conflicting_or_uncorrelated_cli_events_are_integrity_errors(target: str) -> None:
    events = synthetic_cli_events(denied=target.startswith("denial"))
    decoder = harness.ClaudeStreamDecoder()
    if target == "call":
        decoder.accept(events[1])
        events[1]["message"]["content"][0]["input"]["command"] = "different"
        bad = events[1]
    elif target == "result":
        for event in events[:-1]:
            decoder.accept(event)
        events[2]["message"]["content"][0]["content"] = "different"
        bad = events[2]
    elif target == "receipt":
        for event in events:
            decoder.accept(event)
        events[-1]["total_cost_usd"] = 0.02
        bad = events[-1]
    elif target == "reference":
        decoder.accept(events[1])
        events[2]["message"]["content"][0]["tool_use_id"] = "foreign"
        bad = events[2]
    else:
        for event in events[:-1]:
            decoder.accept(event)
        if target == "denial_reference":
            events[-1]["permission_denials"][0]["tool_use_id"] = "foreign"
        else:
            events[-1]["permission_denials"][0]["tool_input"] = {"command": "different"}
        bad = events[-1]
    with pytest.raises(RecordIntegrityError):
        decoder.accept(bad)
    assert decoder.integrity_fault is not None


@pytest.mark.parametrize("metadata", ["missing", None, {}, True, "[]", [None], [{}],
    [{"tool_name": "Bash", "tool_use_id": "", "tool_input": {}}]])
def test_missing_or_malformed_denials_never_grade(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, metadata: object) -> None:
    events = synthetic_cli_events()
    if metadata == "missing":
        del events[-1]["permission_denials"]
    else:
        events[-1]["permission_denials"] = metadata
    calls = []
    monkeypatch.setattr(harness, "grade_submission", lambda *a, **kw: calls.append(kw))
    result = run_event_fixture(tmp_path.resolve(), events)
    assert result.execution == ExecutionOutcome("infrastructure_error", "receipt_integrity")
    assert result.grade.kind == "not_run" and calls == []
    assert not result.evidence_complete
    assert result.usage.estimated_usd == 0.01 and result.usage.completeness == "partial"
    with pytest.raises(RecordIntegrityError, match="receipt_integrity"):
        harness.AttemptStore(tmp_path / "receipt").assert_admission()


def test_recovered_tool_error_is_not_infrastructure_error(tmp_path: Path) -> None:
    result = run_event_fixture(tmp_path.resolve(), synthetic_cli_events(tool_error=True))
    assert result.execution.kind == "completed" and result.grade.kind == "pass"
    assert result.permission_fault is None


def test_earlier_tool_output_cannot_replace_terminal_text(tmp_path: Path) -> None:
    events = synthetic_cli_events(final_text="wrong")
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    task = replace(task, expected_artifact="stdout_text")
    events[2]["message"]["content"][0]["content"] = "after\n"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected, command=emit_events(events),
        results_dir=tmp_path / "receipt", event_format="claude_stream")
    assert result.grade.kind == "fail"
    assert (tmp_path / "receipt/artifacts/final_submission.bin").read_bytes() == b"wrong"


def test_unicode_stream_chunk_boundaries_and_partial_messages() -> None:
    events = synthetic_cli_events(final_text=" 中文\n intact ")
    events.insert(1, {"type": "stream_event", "event": {"type": "content_block_start", "content_block": {"type": "tool_use", "id": "partial"}}})
    encoded = b"".join((json.dumps(e, ensure_ascii=False) + "\n").encode() for e in events)
    decoder = harness.ClaudeStreamDecoder()
    for byte in encoded:
        decoder.feed(bytes([byte]))
    assert decoder.finish().receipt.final_text == " 中文\n intact "
    assert len(decoder.parsed.events) == 3


@pytest.mark.parametrize("subtype,reason,expected_kind,expected_reason", [
    ("error_max_turns", "max_turns", "budget_exhausted", "turn_limit"),
    ("error_max_budget_usd", "max_budget_usd", "budget_exhausted", "cost_limit"),
    ("error_during_execution", "authentication_error", "infrastructure_error", "authentication_error"),
    ("error_during_execution", "transport_error", "infrastructure_error", "transient_transport"),
    ("error_during_execution", "startup_error", "infrastructure_error", "transient_startup"),
])
def test_explicitly_synthetic_unobserved_error_variants(subtype: str, reason: str, expected_kind: str, expected_reason: str) -> None:
    events = synthetic_cli_events()
    events[-1].update(subtype=subtype, terminal_reason=reason, is_error=True)
    decoder = harness.ClaudeStreamDecoder()
    for event in events:
        decoder.accept(event)
    assert decoder.finish().receipt.execution == ExecutionOutcome(expected_kind, expected_reason)


def test_missing_cache_category_remains_unknown() -> None:
    events = synthetic_cli_events()
    del events[-1]["usage"]["cache_read_input_tokens"]
    decoder = harness.ClaudeStreamDecoder()
    for event in events:
        decoder.accept(event)
    receipt = decoder.finish().receipt
    assert receipt.usage.cache_read_tokens is None and receipt.usage.estimated_usd == 0.01
    assert receipt.usage.completeness == "partial"


@pytest.mark.parametrize("actual,expected_reason", [(None, "model_identity_unavailable"), ("wrong-model", "model_mismatch"), ("requested-model", None)])
def test_live_decoder_checks_actual_model_before_grade_eligibility(actual: str | None, expected_reason: str | None) -> None:
    events = synthetic_cli_events()
    events[-1]["modelUsage"] = {actual: {}} if actual is not None else {}
    faults = []
    decoder = harness.ClaudeStreamDecoder(backend="claude_cli", requested_model="requested-model", on_fault=faults.append)
    for event in events:
        decoder.accept(event)
    receipt = decoder.finish().receipt
    if expected_reason:
        assert receipt.execution == ExecutionOutcome("infrastructure_error", expected_reason)
        assert faults == [expected_reason]
    else:
        assert receipt.execution.kind == "completed" and faults == []
    assert receipt.observed_model == actual


def local_test_captured_cli_events_decode(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Explicit local-only selection, no portable synthetic fallback.

Run with --override-ini python_functions=local_test_* and the genuine fixture
    root environment variable. These are actual CLI envelopes with synthetic
    endpoint values; successful decode does not prove real-provider usage.
    """
    source = Path(os.environ["MDTOOLS_U4_CAPTURED_ROOT"])
    for name, denied in (("bash-granted.events.json", False), ("bash-denied.events.json", True)):
        original = (source / name).read_bytes()
        events = json.loads(original)
        root = tmp_path / ("denied" if denied else "granted")
        root.mkdir()
        calls = []
        actual_grader = harness.grade_submission
        def spy(*args: object, **kwargs: object) -> Grade:
            calls.append(kwargs["final_text"])
            return actual_grader(*args, **kwargs)
        with monkeypatch.context() as context:
            context.setattr(harness, "grade_submission", spy)
            result = run_event_fixture(root, events)
        assert result.grade.kind == ("not_run" if denied else "pass")
        assert len(calls) == (0 if denied else 1)
        assert result.usage.input_tokens == 2 and result.usage.output_tokens == 2
        assert result.usage.estimated_usd == events[-1]["total_cost_usd"]
        decoder = harness.ClaudeStreamDecoder()
        for event in events:
            decoder.accept(event)
        parsed = decoder.finish()
        assert parsed.events[0].tool_use_id == parsed.events[1].tool_use_id == "toolu_local_permission_probe"
        assert parsed.receipt.final_text == events[-1]["result"]
        assert (source / name).read_bytes() == original
        if denied:
            denial_index = next(i for i, e in enumerate(events) if e.get("type") == "system" and e.get("subtype") == "permission_denied")
            truncated = events[:denial_index + 1]
            crash_root = tmp_path / "truncated-genuine-copy"
            crash_root.mkdir()
            truncated_result = run_event_fixture(crash_root, truncated)
            assert truncated_result.permission_fault == "permission_denied" and truncated_result.grade.kind == "not_run"
            assert not truncated_result.evidence_complete
            with pytest.raises(RecordIntegrityError, match="permission_denied"):
                harness.AttemptStore(crash_root / "receipt").assert_admission()
