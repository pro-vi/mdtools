from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from tempfile import NamedTemporaryFile, TemporaryDirectory
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
MANIFEST_PATH = SCRIPT_DIR / "cases.json"
PROBE_SCHEMA_VERSION = "target-state-etag-probe.v1"
CANONICAL_DESCRIPTOR_SCHEMA_VERSION = "target-state-etag-descriptor.v1"
CASE_ID_PATTERN = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
EXPECTED_CANDIDATES = (
    "content_only",
    "target_local",
    "ambiguity_reject",
    "document_target_state",
)
EXPECTED_IDENTITY_TRUTHS = ("same_target", "wrong_target")
EXPECTED_CREDITS = ("correct", "wrong_identity", "false-conflict")
EXPECTED_CASE_CLASSES = (
    "duplicate_cross_target_copy",
    "exact_byte_reversion",
    "same_locator_duplicate_shift",
    "unchanged_crlf_bytes",
    "unchanged_multibyte_utf8_bytes",
    "unchanged_reread",
    "unchanged_section_descriptor",
    "unchanged_table_descriptor",
    "unchanged_task_descriptor",
    "unrelated_edit_after_unchanged_target",
)
EXPECTED_CASE_IDS = (
    "block-unchanged-reread",
    "block-duplicate-cross-target-copy",
    "block-same-locator-duplicate-shift",
    "block-unrelated-edit-false-conflict",
    "block-exact-byte-reversion",
    "block-unchanged-crlf-bytes",
    "block-unchanged-multibyte-utf8-bytes",
    "section-unchanged-real-descriptor",
    "table-unchanged-real-descriptor",
    "task-unchanged-real-descriptor",
)


class ProbeError(RuntimeError):
    pass


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the local target-state etag experiment against a supplied md binary."
    )
    parser.add_argument(
        "--md-binary",
        type=Path,
        required=True,
        help="Path to the md binary used for read-command JSON authority.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Write canonical JSON to this path by atomic same-directory replacement.",
    )
    parser.add_argument(
        "--check",
        type=Path,
        help="Byte-compare canonical JSON against an existing file without rewriting it.",
    )
    args = parser.parse_args(argv)
    if args.output is not None and args.check is not None:
        parser.error("--output and --check are mutually exclusive")
    return args


def main(argv: list[str] | None = None) -> int:
    try:
        args = parse_args(argv)
        report_bytes = build_report_bytes(args.md_binary)
        if args.check is not None:
            verify_check_file(args.check, report_bytes)
        elif args.output is not None:
            atomic_write(args.output, report_bytes)
        else:
            sys.stdout.buffer.write(report_bytes)
        return 0
    except ProbeError as exc:
        print(str(exc), file=sys.stderr)
        return 1


def build_report_bytes(md_binary_arg: Path) -> bytes:
    md_binary = ensure_md_binary(md_binary_arg)
    manifest = load_manifest()
    with TemporaryDirectory(prefix="target-state-etag-") as workspace_str:
        workspace = Path(workspace_str)
        case_reports = [
            evaluate_case(case, manifest["query_shapes"], md_binary, workspace)
            for case in manifest["cases"]
        ]
    candidate_summary = build_candidate_summary(case_reports)
    report = {
        "case_count": len(case_reports),
        "candidate_names": list(EXPECTED_CANDIDATES),
        "candidate_summary": candidate_summary,
        "cases": case_reports,
        "descriptor_schema_version": CANONICAL_DESCRIPTOR_SCHEMA_VERSION,
        "manifest_path": "cases.json",
        "manifest_schema_version": manifest["schema_version"],
        "overall_graduation_verdict": build_overall_graduation_verdict(candidate_summary),
        "schema_version": PROBE_SCHEMA_VERSION,
    }
    return canonical_json_bytes(report)


def ensure_md_binary(md_binary_arg: Path) -> Path:
    try:
        md_binary = md_binary_arg.resolve(strict=True)
    except FileNotFoundError as exc:
        raise ProbeError(f"md binary not found: {md_binary_arg}") from exc
    except OSError as exc:
        raise ProbeError(f"failed to resolve md binary: {md_binary_arg}") from exc
    if not md_binary.is_file():
        raise ProbeError(f"md binary is not a file: {md_binary_arg}")
    return md_binary


def load_manifest() -> dict[str, Any]:
    try:
        raw = MANIFEST_PATH.read_text(encoding="utf-8")
    except OSError as exc:
        raise ProbeError(f"failed to read manifest: {MANIFEST_PATH.name}") from exc
    try:
        manifest = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ProbeError(f"invalid manifest JSON in {MANIFEST_PATH.name}: {exc}") from exc
    manifest_map = expect_mapping(manifest, "manifest")
    schema_version = expect_string(manifest_map.get("schema_version"), "manifest.schema_version")
    candidates = expect_list(manifest_map.get("candidates"), "manifest.candidates")
    if [candidate_name(entry, index) for index, entry in enumerate(candidates)] != list(
        EXPECTED_CANDIDATES
    ):
        raise ProbeError("manifest candidates must be the exact four probe candidates in order")
    query_shapes = expect_mapping(manifest_map.get("query_shapes"), "manifest.query_shapes")
    required_case_ids = expect_list(
        manifest_map.get("required_case_ids"),
        "manifest.required_case_ids",
    )
    required_case_id_values = tuple(
        expect_string(value, f"manifest.required_case_ids[{index}]")
        for index, value in enumerate(required_case_ids)
    )
    if required_case_id_values != EXPECTED_CASE_IDS:
        raise ProbeError(
            "manifest.required_case_ids must match the runner-owned case matrix exactly in protocol order"
        )
    cases = expect_list(manifest_map.get("cases"), "manifest.cases")
    normalized_cases = []
    seen_case_ids: set[str] = set()
    for index, case_value in enumerate(cases):
        case = validate_case(case_value, index, query_shapes)
        case_id = case["case_id"]
        if case_id in seen_case_ids:
            raise ProbeError(f"duplicate manifest case_id: {case_id}")
        seen_case_ids.add(case_id)
        normalized_cases.append(case)
    normalized_case_ids = tuple(case["case_id"] for case in normalized_cases)
    if normalized_case_ids != EXPECTED_CASE_IDS:
        raise ProbeError(
            "manifest.cases case_id order must match the runner-owned case matrix exactly in protocol order"
        )
    return {
        "cases": normalized_cases,
        "query_shapes": query_shapes,
        "schema_version": schema_version,
    }


