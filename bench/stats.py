#!/usr/bin/env python3
"""Pure-stdlib statistics helpers for bench v3."""

from __future__ import annotations

import math
import random
from collections import defaultdict
from dataclasses import dataclass
from typing import Any


class MismatchedTaskSetError(ValueError):
    """Raised when paired statistics receive cells with different task IDs."""


@dataclass(frozen=True)
class Interval:
    estimate: float
    low: float
    high: float


@dataclass(frozen=True)
class BootstrapCI:
    estimate: float
    low: float
    high: float
    reps: int
    seed: int


@dataclass(frozen=True)
class FlipCounts:
    pass_pass: int
    pass_fail: int
    fail_pass: int
    fail_fail: int

    @property
    def discordant(self) -> int:
        return self.pass_fail + self.fail_pass

    @property
    def n(self) -> int:
        return self.pass_pass + self.pass_fail + self.fail_pass + self.fail_fail


@dataclass(frozen=True)
class FlipTable:
    majority_of_k: FlipCounts
    all_k: FlipCounts


@dataclass(frozen=True)
class VarianceDecomposition:
    n_tasks: int
    mean_k: float
    task_variance_term: float
    trial_variance_term: float

    @property
    def total(self) -> float:
        return self.task_variance_term + self.trial_variance_term


def _task_id(rec: dict[str, Any]) -> str:
    return str(rec.get("task_id") or rec.get("task"))


def _passed(rec: dict[str, Any]) -> bool:
    if rec.get("runner_error") or rec.get("verdict") == "error":
        return False
    if "correct" in rec:
        return bool(rec["correct"])
    return bool(rec.get("pass"))


