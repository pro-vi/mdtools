#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
REGIME = ROOT / "bench/regimes/longdoc"
INPUTS = REGIME / "inputs"
EXPECTED = REGIME / "expected"


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def _json(path: Path, payload: Any) -> None:
    _write(path, json.dumps(payload, indent=2, sort_keys=True) + "\n")


def _policy(keys: list[str]) -> dict[str, Any]:
    return {
        "kind": "structural",
        "normalize_line_endings": True,
        "ignore_trailing_whitespace": True,
        "compare_frontmatter_json": False,
        "compare_heading_tree": False,
        "compare_block_order": False,
        "compare_link_destinations": False,
        "compare_block_text": False,
        "json_canonical": True,
        "json_required_keys": keys,
    }


def _task(task_id: str, description: str, input_file: str, expected_file: str, keys: list[str]) -> dict[str, Any]:
    return {
        "id": task_id,
        "description": description,
        "provenance": "exploratory-longdoc-regime",
        "input_files": [input_file],
        "expected_output": expected_file,
        "expected_artifact": "json_envelope",
        "difficulty": "regime",
        "scorer": _policy(keys),
    }


def _count_open(items: list[tuple[str, bool]]) -> int:
    return sum(1 for _label, done in items if not done)


def build_changelog() -> tuple[str, dict[str, Any], dict[str, int]]:
    programs = ["Atlas", "Beacon", "Cinder", "Delta", "Echo", "Flux"]
    quarters = ["Q1", "Q2", "Q3", "Q4"]
    streams = ["Core", "Payments", "Search", "Identity", "Analytics", "Storage"]
    owners = ["Mira", "Noor", "Iris", "Pax", "Remy", "Sol"]
    budgets = [4, 6, 8, 10, 12]
    target = ("Atlas", "Q3", 17)
    expected: dict[str, Any] | None = None

    lines = [
        "# Mega Changelog",
        "",
        "This synthetic release ledger repeats headings, values, and checklists across programs.",
        "Targets are identified by structural position, not by unique inline markers.",
        "",
    ]
    for pi, program in enumerate(programs):
        lines.extend([f"## Program {program}", ""])
        for qi, quarter in enumerate(quarters):
            lines.extend([f"### Quarter {quarter}", ""])
            for window in range(1, 91):
                stream = streams[(window + qi + pi) % len(streams)]
                budget = budgets[(window + qi + pi) % len(budgets)]
                owner = owners[(window * 2 + qi + pi) % len(owners)]
                checks = [
                    ("Confirm dashboard bookmark", (window + qi) % 2 == 0),
                    ("Verify fallback ledger", (window + pi) % 3 == 0),
                    ("Publish customer note", (window + qi + pi) % 4 != 0),
                ]
                lines.extend(
                    [
                        "#### Release Window",
                        "",
                        f"- Stream: {stream}",
                        "- Phase: stabilization",
                        "",
                        "##### Migration Notes",
                        "",
                        f"- Retry budget: {budget} minutes",
                        f"- Rollback owner: {owner}",
                    ]
                )
                for label, done in checks:
                    mark = "x" if done else " "
                    lines.append(f"- [{mark}] {label}")
                lines.extend(
                    [
                        "",
                        "##### Risk Log",
                        "",
                        f"- Severity: {['low', 'medium', 'high'][(window + qi) % 3]}",
                        "- Follow-up: revisit after the next train.",
                        "",
                    ]
                )
                if (program, quarter, window) == target:
                    expected = {
                        "program": program,
                        "quarter": quarter,
                        "window_index": window,
                        "retry_budget_minutes": budget,
                        "rollback_owner": owner,
                        "open_checks": _count_open(checks),
                    }

    assert expected is not None
    text = "\n".join(lines) + "\n"
    counts = {
        "lines": len(text.splitlines()),
        "release_window_headings": text.count("#### Release Window"),
        "migration_notes_headings": text.count("##### Migration Notes"),
        "retry_budget_value": text.count(f"Retry budget: {expected['retry_budget_minutes']} minutes"),
        "rollback_owner_value": text.count(f"Rollback owner: {expected['rollback_owner']}"),
    }
    return text, expected, counts


