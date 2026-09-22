"""Synthetic record/shape tests; these are not observed provider error variants."""

from __future__ import annotations

from dataclasses import replace
import json

import pytest

from bench.manifest import sha256_text
from bench.trial_records import (AttemptKey, AttemptStart, AttemptResult, ExecutionOutcome,
    Grade, Usage, TrialDisposition, RecordIntegrityError, admission_fault, decode_record_json,
    derive_trial_disposition, event_from_dict, permission_fault, record_dict)


def synthetic_cli_events(*, tool_id: str = "call:A", denied: bool = False,
                         final_text: str = "after", tool_error: bool = False) -> list[dict[str, object]]:
    """Invented values in CLI envelope shapes, declared synthetic provenance."""
    tool_input = {"command": "echo synthetic", "description": "synthetic-only"}
    events = [{"type": "system", "subtype": "init", "provenance": "synthetic-shape/1"},
        {"type": "assistant", "message": {"content": [{"type": "tool_use", "id": tool_id,
            "name": "Bash", "input": tool_input}]}}]
    if denied:
        events.append({"type": "system", "subtype": "permission_denied", "tool_use_id": tool_id,
                       "tool_name": "Bash", "decision_reason_type": "mode"})
    events.append({"type": "user", "message": {"content": [{"type": "tool_result",
        "tool_use_id": tool_id, "content": "synthetic tool output denied", "is_error": denied or tool_error}]}})
    events.append({"type": "assistant", "message": {"content": [{"type": "text", "text": "earlier text"}]}})
    events.append({"type": "result", "subtype": "success", "terminal_reason": "completed",
        "is_error": False, "result_index": 0, "num_turns": 2, "result": final_text,
        "duration_ms": 12, "usage": {"input_tokens": 3, "output_tokens": 4,
            "cache_read_input_tokens": 0, "cache_creation_input_tokens": 0},
        "modelUsage": {"synthetic-model": {"inputTokens": 3, "outputTokens": 4}},
        "total_cost_usd": 0.01, "permission_denials": [{"tool_name": "Bash", "tool_use_id": tool_id,
            "tool_input": tool_input}] if denied else []})
    return events


def key(ordinal: int = 0) -> AttemptKey:
    return AttemptKey(sha256_text("experiment"), "synthetic", "no-md", 0, ordinal)


def result(*, execution: ExecutionOutcome = ExecutionOutcome("completed"),
           grade: Grade | None = None, ordinal: int = 0, usage: Usage = Usage(),
           permission_reason: str | None = None, complete: bool = True) -> AttemptResult:
    return AttemptResult(key(ordinal), "synthetic", execution,
        grade or (Grade("pass", "synthetic_match") if execution.kind == "completed" else Grade("not_run", execution.reason)),
        usage, {path: sha256_text(path) for path in ("artifacts/stdout.bin", "artifacts/stderr.bin", "events.jsonl", "artifacts/final_submission.bin")},
        complete, sha256_text("manifest"), permission_reason)


def disposition(*results: AttemptResult, retry_allowance: int = 1) -> TrialDisposition:
    return derive_trial_disposition(key().trial, starts=[AttemptStart(r.key, r.backend) for r in results],
                                    results=results, retry_allowance=retry_allowance)


def test_fresh_record_round_trips_and_rejects_historical_booleans() -> None:
    original = result()
    encoded = json.dumps(record_dict(original))
    assert AttemptResult.from_dict(decode_record_json(encoded)) == original
    assert TrialDisposition.from_dict(record_dict(disposition(original))).workflow_success
    with pytest.raises(RecordIntegrityError):
        AttemptResult.from_dict({"correct": True, "runner_error": None})


@pytest.mark.parametrize("updates", [{"schema": "unknown/1"}, {"record": "historical"}, {"evidence_complete": "true"},
    {"exit_code": True}, {"backend": "pi"}, {"artifact_manifest_sha256": "bad"},
    {"artifacts": {"../sibling": "0" * 64}}, {"artifacts": {"file": "bad"}},
    {"permission_fault": "looks denied"}, {"artifacts": {}}])
def test_malformed_result_fields_fail_deterministically(updates: dict[str, object]) -> None:
    with pytest.raises(RecordIntegrityError):
        replace(result(), **updates)


@pytest.mark.parametrize("updates", [{"repetition": True}, {"repetition": "0"}, {"ordinal": -1},
    {"ordinal": 0.0}, {"experiment_id": "bad"}, {"condition": "hybrid"}])
def test_bool_as_index_and_unknown_keys_are_rejected(updates: dict[str, object]) -> None:
    with pytest.raises(RecordIntegrityError):
        replace(key(), **updates)


@pytest.mark.parametrize("kind,reason", [("timeout", "deadline"), ("completed", "error"),
    ("infrastructure_error", "unknown"), ("budget_exhausted", "tokens"), ("interrupted", None)])
def test_execution_variants_are_closed(kind: str, reason: str | None) -> None:
    with pytest.raises(RecordIntegrityError):
        ExecutionOutcome(kind, reason)


