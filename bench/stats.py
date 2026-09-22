"""H c933520 hierarchical bootstrap, adapted to validated TrialView records.

The task-first/trial-second sampling, percentile indices, seed1729 and 10000
replicates match H. Historical boolean adapters and vote collapse are excluded.
"""
from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
import random
from typing import Sequence

from bench.agg_util import TrialView


class MismatchedTaskSetError(ValueError):
    """Paired cells must have identical task/repetition membership."""


@dataclass(frozen=True)
class BootstrapCI:
    estimate: float
    low: float
    high: float
    reps: int
    seed: int


def hierarchical_bootstrap_ci(cell_a: Sequence[TrialView], cell_b: Sequence[TrialView],
                              *, reps: int = 10000, seed: int = 1729) -> BootstrapCI:
    if type(reps) is not int or reps <= 0 or type(seed) is not int or seed < 0:
        raise ValueError("invalid bootstrap settings")
    if len({trial.trial[0] for trial in (*cell_a, *cell_b)}) != 1:
        raise MismatchedTaskSetError("paired statistic cannot pool experiments")
    by_a: dict[str, list[TrialView]] = defaultdict(list)
    by_b: dict[str, list[TrialView]] = defaultdict(list)
    for cell, grouped in ((cell_a, by_a), (cell_b, by_b)):
        keys = [(trial.trial[1], trial.trial[3]) for trial in cell]
        if len(keys) != len(set(keys)):
            raise MismatchedTaskSetError("duplicate task/repetition in paired cell")
        for trial in cell:
            if trial.disposition.kind not in ("succeeded", "task_failed", "operational_failed"):
                raise ValueError("paired statistics require final dispositions")
            grouped[trial.trial[1]].append(trial)
    if not by_a or {task: {trial.trial[3] for trial in trials} for task, trials in by_a.items()} != {
            task: {trial.trial[3] for trial in trials} for task, trials in by_b.items()}:
        raise MismatchedTaskSetError("paired statistic requires identical task/repetition sets")
    task_ids = sorted(by_a)
    rng = random.Random(seed)
    estimate = sum(sum(trial.workflow_success for trial in by_a[task]) / len(by_a[task]) -
                   sum(trial.workflow_success for trial in by_b[task]) / len(by_b[task])
                   for task in task_ids) / len(task_ids)
    diffs = []
    for _ in range(reps):
        sampled_diffs = []
        for _task_slot in task_ids:
            task_id = rng.choice(task_ids)
            a_trials, b_trials = by_a[task_id], by_b[task_id]
            a_rate = sum(rng.choice(a_trials).workflow_success for _ in a_trials) / len(a_trials)
            b_rate = sum(rng.choice(b_trials).workflow_success for _ in b_trials) / len(b_trials)
            sampled_diffs.append(a_rate - b_rate)
        diffs.append(sum(sampled_diffs) / len(sampled_diffs))
    diffs.sort()
    return BootstrapCI(estimate, diffs[int(0.025 * (reps - 1))], diffs[int(0.975 * (reps - 1))], reps, seed)