def candidate_name(candidate_value: Any, index: int) -> str:
    candidate = expect_mapping(candidate_value, f"manifest.candidates[{index}]")
    name = expect_string(candidate.get("name"), f"manifest.candidates[{index}].name")
    expect_string(candidate.get("framing"), f"manifest.candidates[{index}].framing")
    return name


def validate_case(
    case_value: Any,
    index: int,
    query_shapes: dict[str, Any],
) -> dict[str, Any]:
    case = expect_mapping(case_value, f"manifest.cases[{index}]")
    case_id = expect_string(case.get("case_id"), f"manifest.cases[{index}].case_id")
    if CASE_ID_PATTERN.fullmatch(case_id) is None:
        raise ProbeError(f"invalid manifest case_id: {case_id}")
    surface = expect_string(case.get("surface"), f"{case_id}.surface")
    if surface not in {"block", "section", "table", "task"}:
        raise ProbeError(f"{case_id}: unsupported surface {surface!r}")
    expected = validate_expected(case_id, case.get("expected"))
    observed_document_utf8 = expect_string(
        case.get("observed_document_utf8"),
        f"{case_id}.observed_document_utf8",
    )
    current_document_utf8 = expect_string(
        case.get("current_document_utf8"),
        f"{case_id}.current_document_utf8",
    )
    observed_target_query = validate_query(
        case_id,
        "observed_target_query",
        case.get("observed_target_query"),
        surface,
        query_shapes,
    )
    current_target_query = validate_query(
        case_id,
        "current_target_query",
        case.get("current_target_query"),
        surface,
        query_shapes,
    )
    current_domain_query = validate_query(
        case_id,
        "current_domain_query",
        case.get("current_domain_query"),
        surface,
        query_shapes,
    )
    normalized = {
        "case_class": expect_choice(
            case.get("case_class"),
            f"{case_id}.case_class",
            EXPECTED_CASE_CLASSES,
        ),
        "case_id": case_id,
        "current_document_utf8": current_document_utf8,
        "current_domain_query": current_domain_query,
        "current_target_query": current_target_query,
        "expected": expected,
        "identity_truth": expect_choice(
            case.get("identity_truth"),
            f"{case_id}.identity_truth",
            EXPECTED_IDENTITY_TRUTHS,
        ),
        "observed_document_utf8": observed_document_utf8,
        "observed_target_query": observed_target_query,
        "surface": surface,
    }
    if "same_locator_preconditions" in case:
        normalized["same_locator_preconditions"] = validate_same_locator(case_id, case)
    if "notes" in case:
        normalized["notes"] = expect_string(case.get("notes"), f"{case_id}.notes")
    return normalized


def validate_query(
    case_id: str,
    field_name: str,
    query_value: Any,
    case_surface: str,
    query_shapes: dict[str, Any],
) -> dict[str, Any]:
    query = expect_mapping(query_value, f"{case_id}.{field_name}")
    query_type = expect_string(query.get("type"), f"{case_id}.{field_name}.type")
    shape = expect_mapping(
        query_shapes.get(query_type),
        f"manifest.query_shapes.{query_type}",
    )
    expected_surface = expect_string(shape.get("surface"), f"manifest.query_shapes.{query_type}.surface")
    if expected_surface != case_surface:
        raise ProbeError(
            f"{case_id}.{field_name}: query shape surface {expected_surface!r} does not match case surface {case_surface!r}"
        )
    query_surface = expect_string(query.get("surface"), f"{case_id}.{field_name}.surface")
    if query_surface != case_surface:
        raise ProbeError(
            f"{case_id}.{field_name}: query surface {query_surface!r} does not match case surface {case_surface!r}"
        )
    expected_command = expect_string(
        shape.get("command"),
        f"manifest.query_shapes.{query_type}.command",
    )
    query_command = expect_string(query.get("command"), f"{case_id}.{field_name}.command")
    if query_command != expected_command:
        raise ProbeError(f"{case_id}.{field_name}: command does not match manifest.query_shapes")
    required_keys = expect_list(
        shape.get("required_keys"),
        f"manifest.query_shapes.{query_type}.required_keys",
    )
    for key_index, key_value in enumerate(required_keys):
        key = expect_string(
            key_value,
            f"manifest.query_shapes.{query_type}.required_keys[{key_index}]",
        )
        if key not in query:
            raise ProbeError(f"{case_id}.{field_name}: missing required query key {key!r}")
    if query_type in {"block_target", "table_target"}:
        expect_nonnegative_int(query.get("block_index"), f"{case_id}.{field_name}.block_index")
    elif query_type == "section_target":
        expect_string(query.get("selector"), f"{case_id}.{field_name}.selector")
        expect_positive_int(query.get("occurrence"), f"{case_id}.{field_name}.occurrence")
        expect_bool(query.get("contains"), f"{case_id}.{field_name}.contains")
        expect_bool(query.get("ignore_case"), f"{case_id}.{field_name}.ignore_case")
    elif query_type == "section_domain":
        expect_string(query.get("selector"), f"{case_id}.{field_name}.selector")
        expect_bool(query.get("contains"), f"{case_id}.{field_name}.contains")
        expect_bool(query.get("ignore_case"), f"{case_id}.{field_name}.ignore_case")
        expect_positive_int(
            query.get("occurrence_start"),
            f"{case_id}.{field_name}.occurrence_start",
        )
        expect_bool(
            query.get("enumerate_until_missing"),
            f"{case_id}.{field_name}.enumerate_until_missing",
        )
    elif query_type == "task_target":
        expect_nonnegative_int(query.get("result_index"), f"{case_id}.{field_name}.result_index")
        expect_nonnegative_int(query.get("task_index"), f"{case_id}.{field_name}.task_index")
    elif query_type == "tasks_domain":
        expect_nonnegative_int(query.get("result_index"), f"{case_id}.{field_name}.result_index")
    return query


