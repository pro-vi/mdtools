"""Exact H task-first/trial-second bootstrap compatibility checks."""
from __future__ import annotations

from dataclasses import replace

import pytest

from bench.stats import hierarchical_bootstrap_ci, MismatchedTaskSetError
from bench.test_agg_util import views


def test_exact_default_seed_reps_and_deterministic_equal_scores() -> None:
    a = views(passes=(True,))
    b = views(condition="legacy", passes=(True,))
    ci = hierarchical_bootstrap_ci(a, b)
    assert (ci.estimate, ci.low, ci.high, ci.seed, ci.reps) == (0, 0, 0, 1729, 10000)
    assert hierarchical_bootstrap_ci(a, b) == ci


def test_bootstrap_extreme_contrasts_and_semantic_failures() -> None:
    a = views(costs=(0, 0), passes=(True, True))
    b = views(condition="legacy", costs=(0, 0), passes=(False, False))
    ci = hierarchical_bootstrap_ci(a, b)
    assert (ci.estimate, ci.low, ci.high) == (1, 1, 1)
    reverse = hierarchical_bootstrap_ci(b, a)
    assert (reverse.estimate, reverse.low, reverse.high) == (-1, -1, -1)


def test_exact_historical_clustered_sampling_golden() -> None:
    # Independently run H c933520 stats.py with these invented outcomes. H
    # produced these exact percentiles; no historical result was consulted.
    a = views(task="A", costs=(0, 0, 0), passes=(True, True, False)) + views(task="B", costs=(0, 0, 0), passes=(True, False, False))
    b = views(task="A", condition="legacy", costs=(0, 0, 0), passes=(False, True, False)) + views(task="B", condition="legacy", costs=(0, 0, 0), passes=(False, False, False))
    ci = hierarchical_bootstrap_ci(a, b)
    assert (ci.estimate, ci.low, ci.high) == (0.3333333333333333, -0.16666666666666666, 0.8333333333333333)


def test_mismatched_task_repetition_and_experiment_reject() -> None:
    a = views(costs=(0, 0), passes=(True, False))
    b = views(condition="legacy")
    with pytest.raises(MismatchedTaskSetError):
        hierarchical_bootstrap_ci(a, b)
    with pytest.raises(MismatchedTaskSetError):
        hierarchical_bootstrap_ci(a, a + a)
    with pytest.raises(MismatchedTaskSetError):
        hierarchical_bootstrap_ci((), ())
    foreign = replace(b[0], trial=("0" * 64, *b[0].trial[1:]))
    with pytest.raises(MismatchedTaskSetError):
        hierarchical_bootstrap_ci(b, (foreign,))


@pytest.mark.parametrize("reps,seed", [(0, 1729), (True, 1729), (100, True), (100, -1)])
def test_invalid_frozen_statistics_reject(reps: int, seed: int) -> None:
    with pytest.raises(ValueError):
        hierarchical_bootstrap_ci(views(), views(condition="legacy"), reps=reps, seed=seed)
