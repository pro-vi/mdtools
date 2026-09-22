"""H aggregation intent adapted to synthetic validated dispositions."""
from __future__ import annotations

from dataclasses import replace

import pytest

from bench.agg_util import intersection_cost, measurement_coverage, pass_at_1_mean
from bench.report import trial_views
from bench.test_trial_records import result
from bench.trial_records import AttemptStart, ExecutionOutcome, Grade, Usage, RecordIntegrityError


def views(*, condition: str = "no-md", task: str = "synthetic", costs: tuple = (0,), passes: tuple = (True,)) -> tuple:
    results = [replace(result(), key=replace(result().key, condition=condition, task_id=task, repetition=n),
        grade=Grade("pass" if passed else "fail", "synthetic"),
        usage=Usage(estimated_usd=cost, source="synthetic_receipt" if cost is not None else "unavailable",
                    completeness="partial" if cost is not None else "unknown")) for n, (cost, passed) in enumerate(zip(costs, passes))]
    starts = [AttemptStart(attempt.key, attempt.backend) for attempt in results]
    return trial_views(tuple(attempt.key.trial for attempt in results), starts, results)


def test_per_task_rate_keeps_repetitions_without_vote_collapse() -> None:
    trials = views(costs=(0, 0, 0), passes=(True, True, False)) + views(task="synthetic-other", passes=(False,))
    assert pass_at_1_mean(trials) == pytest.approx(1 / 3)
    assert pass_at_1_mean(()) is None


def test_zero_measurement_and_missing_category_have_distinct_coverage() -> None:
    attempt = replace(result(), usage=Usage(estimated_usd=0, input_tokens=0, source="synthetic_receipt", completeness="partial"))
    coverage = measurement_coverage((AttemptStart(attempt.key, attempt.backend),), (attempt,))
    assert coverage["estimated_usd"] == {"known_attempts": 1, "attempts": 1, "known_total": 0, "total": 0}
    assert coverage["output_tokens"]["total"] is None and coverage["output_tokens"]["known_attempts"] == 0
    assert coverage["budget"]["unresolved_attempts"] == 0


def test_intersection_keeps_repetitions_zero_and_units() -> None:
    a = views(costs=(0, 0.4), passes=(True, False))
    b = views(condition="legacy", costs=(0, 0.1), passes=(True, True))
    cost = intersection_cost(a + b, "no-md", "legacy")
    assert cost["shared_successful_trials"] == 1 and cost["delta"] == 0
    missing = intersection_cost(a + views(condition="legacy", costs=(None,), passes=(True,)), "no-md", "legacy")
    assert missing["median_a"] is None and missing["median_b"] is None and missing["measurement"] == "estimated_usd"
    empty = intersection_cost(views(passes=(False,)) + b, "no-md", "legacy")
    assert empty["shared_successful_trials"] == 0 and empty["delta"] is None
    with pytest.raises(ValueError, match="unit"):
        intersection_cost(a + b, "no-md", "legacy", measurement="calls-as-usd")


def test_raw_historical_shapes_are_not_report_authority() -> None:
    with pytest.raises(RecordIntegrityError, match="validated"):
        trial_views((result().key.trial,), (), ({"synthetic": True},))


def test_equal_decimal_retry_costs_do_not_invent_intersection_savings() -> None:
    first = replace(result(), execution=ExecutionOutcome("infrastructure_error", "transient_transport"),
        grade=Grade("not_run", "transient_transport"), usage=Usage(estimated_usd=0.1, source="synthetic_receipt", completeness="partial"))
    second = replace(result(), key=replace(first.key, ordinal=1), usage=Usage(estimated_usd=0.2, source="synthetic_receipt", completeness="partial"))
    retry = trial_views((first.key.trial,), (AttemptStart(first.key, first.backend), AttemptStart(second.key, second.backend)), (first, second))
    baseline = views(condition="legacy", costs=(0.3,))
    cost = intersection_cost(retry + baseline, "no-md", "legacy")
    assert cost["median_a"] == cost["median_b"] == 0.3 and cost["delta"] == 0
