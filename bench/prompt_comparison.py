"""Paired public prompt diagnostics over two ordinary campaign controllers."""
from __future__ import annotations

from dataclasses import replace
import fcntl
import os
import uuid
from pathlib import Path
from typing import Mapping

from bench import harness
from bench.agg_util import MEASUREMENTS, TrialView
from bench.manifest import CampaignConfig, LiveRunGrant, PromptComparisonSpec, canonical_json
from bench.report import report_campaign, summarize, trial_views
from bench.trial_records import (AttemptResult, RecordIntegrityError, ToolCall, ToolResult,
    decode_record_json, record_dict)


def comparison_configs(config: CampaignConfig) -> dict[str, CampaignConfig]:
    if set(config.task_ids) != {"T1", "T2", "T10"} or config.repetitions != 1 or config.retry_allowance != 0:
        raise RecordIntegrityError("comparison config requires T1/T2/T10 once with zero retries")
    return {profile: replace(config, guidance=profile, output_root=str(Path(config.output_root) / profile))
            for profile in ("full_help", "discovery")}


def prepare_comparison(config: CampaignConfig) -> PromptComparisonSpec:
    children = {profile: harness.CampaignSpec.from_dict(harness.configured_campaign(child, prepare_only=True)["spec"])
                for profile, child in comparison_configs(config).items()}
    spec = PromptComparisonSpec.create(children, seed=config.seed)
    root = harness._root(Path(config.output_root))
    if (root / "comparison.json").exists():
        if load_comparison(root).identity != spec.identity:
            raise RecordIntegrityError("changed comparison proposal")
    else:
        harness._write_private(root / "comparison.json", (canonical_json(record_dict(spec)) + "\n").encode())
    return spec


def load_comparison(root: Path) -> PromptComparisonSpec:
    return PromptComparisonSpec.from_dict(decode_record_json(harness._read_source(root, "comparison.json")))


def _observations(spec: PromptComparisonSpec, root: Path) -> tuple[
        dict[str, dict[str, object]], dict[str, tuple[TrialView, ...]], set[tuple[str, str, str, int]]]:
    reports, trials, observed = {}, {}, set()
    for profile, child in spec.campaigns.items():
        directory = root / profile / "campaign"
        if directory.exists():
            loaded, starts, results, _ = harness.load_campaign_bundles((directory,))
            if loaded.identity != child.identity:
                raise RecordIntegrityError("foreign comparison child")
            reports[profile] = report_campaign((directory,))
        else:
            starts, results = [], []
            reports[profile] = summarize(tuple((child.identity, *entry) for entry in child.schedule), [], [], campaign=child)
        observed.update((profile, start.key.task_id, start.key.condition, start.key.repetition) for start in starts)
        trials[profile] = trial_views(tuple((child.identity, *entry) for entry in child.schedule), starts, results, campaign=child)
    if observed != set(spec.schedule[:len(observed)]):
        raise RecordIntegrityError("child records contradict paired schedule prefix")
    return reports, trials, observed