@pytest.mark.parametrize("grade", [Grade("pass", "partial"), Grade("fail", "partial"), Grade("unavailable", "capture")])
def test_infrastructure_failure_has_no_semantic_grade(grade: Grade) -> None:
    with pytest.raises(RecordIntegrityError):
        result(execution=ExecutionOutcome("timed_out", "deadline"), grade=grade)


def test_complete_pass_composition() -> None:
    assert disposition(result()).kind == "succeeded"


def test_completed_failure_keeps_cost() -> None:
    failed = result(grade=Grade("fail", "wrong"), usage=Usage(estimated_usd=0.3, source="synthetic_receipt", completeness="partial"))
    assert disposition(failed).kind == "task_failed"
    assert failed.usage.estimated_usd == 0.3


def test_grader_outage_is_not_semantic_failure() -> None:
    ungraded = result(grade=Grade("unavailable", "capture_unavailable"), complete=False)
    assert disposition(ungraded).kind == "ungradeable"
    assert admission_fault(ungraded) == "capture_unavailable"


def test_missing_cost_does_not_erase_grade() -> None:
    assert result().usage.estimated_usd is None
    assert disposition(result()).workflow_success


def test_retry_composition_keeps_all_costs() -> None:
    first = result(execution=ExecutionOutcome("infrastructure_error", "transient_transport"), usage=Usage(estimated_usd=0.2, source="synthetic_receipt", completeness="partial"))
    second = result(ordinal=1, grade=Grade("fail", "wrong"), usage=Usage(estimated_usd=0.3, source="synthetic_receipt", completeness="partial"))
    selected = disposition(first, second)
    assert selected.kind == "task_failed" and selected.selected == second.key
    assert first.usage.estimated_usd + second.usage.estimated_usd == 0.5


@pytest.mark.parametrize("execution", [ExecutionOutcome("timed_out", "deadline"), ExecutionOutcome("interrupted", "operator_interrupt"),
    ExecutionOutcome("budget_exhausted", "turn_limit"), ExecutionOutcome("budget_exhausted", "cost_limit"),
    ExecutionOutcome("infrastructure_error", "authentication_error"), ExecutionOutcome("infrastructure_error", "configuration_error"),
    ExecutionOutcome("infrastructure_error", "receipt_integrity"), ExecutionOutcome("infrastructure_error", "trace_integrity")])
def test_nonretryable_outcomes_never_relaunch(execution: ExecutionOutcome) -> None:
    assert disposition(result(execution=execution)).kind == "operational_failed"


def test_retry_policy_neighbors() -> None:
    transient = result(execution=ExecutionOutcome("infrastructure_error", "transient_transport"))
    assert disposition(transient).next_ordinal == 1
    assert disposition(transient, retry_allowance=0).kind == "operational_failed"
    assert disposition(transient, replace(transient, key=key(1))).kind == "operational_failed"
    assert disposition(result(grade=Grade("fail", "wrong"))).next_ordinal is None
    with pytest.raises(RecordIntegrityError):
        disposition(result(), result(ordinal=1, grade=Grade("fail", "later_bad")))
    with pytest.raises(RecordIntegrityError):
        disposition(transient, result(ordinal=1), retry_allowance=0)


def test_crash_resume_keeps_unfinished_attempt() -> None:
    start = AttemptStart(key(), "synthetic")
    selected = derive_trial_disposition(key().trial, starts=[start])
    assert selected.kind == "running" and selected.next_ordinal is None
    with pytest.raises(RecordIntegrityError):
        derive_trial_disposition(key().trial, starts=[start, AttemptStart(key(1), "synthetic")])


def test_skipped_trial_never_launches() -> None:
    skipped = derive_trial_disposition(key().trial, skipped_reason="operator_unselected")
    assert skipped.kind == "skipped" and not skipped.workflow_success and skipped.next_ordinal is None
    with pytest.raises(RecordIntegrityError):
        derive_trial_disposition(key().trial, starts=[AttemptStart(key(), "synthetic")], skipped_reason="unselected")


def test_duplicate_missing_or_foreign_records_are_rejected() -> None:
    original = result()
    with pytest.raises(RecordIntegrityError):
        disposition(original, original)
    with pytest.raises(RecordIntegrityError):
        derive_trial_disposition(key().trial, results=[original])
    with pytest.raises(RecordIntegrityError):
        derive_trial_disposition(key().trial, starts=[AttemptStart(replace(key(), task_id="foreign"), "synthetic")])


@pytest.mark.parametrize("field", ["input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens", "estimated_usd", "elapsed_seconds", "tool_output_bytes"])
@pytest.mark.parametrize("invalid", [True, -1, float("nan"), float("inf"), "1"])
def test_usage_rejects_invalid_quantities(field: str, invalid: object) -> None:
    with pytest.raises(RecordIntegrityError):
        Usage(**{field: invalid}, source="synthetic_receipt", completeness="partial")