def build_runbook() -> tuple[str, dict[str, Any], dict[str, int]]:
    regions = ["north", "south", "east", "west", "central", "pacific"]
    services = ["ledger", "catalog", "search", "billing", "auth", "profile", "orders", "metrics"]
    tiers = ["SEV-1", "SEV-2", "SEV-3"]
    backups = ["Iris", "Mira", "Noor", "Pax", "Remy", "Sol"]
    target = ("west", "ledger", 21)
    expected: dict[str, Any] | None = None

    lines = [
        "# Aggregated Operations Runbook",
        "",
        "Each region/service pair repeats the same drill headings and checklist labels.",
        "The target is selected by nested section and ordinal drill position.",
        "",
    ]
    for ri, region in enumerate(regions):
        lines.extend([f"## Region {region}", ""])
        for si, service in enumerate(services):
            lines.extend([f"### Service {service}", ""])
            for drill in range(1, 37):
                tier = tiers[(drill + ri + si) % len(tiers)]
                backup = backups[(drill * 2 + ri + si) % len(backups)]
                rollback_checks = [
                    ("Verify rollback owner", (drill + ri) % 2 == 0),
                    ("Verify rollback owner", (drill + si) % 3 == 0),
                    ("Verify rollback owner", (drill + ri + si) % 5 == 0),
                ]
                lines.extend(
                    [
                        "#### Quarterly Drill",
                        "",
                        "- Scenario: capacity shift",
                        "- Window: maintenance",
                        "",
                        "##### Drill Checklist",
                        "",
                    ]
                )
                for label, done in rollback_checks:
                    mark = "x" if done else " "
                    lines.append(f"- [{mark}] {label}")
                lines.extend(
                    [
                        "- [x] Confirm paging channel",
                        "- [ ] Capture operator note",
                        "",
                        "##### Escalation Matrix",
                        "",
                        "| Tier | Primary | Backup |",
                        "| --- | --- | --- |",
                        f"| {tier} | duty-manager | {backup} |",
                        "",
                    ]
                )
                if (region, service, drill) == target:
                    expected = {
                        "region": region,
                        "service": service,
                        "drill_index": drill,
                        "unchecked_rollback_owner": _count_open(rollback_checks),
                        "escalation_tier": tier,
                        "backup": backup,
                    }

    assert expected is not None
    text = "\n".join(lines) + "\n"
    counts = {
        "lines": len(text.splitlines()),
        "quarterly_drill_headings": text.count("#### Quarterly Drill"),
        "drill_checklist_headings": text.count("##### Drill Checklist"),
        "rollback_label": text.count("Verify rollback owner"),
        "escalation_tier_value": text.count(f"| {expected['escalation_tier']} |"),
        "backup_value": text.count(f"| duty-manager | {expected['backup']} |"),
    }
    return text, expected, counts


def build_spec() -> tuple[str, dict[str, Any], dict[str, int]]:
    statuses = ["required", "conditional", "deferred", "blocked"]
    owners = ["alpha", "beta", "gamma", "delta", "epsilon"]
    patterns = [f"Pattern {label}" for label in "ABCDEFGHIJKL"]
    target = (4, 11, "Pattern B")
    expected: dict[str, Any] | None = None

    lines = [
        "# Book-Length Interop Specification",
        "",
        "Every part/chapter repeats pattern headings, compatibility matrices, and exception blocks.",
        "The target is selected by part, chapter, and repeated pattern name.",
        "",
    ]
    for part in range(1, 6):
        lines.extend([f"## Part {part}", ""])
        for chapter in range(1, 25):
            lines.extend([f"### Chapter {chapter}", ""])
            for pi, pattern in enumerate(patterns):
                boundary_status = statuses[(part + chapter + pi) % len(statuses)]
                audit_status = statuses[(part * 2 + chapter + pi) % len(statuses)]
                owner = owners[(part + chapter + pi) % len(owners)]
                must_count = 1 + ((part + chapter + pi) % 3)
                lines.extend(
                    [
                        f"#### {pattern}",
                        "",
                        "##### Compatibility Matrix",
                        "",
                        "| Requirement | Status | Owner |",
                        "| --- | --- | --- |",
                        f"| Boundary handoff | {boundary_status} | {owner} |",
                        f"| Audit memo retention | {audit_status} | {owner} |",
                        "| Replay ledger | conditional | shared |",
                        "",
                        "##### Exceptions",
                        "",
                    ]
                )
                for idx in range(1, must_count + 1):
                    lines.append(f"- MUST preserve compatibility note {idx}.")
                lines.extend(
                    [
                        "- SHOULD retain downgrade context.",
                        "- MAY defer secondary review.",
                        "",
                    ]
                )
                if (part, chapter, pattern) == target:
                    expected = {
                        "part": part,
                        "chapter": chapter,
                        "pattern": pattern,
                        "boundary_handoff_status": boundary_status,
                        "audit_memo_retention_status": audit_status,
                        "must_exceptions": must_count,
                    }

    assert expected is not None
    text = "\n".join(lines) + "\n"
    counts = {
        "lines": len(text.splitlines()),
        "pattern_b_headings": text.count("#### Pattern B"),
        "compatibility_matrix_headings": text.count("##### Compatibility Matrix"),
        "boundary_status_value": text.count(f"| Boundary handoff | {expected['boundary_handoff_status']} |"),
        "audit_status_value": text.count(f"| Audit memo retention | {expected['audit_memo_retention_status']} |"),
    }
    return text, expected, counts