def _measurements(result: AttemptResult | None, directory: Path) -> dict[str, int | float | None]:
    if result is None:
        return {name: None for name in (*MEASUREMENTS, "total_tokens", "tool_calls", "tool_errors")}
    quantities = {name: getattr(result.usage, name) for name in MEASUREMENTS}
    components = [quantities[name] for name in ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens")]
    quantities["total_tokens"] = sum(components) if all(value is not None for value in components) else None
    decoder = harness.ClaudeStreamDecoder(backend=result.backend, requested_model=result.requested_model)
    try:
        decoder.feed(harness._read_source(directory / "attempts" / harness.campaign_attempt_name(result.key), "events.jsonl"))
        parsed = decoder.finish(require_receipt=False)
    except RecordIntegrityError:
        parsed = decoder.parsed
    quantities["tool_calls"] = sum(isinstance(event, ToolCall) for event in parsed.events)
    quantities["tool_errors"] = sum(isinstance(event, ToolResult) and event.is_error for event in parsed.events)
    return quantities


def report_comparison(root: Path) -> dict[str, object]:
    root = harness._root(root)
    spec = load_comparison(root)
    reports, trials, _ = _observations(spec, root)
    current = harness.harness_source_identity()
    stale = any(member.harness_sha256 != current for child in spec.campaigns.values() for member in child.members.values())
    complete = not stale and all(report["complete"] for report in reports.values())
    rows = []
    for index, entry in enumerate(spec.campaigns["full_help"].schedule):
        pair = {"trial": list(entry), "profiles": {}}
        for profile in spec.campaigns:
            view = trials[profile][index]
            result = next(iter(view.attempts), None)
            pair["profiles"][profile] = {"success": view.workflow_success,
                "disposition": record_dict(view.disposition),
                "execution": record_dict(result.execution) if result else None,
                "grade": record_dict(result.grade) if result else None,
                "measurements": _measurements(result, root / profile / "campaign")}
        rows.append(pair)
    profiles = {}
    for profile in spec.campaigns:
        measures = [row["profiles"][profile]["measurements"] for row in rows]
        totals = {name: {"known_trials": sum(row[name] is not None for row in measures),
                        "known_total": sum(row[name] for row in measures if row[name] is not None),
                        "total": sum(row[name] for row in measures) if all(row[name] is not None for row in measures) else None}
                  for name in measures[0]}
        profiles[profile] = {"planned_trials": len(rows), "successes": sum(row["profiles"][profile]["success"] for row in rows),
            "measurements": totals, "campaign_report": reports[profile]}
    discordance = {"discovery_only": 0, "full_help_only": 0, "both": 0, "neither": 0}
    for row in rows:
        control, treatment = (row["profiles"][name]["success"] for name in ("full_help", "discovery"))
        discordance["both" if control and treatment else "full_help_only" if control else "discovery_only" if treatment else "neither"] += 1
    intersection = [row for row in rows if all(cell["success"] for cell in row["profiles"].values())]
    costs = {}
    for name in profiles["full_help"]["measurements"]:
        differences = [row["profiles"]["discovery"]["measurements"][name] - row["profiles"]["full_help"]["measurements"][name]
            for row in intersection if all(row["profiles"][profile]["measurements"][name] is not None for profile in spec.campaigns)]
        costs[name] = {"measured_pairs": len(differences), "sum_difference": sum(differences) if differences else None}
    cells = [{"task": task, "condition": condition, "planned_pairs": 1,
              "full_help_successes": int(row["profiles"]["full_help"]["success"]),
              "discovery_successes": int(row["profiles"]["discovery"]["success"])}
             for row in rows for task, condition, _ in [row["trial"]]]
    return {"comparison_id": spec.identity, "analysis": spec.analysis, "complete": complete,
        "exit_code": 0 if complete else 1, "planned_pairs": len(rows), "profiles": profiles, "pairs": rows,
        "cells": cells, "paired_success_discordance": discordance if complete else None,
        "success_difference": (profiles["discovery"]["successes"] - profiles["full_help"]["successes"]) / len(rows) if complete else None,
        "successful_intersection": {"pairs": len(intersection), "costs": costs} if complete else None,
        "faults": ["changed_reporting_source"] if stale else [],
        "limits": ["Three public tasks and one repetition are diagnostic, not evidence of general superiority.",
                   "All planned trials remain in the denominator; missing outcomes are unresolved, not scored failures.",
                   "Estimated USD is a nominal token-price equivalent. Actual billing is unverified.",
                   "Discovery is not adopted automatically; no tuning or extra trials are authorized."]}


def run_comparison(config: CampaignConfig, *, grants: Mapping[str, LiveRunGrant] | None = None,
                   prerequisite_bundle: Path | None = None, stop_after: int | None = None) -> dict[str, object]:
    configs = comparison_configs(config)
    root = harness._root(Path(config.output_root))
    spec = load_comparison(root)
    if stop_after is not None and (type(stop_after) is not int or stop_after < 0):
        raise RecordIntegrityError("invalid comparison stop count")
    live = config.claude_executable is not None and config.endpoint is None
    if live:
        if grants is None or set(grants) != set(configs):
            raise RecordIntegrityError("comparison requires explicit grants for both profiles")
        for profile, grant in grants.items():
            if grant.phase != "prompt_comparison":
                raise RecordIntegrityError("wrong comparison grant phase")
            grant.assert_scope(spec.campaigns[profile])
            harness._assert_live_prerequisite(grant, prerequisite_bundle, spec.campaigns[profile])
    elif grants is not None or prerequisite_bundle is not None:
        raise RecordIntegrityError("synthetic comparison cannot consume live consent")
    current_campaigns = {}
    for profile, child in configs.items():
        current, _, _, _ = harness.prepare_campaign_config(child)
        current_campaigns[profile] = current
    if PromptComparisonSpec.create(current_campaigns, seed=config.seed).identity != spec.identity:
        raise RecordIntegrityError("changed comparison configuration")
    lock_path = root / ".comparison.lock"
    harness._assert_no_symlinks(lock_path)
    with os.fdopen(os.open(lock_path, os.O_CREAT | os.O_RDWR, 0o600), "r+b") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        launched, hold = 0, None
        while True:
            reports, _, observed = _observations(spec, root)
            decisions = []
            for profile in configs:
                directory = root / profile / "campaign"
                if directory.exists():
                    child, starts, results, faults = harness.load_campaign_bundles((directory,))
                    decisions.append(harness.campaign_admission(child, starts, results, faults,
                        live_grant=grants[profile] if grants else None))
            if any(report["faults"] for report in reports.values()) or any(action.kind == "hold" for action in decisions):
                hold = "child_campaign_unresolved"
                break
            if len(observed) == len(spec.schedule):
                break
            if stop_after is not None and launched >= stop_after:
                hold = "operator_stop"
                break
            profile = spec.schedule[len(observed)][0]
            summary = harness.configured_campaign(configs[profile], stop_after=1,
                live_grant=grants[profile] if grants else None, prerequisite_bundle=prerequisite_bundle)
            if summary.get("admission_hold") not in (None, "operator_stop"):
                hold = summary["admission_hold"]
                break
            launched += 1
        report = report_comparison(root)
        if hold:
            report.update(admission_hold=hold, exit_code=1)
        temporary = root / (".comparison-report-" + uuid.uuid4().hex + ".json")
        # Derived reports are replaceable; frozen specs and child receipts are not.
        harness._write_private(temporary, (canonical_json(report) + "\n").encode())
        os.replace(temporary, root / "comparison-report.json")
        harness._fsync_directory(root)
        return report
