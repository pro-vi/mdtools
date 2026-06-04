"""Tests for the native-adversary gate (ramp stage 1) in auto_research.py.

The native-adversary is the load-bearing md-edge gate: a candidate is only a real
md-edge candidate once native Edit (claude-cli `native` mode, no md) has been shown
to STRUGGLE on it. These tests cover the two pure decision functions without any
network or harness subprocess — `_native_adversary_verdict` (results → verdict) and
`_resolve_status` (verdict → candidate status).
"""
from __future__ import annotations

import unittest

from bench.auto_research import _native_adversary_verdict, _resolve_status


def _run(task_id: str, correct: bool, *, tokens_in: int = 100, tokens_out: int = 20,
         turns: int = 2, tool_calls: int = 3, mutations: int = 1) -> dict:
    return {
        "task_id": task_id, "correct": correct,
        "tokens_in": tokens_in, "tokens_out": tokens_out,
        "turns": turns, "tool_calls": tool_calls, "mutations": mutations,
    }


class NativeAdversaryVerdictTests(unittest.TestCase):
    def test_native_solves_all_pass(self) -> None:
        results = [_run("C-AR-010", True) for _ in range(3)]
        v = _native_adversary_verdict(results, "C-AR-010")
        self.assertEqual(v["verdict"], "native-solves")
        self.assertTrue(v["native_pass"])
        self.assertEqual(v["native_pass_rate"], 1.0)
        # native Edit solves it correctly → no md correctness lift → recommend reject
        self.assertTrue(v["recommend_reject_no_md_edge"])
        self.assertEqual(v["runs"], 3)

    def test_native_struggles_all_fail(self) -> None:
        results = [_run("C-AR-010", False) for _ in range(3)]
        v = _native_adversary_verdict(results, "C-AR-010")
        self.assertEqual(v["verdict"], "native-struggles-correctness")
        self.assertFalse(v["native_pass"])
        self.assertEqual(v["native_pass_rate"], 0.0)
        # native Edit fails → real md-edge candidate → do NOT reject
        self.assertFalse(v["recommend_reject_no_md_edge"])

    def test_majority_pass_counts_as_native_solves(self) -> None:
        results = [_run("C-AR-010", True), _run("C-AR-010", True), _run("C-AR-010", False)]
        v = _native_adversary_verdict(results, "C-AR-010")
        self.assertTrue(v["native_pass"])
        self.assertEqual(v["verdict"], "native-solves")

    def test_minority_pass_counts_as_struggles(self) -> None:
        results = [_run("C-AR-010", True), _run("C-AR-010", False), _run("C-AR-010", False)]
        v = _native_adversary_verdict(results, "C-AR-010")
        self.assertFalse(v["native_pass"])
        self.assertEqual(v["verdict"], "native-struggles-correctness")

    def test_exactly_half_passes_is_majority(self) -> None:
        # pass_rate >= 0.5 is the gate; 1/2 == 0.5 counts as native solving
        results = [_run("C-AR-010", True), _run("C-AR-010", False)]
        v = _native_adversary_verdict(results, "C-AR-010")
        self.assertTrue(v["native_pass"])

    def test_no_results_is_inconclusive(self) -> None:
        v = _native_adversary_verdict([], "C-AR-010")
        self.assertEqual(v["verdict"], "no-native-results")
        self.assertFalse(v["native_pass"])
        self.assertFalse(v["recommend_reject_no_md_edge"])
        self.assertEqual(v["runs"], 0)

    def test_only_matching_task_id_counted(self) -> None:
        results = [_run("C-AR-010", False), _run("OTHER", True), _run("OTHER", True)]
        v = _native_adversary_verdict(results, "C-AR-010")
        self.assertEqual(v["runs"], 1)
        self.assertFalse(v["native_pass"])

    def test_cost_medians_recorded(self) -> None:
        results = [
            _run("C-AR-010", False, tokens_in=100, tokens_out=10, turns=4),
            _run("C-AR-010", False, tokens_in=200, tokens_out=20, turns=6),
            _run("C-AR-010", False, tokens_in=300, tokens_out=30, turns=8),
        ]
        v = _native_adversary_verdict(results, "C-AR-010")
        # median of per-run (in+out) totals: 110, 220, 330 → 220
        self.assertEqual(v["median_tokens"], 220)
        self.assertEqual(v["median_turns"], 6)


# A candidate that has cleared every unix-pipeline gate (realism yes, hybrid pass,
# unix fail, AST-structural, positive gap) — the native-adversary is the last gate.
_PROMOTABLE = dict(
    realism_ok=True, hybrid_pass=True, unix_pass=False,
    ast_structural=True, gap_label="AST-structural", gap_pp=100.0,
)


class ResolveStatusTests(unittest.TestCase):
    def test_unix_gates_still_short_circuit(self) -> None:
        self.assertEqual(
            _resolve_status(**{**_PROMOTABLE, "realism_ok": False}, native_adversary=None),
            "rejected-planning")
        self.assertEqual(
            _resolve_status(**{**_PROMOTABLE, "hybrid_pass": False}, native_adversary=None),
            "rejected-hybrid-fail-no-gap")
        self.assertEqual(
            _resolve_status(**{**_PROMOTABLE, "unix_pass": True}, native_adversary=None),
            "rejected-both-pass-no-gap")
        self.assertEqual(
            _resolve_status(**{**_PROMOTABLE, "ast_structural": False, "gap_label": "shell-quoting"},
                            native_adversary=None),
            "rejected-shell-quoting")
        self.assertEqual(
            _resolve_status(**{**_PROMOTABLE, "gap_pp": 0.0}, native_adversary=None),
            "rejected-both-fail-no-gap")

    def test_no_native_gate_is_pending_native_adversary(self) -> None:
        # back-compat strengthening: the old unconditional pending-cross-seed is now
        # promotion-pending the native gate when the adversary did not run
        self.assertEqual(
            _resolve_status(**_PROMOTABLE, native_adversary=None),
            "pending-native-adversary")
        self.assertEqual(
            _resolve_status(**_PROMOTABLE, native_adversary={"verdict": "not-run"}),
            "pending-native-adversary")
        self.assertEqual(
            _resolve_status(**_PROMOTABLE,
                            native_adversary={"verdict": "no-native-results",
                                              "recommend_reject_no_md_edge": False}),
            "pending-native-adversary")

    def test_native_solves_rejects_no_md_edge(self) -> None:
        na = {"verdict": "native-solves", "recommend_reject_no_md_edge": True}
        self.assertEqual(_resolve_status(**_PROMOTABLE, native_adversary=na),
                         "rejected-no-md-edge")

    def test_native_struggles_clears_to_pending_cross_seed(self) -> None:
        na = {"verdict": "native-struggles-correctness", "recommend_reject_no_md_edge": False}
        self.assertEqual(_resolve_status(**_PROMOTABLE, native_adversary=na),
                         "pending-cross-seed")

    def test_native_gate_only_applies_after_unix_gates(self) -> None:
        # even a native-struggles verdict cannot rescue a unix-rejected candidate
        na = {"verdict": "native-struggles-correctness", "recommend_reject_no_md_edge": False}
        self.assertEqual(
            _resolve_status(**{**_PROMOTABLE, "unix_pass": True}, native_adversary=na),
            "rejected-both-pass-no-gap")


if __name__ == "__main__":
    unittest.main()