def build_tasks() -> list[dict[str, Any]]:
    return [
        _task(
            "LD-CHANGELOG-01",
            "In mega_changelog.md, navigate by structure: Program Atlas -> Quarter Q3 -> the 17th `Release Window` subsection, counting only Release Window subsections inside that quarter. From its `Migration Notes`, print JSON with keys program, quarter, window_index, retry_budget_minutes, rollback_owner, open_checks. The same headings, checklist labels, owners, and retry-budget values recur throughout the document.",
            "bench/regimes/longdoc/inputs/mega_changelog.md",
            "bench/regimes/longdoc/expected/LD-CHANGELOG-01.json",
            ["program", "quarter", "window_index", "retry_budget_minutes", "rollback_owner", "open_checks"],
        ),
        _task(
            "LD-RUNBOOK-02",
            "In ops_runbook.md, navigate by structure: Region west -> Service ledger -> the 21st `Quarterly Drill` subsection, counting only Quarterly Drill subsections inside that service. From its Drill Checklist and Escalation Matrix, print JSON with keys region, service, drill_index, unchecked_rollback_owner, escalation_tier, backup. The same region/service/drill headings and rollback labels recur throughout the document.",
            "bench/regimes/longdoc/inputs/ops_runbook.md",
            "bench/regimes/longdoc/expected/LD-RUNBOOK-02.json",
            ["region", "service", "drill_index", "unchecked_rollback_owner", "escalation_tier", "backup"],
        ),
        _task(
            "LD-SPEC-03",
            "In interop_spec.md, navigate by structure: Part 4 -> Chapter 11 -> `Pattern B`. From its Compatibility Matrix and Exceptions section, print JSON with keys part, chapter, pattern, boundary_handoff_status, audit_memo_retention_status, must_exceptions. Pattern B, Compatibility Matrix, and the status values recur throughout the document.",
            "bench/regimes/longdoc/inputs/interop_spec.md",
            "bench/regimes/longdoc/expected/LD-SPEC-03.json",
            ["part", "chapter", "pattern", "boundary_handoff_status", "audit_memo_retention_status", "must_exceptions"],
        ),
    ]


def main() -> None:
    INPUTS.mkdir(parents=True, exist_ok=True)
    EXPECTED.mkdir(parents=True, exist_ok=True)

    changelog, changelog_expected, changelog_counts = build_changelog()
    runbook, runbook_expected, runbook_counts = build_runbook()
    spec, spec_expected, spec_counts = build_spec()

    _write(INPUTS / "mega_changelog.md", changelog)
    _write(INPUTS / "ops_runbook.md", runbook)
    _write(INPUTS / "interop_spec.md", spec)
    _json(EXPECTED / "LD-CHANGELOG-01.json", changelog_expected)
    _json(EXPECTED / "LD-RUNBOOK-02.json", runbook_expected)
    _json(EXPECTED / "LD-SPEC-03.json", spec_expected)

    tasks = build_tasks()
    _json(REGIME / "tasks.json", tasks)
    _json(REGIME / "task_ids.json", [task["id"] for task in tasks])
    _json(
        REGIME / "fixture_validation.json",
        {
            "LD-CHANGELOG-01": changelog_counts,
            "LD-RUNBOOK-02": runbook_counts,
            "LD-SPEC-03": spec_counts,
        },
    )


if __name__ == "__main__":
    main()
