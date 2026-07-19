from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import struct
import subprocess
import sys
from pathlib import Path
from tempfile import NamedTemporaryFile, TemporaryDirectory
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
MANIFEST_PATH = SCRIPT_DIR / "cases.json"
PROBE_SCHEMA_VERSION = "non-block-target-identity-probe.v1"
CANONICAL_DESCRIPTOR_SCHEMA_VERSION = "non-block-target-identity-descriptor.v1"
TOKEN_DOMAIN_LABEL = b"non-block-target-identity-token"
CASE_ID_PATTERN = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
RECONSTRUCTION_FORMULA = (
    "observed_prefix_before_first_matching_target + "
    "observed_suffix_starting_at_second_matching_target"
)
EXPECTED_CANDIDATES = (
    "content_only",
    "target_local",
    "ambiguity_reject",
    "document_target_state",
)
EXPECTED_IDENTITY_TRUTHS = ("same_target", "wrong_target")
EXPECTED_CASE_CLASSES = (
    "unchanged_reread",
    "duplicate_cross_target_copy",
    "same_locator_duplicate_shift",
    "unrelated_edit_after_unchanged_target",
)
EXPECTED_CASE_MATRIX = (
    ("section-unchanged-reread", "same_target"),
    ("section-duplicate-cross-target-copy", "wrong_target"),
    ("section-same-locator-duplicate-shift", "wrong_target"),
    ("section-unrelated-edit-false-conflict", "same_target"),
    ("table-unchanged-reread", "same_target"),
    ("table-duplicate-cross-target-copy", "wrong_target"),
    ("table-same-locator-duplicate-shift", "wrong_target"),
    ("table-unrelated-edit-false-conflict", "same_target"),
    ("task-unchanged-reread", "same_target"),
    ("task-duplicate-cross-target-copy", "wrong_target"),
    ("task-same-locator-duplicate-shift", "wrong_target"),
    ("task-unrelated-edit-false-conflict", "same_target"),
)
EXPECTED_CASE_IDS = tuple(case_id for case_id, _identity_truth in EXPECTED_CASE_MATRIX)
EXPECTED_CASE_IDENTITY_TRUTHS = {
    case_id: identity_truth for case_id, identity_truth in EXPECTED_CASE_MATRIX
}
EXPECTED_MANIFEST_SEMANTIC_SHA256 = "d0b47e5c8ee38800e383cb25c82484ed01dd497008bd4faf0625e0799d3c41b9"
EXPECTED_SAME_LOCATOR_LINEAGE_KEYS = (
    "require_observed_match_count",
    "require_current_match_count",
    "require_distinct_increasing_byte_starts",
    "require_reconstructed_current_document_sha256",
    "require_reconstruction_equality",
)
EXPECTED_SAME_LOCATOR_PRECONDITION_KEYS = (
    "require_target_bytes_equal",
    "require_canonical_descriptor_equal",
    "require_document_bytes_different",
    "mechanical_failure_on_violation",
)