def validate_expected(case_id: str, expected_value: Any) -> dict[str, Any]:
    expected = expect_mapping(expected_value, f"{case_id}.expected")
    if set(expected.keys()) != set(EXPECTED_CANDIDATES):
        raise ProbeError(f"{case_id}: expected candidate map must contain the exact four candidate names")
    normalized: dict[str, Any] = {}
    for candidate_name_value in EXPECTED_CANDIDATES:
        candidate_expected = expect_mapping(
            expected.get(candidate_name_value),
            f"{case_id}.expected.{candidate_name_value}",
        )
        decision = expect_string(
            candidate_expected.get("decision"),
            f"{case_id}.expected.{candidate_name_value}.decision",
        )
        if decision not in {"accept", "reject"}:
            raise ProbeError(f"{case_id}: invalid expected decision for {candidate_name_value}")
        credit = expect_choice(
            candidate_expected.get("credit"),
            f"{case_id}.expected.{candidate_name_value}.credit",
            EXPECTED_CREDITS,
        )
        normalized[candidate_name_value] = {"credit": credit, "decision": decision}
    return normalized


def validate_same_locator(case_id: str, case: dict[str, Any]) -> dict[str, Any]:
    same_locator = expect_mapping(
        case.get("same_locator_preconditions"),
        f"{case_id}.same_locator_preconditions",
    )
    normalized: dict[str, Any] = {}
    for key in (
        "require_target_bytes_equal",
        "require_canonical_descriptor_equal",
        "require_current_match_count",
        "require_document_bytes_different",
        "mechanical_failure_on_violation",
    ):
        if key == "require_current_match_count":
            normalized[key] = expect_nonnegative_int(
                same_locator.get(key),
                f"{case_id}.same_locator_preconditions.{key}",
            )
        else:
            normalized[key] = expect_bool(
                same_locator.get(key),
                f"{case_id}.same_locator_preconditions.{key}",
            )
    return normalized


def evaluate_case(
    case: dict[str, Any],
    query_shapes: dict[str, Any],
    md_binary: Path,
    workspace: Path,
) -> dict[str, Any]:
    case_id = case["case_id"]
    observed_path = workspace / f"{case_id}.observed.md"
    current_path = workspace / f"{case_id}.current.md"
    observed_document_utf8 = case["observed_document_utf8"]
    current_document_utf8 = case["current_document_utf8"]
    observed_document_bytes = observed_document_utf8.encode("utf-8")
    current_document_bytes = current_document_utf8.encode("utf-8")
    write_bytes(observed_path, observed_document_bytes)
    write_bytes(current_path, current_document_bytes)
    observed_target = resolve_target(
        case_id,
        "observed_target_query",
        case["surface"],
        case["observed_target_query"],
        query_shapes,
        md_binary,
        workspace,
        observed_path,
        observed_document_bytes,
    )
    current_target = resolve_target(
        case_id,
        "current_target_query",
        case["surface"],
        case["current_target_query"],
        query_shapes,
        md_binary,
        workspace,
        current_path,
        current_document_bytes,
    )
    current_domain = resolve_domain(
        case_id,
        case["surface"],
        case["current_domain_query"],
        query_shapes,
        md_binary,
        workspace,
        current_path,
        current_document_bytes,
    )
    ambiguity_matches = [
        entry for entry in current_domain if entry["target_bytes"] == observed_target["target_bytes"]
    ]
    ambiguity_count = len(ambiguity_matches)
    same_locator_report = evaluate_same_locator_preconditions(
        case,
        observed_target,
        current_target,
        observed_document_bytes,
        current_document_bytes,
        ambiguity_count,
    )
    candidate_results = build_candidate_results(
        case,
        observed_target,
        current_target,
        current_domain,
        ambiguity_matches,
        observed_document_bytes,
        current_document_bytes,
    )
    return {
        "abstract_projection_commands": {
            "current_domain": case["current_domain_query"]["command"],
            "current_target": case["current_target_query"]["command"],
            "observed_target": case["observed_target_query"]["command"],
        },
        "ambiguity_match_count": ambiguity_count,
        "candidate_results": candidate_results,
        "case_class": case["case_class"],
        "case_id": case_id,
        "current_descriptor": current_target["descriptor"],
        "current_document": payload_record(current_document_utf8, current_document_bytes),
        "current_domain_descriptors": [entry["descriptor"] for entry in current_domain],
        "current_domain_target_sha256": [sha256_hex(entry["target_bytes"]) for entry in current_domain],
        "current_target": payload_record(current_target["target_text"], current_target["target_bytes"]),
        "expected_identity_truth": case["identity_truth"],
        "notes": case.get("notes"),
        "observed_descriptor": observed_target["descriptor"],
        "observed_document": payload_record(observed_document_utf8, observed_document_bytes),
        "observed_target": payload_record(observed_target["target_text"], observed_target["target_bytes"]),
        "same_locator_preconditions": same_locator_report,
        "schema_version": PROBE_SCHEMA_VERSION,
        "surface": case["surface"],
    }


