from __future__ import annotations

import math
import unittest

from bench.stats import (
    MismatchedTaskSetError,
    exact_sign_test,
    flip_table,
    hierarchical_bootstrap_ci,
    variance_decomposition,
    wilson_ci,
)


def trial(task: str, passed: bool, run_index: int = 0) -> dict:
    return {"task_id": task, "correct": passed, "run_index": run_index}


class BenchStatsTests(unittest.TestCase):
    def test_exact_sign_test_matches_hand_computed_binomial(self) -> None:
        self.assertAlmostEqual(exact_sign_test(12, 1), 28 / 8192)

    def test_flip_table_reports_majority_and_all_k(self) -> None:
        cell_a = [
            trial("T1", True, 0),
            trial("T1", True, 1),
            trial("T1", False, 2),
            trial("T2", True, 0),
            trial("T2", False, 1),
            trial("T2", False, 2),
        ]
        cell_b = [
            trial("T1", False, 0),
            trial("T1", False, 1),
            trial("T1", True, 2),
            trial("T2", True, 0),
            trial("T2", True, 1),
            trial("T2", True, 2),
        ]
        table = flip_table(cell_a, cell_b)
        self.assertEqual(table.majority_of_k.pass_fail, 1)
        self.assertEqual(table.majority_of_k.fail_pass, 1)
        self.assertEqual(table.all_k.fail_fail, 1)
        self.assertEqual(table.all_k.fail_pass, 1)

    def test_tool_errors_are_scored_not_infra_failures(self) -> None:
        cell_a = [
            {**trial("T1", True, 0), "runner_error": "tool_error: edit recovered"},
            {**trial("T1", True, 1), "runner_error": "tool_error: md unavailable"},
            trial("T1", False, 2),
        ]
        cell_b = [
            trial("T1", False, 0),
            trial("T1", False, 1),
            trial("T1", True, 2),
        ]
        table = flip_table(cell_a, cell_b)
        self.assertEqual(table.majority_of_k.pass_fail, 1)
        self.assertEqual(table.majority_of_k.fail_pass, 0)

    def test_verdict_error_still_counts_as_infra_failure(self) -> None:
        cell_a = [
            {**trial("T1", True, 0), "verdict": "error"},
            {**trial("T1", True, 1), "verdict": "error"},
            trial("T1", False, 2),
        ]
        cell_b = [
            trial("T1", False, 0),
            trial("T1", False, 1),
            trial("T1", True, 2),
        ]
        table = flip_table(cell_a, cell_b)
        self.assertEqual(table.majority_of_k.pass_fail, 0)
        self.assertEqual(table.majority_of_k.fail_fail, 1)

    def test_bootstrap_ci_large_effect_excludes_zero(self) -> None:
        cell_a = [trial(f"T{i}", True, r) for i in range(8) for r in range(5)]
        cell_b = [trial(f"T{i}", False, r) for i in range(8) for r in range(5)]
        ci = hierarchical_bootstrap_ci(cell_a, cell_b, reps=500, seed=7)
        self.assertEqual(ci.estimate, 1.0)
        self.assertGreater(ci.low, 0)
        self.assertEqual(ci.reps, 500)
        self.assertEqual(ci.seed, 7)

    def test_zero_discordant_tasks_do_not_crash(self) -> None:
        cell_a = [trial("T1", True), trial("T2", False)]
        cell_b = [trial("T1", True), trial("T2", False)]
        table = flip_table(cell_a, cell_b)
        self.assertEqual(table.majority_of_k.discordant, 0)
        self.assertEqual(exact_sign_test(0, 0), 1.0)
        ci = hierarchical_bootstrap_ci(cell_a, cell_b, reps=100, seed=1)
        self.assertLessEqual(ci.low, 0)
        self.assertGreaterEqual(ci.high, 0)

    def test_mismatched_task_sets_raise(self) -> None:
        with self.assertRaises(MismatchedTaskSetError):
            flip_table([trial("T1", True)], [trial("T2", True)])
        with self.assertRaises(MismatchedTaskSetError):
            hierarchical_bootstrap_ci([trial("T1", True)], [trial("T2", True)], reps=10)

    def test_wilson_ci_known_symmetric_case(self) -> None:
        interval = wilson_ci(50, 100)
        self.assertEqual(interval.estimate, 0.5)
        self.assertTrue(0.40 < interval.low < 0.41)
        self.assertTrue(0.59 < interval.high < 0.60)
        self.assertEqual(wilson_ci(0, 0).low, 0.0)

    def test_variance_decomposition_reports_both_terms(self) -> None:
        trials = [
            trial("T1", True, 0),
            trial("T1", True, 1),
            trial("T2", True, 0),
            trial("T2", False, 1),
            trial("T3", False, 0),
            trial("T3", False, 1),
        ]
        out = variance_decomposition(trials)
        self.assertEqual(out.n_tasks, 3)
        self.assertEqual(out.mean_k, 2)
        self.assertGreater(out.task_variance_term, 0)
        self.assertGreater(out.trial_variance_term, 0)
        self.assertTrue(math.isclose(out.total, out.task_variance_term + out.trial_variance_term))


if __name__ == "__main__":
    unittest.main()
