"""Pure aggregation selectively recovered from H c933520: per-task rates.

Only validated dispositions choose successes. Nullable measurements retain their
own units; all attempted costs, including retries, remain in totals.
"""
from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from decimal import Decimal
import statistics
from typing import Sequence

from bench.trial_records import AttemptResult, AttemptStart, TrialDisposition

MEASUREMENTS = ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens",
                "estimated_usd", "elapsed_seconds", "tool_output_bytes")


def estimated_usd_total(results: Sequence[AttemptResult]) -> Decimal:
    """Sum the reported decimal amounts without binary-float budget drift."""
    return sum((Decimal(str(result.usage.estimated_usd)) for result in results
                if result.usage.estimated_usd is not None), Decimal(0))


def can_reserve_estimated_usd(known_usd: float, reservation_usd: float, grant_usd: float) -> bool:
    return Decimal(str(known_usd)) + Decimal(str(reservation_usd)) <= Decimal(str(grant_usd))


@dataclass(frozen=True)
class TrialView:
    trial: tuple[str, str, str, int]
    disposition: TrialDisposition
    attempts: tuple[AttemptResult, ...]

    @property
    def workflow_success(self) -> bool:
        return self.disposition.workflow_success


def pass_at_1_mean(trials: Sequence[TrialView]) -> float | None:
    by_task: dict[str, list[TrialView]] = defaultdict(list)
    for trial in trials:
        by_task[trial.trial[1]].append(trial)
    if not by_task:
        return None
    return sum(sum(trial.workflow_success for trial in cell) / len(cell)
               for cell in by_task.values()) / len(by_task)


def measurement_coverage(starts: Sequence[AttemptStart], results: Sequence[AttemptResult]) -> dict[str, object]:
    quantities = {}
    for name in MEASUREMENTS:
        known = [getattr(result.usage, name) for result in results if getattr(result.usage, name) is not None]
        total = float(estimated_usd_total(results)) if name == "estimated_usd" else sum(known)
        quantities[name] = {"known_attempts": len(known), "attempts": len(starts),
            "known_total": total, "total": total if len(known) == len(starts) else None}
    by_key = {result.key: result for result in results}
    unresolved = [start for start in starts if start.key not in by_key or by_key[start.key].usage.estimated_usd is None]
    reservations = [start.reservation_usd for start in unresolved]
    known_usd = estimated_usd_total(results)
    reserved_usd = sum((Decimal(str(r)) for r in reservations if r is not None), Decimal(0))
    quantities["budget"] = {"known_estimated_usd": float(known_usd),
        "unresolved_attempts": len(unresolved), "unresolved_reservation_usd": float(reserved_usd),
        "conservative_exposure_usd": float(known_usd + reserved_usd) if all(r is not None for r in reservations) else None}
    return quantities


def intersection_cost(trials: Sequence[TrialView], condition_a: str, condition_b: str,
                      *, measurement: str = "estimated_usd") -> dict[str, object]:
    if measurement not in MEASUREMENTS:
        raise ValueError("unknown measurement unit")
    by_condition = {condition: {(trial.trial[1], trial.trial[3]): trial for trial in trials if trial.trial[2] == condition}
                    for condition in (condition_a, condition_b)}
    a, b = by_condition[condition_a], by_condition[condition_b]
    shared = sorted(key for key in a.keys() & b.keys() if a[key].workflow_success and b[key].workflow_success)
    values = []
    for key in shared:
        costs = []
        for trial in (a[key], b[key]):
            measured = [getattr(attempt.usage, measurement) for attempt in trial.attempts]
            total = float(estimated_usd_total(trial.attempts)) if measurement == "estimated_usd" else sum(value for value in measured if value is not None)
            costs.append(total if measured and all(value is not None for value in measured) else None)
        values.append(costs)
    complete = bool(values) and all(value is not None for pair in values for value in pair)
    median_a = statistics.median(pair[0] for pair in values) if complete else None
    median_b = statistics.median(pair[1] for pair in values) if complete else None
    return {"condition_a": condition_a, "condition_b": condition_b, "measurement": measurement,
        "shared_successful_trials": len(shared), "measured_pairs": sum(all(value is not None for value in pair) for pair in values),
        "median_a": median_a, "median_b": median_b,
        "delta": median_a - median_b if complete else None}
