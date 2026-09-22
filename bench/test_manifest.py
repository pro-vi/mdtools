"""H content hashing intent, adapted to validated content identities."""

from dataclasses import replace

import pytest
from bench.manifest import ExperimentSpec, sha256_text
from bench.trial_records import RecordIntegrityError, record_dict


def synthetic_spec() -> ExperimentSpec:
    digest = sha256_text
    return ExperimentSpec("synthetic", digest("task"), digest("prompt"), {"input.md": digest("input")},
        digest("expected"), ("/synthetic/python", "-c", "pass"), digest("binary"), digest("lock"), digest("harness"), digest("grader"),
        answer_policy_sha256=digest("answer"))


def test_condition_and_toolkit_content_bind_synthetic_identity() -> None:
    spec = synthetic_spec()
    bound = replace(spec, condition_sha256=sha256_text("condition"), toolkit_sha256={"cat": sha256_text("cat-bytes")})
    assert spec.identity != bound.identity
    assert bound.task_sha256 == spec.task_sha256
    assert bound.harness_sha256 == spec.harness_sha256
    assert bound.grader_sha256 == spec.grader_sha256
    assert replace(bound, toolkit_sha256={"cat": sha256_text("changed")}).identity != bound.identity


def test_identity_is_content_not_path() -> None:
    spec = synthetic_spec()
    moved = replace(spec, command=("/relocated/python", *spec.command[1:]), locators={"runner": "/relocated/python"})
    spec.assert_same_identity(moved)
    assert ExperimentSpec.from_dict(record_dict(moved)) == moved
    with pytest.raises(RecordIntegrityError, match="changed experiment"):
        spec.assert_same_identity(replace(moved, executable_sha256=sha256_text("changed")))


@pytest.mark.parametrize("field", ["task_sha256", "prompt_sha256", "harness_sha256", "grader_sha256", "expected_sha256", "dependency_lock_sha256", "answer_policy_sha256", "runner_configuration_sha256"])
def test_changed_content_refuses_resume(field: str) -> None:
    spec = synthetic_spec()
    with pytest.raises(RecordIntegrityError, match="changed experiment"):
        spec.assert_same_identity(replace(spec, **{field: sha256_text("different")}))


@pytest.mark.parametrize("updates", [{"schema": "old"}, {"task_sha256": "task"}, {"retry_allowance": True},
    {"limits": {"timeout_seconds": -1, "max_turns": 30}}, {"limits": {"timeout_seconds": 10, "max_turns": True}},
    {"input_sha256": {}}, {"answer_policy_sha256": None}, {"backend": "claude_cli"}])
def test_incomplete_or_malformed_identity_is_rejected(updates: dict[str, object]) -> None:
    with pytest.raises(RecordIntegrityError):
        replace(synthetic_spec(), **updates)


def test_one_model_identity_binds_explicit_thinking_policy_without_required_effort() -> None:
    # Identity mechanics only: no configuration viability or paid admission.
    digest = sha256_text("synthetic-live-configuration-identity")
    haiku = replace(synthetic_spec(), backend="claude_cli", requested_model="claude-haiku-4-5-20251001",
        effort=None, thinking_policy="manual-thinking-policy-explicitly-frozen-by-controller",
        runner_source_sha256=digest, runner_pin_sha256=digest, runner_configuration_sha256=digest,
        launcher_sha256=digest, profile_sha256=digest, condition_sha256=digest)
    assert haiku.effort is None
    assert ExperimentSpec.from_dict(record_dict(haiku)) == haiku
    sonnet = replace(haiku, requested_model="claude-sonnet-5", effort="high", thinking_policy="adaptive-effort")
    assert sonnet.identity != haiku.identity
    assert replace(haiku, thinking_policy="thinking-disabled").identity != haiku.identity
    with pytest.raises(RecordIntegrityError):
        replace(haiku, thinking_policy=None)