def test_usage_missingness_neighbors() -> None:
    zero = Usage(0, 0, 0, 0, 0, 0, 0, "synthetic_receipt", "complete")
    assert zero.estimated_usd == 0 and zero.input_tokens == 0
    partial = replace(zero, cache_read_tokens=None, completeness="partial")
    assert partial.cache_read_tokens is None and partial.estimated_usd == 0
    with pytest.raises(RecordIntegrityError):
        replace(partial, completeness="complete")
    with pytest.raises(RecordIntegrityError):
        Usage(estimated_usd=0)


@pytest.mark.parametrize("reason", ["turn_limit", "cost_limit"])
def test_accounted_live_limit_is_study_evidence_without_semantic_grade(reason: str) -> None:
    limited = replace(result(execution=ExecutionOutcome("budget_exhausted", reason)),
        backend="claude_cli", requested_model="synthetic-model", observed_model="synthetic-model",
        usage=Usage(1, 1, 0, 0, 0.01, 1, 0, "claude_terminal", "complete"))
    assert limited.live_study_evidence
    assert limited.grade.kind == "not_run"
    assert not replace(limited, evidence_complete=False).live_study_evidence
    assert not replace(limited, observed_model=None).live_study_evidence
    assert not replace(limited, usage=Usage()).live_study_evidence
    assert not replace(limited, execution=ExecutionOutcome("infrastructure_error", "transient_transport"),
        grade=Grade("not_run", "transient_transport")).live_study_evidence


def test_synthetic_records_never_qualify_as_live_evidence() -> None:
    assert not result().live_study_evidence
    with pytest.raises(RecordIntegrityError):
        replace(result(), usage=Usage(estimated_usd=0, source="claude_terminal", completeness="partial"))


def test_success_receipt_with_denial_is_not_task_failure() -> None:
    denied = result(execution=ExecutionOutcome("infrastructure_error", "permission_denied"), permission_reason="permission_denied")
    assert disposition(denied).kind == "operational_failed"
    assert admission_fault(denied) == "permission_denied"
    assert disposition(denied).next_ordinal is None


def test_permission_fault_survives_primary_timeout_or_interruption() -> None:
    for execution in (ExecutionOutcome("timed_out", "deadline"), ExecutionOutcome("interrupted", "operator_interrupt")):
        denied = result(execution=execution, permission_reason="permission_denied")
        assert denied.execution == execution
        assert disposition(denied).reason == "permission_denied"


@pytest.mark.parametrize("denials", [None, {}, "[]", True, [None], [{}], [{"tool_name": "Bash", "tool_use_id": "", "tool_input": {}}]])
def test_required_permission_metadata_fails_closed(denials: object) -> None:
    with pytest.raises(RecordIntegrityError):
        permission_fault({"permission_denials": denials}, terminal=True)


def test_record_definitions_are_deeply_immutable() -> None:
    original = result()
    with pytest.raises(TypeError):
        original.artifacts["new"] = "0" * 64
    parsed = record_dict(original)
    parsed["artifacts"]["new"] = "0" * 64
    assert "new" not in original.artifacts


@pytest.mark.parametrize("payload", ['{"a":1,"a":2}', '{"a":NaN}', '{"a":Infinity}', '{"a":1e999}', '{"a":'])
def test_invalid_record_json_fails_closed(payload: str) -> None:
    with pytest.raises(RecordIntegrityError):
        decode_record_json(payload)


def test_unknown_normalized_event_or_disposition_is_rejected() -> None:
    with pytest.raises(RecordIntegrityError):
        event_from_dict({"kind": "permission_denied"})
    with pytest.raises(RecordIntegrityError):
        TrialDisposition("best_grade", None, None, None)


@pytest.mark.parametrize("actual", [None, "wrong-model"])
@pytest.mark.parametrize("kind", ["pass", "fail"])
def test_live_semantic_grade_requires_actual_requested_model(actual: str | None, kind: str) -> None:
    with pytest.raises(RecordIntegrityError, match="model"):
        replace(result(grade=Grade(kind, "synthetic_grade")), backend="claude_cli", requested_model="requested-model",
                observed_model=actual, usage=Usage(estimated_usd=0, source="claude_terminal", completeness="partial"))


def test_provider_failure_without_observed_model_keeps_original_classification() -> None:
    failed = replace(result(execution=ExecutionOutcome("infrastructure_error", "transient_transport")),
                     backend="claude_cli", requested_model="requested-model", observed_model=None)
    assert admission_fault(failed) is None
    selected = derive_trial_disposition(failed.key.trial,
        starts=[AttemptStart(failed.key, "claude_cli", 0.1, "requested-model")], results=[failed])
    assert selected.kind == "retry_pending"


@pytest.mark.parametrize("actual,reason", [(None, "model_identity_unavailable"), ("wrong-model", "model_mismatch")])
def test_ungradeable_model_identity_fault_blocks_following_admission(actual: str | None, reason: str) -> None:
    invalid_model = replace(result(grade=Grade("unavailable", "model_not_gradeable")), backend="claude_cli",
                            requested_model="requested-model", observed_model=actual)
    assert admission_fault(invalid_model) == reason
    assert not invalid_model.live_study_evidence