class ProbeError(RuntimeError):
    pass


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the local non-block target-identity experiment against a supplied md binary."
        )
    )
    parser.add_argument(
        "--md-binary",
        type=Path,
        required=True,
        help="Path to the md binary used for live JSON descriptor authority.",
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
    with TemporaryDirectory(prefix="non-block-target-identity-") as workspace_str:
        workspace = Path(workspace_str)
        case_reports = [evaluate_case(case, manifest["query_shapes"], md_binary, workspace) for case in manifest["cases"]]
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
    actual_manifest_semantic_sha256 = manifest_semantic_sha256(manifest_map)
    if actual_manifest_semantic_sha256 != EXPECTED_MANIFEST_SEMANTIC_SHA256:
        raise ProbeError(
            f"{MANIFEST_PATH.name} runner-owned canonical semantic digest mismatch: "
            f"actual {actual_manifest_semantic_sha256}, "
            f"expected {EXPECTED_MANIFEST_SEMANTIC_SHA256}"
        )
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
    expect_string(candidate.get("decision_gate"), f"manifest.candidates[{index}].decision_gate")
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
    if "expected" in case:
        raise ProbeError(f"{case_id}: manifest must not contain candidate expected decisions")
    surface = expect_string(case.get("surface"), f"{case_id}.surface")
    if surface not in {"section", "table", "task"}:
        raise ProbeError(f"{case_id}: unsupported surface {surface!r}")
    identity_truth = validate_identity_truth(case_id, case.get("identity_truth"))
    normalized = {
        "case_class": expect_choice(
            case.get("case_class"),
            f"{case_id}.case_class",
            EXPECTED_CASE_CLASSES,
        ),
        "case_id": case_id,
        "current_document_utf8": expect_string(
            case.get("current_document_utf8"),
            f"{case_id}.current_document_utf8",
        ),
        "current_domain_query": validate_query(
            case_id,
            "current_domain_query",
            case.get("current_domain_query"),
            surface,
            query_shapes,
        ),
        "current_target_query": validate_query(
            case_id,
            "current_target_query",
            case.get("current_target_query"),
            surface,
            query_shapes,
        ),
        "identity_truth": identity_truth,
        "observed_document_utf8": expect_string(
            case.get("observed_document_utf8"),
            f"{case_id}.observed_document_utf8",
        ),
        "observed_target_query": validate_query(
            case_id,
            "observed_target_query",
            case.get("observed_target_query"),
            surface,
            query_shapes,
        ),
        "surface": surface,
    }
    is_same_locator = normalized["case_class"] == "same_locator_duplicate_shift"
    if is_same_locator:
        normalized["same_locator_preconditions"] = validate_same_locator_preconditions(case_id, case)
        normalized["same_locator_lineage"] = validate_same_locator_lineage(case_id, case)
    elif "same_locator_preconditions" in case or "same_locator_lineage" in case:
        raise ProbeError(f"{case_id}: non-same-locator cases must not carry same-locator mappings")
    return normalized


def validate_identity_truth(case_id: str, identity_truth_value: Any) -> str:
    identity_truth = expect_choice(
        identity_truth_value,
        f"{case_id}.identity_truth",
        EXPECTED_IDENTITY_TRUTHS,
    )
    expected_identity_truth = EXPECTED_CASE_IDENTITY_TRUTHS.get(case_id)
    if expected_identity_truth is not None and identity_truth != expected_identity_truth:
        raise ProbeError(
            f"{case_id}.identity_truth: runner-owned identity truth mismatch for case "
            f"{case_id}: got {identity_truth!r}, expected {expected_identity_truth!r}"
        )
    return expected_identity_truth or identity_truth


def validate_query(
    case_id: str,
    field_name: str,
    query_value: Any,
    case_surface: str,
    query_shapes: dict[str, Any],
) -> dict[str, Any]:
    query = expect_mapping(query_value, f"{case_id}.{field_name}")
    query_type = expect_string(query.get("type"), f"{case_id}.{field_name}.type")
    shape = expect_mapping(query_shapes.get(query_type), f"manifest.query_shapes.{query_type}")
    expected_surface = expect_string(
        shape.get("surface"),
        f"manifest.query_shapes.{query_type}.surface",
    )
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
    if query_type == "section_target":
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
    elif query_type == "table_target":
        expect_nonnegative_int(query.get("block_index"), f"{case_id}.{field_name}.block_index")
    elif query_type == "task_target":
        expect_nonnegative_int(query.get("result_index"), f"{case_id}.{field_name}.result_index")
        expect_nonnegative_int(query.get("task_index"), f"{case_id}.{field_name}.task_index")
    return query


def validate_same_locator_preconditions(case_id: str, case: dict[str, Any]) -> dict[str, Any]:
    raw = expect_mapping(
        case.get("same_locator_preconditions"),
        f"{case_id}.same_locator_preconditions",
    )
    if set(raw.keys()) != set(EXPECTED_SAME_LOCATOR_PRECONDITION_KEYS):
        raise ProbeError(f"{case_id}.same_locator_preconditions: unexpected key set")
    return {
        "require_target_bytes_equal": expect_bool(
            raw.get("require_target_bytes_equal"),
            f"{case_id}.same_locator_preconditions.require_target_bytes_equal",
        ),
        "require_canonical_descriptor_equal": expect_bool(
            raw.get("require_canonical_descriptor_equal"),
            f"{case_id}.same_locator_preconditions.require_canonical_descriptor_equal",
        ),
        "require_document_bytes_different": expect_bool(
            raw.get("require_document_bytes_different"),
            f"{case_id}.same_locator_preconditions.require_document_bytes_different",
        ),
        "mechanical_failure_on_violation": expect_bool(
            raw.get("mechanical_failure_on_violation"),
            f"{case_id}.same_locator_preconditions.mechanical_failure_on_violation",
        ),
    }


def validate_same_locator_lineage(case_id: str, case: dict[str, Any]) -> dict[str, Any]:
    raw = expect_mapping(case.get("same_locator_lineage"), f"{case_id}.same_locator_lineage")
    if set(raw.keys()) != set(EXPECTED_SAME_LOCATOR_LINEAGE_KEYS):
        raise ProbeError(f"{case_id}.same_locator_lineage: unexpected key set")
    reconstruction_formula = expect_string(
        raw.get("require_reconstruction_equality"),
        f"{case_id}.same_locator_lineage.require_reconstruction_equality",
    )
    if reconstruction_formula != RECONSTRUCTION_FORMULA:
        raise ProbeError(
            f"{case_id}.same_locator_lineage.require_reconstruction_equality must equal the runner-owned reconstruction formula"
        )
    return {
        "require_observed_match_count": expect_nonnegative_int(
            raw.get("require_observed_match_count"),
            f"{case_id}.same_locator_lineage.require_observed_match_count",
        ),
        "require_current_match_count": expect_nonnegative_int(
            raw.get("require_current_match_count"),
            f"{case_id}.same_locator_lineage.require_current_match_count",
        ),
        "require_distinct_increasing_byte_starts": expect_bool(
            raw.get("require_distinct_increasing_byte_starts"),
            f"{case_id}.same_locator_lineage.require_distinct_increasing_byte_starts",
        ),
        "require_reconstructed_current_document_sha256": expect_string(
            raw.get("require_reconstructed_current_document_sha256"),
            f"{case_id}.same_locator_lineage.require_reconstructed_current_document_sha256",
        ),
        "require_reconstruction_equality": reconstruction_formula,
    }


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
        md_binary,
        workspace,
        current_path,
        current_document_bytes,
    )
    observed_domain = resolve_domain(
        case_id,
        "observed_domain",
        case["surface"],
        case["current_domain_query"],
        md_binary,
        workspace,
        observed_path,
        observed_document_bytes,
    )
    current_domain = resolve_domain(
        case_id,
        "current_domain",
        case["surface"],
        case["current_domain_query"],
        md_binary,
        workspace,
        current_path,
        current_document_bytes,
    )
    observed_matches = [
        entry for entry in observed_domain if entry["target_bytes"] == observed_target["target_bytes"]
    ]
    current_matches = [
        entry for entry in current_domain if entry["target_bytes"] == observed_target["target_bytes"]
    ]
    same_locator_report = evaluate_same_locator(
        case,
        observed_target,
        current_target,
        observed_matches,
        current_matches,
        observed_document_bytes,
        current_document_bytes,
    )
    candidate_results = build_candidate_results(
        case,
        observed_target,
        current_target,
        current_matches,
        observed_document_bytes,
        current_document_bytes,
    )
    report: dict[str, Any] = {
        "abstract_projection_commands": {
            "current_domain": case["current_domain_query"]["command"],
            "current_target": case["current_target_query"]["command"],
            "observed_target": case["observed_target_query"]["command"],
        },
        "candidate_results": candidate_results,
        "case_class": case["case_class"],
        "case_id": case_id,
        "current_descriptor": current_target["descriptor"],
        "current_document": payload_record(current_document_utf8, current_document_bytes),
        "current_domain_descriptors": [entry["descriptor"] for entry in current_domain],
        "current_domain_match_count": len(current_matches),
        "current_target": payload_record(current_target["target_text"], current_target["target_bytes"]),
        "identity_truth": case["identity_truth"],
        "observed_descriptor": observed_target["descriptor"],
        "observed_document": payload_record(observed_document_utf8, observed_document_bytes),
        "observed_domain_match_count": len(observed_matches),
        "observed_target": payload_record(observed_target["target_text"], observed_target["target_bytes"]),
        "schema_version": PROBE_SCHEMA_VERSION,
        "surface": case["surface"],
    }
    if same_locator_report is not None:
        report["same_locator_lineage"] = same_locator_report
    return report