def _by_task(cell: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for trial in cell:
        grouped[_task_id(trial)].append(trial)
    return dict(grouped)


def _require_same_tasks(
    cell_a: list[dict[str, Any]],
    cell_b: list[dict[str, Any]],
) -> tuple[dict[str, list[dict[str, Any]]], dict[str, list[dict[str, Any]]], list[str]]:
    by_a = _by_task(cell_a)
    by_b = _by_task(cell_b)
    tasks_a = set(by_a)
    tasks_b = set(by_b)
    if tasks_a != tasks_b:
        raise MismatchedTaskSetError(
            "paired statistic requires identical task sets: "
            f"only_a={sorted(tasks_a - tasks_b)}, only_b={sorted(tasks_b - tasks_a)}"
        )
    if not tasks_a:
        raise ValueError("paired statistic requires at least one task")
    return by_a, by_b, sorted(tasks_a)


def _collapse_majority(trials: list[dict[str, Any]]) -> bool:
    return sum(1 for trial in trials if _passed(trial)) * 2 > len(trials)


def _collapse_all_k(trials: list[dict[str, Any]]) -> bool:
    return bool(trials) and all(_passed(trial) for trial in trials)


def _counts_for(
    by_a: dict[str, list[dict[str, Any]]],
    by_b: dict[str, list[dict[str, Any]]],
    task_ids: list[str],
    collapse,
) -> FlipCounts:
    pass_pass = pass_fail = fail_pass = fail_fail = 0
    for task_id in task_ids:
        a = collapse(by_a[task_id])
        b = collapse(by_b[task_id])
        if a and b:
            pass_pass += 1
        elif a and not b:
            pass_fail += 1
        elif not a and b:
            fail_pass += 1
        else:
            fail_fail += 1
    return FlipCounts(
        pass_pass=pass_pass,
        pass_fail=pass_fail,
        fail_pass=fail_pass,
        fail_fail=fail_fail,
    )


def flip_table(cell_a: list[dict[str, Any]], cell_b: list[dict[str, Any]]) -> FlipTable:
    """Return paired task flips after collapsing k trials.

    Validity: only use on paired cells with identical task sets. The majority-of-k
    collapse is suitable for exact collapsed-task tests; the all-k collapse reports
    reliability but is stricter than pass@1 and should not be treated as per-trial
    evidence.
    """
    by_a, by_b, task_ids = _require_same_tasks(cell_a, cell_b)
    return FlipTable(
        majority_of_k=_counts_for(by_a, by_b, task_ids, _collapse_majority),
        all_k=_counts_for(by_a, by_b, task_ids, _collapse_all_k),
    )


def exact_sign_test(n01: int, n10: int) -> float:
    """Two-sided exact binomial sign/McNemar p-value for discordant task counts.

    Validity: use only after each task has been collapsed to one binary outcome.
    It is not valid for raw per-trial pass@1 records because repeated trials are
    clustered within task.
    """
    if n01 < 0 or n10 < 0:
        raise ValueError("discordant counts must be non-negative")
    n = n01 + n10
    if n == 0:
        return 1.0
    tail = sum(math.comb(n, i) for i in range(0, min(n01, n10) + 1)) / (2 ** n)
    return min(1.0, 2 * tail)


def _task_rate(trials: list[dict[str, Any]]) -> float:
    return sum(1 for trial in trials if _passed(trial)) / len(trials)


def hierarchical_bootstrap_ci(
    cell_a: list[dict[str, Any]],
    cell_b: list[dict[str, Any]],
    *,
    reps: int = 10_000,
    seed: int = 1729,
) -> BootstrapCI:
    """Clustered bootstrap percentile CI for pass@1-mean difference A minus B.

    Validity: use on paired cells with identical task sets. The bootstrap resamples
    tasks first, then trials within each sampled task, preserving task-level
    clustering that raw per-trial tests would ignore.
    """
    if reps <= 0:
        raise ValueError("reps must be positive")
    by_a, by_b, task_ids = _require_same_tasks(cell_a, cell_b)
    rng = random.Random(seed)
    estimate = sum(_task_rate(by_a[t]) - _task_rate(by_b[t]) for t in task_ids) / len(task_ids)
    diffs: list[float] = []
    for _ in range(reps):
        sampled_diffs = []
        for _task_slot in task_ids:
            task_id = rng.choice(task_ids)
            a_trials = by_a[task_id]
            b_trials = by_b[task_id]
            a_rate = sum(1 for _ in a_trials if _passed(rng.choice(a_trials))) / len(a_trials)
            b_rate = sum(1 for _ in b_trials if _passed(rng.choice(b_trials))) / len(b_trials)
            sampled_diffs.append(a_rate - b_rate)
        diffs.append(sum(sampled_diffs) / len(sampled_diffs))
    diffs.sort()
    low = diffs[int(0.025 * (reps - 1))]
    high = diffs[int(0.975 * (reps - 1))]
    return BootstrapCI(estimate=estimate, low=low, high=high, reps=reps, seed=seed)


def wilson_ci(successes: int, n: int, *, z: float = 1.959963984540054) -> Interval:
    """Wilson score interval for one binomial rate.

    Validity: use for descriptive single-cell rates. It does not account for paired
    task comparisons or repeated-trial clustering across tasks.
    """
    if successes < 0 or n < 0 or successes > n:
        raise ValueError("successes and n must satisfy 0 <= successes <= n")
    if n == 0:
        return Interval(estimate=0.0, low=0.0, high=0.0)
    phat = successes / n
    denom = 1 + z * z / n
    center = (phat + z * z / (2 * n)) / denom
    margin = z * math.sqrt((phat * (1 - phat) + z * z / (4 * n)) / n) / denom
    return Interval(estimate=phat, low=max(0.0, center - margin), high=min(1.0, center + margin))


def variance_decomposition(trials: list[dict[str, Any]]) -> VarianceDecomposition:
    """Approximate task-vs-trial contribution to variance of the mean pass rate.

    Validity: use as a planning diagnostic for whether more budget should buy more
    tasks or more trials. It assumes task-level pass probabilities are estimated by
    observed trial fractions and is not a substitute for the paired bootstrap CI.
    """
    by_task = _by_task(trials)
    if not by_task:
        raise ValueError("variance decomposition requires at least one task")
    rates = [_task_rate(task_trials) for task_trials in by_task.values()]
    n_tasks = len(rates)
    mean_rate = sum(rates) / n_tasks
    task_var = (
        sum((rate - mean_rate) ** 2 for rate in rates) / (n_tasks - 1)
        if n_tasks > 1
        else 0.0
    )
    task_term = task_var / n_tasks
    trial_term = sum(
        rate * (1 - rate) / len(task_trials)
        for rate, task_trials in zip(rates, by_task.values())
    ) / (n_tasks * n_tasks)
    mean_k = sum(len(task_trials) for task_trials in by_task.values()) / n_tasks
    return VarianceDecomposition(
        n_tasks=n_tasks,
        mean_k=mean_k,
        task_variance_term=task_term,
        trial_variance_term=trial_term,
    )
