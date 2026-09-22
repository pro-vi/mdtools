"""One disposition-based CLI evaluation report path (selective H recovery)."""
from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import asdict
from pathlib import Path
import sys
from typing import Sequence

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from bench.agg_util import (MEASUREMENTS, TrialView, can_reserve_estimated_usd, estimated_usd_total, intersection_cost,
                            measurement_coverage, pass_at_1_mean)
from bench.manifest import CampaignSpec, canonical_json, sha256_file
from bench.stats import hierarchical_bootstrap_ci
from bench.trial_records import (AttemptStart, AttemptResult, CONDITIONS, RecordIntegrityError,
    AttemptKey, admission_fault, derive_trial_disposition, record_dict)


def trial_views(schedule: Sequence[tuple[str, str, str, int]], starts: Sequence[AttemptStart],
                results: Sequence[AttemptResult], *, campaign: CampaignSpec | None = None,
                retry_allowance: int = 1) -> tuple[TrialView, ...]:
    if any(type(trial) is not tuple or len(trial) != 4 for trial in schedule):
        raise RecordIntegrityError("invalid report schedule")
    for trial in schedule:
        AttemptKey(*trial, 0)
    if len({trial[0] for trial in schedule}) > 1:
        raise RecordIntegrityError("report cannot pool experiments")
    if any(not isinstance(start, AttemptStart) for start in starts) or any(not isinstance(result, AttemptResult) for result in results):
        raise RecordIntegrityError("report requires validated records")
    if len(set(schedule)) != len(schedule):
        raise RecordIntegrityError("duplicate scheduled trial")
    if len({start.key for start in starts}) != len(starts) or len({result.key for result in results}) != len(results):
        raise RecordIntegrityError("duplicate attempt across bundles")
    if any(record.key.trial not in schedule for record in (*starts, *results)):
        raise RecordIntegrityError("foreign/unscheduled attempt")
    views = []
    for trial in schedule:
        trial_starts = [start for start in starts if start.key.trial == trial]
        trial_results = sorted((result for result in results if result.key.trial == trial), key=lambda result: result.key.ordinal)
        allowance = campaign.retry_allowance(trial) if campaign else retry_allowance
        disposition = derive_trial_disposition(trial, starts=trial_starts, results=trial_results, retry_allowance=allowance)
        views.append(TrialView(trial, disposition, tuple(trial_results)))
    return tuple(views)