def resolve_target(
    case_id: str,
    query_name: str,
    surface: str,
    query: dict[str, Any],
    query_shapes: dict[str, Any],
    md_binary: Path,
    workspace: Path,
    document_path: Path,
    document_bytes: bytes,
) -> dict[str, Any]:
    query_type = query["type"]
    if surface == "block" and query_type == "block_target":
        result = run_md_json(
            md_binary,
            build_blocks_command(document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        blocks = expect_list(result.get("blocks"), f"{case_id}.{query_name}.blocks")
        index = query["block_index"]
        if index >= len(blocks):
            raise ProbeError(f"{case_id}.{query_name}: block_index {index} out of range")
        entry = expect_mapping(blocks[index], f"{case_id}.{query_name}.blocks[{index}]")
        descriptor = canonical_block_descriptor(case_id, query_name, entry)
        target_bytes, target_text = slice_target_bytes(
            document_bytes,
            descriptor["span"],
            f"{case_id}.{query_name}",
        )
        return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}
    if surface == "section" and query_type == "section_target":
        result = run_md_json(
            md_binary,
            build_section_command(query, document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        section = expect_mapping(result.get("section"), f"{case_id}.{query_name}.section")
        descriptor = canonical_section_descriptor(case_id, query_name, section)
        target_bytes, target_text = slice_target_bytes(
            document_bytes,
            descriptor["span"],
            f"{case_id}.{query_name}",
        )
        return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}
    if surface == "table" and query_type == "table_target":
        result = run_md_json(
            md_binary,
            build_table_read_command(query, document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        descriptor = canonical_table_descriptor(case_id, query_name, expect_mapping(result, f"{case_id}.{query_name}.result"))
        target_bytes, target_text = slice_target_bytes(
            document_bytes,
            descriptor["span"],
            f"{case_id}.{query_name}",
        )
        return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}
    if surface == "task" and query_type == "task_target":
        result = run_md_json(
            md_binary,
            build_tasks_command(document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        results = expect_list(result.get("results"), f"{case_id}.{query_name}.results")
        result_index = query["result_index"]
        if result_index >= len(results):
            raise ProbeError(f"{case_id}.{query_name}: result_index {result_index} out of range")
        file_result = expect_mapping(
            results[result_index],
            f"{case_id}.{query_name}.results[{result_index}]",
        )
        tasks = expect_list(
            file_result.get("tasks"),
            f"{case_id}.{query_name}.results[{result_index}].tasks",
        )
        task_index = query["task_index"]
        if task_index >= len(tasks):
            raise ProbeError(f"{case_id}.{query_name}: task_index {task_index} out of range")
        entry = expect_mapping(
            tasks[task_index],
            f"{case_id}.{query_name}.results[{result_index}].tasks[{task_index}]",
        )
        descriptor = canonical_task_descriptor(case_id, query_name, entry)
        target_bytes, target_text = slice_target_bytes(
            document_bytes,
            descriptor["span"],
            f"{case_id}.{query_name}",
        )
        return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}
    raise ProbeError(
        f"{case_id}.{query_name}: unsupported query type {query_type!r} for surface {surface!r}"
    )


def resolve_domain(
    case_id: str,
    surface: str,
    query: dict[str, Any],
    query_shapes: dict[str, Any],
    md_binary: Path,
    workspace: Path,
    document_path: Path,
    document_bytes: bytes,
) -> list[dict[str, Any]]:
    query_type = query["type"]
    if surface == "block" and query_type == "blocks_domain":
        result = run_md_json(
            md_binary,
            build_blocks_command(document_path.name),
            workspace,
            f"{case_id}.current_domain_query",
        )
        blocks = expect_list(result.get("blocks"), f"{case_id}.current_domain_query.blocks")
        return [
            build_domain_entry(
                canonical_block_descriptor(
                    case_id,
                    f"current_domain_query.blocks[{index}]",
                    expect_mapping(block, f"{case_id}.current_domain_query.blocks[{index}]"),
                ),
                document_bytes,
                f"{case_id}.current_domain_query.blocks[{index}]",
            )
            for index, block in enumerate(blocks)
        ]
    if surface == "section" and query_type == "section_domain":
        return resolve_section_domain(case_id, query, md_binary, workspace, document_path, document_bytes)
    if surface == "table" and query_type == "tables_domain":
        result = run_md_json(
            md_binary,
            build_tables_command(document_path.name),
            workspace,
            f"{case_id}.current_domain_query",
        )
        tables = expect_list(result.get("tables"), f"{case_id}.current_domain_query.tables")
        return [
            build_domain_entry(
                canonical_table_descriptor(
                    case_id,
                    f"current_domain_query.tables[{index}]",
                    expect_mapping(table, f"{case_id}.current_domain_query.tables[{index}]"),
                ),
                document_bytes,
                f"{case_id}.current_domain_query.tables[{index}]",
            )
            for index, table in enumerate(tables)
        ]
    if surface == "task" and query_type == "tasks_domain":
        result = run_md_json(
            md_binary,
            build_tasks_command(document_path.name),
            workspace,
            f"{case_id}.current_domain_query",
        )
        results = expect_list(result.get("results"), f"{case_id}.current_domain_query.results")
        result_index = query["result_index"]
        if result_index >= len(results):
            raise ProbeError(f"{case_id}.current_domain_query: result_index {result_index} out of range")
        file_result = expect_mapping(
            results[result_index],
            f"{case_id}.current_domain_query.results[{result_index}]",
        )
        tasks = expect_list(
            file_result.get("tasks"),
            f"{case_id}.current_domain_query.results[{result_index}].tasks",
        )
        return [
            build_domain_entry(
                canonical_task_descriptor(
                    case_id,
                    f"current_domain_query.results[{result_index}].tasks[{task_index}]",
                    expect_mapping(
                        task,
                        f"{case_id}.current_domain_query.results[{result_index}].tasks[{task_index}]",
                    ),
                ),
                document_bytes,
                f"{case_id}.current_domain_query.results[{result_index}].tasks[{task_index}]",
            )
            for task_index, task in enumerate(tasks)
        ]
    raise ProbeError(
        f"{case_id}.current_domain_query: unsupported query type {query_type!r} for surface {surface!r}"
    )


def resolve_section_domain(
    case_id: str,
    query: dict[str, Any],
    md_binary: Path,
    workspace: Path,
    document_path: Path,
    document_bytes: bytes,
) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    occurrence = query["occurrence_start"]
    while True:
        command = build_section_command(
            {
                "selector": query["selector"],
                "occurrence": occurrence,
                "contains": query["contains"],
                "ignore_case": query["ignore_case"],
            },
            document_path.name,
        )
        outcome = run_md_command(
            md_binary,
            command,
            workspace,
            f"{case_id}.current_domain_query.occurrence[{occurrence}]",
        )
        if outcome["returncode"] != 0:
            if outcome["returncode"] == 1 and query["enumerate_until_missing"]:
                if occurrence == query["occurrence_start"]:
                    raise ProbeError(f"{case_id}.current_domain_query: first section occurrence did not resolve")
                break
            stderr_text = decode_utf8(
                outcome["stderr"],
                f"{case_id}.current_domain_query.occurrence[{occurrence}].stderr",
            ).strip()
            raise ProbeError(
                f"{case_id}.current_domain_query: md exited with code {outcome['returncode']} at section occurrence {occurrence}: {stderr_text}"
            )
        stdout_text = decode_utf8(
            outcome["stdout"],
            f"{case_id}.current_domain_query.occurrence[{occurrence}].stdout",
        )
        try:
            result = json.loads(stdout_text)
        except json.JSONDecodeError as exc:
            raise ProbeError(
                f"{case_id}.current_domain_query: invalid md JSON at section occurrence {occurrence}: {exc}"
            ) from exc
        section = expect_mapping(result, f"{case_id}.current_domain_query.occurrence[{occurrence}].result").get("section")
        descriptor = canonical_section_descriptor(
            case_id,
            f"current_domain_query.occurrence[{occurrence}]",
            expect_mapping(
                section,
                f"{case_id}.current_domain_query.occurrence[{occurrence}].section",
            ),
        )
        entries.append(
            build_domain_entry(
                descriptor,
                document_bytes,
                f"{case_id}.current_domain_query.occurrence[{occurrence}]",
            )
        )
        occurrence += 1
    return entries


def build_domain_entry(
    descriptor: dict[str, Any],
    document_bytes: bytes,
    where: str,
) -> dict[str, Any]:
    target_bytes, target_text = slice_target_bytes(document_bytes, descriptor["span"], where)
    return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}


def evaluate_same_locator_preconditions(
    case: dict[str, Any],
    observed_target: dict[str, Any],
    current_target: dict[str, Any],
    observed_document_bytes: bytes,
    current_document_bytes: bytes,
    ambiguity_count: int,
) -> dict[str, Any] | None:
    if "same_locator_preconditions" not in case:
        return None
    same_locator = case["same_locator_preconditions"]
    report = {
        "canonical_descriptor_equal": observed_target["descriptor"] == current_target["descriptor"],
        "current_match_count": ambiguity_count,
        "document_bytes_different": observed_document_bytes != current_document_bytes,
        "mechanical_failure_on_violation": same_locator["mechanical_failure_on_violation"],
        "target_bytes_equal": observed_target["target_bytes"] == current_target["target_bytes"],
    }
    failures = []
    if same_locator["require_target_bytes_equal"] and not report["target_bytes_equal"]:
        failures.append("target bytes are not equal")
    if same_locator["require_canonical_descriptor_equal"] and not report["canonical_descriptor_equal"]:
        failures.append("canonical descriptors are not equal")
    if same_locator["require_current_match_count"] != report["current_match_count"]:
        failures.append(
            f"current ambiguity count is {report['current_match_count']} instead of {same_locator['require_current_match_count']}"
        )
    if same_locator["require_document_bytes_different"] and not report["document_bytes_different"]:
        failures.append("document bytes are not different")
    if failures and same_locator["mechanical_failure_on_violation"]:
        raise ProbeError(f"{case['case_id']}: same-locator preconditions failed: {'; '.join(failures)}")
    return report


def build_candidate_results(
    case: dict[str, Any],
    observed_target: dict[str, Any],
    current_target: dict[str, Any],
    current_domain: list[dict[str, Any]],
    ambiguity_matches: list[dict[str, Any]],
    observed_document_bytes: bytes,
    current_document_bytes: bytes,
) -> dict[str, Any]:
    results: dict[str, Any] = {}
    observed_descriptor_bytes = canonical_descriptor_bytes(observed_target["descriptor"])
    current_descriptor_bytes = canonical_descriptor_bytes(current_target["descriptor"])
    observed_target_bytes = observed_target["target_bytes"]
    current_target_bytes = current_target["target_bytes"]
    expected = case["expected"]
    payloads = {
        "content_only": {
            "observed": [observed_target_bytes],
            "current": [current_target_bytes],
        },
        "target_local": {
            "observed": [observed_descriptor_bytes, observed_target_bytes],
            "current": [current_descriptor_bytes, current_target_bytes],
        },
        "ambiguity_reject": {
            "observed": [observed_target_bytes],
            "current": [current_target_bytes],
        },
        "document_target_state": {
            "observed": [observed_descriptor_bytes, observed_target_bytes, observed_document_bytes],
            "current": [current_descriptor_bytes, current_target_bytes, current_document_bytes],
        },
    }
    for candidate_name_value in EXPECTED_CANDIDATES:
        observed_token = token_digest_hex(
            candidate_name_value,
            case["surface"],
            payloads[candidate_name_value]["observed"],
        )
        current_token = token_digest_hex(
            candidate_name_value,
            case["surface"],
            payloads[candidate_name_value]["current"],
        )
        if candidate_name_value == "content_only":
            decision = "accept" if observed_token == current_token else "reject"
        elif candidate_name_value == "target_local":
            decision = "accept" if observed_token == current_token else "reject"
        elif candidate_name_value == "ambiguity_reject":
            decision = "accept" if len(ambiguity_matches) == 1 and observed_token == current_token else "reject"
        else:
            decision = "accept" if observed_token == current_token else "reject"
        expected_decision = expected[candidate_name_value]["decision"]
        expected_credit = expected[candidate_name_value]["credit"]
        wrong_identity_accept = decision == "accept" and case["identity_truth"] != "same_target"
        required_same_state_reject = decision == "reject" and case["identity_truth"] == "same_target"
        unrelated_edit_conflict = (
            decision == "reject"
            and case["case_class"] == "unrelated_edit_after_unchanged_target"
        )
        whole_document_false_conflict_cost = (
            candidate_name_value == "document_target_state" and unrelated_edit_conflict
        )
        if wrong_identity_accept:
            graduation_verdict = "fails_wrong_identity"
        elif whole_document_false_conflict_cost:
            graduation_verdict = "fails_whole_document_false_conflict"
        elif required_same_state_reject:
            graduation_verdict = "fails_required_same_state"
        else:
            graduation_verdict = "graduates"
        results[candidate_name_value] = {
            "credit": expected_credit,
            "current_token_sha256": current_token,
            "decision": decision,
            "expectation_match": decision == expected_decision,
            "expected_decision": expected_decision,
            "observed_token_sha256": observed_token,
            "required_same_state_reject": required_same_state_reject,
            "unrelated_edit_conflict": unrelated_edit_conflict,
            "whole_document_false_conflict_cost": whole_document_false_conflict_cost,
            "wrong_identity_accept": wrong_identity_accept,
        }
    return results


def build_candidate_summary(case_reports: list[dict[str, Any]]) -> dict[str, Any]:
    summary: dict[str, Any] = {}
    for candidate_name_value in EXPECTED_CANDIDATES:
        accepts = 0
        rejects = 0
        wrong_identity_accepts = 0
        required_same_state_rejects = 0
        unrelated_edit_conflicts = 0
        expectation_matches = 0
        for case_report in case_reports:
            result = case_report["candidate_results"][candidate_name_value]
            if result["decision"] == "accept":
                accepts += 1
            else:
                rejects += 1
            if result["wrong_identity_accept"]:
                wrong_identity_accepts += 1
            if result["required_same_state_reject"]:
                required_same_state_rejects += 1
            if result["unrelated_edit_conflict"]:
                unrelated_edit_conflicts += 1
            if result["expectation_match"]:
                expectation_matches += 1
        graduation_verdict = select_graduation_verdict(
            candidate_name_value,
            wrong_identity_accepts,
            unrelated_edit_conflicts,
            required_same_state_rejects,
        )
        summary[candidate_name_value] = {
            "accepts": accepts,
            "disposition": "promote" if graduation_verdict == "graduates" else "demote",
            "expectation_matches": expectation_matches,
            "graduation_verdict": graduation_verdict,
            "rejects": rejects,
            "required_same_state_rejects": required_same_state_rejects,
            "unrelated_edit_conflicts": unrelated_edit_conflicts,
            "wrong_identity_accepts": wrong_identity_accepts,
        }
    return summary


def select_graduation_verdict(
    candidate_name_value: str,
    wrong_identity_accepts: int,
    unrelated_edit_conflicts: int,
    required_same_state_rejects: int,
) -> str:
    if wrong_identity_accepts:
        return "fails_wrong_identity"
    if candidate_name_value == "document_target_state" and unrelated_edit_conflicts:
        return "fails_whole_document_false_conflict"
    if required_same_state_rejects:
        return "fails_required_same_state"
    return "graduates"


def build_overall_graduation_verdict(candidate_summary: dict[str, Any]) -> dict[str, Any]:
    candidate_verdicts = {
        candidate_name_value: candidate_summary[candidate_name_value]["graduation_verdict"]
        for candidate_name_value in EXPECTED_CANDIDATES
    }
    graduating_candidates = [
        candidate_name_value
        for candidate_name_value in EXPECTED_CANDIDATES
        if candidate_summary[candidate_name_value]["disposition"] == "promote"
    ]
    demoted_candidates = [
        candidate_name_value
        for candidate_name_value in EXPECTED_CANDIDATES
        if candidate_summary[candidate_name_value]["disposition"] == "demote"
    ]
    if not graduating_candidates:
        verdict = "no_candidate_graduates"
        selected_candidate = None
    elif len(graduating_candidates) == 1:
        verdict = "single_candidate_graduates"
        selected_candidate = graduating_candidates[0]
    else:
        verdict = "multiple_candidates_graduate"
        selected_candidate = None
    return {
        "candidate_verdicts": candidate_verdicts,
        "demoted_candidates": demoted_candidates,
        "graduating_candidates": graduating_candidates,
        "selected_candidate": selected_candidate,
        "verdict": verdict,
        "whole_document_false_conflict_cost": candidate_summary["document_target_state"][
            "unrelated_edit_conflicts"
        ],
    }


def build_blocks_command(document_name: str) -> list[str]:
    return ["blocks", document_name, "--json"]


def build_section_command(query: dict[str, Any], document_name: str) -> list[str]:
    command = ["section", query["selector"], document_name]
    if query["occurrence"] is not None:
        command.extend(["--occurrence", str(query["occurrence"])])
    if query["contains"]:
        command.append("--contains")
    if query["ignore_case"]:
        command.append("--ignore-case")
    command.append("--json")
    return command


def build_tables_command(document_name: str) -> list[str]:
    return ["table", document_name, "--json"]


def build_table_read_command(query: dict[str, Any], document_name: str) -> list[str]:
    return ["table", document_name, "--index", str(query["block_index"]), "--json"]


def build_tasks_command(document_name: str) -> list[str]:
    return ["tasks", document_name, "--json"]


def run_md_json(
    md_binary: Path,
    args: list[str],
    workspace: Path,
    where: str,
) -> dict[str, Any]:
    outcome = run_md_command(md_binary, args, workspace, where)
    if outcome["returncode"] != 0:
        stderr_text = decode_utf8(outcome["stderr"], f"{where}.stderr").strip()
        raise ProbeError(f"{where}: md exited with code {outcome['returncode']}: {stderr_text}")
    stdout_text = decode_utf8(outcome["stdout"], f"{where}.stdout")
    try:
        value = json.loads(stdout_text)
    except json.JSONDecodeError as exc:
        raise ProbeError(f"{where}: invalid md JSON: {exc}") from exc
    return expect_mapping(value, f"{where}.result")


def run_md_command(
    md_binary: Path,
    args: list[str],
    workspace: Path,
    where: str,
) -> dict[str, Any]:
    command = [str(md_binary), *args]
    try:
        completed = subprocess.run(
            command,
            capture_output=True,
            check=False,
            cwd=str(workspace),
            env={},
            shell=False,
        )
    except OSError as exc:
        raise ProbeError(f"{where}: failed to invoke md binary") from exc
    return {
        "returncode": completed.returncode,
        "stderr": completed.stderr,
        "stdout": completed.stdout,
    }


def canonical_block_descriptor(case_id: str, where: str, entry: dict[str, Any]) -> dict[str, Any]:
    descriptor = {
        "index": expect_nonnegative_int(entry.get("index"), f"{case_id}.{where}.index"),
        "span": normalize_span(case_id, where, entry.get("span")),
    }
    return descriptor


def canonical_section_descriptor(case_id: str, where: str, section: dict[str, Any]) -> dict[str, Any]:
    descriptor = {
        "heading": normalize_optional_mapping(case_id, f"{where}.heading", section.get("heading")),
        "selector": normalize_section_selector(case_id, where, section.get("selector")),
        "span": normalize_span(case_id, where, section.get("span")),
    }
    return descriptor


def canonical_table_descriptor(case_id: str, where: str, entry: dict[str, Any]) -> dict[str, Any]:
    descriptor = {
        "block_index": expect_nonnegative_int(
            entry.get("block_index"),
            f"{case_id}.{where}.block_index",
        ),
        "span": normalize_span(case_id, where, entry.get("span")),
    }
    return descriptor


def canonical_task_descriptor(case_id: str, where: str, entry: dict[str, Any]) -> dict[str, Any]:
    child_path = expect_list(entry.get("child_path"), f"{case_id}.{where}.child_path")
    descriptor = {
        "child_path": [
            expect_nonnegative_int(value, f"{case_id}.{where}.child_path[{index}]")
            for index, value in enumerate(child_path)
        ],
        "loc": expect_string(entry.get("loc"), f"{case_id}.{where}.loc"),
        "span": normalize_span(case_id, where, entry.get("span")),
    }
    return descriptor


def normalize_section_selector(case_id: str, where: str, value: Any) -> dict[str, Any]:
    selector = expect_mapping(value, f"{case_id}.{where}.selector")
    normalized = {
        "heading_text": normalize_optional_string(
            selector.get("heading_text"),
            f"{case_id}.{where}.selector.heading_text",
        ),
        "kind": expect_string(selector.get("kind"), f"{case_id}.{where}.selector.kind"),
        "match_mode": expect_string(
            selector.get("match_mode"),
            f"{case_id}.{where}.selector.match_mode",
        ),
        "occurrence": normalize_optional_int(
            selector.get("occurrence"),
            f"{case_id}.{where}.selector.occurrence",
        ),
    }
    return normalized


def normalize_span(case_id: str, where: str, value: Any) -> dict[str, int]:
    span = expect_mapping(value, f"{case_id}.{where}.span")
    line_start = expect_positive_int(span.get("line_start"), f"{case_id}.{where}.span.line_start")
    line_end = expect_positive_int(span.get("line_end"), f"{case_id}.{where}.span.line_end")
    byte_start = expect_nonnegative_int(span.get("byte_start"), f"{case_id}.{where}.span.byte_start")
    byte_end = expect_nonnegative_int(span.get("byte_end"), f"{case_id}.{where}.span.byte_end")
    if byte_end < byte_start:
        raise ProbeError(f"{case_id}.{where}.span.byte_end must be >= byte_start")
    if line_end < line_start:
        raise ProbeError(f"{case_id}.{where}.span.line_end must be >= line_start")
    return {
        "byte_end": byte_end,
        "byte_start": byte_start,
        "line_end": line_end,
        "line_start": line_start,
    }


def slice_target_bytes(
    document_bytes: bytes,
    span: dict[str, int],
    where: str,
) -> tuple[bytes, str]:
    start = span["byte_start"]
    end = span["byte_end"]
    if end > len(document_bytes):
        raise ProbeError(f"{where}: span.byte_end {end} exceeds document byte length {len(document_bytes)}")
    target_bytes = document_bytes[start:end]
    try:
        target_text = target_bytes.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ProbeError(f"{where}: target bytes are not valid UTF-8 at the reported span") from exc
    return target_bytes, target_text


def token_digest_hex(candidate_name_value: str, surface: str, fields: list[bytes]) -> str:
    return hashlib.sha256(token_preimage(candidate_name_value, surface, fields)).hexdigest()


def token_preimage(candidate_name_value: str, surface: str, fields: list[bytes]) -> bytes:
    label = b"target-state-etag-token"
    parts = [label]
    for fixed_field in (
        PROBE_SCHEMA_VERSION.encode("ascii"),
        candidate_name_value.encode("ascii"),
        surface.encode("ascii"),
    ):
        parts.append(len(fixed_field).to_bytes(8, "big"))
        parts.append(fixed_field)
    for field in fields:
        parts.append(len(field).to_bytes(8, "big"))
        parts.append(field)
    return b"".join(parts)


def payload_record(text: str, payload: bytes) -> dict[str, Any]:
    return {
        "byte_length": len(payload),
        "sha256": sha256_hex(payload),
        "utf8": text,
    }


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def order_report_value(value: Any) -> Any:
    if isinstance(value, dict):
        ordered: dict[str, Any] = {}
        if set(value.keys()) == set(EXPECTED_CANDIDATES):
            key_order = EXPECTED_CANDIDATES
        else:
            key_order = sorted(value.keys())
        for key in key_order:
            ordered[key] = order_report_value(value[key])
        return ordered
    if isinstance(value, list):
        return [order_report_value(item) for item in value]
    return value


def canonical_descriptor_bytes(descriptor: dict[str, Any]) -> bytes:
    return json.dumps(
        descriptor,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def canonical_json_bytes(value: dict[str, Any]) -> bytes:
    return (
        json.dumps(
            order_report_value(value),
            ensure_ascii=False,
            indent=2,
            sort_keys=False,
        )
        + "\n"
    ).encode("utf-8")


def verify_check_file(check_path: Path, report_bytes: bytes) -> None:
    try:
        existing = check_path.read_bytes()
    except OSError as exc:
        raise ProbeError(f"failed to read --check file: {check_path}") from exc
    if existing != report_bytes:
        raise ProbeError(f"--check mismatch: {check_path}")


def atomic_write(output_path: Path, report_bytes: bytes) -> None:
    parent = output_path.parent
    if not parent.exists():
        raise ProbeError(f"--output parent directory does not exist: {parent}")
    temp_name: str | None = None
    try:
        with NamedTemporaryFile(
            dir=str(parent),
            prefix=f".{output_path.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temp_name = handle.name
            handle.write(report_bytes)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, output_path)
        temp_name = None
    except OSError as exc:
        raise ProbeError(f"failed to atomically write --output file: {output_path}") from exc
    finally:
        if temp_name is not None:
            try:
                os.unlink(temp_name)
            except OSError:
                pass


def write_bytes(path: Path, payload: bytes) -> None:
    try:
        path.write_bytes(payload)
    except OSError as exc:
        raise ProbeError(f"failed to write case document: {path.name}") from exc


def decode_utf8(payload: bytes, where: str) -> str:
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ProbeError(f"{where}: output is not valid UTF-8") from exc


def expect_mapping(value: Any, where: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ProbeError(f"{where} must be an object")
    return value


def expect_list(value: Any, where: str) -> list[Any]:
    if not isinstance(value, list):
        raise ProbeError(f"{where} must be an array")
    return value


def expect_string(value: Any, where: str) -> str:
    if not isinstance(value, str):
        raise ProbeError(f"{where} must be a string")
    return value


def expect_choice(value: Any, where: str, choices: tuple[str, ...]) -> str:
    normalized = expect_string(value, where)
    if normalized not in choices:
        raise ProbeError(
            f"{where}: invalid value {normalized!r}; allowed values: {choices!r}"
        )
    return normalized


def expect_bool(value: Any, where: str) -> bool:
    if not isinstance(value, bool):
        raise ProbeError(f"{where} must be a boolean")
    return value


def expect_nonnegative_int(value: Any, where: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ProbeError(f"{where} must be a nonnegative integer")
    return value


def expect_positive_int(value: Any, where: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ProbeError(f"{where} must be a positive integer")
    return value


def normalize_optional_string(value: Any, where: str) -> str | None:
    if value is None:
        return None
    return expect_string(value, where)


def normalize_optional_int(value: Any, where: str) -> int | None:
    if value is None:
        return None
    return expect_nonnegative_int(value, where)


def normalize_optional_mapping(case_id: str, where: str, value: Any) -> dict[str, Any] | None:
    if value is None:
        return None
    mapping = expect_mapping(value, f"{case_id}.{where}")
    normalized: dict[str, Any] = {}
    for key, item in mapping.items():
        if key == "span":
            normalized[key] = normalize_span(case_id, where, item)
        elif key in {"block_index", "level"}:
            normalized[key] = expect_nonnegative_int(item, f"{case_id}.{where}.{key}")
        else:
            normalized[key] = item
    return normalized


if __name__ == "__main__":
    raise SystemExit(main())