def resolve_target(
    case_id: str,
    query_name: str,
    surface: str,
    query: dict[str, Any],
    md_binary: Path,
    workspace: Path,
    document_path: Path,
    document_bytes: bytes,
) -> dict[str, Any]:
    if surface == "section":
        result = run_md_json(
            md_binary,
            build_section_command(query, document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        section = expect_mapping(result.get("section"), f"{case_id}.{query_name}.section")
        descriptor = canonical_section_descriptor(case_id, query_name, section)
    elif surface == "table":
        result = run_md_json(
            md_binary,
            build_table_read_command(query, document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        descriptor = canonical_table_descriptor(
            case_id,
            query_name,
            expect_mapping(result, f"{case_id}.{query_name}.result"),
        )
    elif surface == "task":
        result = run_md_json(
            md_binary,
            build_tasks_command(document_path.name),
            workspace,
            f"{case_id}.{query_name}",
        )
        descriptor = task_descriptor_from_result(case_id, query_name, result, query)
    else:
        raise ProbeError(f"{case_id}.{query_name}: unsupported surface {surface!r}")
    target_bytes, target_text = slice_target_bytes(
        document_bytes,
        descriptor["span"],
        f"{case_id}.{query_name}",
    )
    return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}


def resolve_domain(
    case_id: str,
    domain_name: str,
    surface: str,
    query: dict[str, Any],
    md_binary: Path,
    workspace: Path,
    document_path: Path,
    document_bytes: bytes,
) -> list[dict[str, Any]]:
    if surface == "section":
        return resolve_section_domain(
            case_id,
            domain_name,
            query,
            md_binary,
            workspace,
            document_path,
            document_bytes,
        )
    if surface == "table":
        result = run_md_json(
            md_binary,
            build_tables_command(document_path.name),
            workspace,
            f"{case_id}.{domain_name}",
        )
        tables = expect_list(result.get("tables"), f"{case_id}.{domain_name}.tables")
        return [
            build_domain_entry(
                canonical_table_descriptor(
                    case_id,
                    f"{domain_name}.tables[{index}]",
                    expect_mapping(entry, f"{case_id}.{domain_name}.tables[{index}]"),
                ),
                document_bytes,
                f"{case_id}.{domain_name}.tables[{index}]",
            )
            for index, entry in enumerate(tables)
        ]
    if surface == "task":
        result = run_md_json(
            md_binary,
            build_tasks_command(document_path.name),
            workspace,
            f"{case_id}.{domain_name}",
        )
        descriptors = task_domain_descriptors(case_id, domain_name, result)
        return [
            build_domain_entry(descriptor, document_bytes, f"{case_id}.{domain_name}[{index}]")
            for index, descriptor in enumerate(descriptors)
        ]
    raise ProbeError(f"{case_id}.{domain_name}: unsupported surface {surface!r}")


def resolve_section_domain(
    case_id: str,
    domain_name: str,
    query: dict[str, Any],
    md_binary: Path,
    workspace: Path,
    document_path: Path,
    document_bytes: bytes,
) -> list[dict[str, Any]]:
    descriptors: list[dict[str, Any]] = []
    occurrence = query["occurrence_start"]
    while True:
        command_query = {
            "selector": query["selector"],
            "occurrence": occurrence,
            "contains": query["contains"],
            "ignore_case": query["ignore_case"],
        }
        outcome = run_md_command(
            md_binary,
            build_section_command(command_query, document_path.name),
            workspace,
            f"{case_id}.{domain_name}.occurrence[{occurrence}]",
        )
        if outcome["returncode"] != 0:
            if not descriptors and query["enumerate_until_missing"]:
                stderr_text = decode_utf8(
                    outcome["stderr"],
                    f"{case_id}.{domain_name}.stderr",
                ).strip()
                raise ProbeError(
                    f"{case_id}.{domain_name}: section domain query failed before first match: {stderr_text}"
                )
            break
        stdout_text = decode_utf8(
            outcome["stdout"],
            f"{case_id}.{domain_name}.stdout[{occurrence}]",
        )
        try:
            value = json.loads(stdout_text)
        except json.JSONDecodeError as exc:
            raise ProbeError(f"{case_id}.{domain_name}: invalid md JSON: {exc}") from exc
        result = expect_mapping(value, f"{case_id}.{domain_name}.result[{occurrence}]")
        section = expect_mapping(
            result.get("section"),
            f"{case_id}.{domain_name}.section[{occurrence}]",
        )
        descriptor = canonical_section_descriptor(
            case_id,
            f"{domain_name}.section[{occurrence}]",
            section,
        )
        descriptors.append(
            build_domain_entry(
                descriptor,
                document_bytes,
                f"{case_id}.{domain_name}.section[{occurrence}]",
            )
        )
        occurrence += 1
    return descriptors


def evaluate_same_locator(
    case: dict[str, Any],
    observed_target: dict[str, Any],
    current_target: dict[str, Any],
    observed_matches: list[dict[str, Any]],
    current_matches: list[dict[str, Any]],
    observed_document_bytes: bytes,
    current_document_bytes: bytes,
) -> dict[str, Any] | None:
    if "same_locator_lineage" not in case:
        return None
    preconditions = case["same_locator_preconditions"]
    lineage = case["same_locator_lineage"]
    descriptor_equal = observed_target["descriptor"] == current_target["descriptor"]
    target_bytes_equal = observed_target["target_bytes"] == current_target["target_bytes"]
    document_bytes_different = observed_document_bytes != current_document_bytes
    report = {
        "canonical_descriptor_equal": descriptor_equal,
        "current_match_count": len(current_matches),
        "document_bytes_different": document_bytes_different,
        "mechanical_failure_on_violation": preconditions["mechanical_failure_on_violation"],
        "observed_match_count": len(observed_matches),
        "target_bytes_equal": target_bytes_equal,
    }
    failures = []
    if preconditions["require_target_bytes_equal"] and not target_bytes_equal:
        failures.append("target bytes are not equal")
    if preconditions["require_canonical_descriptor_equal"] and not descriptor_equal:
        failures.append("canonical descriptors are not equal")
    if preconditions["require_document_bytes_different"] and not document_bytes_different:
        failures.append("document bytes are not different")
    if len(observed_matches) != lineage["require_observed_match_count"]:
        failures.append(
            "observed exact-byte match count is "
            f"{len(observed_matches)} instead of {lineage['require_observed_match_count']}"
        )
    if len(current_matches) != lineage["require_current_match_count"]:
        failures.append(
            "current exact-byte match count is "
            f"{len(current_matches)} instead of {lineage['require_current_match_count']}"
        )
    if len(observed_matches) < 2:
        failures.append("observed duplicate set is smaller than two matches")
        first_span = None
        second_span = None
        distinct_increasing = False
        reconstructed_current_document = b""
    else:
        first_span = observed_matches[0]["descriptor"]["span"]
        second_span = observed_matches[1]["descriptor"]["span"]
        distinct_increasing = (
            first_span["byte_start"] < second_span["byte_start"]
            and first_span["byte_start"] != second_span["byte_start"]
        )
        reconstructed_current_document = (
            observed_document_bytes[: first_span["byte_start"]]
            + observed_document_bytes[second_span["byte_start"] :]
        )
    if lineage["require_distinct_increasing_byte_starts"] and not distinct_increasing:
        failures.append("observed duplicate spans are not distinct increasing byte starts")
    reconstructed_current_document_sha256 = sha256_hex(reconstructed_current_document)
    reconstruction_equality = reconstructed_current_document == current_document_bytes
    if reconstructed_current_document_sha256 != lineage["require_reconstructed_current_document_sha256"]:
        failures.append(
            "reconstructed current document sha256 is "
            f"{reconstructed_current_document_sha256} instead of "
            f"{lineage['require_reconstructed_current_document_sha256']}"
        )
    if not reconstruction_equality:
        failures.append("reconstructed current document does not equal current document bytes")
    if lineage["require_reconstruction_equality"] != RECONSTRUCTION_FORMULA:
        failures.append("reconstruction formula changed unexpectedly")
    report.update(
        {
            "current_match_spans": [entry["descriptor"]["span"] for entry in current_matches],
            "distinct_increasing_byte_starts": distinct_increasing,
            "first_observed_matching_span": first_span,
            "reconstructed_current_document_sha256": reconstructed_current_document_sha256,
            "reconstruction_equality": reconstruction_equality,
            "reconstruction_formula": RECONSTRUCTION_FORMULA,
            "second_observed_matching_span": second_span,
        }
    )
    if failures and preconditions["mechanical_failure_on_violation"]:
        raise ProbeError(f"{case['case_id']}: same-locator preconditions failed: {'; '.join(failures)}")
    return report


def build_candidate_results(
    case: dict[str, Any],
    observed_target: dict[str, Any],
    current_target: dict[str, Any],
    current_matches: list[dict[str, Any]],
    observed_document_bytes: bytes,
    current_document_bytes: bytes,
) -> dict[str, Any]:
    results: dict[str, Any] = {}
    observed_descriptor_bytes = canonical_descriptor_bytes(observed_target["descriptor"])
    current_descriptor_bytes = canonical_descriptor_bytes(current_target["descriptor"])
    observed_target_bytes = observed_target["target_bytes"]
    current_target_bytes = current_target["target_bytes"]
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
        if candidate_name_value == "ambiguity_reject":
            decision = "accept" if len(current_matches) == 1 and observed_token == current_token else "reject"
        else:
            decision = "accept" if observed_token == current_token else "reject"
        wrong_identity_accept = decision == "accept" and case["identity_truth"] == "wrong_target"
        required_same_state_reject = decision == "reject" and case["identity_truth"] == "same_target"
        unrelated_edit_conflict = (
            decision == "reject"
            and case["case_class"] == "unrelated_edit_after_unchanged_target"
        )
        if wrong_identity_accept:
            graduation_verdict = "fails_wrong_identity"
        elif candidate_name_value == "document_target_state" and unrelated_edit_conflict:
            graduation_verdict = "fails_whole_document_false_conflict"
        elif required_same_state_reject:
            graduation_verdict = "fails_required_same_state"
        else:
            graduation_verdict = "graduates"
        results[candidate_name_value] = {
            "current_token_sha256": current_token,
            "decision": decision,
            "graduation_verdict": graduation_verdict,
            "observed_token_sha256": observed_token,
            "required_same_state_reject": required_same_state_reject,
            "unrelated_edit_conflict": unrelated_edit_conflict,
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
        graduation_verdict = select_graduation_verdict(
            candidate_name_value,
            wrong_identity_accepts,
            unrelated_edit_conflicts,
            required_same_state_rejects,
        )
        summary[candidate_name_value] = {
            "accepts": accepts,
            "disposition": "promote" if graduation_verdict == "graduates" else "demote",
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


def build_section_command(query: dict[str, Any], document_name: str) -> list[str]:
    command = ["section", query["selector"], document_name]
    if query.get("occurrence") is not None:
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


def canonical_section_descriptor(case_id: str, where: str, section: dict[str, Any]) -> dict[str, Any]:
    return {
        "heading": normalize_json_value(case_id, f"{where}.heading", section.get("heading")),
        "selector": normalize_json_value(case_id, f"{where}.selector", section.get("selector")),
        "span": normalize_span(case_id, where, section.get("span")),
    }


def canonical_table_descriptor(case_id: str, where: str, result: dict[str, Any]) -> dict[str, Any]:
    return {
        "block_index": expect_nonnegative_int(result.get("block_index"), f"{case_id}.{where}.block_index"),
        "span": normalize_span(case_id, where, result.get("span")),
    }


def task_descriptor_from_result(
    case_id: str,
    where: str,
    result: dict[str, Any],
    query: dict[str, Any],
) -> dict[str, Any]:
    results = expect_list(result.get("results"), f"{case_id}.{where}.results")
    result_index = query["result_index"]
    if result_index >= len(results):
        raise ProbeError(f"{case_id}.{where}: result_index {result_index} out of range")
    file_result = expect_mapping(results[result_index], f"{case_id}.{where}.results[{result_index}]")
    tasks = expect_list(file_result.get("tasks"), f"{case_id}.{where}.results[{result_index}].tasks")
    task_index = query["task_index"]
    if task_index >= len(tasks):
        raise ProbeError(f"{case_id}.{where}: task_index {task_index} out of range")
    entry = expect_mapping(
        tasks[task_index],
        f"{case_id}.{where}.results[{result_index}].tasks[{task_index}]",
    )
    return canonical_task_descriptor(case_id, where, entry)


def task_domain_descriptors(case_id: str, where: str, result: dict[str, Any]) -> list[dict[str, Any]]:
    results = expect_list(result.get("results"), f"{case_id}.{where}.results")
    descriptors = []
    for result_index, file_result_value in enumerate(results):
        file_result = expect_mapping(
            file_result_value,
            f"{case_id}.{where}.results[{result_index}]",
        )
        tasks = expect_list(
            file_result.get("tasks"),
            f"{case_id}.{where}.results[{result_index}].tasks",
        )
        for task_index, entry_value in enumerate(tasks):
            entry = expect_mapping(
                entry_value,
                f"{case_id}.{where}.results[{result_index}].tasks[{task_index}]",
            )
            descriptors.append(
                canonical_task_descriptor(
                    case_id,
                    f"{where}.results[{result_index}].tasks[{task_index}]",
                    entry,
                )
            )
    return descriptors


def canonical_task_descriptor(case_id: str, where: str, entry: dict[str, Any]) -> dict[str, Any]:
    child_path = expect_list(entry.get("child_path"), f"{case_id}.{where}.child_path")
    return {
        "child_path": [
            expect_nonnegative_int(value, f"{case_id}.{where}.child_path[{index}]")
            for index, value in enumerate(child_path)
        ],
        "loc": expect_string(entry.get("loc"), f"{case_id}.{where}.loc"),
        "span": normalize_span(case_id, where, entry.get("span")),
    }


def build_domain_entry(
    descriptor: dict[str, Any],
    document_bytes: bytes,
    where: str,
) -> dict[str, Any]:
    target_bytes, target_text = slice_target_bytes(document_bytes, descriptor["span"], where)
    return {"descriptor": descriptor, "target_bytes": target_bytes, "target_text": target_text}


def slice_target_bytes(
    document_bytes: bytes,
    span: dict[str, int],
    where: str,
) -> tuple[bytes, str]:
    start = span["byte_start"]
    end = span["byte_end"]
    if start > end or end > len(document_bytes):
        raise ProbeError(f"{where}: span is out of range for target byte slicing")
    target_bytes = document_bytes[start:end]
    return target_bytes, decode_utf8(target_bytes, f"{where}.target_bytes")


def normalize_span(case_id: str, where: str, span_value: Any) -> dict[str, int]:
    span = expect_mapping(span_value, f"{case_id}.{where}.span")
    return {
        "byte_end": expect_nonnegative_int(span.get("byte_end"), f"{case_id}.{where}.span.byte_end"),
        "byte_start": expect_nonnegative_int(
            span.get("byte_start"),
            f"{case_id}.{where}.span.byte_start",
        ),
        "line_end": expect_nonnegative_int(span.get("line_end"), f"{case_id}.{where}.span.line_end"),
        "line_start": expect_nonnegative_int(
            span.get("line_start"),
            f"{case_id}.{where}.span.line_start",
        ),
    }


def normalize_json_value(case_id: str, where: str, value: Any) -> Any:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        return [normalize_json_value(case_id, f"{where}[{index}]", item) for index, item in enumerate(value)]
    if isinstance(value, dict):
        normalized: dict[str, Any] = {}
        for key in sorted(value.keys()):
            if not isinstance(key, str):
                raise ProbeError(f"{case_id}.{where}: descriptor keys must be strings")
            normalized[key] = normalize_json_value(case_id, f"{where}.{key}", value[key])
        return normalized
    raise ProbeError(f"{case_id}.{where}: unsupported descriptor value type {type(value).__name__}")


def payload_record(text: str, payload: bytes) -> dict[str, Any]:
    return {
        "sha256": sha256_hex(payload),
        "text": text,
        "utf8_len": len(payload),
    }


def token_digest_hex(candidate_name_value: str, surface: str, payload_fields: list[bytes]) -> str:
    payload = bytearray()
    payload.extend(TOKEN_DOMAIN_LABEL)
    payload.extend(length_prefixed(PROBE_SCHEMA_VERSION.encode("utf-8")))
    payload.extend(length_prefixed(candidate_name_value.encode("utf-8")))
    payload.extend(length_prefixed(surface.encode("utf-8")))
    for field in payload_fields:
        payload.extend(length_prefixed(field))
    return hashlib.sha256(payload).hexdigest()


def length_prefixed(value: bytes) -> bytes:
    return struct.pack(">Q", len(value)) + value


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def manifest_semantic_sha256(manifest_map: dict[str, Any]) -> str:
    canonical_manifest = json.dumps(
        manifest_map,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return sha256_hex(canonical_manifest)


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
        raise ProbeError(f"failed to write temporary file: {path.name}") from exc


def decode_utf8(payload: bytes, where: str) -> str:
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ProbeError(f"{where}: invalid UTF-8") from exc


def expect_mapping(value: Any, where: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ProbeError(f"{where}: expected object")
    return value


def expect_list(value: Any, where: str) -> list[Any]:
    if not isinstance(value, list):
        raise ProbeError(f"{where}: expected array")
    return value


def expect_string(value: Any, where: str) -> str:
    if not isinstance(value, str):
        raise ProbeError(f"{where}: expected string")
    return value


def expect_bool(value: Any, where: str) -> bool:
    if not isinstance(value, bool):
        raise ProbeError(f"{where}: expected boolean")
    return value


def expect_nonnegative_int(value: Any, where: str) -> int:
    if not isinstance(value, int) or value < 0:
        raise ProbeError(f"{where}: expected non-negative integer")
    return value


def expect_positive_int(value: Any, where: str) -> int:
    if not isinstance(value, int) or value <= 0:
        raise ProbeError(f"{where}: expected positive integer")
    return value


def expect_choice(value: Any, where: str, choices: tuple[str, ...]) -> str:
    text = expect_string(value, where)
    if text not in choices:
        raise ProbeError(f"{where}: expected one of {', '.join(choices)}")
    return text


if __name__ == "__main__":
    raise SystemExit(main())
