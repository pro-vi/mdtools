"""One campaign policy shared by orchestration and the guarded executor."""
from __future__ import annotations

from dataclasses import replace

import pytest

from bench import harness
from bench.test_report_inputs import campaign_case
from bench.test_trial_records import result as result_shape
from bench.trial_records import AttemptKey, AttemptStart, ExecutionOutcome, Grade, RecordIntegrityError


def test_admission_requires_coherent_variant_fields() -> None:
    with pytest.raises(RecordIntegrityError):
        harness.CampaignAdmission("launch")
    with pytest.raises(RecordIntegrityError):
        harness.CampaignAdmission("finished", reason="unexpected")
    with pytest.raises(RecordIntegrityError):
        harness.CampaignAdmission("hold")


@pytest.mark.parametrize("state,kind,reason", [
    ("empty", "launch", None), ("unfinished", "hold", "unfinished_attempt"),
    ("fault", "hold", "permission_denied"), ("limited", "launch", None),
    ("complete", "finished", None), ("operator", "hold", "operator_stop"),
])
def test_campaign_admission_states(campaign_case: tuple, state: str, kind: str, reason: str | None) -> None:
    _, _, spec, _ = campaign_case
    starts, results, faults = [], [], []
    if state in ("unfinished", "limited", "complete"):
        for entry in (spec.schedule if state == "complete" else spec.schedule[:1]):
            key = AttemptKey(spec.identity, *entry, 0)
            starts.append(AttemptStart(key, "synthetic"))
            if state != "unfinished":
                result = replace(result_shape(), key=key)
                if state == "limited":
                    result = replace(result, execution=ExecutionOutcome("budget_exhausted", "turn_limit"),
                        grade=Grade("not_run", "turn_limit"))
                results.append(result)
    if state == "fault":
        faults.append("permission_denied")
    action = harness.campaign_admission(spec, starts, results, faults, operator_stop=state == "operator")
    assert (action.kind, action.reason) == (kind, reason)
    if kind == "launch":
        assert action.key == AttemptKey(spec.identity, *spec.schedule[int(state == "limited")], 0)


def test_contract_prefix_and_ordinary_limit_are_distinct(campaign_case: tuple) -> None:
    _, _, spec, _ = campaign_case
    key = AttemptKey(spec.identity, *spec.schedule[0], 0)
    starts = [AttemptStart(key, "synthetic")]
    limited = replace(result_shape(execution=ExecutionOutcome("budget_exhausted", "turn_limit")), key=key)
    assert harness.campaign_admission(spec, starts, [limited], []).kind == "launch"
    prefix = replace(spec, prefix_length=6)
    prefix_key = replace(key, experiment_id=prefix.identity)
    assert harness.campaign_admission(prefix, [replace(starts[0], key=prefix_key)],
        [replace(limited, key=prefix_key)], []).reason == "incomplete_contract_prefix"


def test_final_unknown_cost_cannot_finish_budgeted_campaign(campaign_case: tuple) -> None:
    _, _, original, _ = campaign_case
    spec = replace(original, grant_usd=1.0, reservation_usd=0.01)
    starts, results = [], []
    for entry in spec.schedule:
        key = AttemptKey(spec.identity, *entry, 0)
        starts.append(AttemptStart(key, "synthetic", reservation_usd=0.01))
        results.append(replace(result_shape(), key=key))
    results[-1] = replace(results[-1], usage=replace(results[-1].usage,
        estimated_usd=None, completeness="partial"))
    action = harness.campaign_admission(spec, starts, results, [])
    assert (action.kind, action.reason) == ("hold", "unknown_estimated_cost")