def summarize(schedule: Sequence[tuple[str, str, str, int]], starts: Sequence[AttemptStart],
              results: Sequence[AttemptResult], *, campaign: CampaignSpec | None = None,
              persisted_faults: Sequence[str] = (), retry_allowance: int = 1) -> dict[str, object]:
    views = trial_views(schedule, starts, results, campaign=campaign, retry_allowance=retry_allowance)
    coverage = measurement_coverage(starts, results)
    faults = sorted(set(persisted_faults) | {fault for result in results if (fault := admission_fault(result))})
    if campaign and campaign.grant_usd is not None and coverage["budget"]["unresolved_attempts"]:
        faults = sorted(set(faults) | {"unknown_estimated_cost"})
    if campaign and any(view.disposition.kind not in ("pending", "succeeded", "task_failed")
                        for view in views[:campaign.prefix_length]):
        faults = sorted(set(faults) | {"incomplete_contract_prefix"})
    if campaign and next(iter(campaign.members.values())).backend == "claude_cli":
        if campaign.reservation_usd is not None and any(result.usage.estimated_usd is not None and
                result.usage.estimated_usd > campaign.reservation_usd for result in results):
            faults = sorted(set(faults) | {"attempt_cost_limit"})
        if campaign.grant_usd is not None and not can_reserve_estimated_usd(float(estimated_usd_total(results)), 0, campaign.grant_usd):
            faults = sorted(set(faults) | {"campaign_cost_limit"})
    terminal = ("succeeded", "task_failed", "operational_failed")
    missing = [list(view.trial[1:]) for view in views if view.disposition.kind not in terminal]
    complete = not missing and not faults
    cells = {}
    result_by_key = {result.key: result for result in results}
    for condition in CONDITIONS:
        cell = [view for view in views if view.trial[2] == condition]
        selected = [result_by_key[view.disposition.selected] for view in cell if view.disposition.selected in result_by_key]
        graded = [result for result in selected if result.grade.kind in ("pass", "fail")]
        measurements = measurement_coverage([start for start in starts if start.key.condition == condition],
            [result for result in results if result.key.condition == condition])
        tokens = [measurements[name]["total"] for name in
                  ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens")]
        cells[condition] = {"requested_trials": len(cell), "successes": sum(view.workflow_success for view in cell),
            "dispositions": dict(Counter(view.disposition.kind for view in cell)),
            "pass_at_1_mean": pass_at_1_mean(cell) if complete else None,
            "graded_trials": len(graded),
            "semantic_pass_rate": sum(result.grade.kind == "pass" for result in graded) / len(graded) if graded else None,
            "coverage": measurements, "total_tokens": sum(tokens) if all(value is not None for value in tokens) else None}
    comparisons = []
    if complete and all(cells[condition]["requested_trials"] for condition in CONDITIONS):
        for a, b in (("legacy", "no-md"), ("current-compact", "no-md"), ("current-compact", "legacy")):
            settings = next(iter(campaign.members.values())).statistical_settings if campaign else {"seed": 1729, "reps": 10000}
            comparisons.append({"condition_a": a, "condition_b": b,
                "workflow_success_difference": asdict(hierarchical_bootstrap_ci(
                    [view for view in views if view.trial[2] == a], [view for view in views if view.trial[2] == b], **settings)),
                "successful_intersection_cost": intersection_cost(views, a, b),
                "successful_intersection_measurements": {name: intersection_cost(views, a, b, measurement=name)
                    for name in MEASUREMENTS}})
    return {"schema": "mdtools.cli-eval/1", "record": "campaign_report", "experiment_id": schedule[0][0] if schedule else None,
        "complete": complete, "exit_code": 0 if complete else 1,
        "comparative_performance_eligible": complete and bool(comparisons),
        "core_study_complete": bool(complete and campaign and campaign.core_study and len(views) == 360 and
            all(next(result for result in view.attempts if result.key == view.disposition.selected).live_study_evidence for view in views)),
        "faults": faults, "missing_or_unresolved_trials": missing, "cells": cells,
        "execution_counts": dict(Counter(result.execution.kind for result in results)),
        "grade_counts": dict(Counter(result.grade.kind for result in results)),
        "coverage": coverage, "comparisons": comparisons,
        "failures": [{"trial": list(view.trial[1:]), "disposition": record_dict(view.disposition),
            "attempts": [{"ordinal": result.key.ordinal, "execution": record_dict(result.execution),
                          "grade": record_dict(result.grade)} for result in view.attempts]} for view in views
                     if view.disposition.kind in ("task_failed", "operational_failed", "ungradeable")],
        "trials": [{"trial": list(view.trial[1:]), "disposition": record_dict(view.disposition)} for view in views],
        "limits": ["Provider-reported estimated USD is not an invoice.",
                   "Synthetic reports do not establish live study or native containment evidence.",
                   "Successful-intersection costs exclude failed trials; total coverage retains every attempt."]}


def attempt_report(starts: Sequence[AttemptStart], results: Sequence[AttemptResult], *, retry_allowance: int = 1) -> dict[str, object]:
    if not starts or not results:
        raise RecordIntegrityError("attempt summary requires records")
    summary = summarize((starts[0].key.trial,), starts, results, retry_allowance=retry_allowance)
    selected = summary["trials"][0]["disposition"]["selected"]
    result = next((result for result in results if record_dict(result.key) == selected), None)
    if result is None:
        raise RecordIntegrityError("selected attempt is unfinished; use the campaign report")
    # Stable single-attempt fields retained for existing offline consumers.
    summary.update({"backend": result.backend, "task_id": result.key.task_id,
        "execution": record_dict(result.execution), "grade": record_dict(result.grade),
        "disposition": summary["trials"][0]["disposition"], "live_study_evidence": result.live_study_evidence})
    return summary


def report_campaign(bundles: Sequence[Path]) -> dict[str, object]:
    from bench.harness import (load_campaign_bundles, load_authorization_evidence, harness_source_identity,
                              _root, _assert_live_prerequisite, AttemptStore, campaign_attempt_name)
    spec, starts, results, faults = load_campaign_bundles(bundles)
    current_sources = harness_source_identity()
    current_grader = sha256_file(Path(__file__).with_name("neutral_scorer.py"))
    mismatches = {name: {"recorded": sorted({getattr(member, name) for member in spec.members.values()}), "current": current}
                  for name, current in (("harness_sha256", current_sources), ("grader_sha256", current_grader))
                  if any(getattr(member, name) != current for member in spec.members.values())}
    if mismatches:
        faults.append("changed_reporting_source")
    metadata = {}
    if next(iter(spec.members.values())).backend == "claude_cli":
        grant, prerequisite_bundle = load_authorization_evidence(_root(bundles[0]), spec)
        for bundle in bundles[1:]:
            if load_authorization_evidence(_root(bundle), spec) != (grant, prerequisite_bundle):
                raise RecordIntegrityError("conflicting authorization evidence across bundles")
        _assert_live_prerequisite(grant, prerequisite_bundle, spec)
        metadata = {"live_phase": grant.phase, "explicit_max_attempts": grant.max_attempts,
                    "authorization_evidence_is_launch_authority": False}
        if grant.phase == "canary" and any(result.grade.kind == "fail" for result in results):
            faults.append("canary_failed")
        if prerequisite_bundle is not None:
            prior, prior_starts, prior_results, _ = load_campaign_bundles((prerequisite_bundle,))
            if prior.identity != grant.prerequisite_experiment_id:
                raise RecordIntegrityError("prerequisite evidence identity mismatch")
            metadata["prerequisite_evidence"] = {"experiment_id": prior.identity,
                "coverage": measurement_coverage(prior_starts, prior_results), "budget_pooled_with_current_campaign": False}
    schedule = tuple((spec.identity, *entry) for entry in spec.schedule)
    summary = summarize(schedule, starts, results, campaign=spec, persisted_faults=faults)
    configuration = next(iter(spec.members.values()))
    summary.update({"backend": configuration.backend, "requested_model": configuration.requested_model,
                    "effort": configuration.effort, "thinking_policy": configuration.thinking_policy})
    summary.update(metadata)
    summary["source_mismatches"] = mismatches
    formats = {condition: {} for condition in CONDITIONS}
    by_name = {campaign_attempt_name(result.key): result for result in results}
    for bundle in bundles:
        for path in (_root(bundle) / "attempts").iterdir():
            if path.name not in by_name:
                continue
            result = by_name[path.name]
            observed = AttemptStore(path).submission_format(result.grade, result.artifacts)
            family = observed.answer_policy["artifact"] if observed is not None else "unavailable"
            verdict = (f"{observed.json_form}:{result.grade.kind}" if observed is not None and observed.json_form is not None
                else result.grade.kind)
            cell = formats[result.key.condition].setdefault(family, {})
            cell[verdict] = cell.get(verdict, 0) + 1
    summary["submission_formats"] = formats
    return summary


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Offline report regeneration; never launches a runner.")
    parser.add_argument("bundles", nargs="+", type=Path)
    args = parser.parse_args(argv)
    try:
        summary = report_campaign(args.bundles)
    except (RecordIntegrityError, OSError, ValueError) as exc:
        print(canonical_json({"complete": False, "exit_code": 2, "comparisons": [], "fault": str(exc)}))
        return 2
    print(canonical_json(summary))
    return summary["exit_code"]


if __name__ == "__main__":
    raise SystemExit(main())
