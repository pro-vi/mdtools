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
REPO_ROOT = SCRIPT_DIR.parent.parent
MANIFEST_PATH = SCRIPT_DIR / "cases.json"
PROBE_SCHEMA_VERSION = "position-bound-target-identity-probe.v1"
REPORT_SCHEMA_VERSION = "position-bound-target-identity-report.v1"
CANONICAL_DESCRIPTOR_SCHEMA_VERSION = "position-bound-target-identity-descriptor.v1"
TOKEN_DOMAIN_LABEL = b"position-bound-target-identity-token"
EXPECTED_MD_SCHEMA_VERSION = "mdtools.v1"
EXPECTED_MANIFEST_SCHEMA_VERSION = "position-bound-target-identity-manifest.v1"
EXPECTED_MANIFEST_KIND = "deterministic-position-bound-query-matrix"
EXPECTED_DATE_LOCKED = "2026-07-20"
EXPECTED_PROTOCOL_SHA256 = "579dd0bc5eb8cfa2e65834fd976517af10c008c2006e3687e124c911b2b77445"
EXPECTED_MANIFEST_SEMANTIC_SHA256 = "6de55bce9445bfa9c1011b466579cbc380b7704ebb6f9b76d36fc524b0543584"
CASE_ID_PATTERN = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
SHA256_PATTERN = re.compile(r"^[0-9a-f]{64}$")
EXPECTED_CANDIDATE_ORDER = (
    "preceding_block",
    "following_block",
    "adjacent_blocks",
    "byte_window_64",
)
EXPECTED_CASE_ORDER = (
    "block-unchanged-lf-reread",
    "block-unchanged-crlf-reread",
    "block-unchanged-multibyte-utf8-reread",
    "block-outside-context-unrelated-edit",
    "block-preceding-neighbor-edit-inside-prefix-window",
    "block-following-neighbor-edit-inside-suffix-window",
    "block-forward-survivor-same-locator-duplicate-substitution",
    "block-backward-survivor-same-locator-duplicate-substitution",
    "block-cloned-adjacent-blocks-distinguishing-byte-window",
    "block-byte-identical-bounded-neighborhood-duplicate-substitution",
)
EXPECTED_CASE_CLASSES = (
    "unchanged_reread",
    "outside_context_unrelated_edit",
    "preceding_neighbor_edit_inside_prefix_window",
    "following_neighbor_edit_inside_suffix_window",
    "forward_survivor_same_locator_duplicate_substitution",
    "backward_survivor_same_locator_duplicate_substitution",
    "cloned_adjacent_blocks_distinguishing_byte_window",
    "byte_identical_bounded_neighborhood_duplicate_substitution",
)
EXPECTED_IDENTITY_TRUTHS = ("same_target", "wrong_target")
EXPECTED_RELATIONS = ("equal", "different")
EXPECTED_CANDIDATE_DECISIONS = ("accept", "reject")
EXPECTED_CREDITS = ("correct", "wrong_identity", "false_conflict")
EXPECTED_GRADUATION_VERDICTS = (
    "graduates",
    "demoted_wrong_identity",
    "demoted_same_target_reject",
)
EXPECTED_AGGREGATE_VERDICTS = (
    "no_bounded_context_candidate_graduates",
    "single_candidate_graduates",
    "multiple_candidates_graduate",
)
EXPECTED_PRECEDING_BOUNDARY_STATES = ("bof", "present")
EXPECTED_FOLLOWING_BOUNDARY_STATES = ("eof", "present")
CANONICAL_OBSERVED_CASE_PATH_TEMPLATE = "cases/{case_id}/observed.md"
CANONICAL_CURRENT_CASE_PATH_TEMPLATE = "cases/{case_id}/current.md"
EXPECTED_CASE_CONTRACTS = {
    "block-unchanged-lf-reread": {
        "case_class": "unchanged_reread",
        "identity_truth": "same_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "equal",
            "observed_exact_target_match_count": 1,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "equal",
                "adjacent_blocks": "equal",
                "byte_window_64": "equal",
            },
            "mechanical_failure_on_violation": True,
        },
    },
    "block-unchanged-crlf-reread": {
        "case_class": "unchanged_reread",
        "identity_truth": "same_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "equal",
            "observed_exact_target_match_count": 1,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "equal",
                "adjacent_blocks": "equal",
                "byte_window_64": "equal",
            },
            "mechanical_failure_on_violation": True,
        },
    },
    "block-unchanged-multibyte-utf8-reread": {
        "case_class": "unchanged_reread",
        "identity_truth": "same_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "equal",
            "observed_exact_target_match_count": 1,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "equal",
                "adjacent_blocks": "equal",
                "byte_window_64": "equal",
            },
            "mechanical_failure_on_violation": True,
        },
    },
    "block-outside-context-unrelated-edit": {
        "case_class": "outside_context_unrelated_edit",
        "identity_truth": "same_target",
        "observed_target_selector": {"block_index": 3},
        "current_target_selector": {"block_index": 3},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 1,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "equal",
                "adjacent_blocks": "equal",
                "byte_window_64": "equal",
            },
            "mechanical_failure_on_violation": True,
        },
    },
    "block-preceding-neighbor-edit-inside-prefix-window": {
        "case_class": "preceding_neighbor_edit_inside_prefix_window",
        "identity_truth": "same_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 1,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "different",
                "following_block": "equal",
                "adjacent_blocks": "different",
                "byte_window_64": "different",
            },
            "mechanical_failure_on_violation": True,
        },
    },
    "block-following-neighbor-edit-inside-suffix-window": {
        "case_class": "following_neighbor_edit_inside_suffix_window",
        "identity_truth": "same_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 1,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "different",
                "adjacent_blocks": "different",
                "byte_window_64": "different",
            },
            "mechanical_failure_on_violation": True,
        },
    },
    "block-forward-survivor-same-locator-duplicate-substitution": {
        "case_class": "forward_survivor_same_locator_duplicate_substitution",
        "identity_truth": "wrong_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 2,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "different",
                "adjacent_blocks": "different",
                "byte_window_64": "different",
            },
            "mechanical_failure_on_violation": True,
        },
        "same-locator": True,
        "observed_target_duplicate_ordinal": 0,
        "survivor_duplicate_ordinal": 1,
        "require_reconstructed_current_document_sha256": "cbfb3cdc1da5cb66407d750f0091a6c65e1a983d6185c9510b0e62dc5fde36b1",
    },
    "block-backward-survivor-same-locator-duplicate-substitution": {
        "case_class": "backward_survivor_same_locator_duplicate_substitution",
        "identity_truth": "wrong_target",
        "observed_target_selector": {"block_index": 5},
        "current_target_selector": {"block_index": 5},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 2,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "different",
                "following_block": "equal",
                "adjacent_blocks": "different",
                "byte_window_64": "different",
            },
            "mechanical_failure_on_violation": True,
        },
        "same-locator": True,
        "observed_target_duplicate_ordinal": 1,
        "survivor_duplicate_ordinal": 0,
        "require_reconstructed_current_document_sha256": "9b27a84f80ae4405acf6bcf4b34be1936179db10494b632fa6dfe163109d9979",
        "backward_prefix_insertion_bytes": "inserted lead shard\n\noffset filler path\n\nnew local prefix marker!\n\n",
    },
    "block-cloned-adjacent-blocks-distinguishing-byte-window": {
        "case_class": "cloned_adjacent_blocks_distinguishing_byte_window",
        "identity_truth": "wrong_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 2,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "equal",
                "adjacent_blocks": "equal",
                "byte_window_64": "different",
            },
            "mechanical_failure_on_violation": True,
        },
        "same-locator": True,
        "observed_target_duplicate_ordinal": 0,
        "survivor_duplicate_ordinal": 1,
        "require_reconstructed_current_document_sha256": "223a515ca423c99cc32466dfeef9743f55ba990306cec0c3b7c425b29f6efe7d",
    },
    "block-byte-identical-bounded-neighborhood-duplicate-substitution": {
        "case_class": "byte_identical_bounded_neighborhood_duplicate_substitution",
        "identity_truth": "wrong_target",
        "observed_target_selector": {"block_index": 2},
        "current_target_selector": {"block_index": 2},
        "mechanical_preconditions": {
            "target_bytes_relation": "equal",
            "live_descriptor_relation": "equal",
            "document_bytes_relation": "different",
            "observed_exact_target_match_count": 2,
            "current_exact_target_match_count": 1,
            "candidate_context_relations": {
                "preceding_block": "equal",
                "following_block": "equal",
                "adjacent_blocks": "equal",
                "byte_window_64": "equal",
            },
            "mechanical_failure_on_violation": True,
        },
        "same-locator": True,
        "observed_target_duplicate_ordinal": 0,
        "survivor_duplicate_ordinal": 1,
        "require_reconstructed_current_document_sha256": "c9dfe514e84de3773fbd73fc48e7d9504c024a6852a601dc77d564b0611d4070",
    },
}
EXPECTED_WRONG_TARGET_ORDINALS = {
    case_id: (
        contract["observed_target_duplicate_ordinal"],
        contract["survivor_duplicate_ordinal"],
    )
    for case_id, contract in EXPECTED_CASE_CONTRACTS.items()
    if "observed_target_duplicate_ordinal" in contract
}
EXPECTED_RECONSTRUCTION_FAMILY = {
    "block-forward-survivor-same-locator-duplicate-substitution": "D[0:a] + D[c:]",
    "block-backward-survivor-same-locator-duplicate-substitution": "D[0:a] + I + D[a:b] + D[d:]",
    "block-cloned-adjacent-blocks-distinguishing-byte-window": "D[0:a] + D[c:]",
    "block-byte-identical-bounded-neighborhood-duplicate-substitution": "D[0:a] + D[c:]",
}
EXPECTED_CANDIDATE_PAYLOAD_FIELDS = {
    "preceding_block": (
        "target_bytes",
        "preceding_boundary_state",
        "preceding_block_bytes",
    ),
    "following_block": (
        "target_bytes",
        "following_boundary_state",
        "following_block_bytes",
    ),
    "adjacent_blocks": (
        "target_bytes",
        "preceding_boundary_state",
        "preceding_block_bytes",
        "following_boundary_state",
        "following_block_bytes",
    ),
    "byte_window_64": (
        "target_bytes",
        "prefix_window_bytes",
        "prefix_hits_bof",
        "suffix_window_bytes",
        "suffix_hits_eof",
    ),
}
EXPECTED_CASE_RESULT_KEYS = (
    "case_class",
    "case_id",
    "candidate_results",
    "descriptor_evidence",
    "identity_truth",
    "lineage_evidence",
    "mechanical_proofs",
    "reported_command_vectors",
    "schema_version",
    "target_bytes_evidence",
)
BLOCK_ENTRY_KEYS = {"etag", "index", "kind", "preview", "span"}
BLOCKS_RESULT_KEYS = {"blocks", "file", "schema_version"}
MANIFEST_TOP_LEVEL_KEYS = {
    "candidate_order",
    "cases",
    "date_locked",
    "manifest_kind",
    "protocol_sha256",
    "required_case_ids",
    "schema_version",
}
MECHANICAL_PRECONDITION_KEYS = {
    "candidate_context_relations",
    "current_exact_target_match_count",
    "document_bytes_relation",
    "live_descriptor_relation",
    "mechanical_failure_on_violation",
    "observed_exact_target_match_count",
    "target_bytes_relation",
}
TARGET_SELECTOR_KEYS = {"block_index"}
SPAN_KEYS = {"byte_end", "byte_start", "line_end", "line_start"}


class ProbeError(RuntimeError):
    pass


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the local block-only position-bound target-identity probe against an "
            "operator-designated md binary authenticated by a supplied byte digest."
        )
    )
    parser.add_argument(
        "--md-binary",
        type=Path,
        required=True,
        help="Repository-local path to the operator-designated md binary bytes to authenticate.",
    )
    parser.add_argument(
        "--md-binary-sha256",
        required=True,
        help=(
            "Expected SHA-256 for the exact operator-designated md binary bytes only; "
            "this authenticates those bytes and does not attest provenance."
        ),
    )
    parser.add_argument(
        "--check",
        type=Path,
        help=(
            "Repository-local path to an existing file for byte-comparison without "
            "rewriting it."
        ),
    )
    parser.add_argument(
        "--output",
        type=Path,
        help=(
            "Repository-local path where canonical JSON is written by atomic "
            "same-directory replacement."
        ),
    )
    args = parser.parse_args(argv)
    if args.check is not None and args.output is not None:
        parser.error("--check and --output are mutually exclusive")
    return args


def main(argv: list[str] | None = None) -> int:
    try:
        args = parse_args(argv)
        report_bytes = build_report_bytes(args.md_binary, args.md_binary_sha256)
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


def build_report_bytes(md_binary_arg: Path, expected_md_binary_sha256: str) -> bytes:
    md_binary, md_binary_sha256 = ensure_md_binary(md_binary_arg, expected_md_binary_sha256)
    manifest = load_manifest()
    with TemporaryDirectory(prefix="position-bound-target-identity-") as workspace_str:
        workspace = Path(workspace_str)
        case_reports = [
            evaluate_case(case, md_binary, md_binary_sha256, workspace)
            for case in manifest["cases"]
        ]
    candidate_summary = build_candidate_summary(case_reports)
    aggregate_verdict = build_aggregate_verdict(candidate_summary)
    report = {
        "aggregate_verdict": aggregate_verdict,
        "candidate_order": list(EXPECTED_CANDIDATE_ORDER),
        "candidate_summary": candidate_summary,
        "cases": case_reports,
        "descriptor_schema_version": CANONICAL_DESCRIPTOR_SCHEMA_VERSION,
        "manifest_path": "cases.json",
        "manifest_semantic_sha256": manifest["manifest_semantic_sha256"],
        "manifest_schema_version": manifest["schema_version"],
        "md_binary_sha256": md_binary_sha256,
        "protocol_sha256": EXPECTED_PROTOCOL_SHA256,
        "protocol_version": REPORT_SCHEMA_VERSION,
        "report_schema_version": REPORT_SCHEMA_VERSION,
        "required_case_ids": list(EXPECTED_CASE_ORDER),
        "schema_version": PROBE_SCHEMA_VERSION,
    }
    return canonical_json_bytes(report)


def ensure_md_binary(md_binary_arg: Path, expected_sha256: str) -> tuple[Path, str]:
    # This digest authenticates only the operator-designated file bytes at the resolved
    # repository-local path. It does not establish provenance, publisher identity, or trust.
    if SHA256_PATTERN.fullmatch(expected_sha256) is None:
        raise ProbeError("--md-binary-sha256 must be exactly 64 lowercase hexadecimal characters")
    md_binary = resolve_repo_local_path(md_binary_arg, "md binary", strict=True)
    if not md_binary.is_file():
        raise ProbeError(f"md binary is not a regular file: {md_binary_arg}")
    if not os.access(md_binary, os.X_OK):
        raise ProbeError(f"md binary is not executable: {md_binary_arg}")
    try:
        actual_sha256 = sha256_hex(md_binary.read_bytes())
    except OSError as exc:
        raise ProbeError(f"failed to read md binary bytes: {md_binary_arg}") from exc
    if actual_sha256 != expected_sha256:
        raise ProbeError(
            f"md binary SHA-256 mismatch: expected {expected_sha256}, got {actual_sha256}"
        )
    return md_binary, actual_sha256


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
    exact_key_set(manifest_map, MANIFEST_TOP_LEVEL_KEYS, "manifest")
    manifest_semantic_sha256 = manifest_semantic_sha256_hex(manifest_map)
    if manifest_semantic_sha256 != EXPECTED_MANIFEST_SEMANTIC_SHA256:
        raise ProbeError(
            f"{MANIFEST_PATH.name} canonical semantic digest mismatch: "
            f"expected {EXPECTED_MANIFEST_SEMANTIC_SHA256}, got {manifest_semantic_sha256}"
        )
    schema_version = expect_exact_string(
        manifest_map.get("schema_version"),
        "manifest.schema_version",
        EXPECTED_MANIFEST_SCHEMA_VERSION,
    )
    expect_exact_string(
        manifest_map.get("manifest_kind"),
        "manifest.manifest_kind",
        EXPECTED_MANIFEST_KIND,
    )
    expect_exact_string(
        manifest_map.get("date_locked"),
        "manifest.date_locked",
        EXPECTED_DATE_LOCKED,
    )
    expect_exact_string(
        manifest_map.get("protocol_sha256"),
        "manifest.protocol_sha256",
        EXPECTED_PROTOCOL_SHA256,
    )
    candidate_order = expect_list(manifest_map.get("candidate_order"), "manifest.candidate_order")
    if tuple(expect_string(value, f"manifest.candidate_order[{index}]") for index, value in enumerate(candidate_order)) != EXPECTED_CANDIDATE_ORDER:
        raise ProbeError("manifest.candidate_order must match the runner-owned candidate order exactly")
    required_case_ids = expect_list(
        manifest_map.get("required_case_ids"),
        "manifest.required_case_ids",
    )
    if tuple(expect_string(value, f"manifest.required_case_ids[{index}]") for index, value in enumerate(required_case_ids)) != EXPECTED_CASE_ORDER:
        raise ProbeError("manifest.required_case_ids must match the runner-owned case order exactly")
    case_values = expect_list(manifest_map.get("cases"), "manifest.cases")
    if len(case_values) != len(EXPECTED_CASE_ORDER):
        raise ProbeError("manifest.cases must contain the exact runner-owned ten-case matrix")
    cases = []
    for index, case_value in enumerate(case_values):
        cases.append(validate_case(case_value, index))
    return {
        "cases": cases,
        "manifest_semantic_sha256": manifest_semantic_sha256,
        "schema_version": schema_version,
    }


def validate_case(case_value: Any, index: int) -> dict[str, Any]:
    case = expect_mapping(case_value, f"manifest.cases[{index}]")
    case_id = expect_string(case.get("case_id"), f"manifest.cases[{index}].case_id")
    if CASE_ID_PATTERN.fullmatch(case_id) is None:
        raise ProbeError(f"invalid manifest case_id: {case_id}")
    expected_case_id = EXPECTED_CASE_ORDER[index]
    if case_id != expected_case_id:
        raise ProbeError(
            f"manifest case order mismatch at index {index}: expected {expected_case_id!r}, got {case_id!r}"
        )
    contract = EXPECTED_CASE_CONTRACTS[case_id]
    expected_keys = {
        "case_class",
        "case_id",
        "current_document_utf8",
        "current_target_selector",
        "identity_truth",
        "mechanical_preconditions",
        "observed_document_utf8",
        "observed_target_selector",
    }
    if "same-locator" in contract:
        expected_keys.update(
            {
                "observed_target_duplicate_ordinal",
                "require_reconstructed_current_document_sha256",
                "same-locator",
                "survivor_duplicate_ordinal",
            }
        )
    if "backward_prefix_insertion_bytes" in contract:
        expected_keys.add("backward_prefix_insertion_bytes")
    exact_key_set(case, expected_keys, case_id)
    normalized = {
        "case_class": expect_exact_string(case.get("case_class"), f"{case_id}.case_class", contract["case_class"]),
        "case_id": case_id,
        "current_document_utf8": expect_string(
            case.get("current_document_utf8"),
            f"{case_id}.current_document_utf8",
        ),
        "current_target_selector": validate_target_selector(
            case_id,
            "current_target_selector",
            case.get("current_target_selector"),
            contract["current_target_selector"],
        ),
        "identity_truth": expect_exact_string(
            case.get("identity_truth"),
            f"{case_id}.identity_truth",
            contract["identity_truth"],
        ),
        "mechanical_preconditions": validate_mechanical_preconditions(
            case_id,
            case.get("mechanical_preconditions"),
            contract["mechanical_preconditions"],
        ),
        "observed_document_utf8": expect_string(
            case.get("observed_document_utf8"),
            f"{case_id}.observed_document_utf8",
        ),
        "observed_target_selector": validate_target_selector(
            case_id,
            "observed_target_selector",
            case.get("observed_target_selector"),
            contract["observed_target_selector"],
        ),
    }
    if "same-locator" in contract:
        normalized["same-locator"] = expect_bool(case.get("same-locator"), f"{case_id}.same-locator")
        if normalized["same-locator"] != contract["same-locator"]:
            raise ProbeError(f"{case_id}.same-locator must be the literal true runner-owned contract value")
        normalized["observed_target_duplicate_ordinal"] = expect_nonnegative_int(
            case.get("observed_target_duplicate_ordinal"),
            f"{case_id}.observed_target_duplicate_ordinal",
        )
        normalized["survivor_duplicate_ordinal"] = expect_nonnegative_int(
            case.get("survivor_duplicate_ordinal"),
            f"{case_id}.survivor_duplicate_ordinal",
        )
        normalized["require_reconstructed_current_document_sha256"] = expect_sha256_string(
            case.get("require_reconstructed_current_document_sha256"),
            f"{case_id}.require_reconstructed_current_document_sha256",
        )
        expected_observed_ordinal, expected_survivor_ordinal = EXPECTED_WRONG_TARGET_ORDINALS[case_id]
        if normalized["observed_target_duplicate_ordinal"] != expected_observed_ordinal:
            raise ProbeError(
                f"{case_id}.observed_target_duplicate_ordinal must be {expected_observed_ordinal}"
            )
        if normalized["survivor_duplicate_ordinal"] != expected_survivor_ordinal:
            raise ProbeError(
                f"{case_id}.survivor_duplicate_ordinal must be {expected_survivor_ordinal}"
            )
        if (
            normalized["require_reconstructed_current_document_sha256"]
            != contract["require_reconstructed_current_document_sha256"]
        ):
            raise ProbeError(
                f"{case_id}.require_reconstructed_current_document_sha256 does not match the runner-owned contract"
            )
    if "backward_prefix_insertion_bytes" in contract:
        insertion_bytes = expect_string(
            case.get("backward_prefix_insertion_bytes"),
            f"{case_id}.backward_prefix_insertion_bytes",
        )
        if insertion_bytes != contract["backward_prefix_insertion_bytes"]:
            raise ProbeError(f"{case_id}.backward_prefix_insertion_bytes does not match the runner-owned contract")
        normalized["backward_prefix_insertion_bytes"] = insertion_bytes
    return normalized


def validate_target_selector(
    case_id: str,
    field_name: str,
    selector_value: Any,
    expected_selector: dict[str, int],
) -> dict[str, int]:
    selector = expect_mapping(selector_value, f"{case_id}.{field_name}")
    exact_key_set(selector, TARGET_SELECTOR_KEYS, f"{case_id}.{field_name}")
    block_index = expect_nonnegative_int(selector.get("block_index"), f"{case_id}.{field_name}.block_index")
    if block_index != expected_selector["block_index"]:
        raise ProbeError(
            f"{case_id}.{field_name}.block_index must be {expected_selector['block_index']}"
        )
    return {"block_index": block_index}


def validate_mechanical_preconditions(
    case_id: str,
    value: Any,
    expected: dict[str, Any],
) -> dict[str, Any]:
    preconditions = expect_mapping(value, f"{case_id}.mechanical_preconditions")
    exact_key_set(preconditions, MECHANICAL_PRECONDITION_KEYS, f"{case_id}.mechanical_preconditions")
    candidate_context_relations = expect_mapping(
        preconditions.get("candidate_context_relations"),
        f"{case_id}.mechanical_preconditions.candidate_context_relations",
    )
    exact_key_set(
        candidate_context_relations,
        set(EXPECTED_CANDIDATE_ORDER),
        f"{case_id}.mechanical_preconditions.candidate_context_relations",
    )
    normalized = {
        "target_bytes_relation": expect_choice(
            preconditions.get("target_bytes_relation"),
            f"{case_id}.mechanical_preconditions.target_bytes_relation",
            EXPECTED_RELATIONS,
        ),
        "live_descriptor_relation": expect_choice(
            preconditions.get("live_descriptor_relation"),
            f"{case_id}.mechanical_preconditions.live_descriptor_relation",
            EXPECTED_RELATIONS,
        ),
        "document_bytes_relation": expect_choice(
            preconditions.get("document_bytes_relation"),
            f"{case_id}.mechanical_preconditions.document_bytes_relation",
            EXPECTED_RELATIONS,
        ),
        "observed_exact_target_match_count": expect_nonnegative_int(
            preconditions.get("observed_exact_target_match_count"),
            f"{case_id}.mechanical_preconditions.observed_exact_target_match_count",
        ),
        "current_exact_target_match_count": expect_nonnegative_int(
            preconditions.get("current_exact_target_match_count"),
            f"{case_id}.mechanical_preconditions.current_exact_target_match_count",
        ),
        "candidate_context_relations": {
            candidate_name: expect_choice(
                candidate_context_relations.get(candidate_name),
                f"{case_id}.mechanical_preconditions.candidate_context_relations.{candidate_name}",
                EXPECTED_RELATIONS,
            )
            for candidate_name in EXPECTED_CANDIDATE_ORDER
        },
        "mechanical_failure_on_violation": expect_bool(
            preconditions.get("mechanical_failure_on_violation"),
            f"{case_id}.mechanical_preconditions.mechanical_failure_on_violation",
        ),
    }
    if normalized != expected:
        raise ProbeError(f"{case_id}.mechanical_preconditions does not match the runner-owned contract")
    return normalized


def evaluate_case(
    case: dict[str, Any],
    md_binary: Path,
    md_binary_sha256: str,
    workspace: Path,
) -> dict[str, Any]:
    case_id = case["case_id"]
    case_dir = workspace / case_id
    case_dir.mkdir(parents=True, exist_ok=True)
    observed_path = (case_dir / "observed.md").resolve()
    current_path = (case_dir / "current.md").resolve()
    observed_document_bytes = case["observed_document_utf8"].encode("utf-8")
    current_document_bytes = case["current_document_utf8"].encode("utf-8")
    write_bytes(observed_path, observed_document_bytes)
    write_bytes(current_path, current_document_bytes)

    observed_blocks = run_blocks_query(
        md_binary,
        observed_path,
        workspace,
        f"{case_id}.observed_blocks",
        observed_document_bytes,
    )
    current_blocks = run_blocks_query(
        md_binary,
        current_path,
        workspace,
        f"{case_id}.current_blocks",
        current_document_bytes,
    )
    observed_context = resolve_target_context(
        case_id,
        observed_blocks,
        case["observed_target_selector"]["block_index"],
        observed_document_bytes,
        "observed",
    )
    current_context = resolve_target_context(
        case_id,
        current_blocks,
        case["current_target_selector"]["block_index"],
        current_document_bytes,
        "current",
    )
    observed_matches = exact_target_matches(observed_blocks, observed_context["target_bytes"])
    current_matches = exact_target_matches(current_blocks, observed_context["target_bytes"])
    context_relations = derive_candidate_context_relations(observed_context, current_context)
    measured_relations = {
        "target_bytes_relation": equality_relation(
            observed_context["target_bytes"],
            current_context["target_bytes"],
        ),
        "live_descriptor_relation": equality_relation(
            canonical_descriptor_bytes(observed_context["descriptor"]),
            canonical_descriptor_bytes(current_context["descriptor"]),
        ),
        "document_bytes_relation": equality_relation(
            observed_document_bytes,
            current_document_bytes,
        ),
        "observed_exact_target_match_count": len(observed_matches),
        "current_exact_target_match_count": len(current_matches),
        "candidate_context_relations": context_relations,
    }
    enforce_mechanical_preconditions(case, measured_relations)
    boundary_proofs = build_boundary_proofs(
        case,
        observed_context,
        current_context,
        observed_document_bytes,
        current_document_bytes,
    )
    lineage_evidence = build_lineage_evidence(
        case,
        observed_context,
        current_context,
        observed_matches,
        current_matches,
        observed_document_bytes,
        current_document_bytes,
    )
    candidate_results = build_candidate_results(
        case,
        observed_context,
        current_context,
    )
    result = {
        "candidate_results": candidate_results,
        "case_class": case["case_class"],
        "case_id": case_id,
        "descriptor_evidence": {
            "current": descriptor_evidence_record(current_context),
            "observed": descriptor_evidence_record(observed_context),
        },
        "identity_truth": case["identity_truth"],
        "lineage_evidence": lineage_evidence,
        "mechanical_proofs": {
            "block_domain_start": {
                "current": current_context["block_domain_start"],
                "observed": observed_context["block_domain_start"],
            },
            "boundary_proofs": boundary_proofs,
            "candidate_context_relations": context_relations,
            "measured_relations": measured_relations,
        },
        "reported_command_vectors": {
            "current": build_report_command_vector(md_binary, case_id, "current"),
            "observed": build_report_command_vector(md_binary, case_id, "observed"),
        },
        "schema_version": PROBE_SCHEMA_VERSION,
        "target_bytes_evidence": {
            "current": payload_record(current_context["target_bytes"]),
            "observed": payload_record(observed_context["target_bytes"]),
        },
    }
    exact_key_set(result, set(EXPECTED_CASE_RESULT_KEYS), f"{case_id}.result")
    return result


def run_blocks_query(
    md_binary: Path,
    document_path: Path,
    workspace: Path,
    where: str,
    document_bytes: bytes,
) -> list[dict[str, Any]]:
    outcome = run_md_command(
        md_binary,
        [str(md_binary), "blocks", str(document_path), "--json"],
        workspace,
        where,
    )
    if outcome["returncode"] != 0:
        stderr_text = decode_utf8(outcome["stderr"], f"{where}.stderr").strip()
        raise ProbeError(f"{where}: md exited with code {outcome['returncode']}: {stderr_text}")
    stdout_text = decode_utf8(outcome["stdout"], f"{where}.stdout")
    try:
        result = json.loads(stdout_text)
    except json.JSONDecodeError as exc:
        raise ProbeError(f"{where}: invalid md JSON: {exc}") from exc
    result_map = expect_mapping(result, f"{where}.result")
    exact_key_set(result_map, BLOCKS_RESULT_KEYS, f"{where}.result")
    expect_exact_string(
        result_map.get("schema_version"),
        f"{where}.result.schema_version",
        EXPECTED_MD_SCHEMA_VERSION,
    )
    expect_string(result_map.get("file"), f"{where}.result.file")
    block_values = expect_list(result_map.get("blocks"), f"{where}.result.blocks")
    if not block_values:
        raise ProbeError(f"{where}: md blocks returned no top-level blocks")
    blocks = []
    seen_indices: set[int] = set()
    for index, entry_value in enumerate(block_values):
        entry = expect_mapping(entry_value, f"{where}.result.blocks[{index}]")
        exact_key_set(entry, BLOCK_ENTRY_KEYS, f"{where}.result.blocks[{index}]")
        block_index = expect_nonnegative_int(entry.get("index"), f"{where}.result.blocks[{index}].index")
        if block_index in seen_indices:
            raise ProbeError(f"{where}: duplicate block index {block_index}")
        seen_indices.add(block_index)
        kind = expect_string(entry.get("kind"), f"{where}.result.blocks[{index}].kind")
        span = normalize_span(entry.get("span"), f"{where}.result.blocks[{index}].span")
        expect_string(entry.get("etag"), f"{where}.result.blocks[{index}].etag")
        expect_string(entry.get("preview"), f"{where}.result.blocks[{index}].preview")
        target_bytes = slice_target_bytes(document_bytes, span, f"{where}.result.blocks[{index}]")
        blocks.append(
            {
                "descriptor": {
                    "index": block_index,
                    "kind": kind,
                    "span": span,
                },
                "target_bytes": target_bytes,
            }
        )
    return blocks


def resolve_target_context(
    case_id: str,
    blocks: list[dict[str, Any]],
    target_block_index: int,
    document_bytes: bytes,
    label: str,
) -> dict[str, Any]:
    block_domain_start = min(
        block["descriptor"]["span"]["byte_start"] for block in blocks
    )
    for position, block in enumerate(blocks):
        if block["descriptor"]["index"] == target_block_index:
            preceding = blocks[position - 1] if position > 0 else None
            following = blocks[position + 1] if position + 1 < len(blocks) else None
            span = block["descriptor"]["span"]
            byte_start = span["byte_start"]
            byte_end = span["byte_end"]
            prefix_start = max(block_domain_start, byte_start - 64)
            suffix_end = min(len(document_bytes), byte_end + 64)
            return {
                "block_domain_start": block_domain_start,
                "descriptor": block["descriptor"],
                "following_block_bytes": following["target_bytes"] if following is not None else b"",
                "following_boundary_state": "present" if following is not None else "eof",
                "following_descriptor": following["descriptor"] if following is not None else None,
                "label": label,
                "prefix_hits_bof": prefix_start == block_domain_start,
                "prefix_window_bytes": document_bytes[prefix_start:byte_start],
                "preceding_block_bytes": preceding["target_bytes"] if preceding is not None else b"",
                "preceding_boundary_state": "present" if preceding is not None else "bof",
                "preceding_descriptor": preceding["descriptor"] if preceding is not None else None,
                "suffix_hits_eof": suffix_end == len(document_bytes),
                "suffix_window_bytes": document_bytes[byte_end:suffix_end],
                "target_bytes": block["target_bytes"],
            }
    raise ProbeError(f"{case_id}.{label}: target block_index {target_block_index} was not found in live blocks")


def exact_target_matches(blocks: list[dict[str, Any]], target_bytes: bytes) -> list[dict[str, Any]]:
    matches = [block for block in blocks if block["target_bytes"] == target_bytes]
    return sorted(matches, key=lambda block: block["descriptor"]["span"]["byte_start"])


def derive_candidate_context_relations(
    observed_context: dict[str, Any],
    current_context: dict[str, Any],
) -> dict[str, str]:
    preceding_block = equality_relation(
        boundary_state_bytes(observed_context["preceding_boundary_state"])
        + observed_context["preceding_block_bytes"],
        boundary_state_bytes(current_context["preceding_boundary_state"])
        + current_context["preceding_block_bytes"],
    )
    following_block = equality_relation(
        boundary_state_bytes(observed_context["following_boundary_state"])
        + observed_context["following_block_bytes"],
        boundary_state_bytes(current_context["following_boundary_state"])
        + current_context["following_block_bytes"],
    )
    adjacent_blocks = equality_relation(
        boundary_state_bytes(observed_context["preceding_boundary_state"])
        + observed_context["preceding_block_bytes"]
        + boundary_state_bytes(observed_context["following_boundary_state"])
        + observed_context["following_block_bytes"],
        boundary_state_bytes(current_context["preceding_boundary_state"])
        + current_context["preceding_block_bytes"]
        + boundary_state_bytes(current_context["following_boundary_state"])
        + current_context["following_block_bytes"],
    )
    byte_window_64 = equality_relation(
        observed_context["prefix_window_bytes"]
        + boolean_payload(observed_context["prefix_hits_bof"])
        + observed_context["suffix_window_bytes"]
        + boolean_payload(observed_context["suffix_hits_eof"]),
        current_context["prefix_window_bytes"]
        + boolean_payload(current_context["prefix_hits_bof"])
        + current_context["suffix_window_bytes"]
        + boolean_payload(current_context["suffix_hits_eof"]),
    )
    return {
        "preceding_block": preceding_block,
        "following_block": following_block,
        "adjacent_blocks": adjacent_blocks,
        "byte_window_64": byte_window_64,
    }


def enforce_mechanical_preconditions(case: dict[str, Any], measured: dict[str, Any]) -> None:
    expected = case["mechanical_preconditions"]
    failures: list[str] = []
    for key in (
        "target_bytes_relation",
        "live_descriptor_relation",
        "document_bytes_relation",
        "observed_exact_target_match_count",
        "current_exact_target_match_count",
    ):
        if measured[key] != expected[key]:
            failures.append(f"{key}: expected {expected[key]!r}, got {measured[key]!r}")
    for candidate_name in EXPECTED_CANDIDATE_ORDER:
        actual_relation = measured["candidate_context_relations"][candidate_name]
        expected_relation = expected["candidate_context_relations"][candidate_name]
        if actual_relation != expected_relation:
            failures.append(
                f"candidate_context_relations.{candidate_name}: expected {expected_relation!r}, got {actual_relation!r}"
            )
    if failures and expected["mechanical_failure_on_violation"]:
        raise ProbeError(f"{case['case_id']}: mechanical preconditions failed: {'; '.join(failures)}")


def build_boundary_proofs(
    case: dict[str, Any],
    observed_context: dict[str, Any],
    current_context: dict[str, Any],
    observed_document_bytes: bytes,
    current_document_bytes: bytes,
) -> dict[str, Any]:
    observed_span = observed_context["descriptor"]["span"]
    current_span = current_context["descriptor"]["span"]
    diff_bounds = difference_bounds(observed_document_bytes, current_document_bytes)
    proofs = {
        "current_span": current_span,
        "observed_span": observed_span,
        "prefix_window_bytes": {
            "current_sha256": sha256_hex(current_context["prefix_window_bytes"]),
            "observed_sha256": sha256_hex(observed_context["prefix_window_bytes"]),
        },
        "suffix_window_bytes": {
            "current_sha256": sha256_hex(current_context["suffix_window_bytes"]),
            "observed_sha256": sha256_hex(observed_context["suffix_window_bytes"]),
        },
        "target_boundaries": {
            "current": current_span,
            "observed": observed_span,
        },
    }
    case_class = case["case_class"]
    if case_class == "outside_context_unrelated_edit":
        if diff_bounds is None:
            raise ProbeError(f"{case['case_id']}: expected a document difference for outside-context proof")
        proofs["outside_context_difference"] = prove_outside_context_difference(
            case["case_id"],
            diff_bounds,
            observed_context,
            current_context,
        )
    if case_class == "preceding_neighbor_edit_inside_prefix_window":
        if diff_bounds is None:
            raise ProbeError(f"{case['case_id']}: expected a document difference for prefix-neighbor proof")
        proofs["neighbor_edit"] = prove_neighbor_edit(
            case["case_id"],
            "preceding",
            diff_bounds,
            observed_context,
            current_context,
        )
    if case_class == "following_neighbor_edit_inside_suffix_window":
        if diff_bounds is None:
            raise ProbeError(f"{case['case_id']}: expected a document difference for suffix-neighbor proof")
        proofs["neighbor_edit"] = prove_neighbor_edit(
            case["case_id"],
            "following",
            diff_bounds,
            observed_context,
            current_context,
        )
    return proofs


def prove_outside_context_difference(
    case_id: str,
    diff_bounds: dict[str, int],
    observed_context: dict[str, Any],
    current_context: dict[str, Any],
) -> dict[str, Any]:
    observed_intervals = relevant_intervals(observed_context)
    current_intervals = relevant_intervals(current_context)
    observed_change = (diff_bounds["observed_start"], diff_bounds["observed_end"])
    current_change = (diff_bounds["current_start"], diff_bounds["current_end"])
    observed_outside = all(
        not intervals_overlap(observed_change, interval)
        for interval in observed_intervals.values()
    )
    current_outside = all(
        not intervals_overlap(current_change, interval)
        for interval in current_intervals.values()
    )
    if not observed_outside or not current_outside:
        raise ProbeError(f"{case_id}: outside-context difference intersects the target-adjacent bounded domain")
    return {
        "current_change": {"end": current_change[1], "start": current_change[0]},
        "observed_change": {"end": observed_change[1], "start": observed_change[0]},
    }


def prove_neighbor_edit(
    case_id: str,
    direction: str,
    diff_bounds: dict[str, int],
    observed_context: dict[str, Any],
    current_context: dict[str, Any],
) -> dict[str, Any]:
    observed_change = (diff_bounds["observed_start"], diff_bounds["observed_end"])
    current_change = (diff_bounds["current_start"], diff_bounds["current_end"])
    if direction == "preceding":
        observed_descriptor = observed_context["preceding_descriptor"]
        current_descriptor = current_context["preceding_descriptor"]
        observed_window = (
            observed_context["descriptor"]["span"]["byte_start"] - len(observed_context["prefix_window_bytes"]),
            observed_context["descriptor"]["span"]["byte_start"],
        )
        current_window = (
            current_context["descriptor"]["span"]["byte_start"] - len(current_context["prefix_window_bytes"]),
            current_context["descriptor"]["span"]["byte_start"],
        )
    else:
        observed_descriptor = observed_context["following_descriptor"]
        current_descriptor = current_context["following_descriptor"]
        observed_window = (
            observed_context["descriptor"]["span"]["byte_end"],
            observed_context["descriptor"]["span"]["byte_end"] + len(observed_context["suffix_window_bytes"]),
        )
        current_window = (
            current_context["descriptor"]["span"]["byte_end"],
            current_context["descriptor"]["span"]["byte_end"] + len(current_context["suffix_window_bytes"]),
        )
    if observed_descriptor is None or current_descriptor is None:
        raise ProbeError(f"{case_id}: {direction} neighbor proof requires a live adjacent block")
    observed_neighbor = interval_from_span(observed_descriptor["span"])
    current_neighbor = interval_from_span(current_descriptor["span"])
    if not interval_contains(observed_neighbor, observed_change) or not interval_contains(current_neighbor, current_change):
        raise ProbeError(f"{case_id}: changed bytes do not stay inside the required {direction} adjacent block")
    if not interval_contains(observed_window, observed_change) or not interval_contains(current_window, current_change):
        raise ProbeError(f"{case_id}: changed bytes do not stay inside the required {direction} 64-byte window")
    return {
        "current_change": {"end": current_change[1], "start": current_change[0]},
        "current_neighbor": {"end": current_neighbor[1], "start": current_neighbor[0]},
        "observed_change": {"end": observed_change[1], "start": observed_change[0]},
        "observed_neighbor": {"end": observed_neighbor[1], "start": observed_neighbor[0]},
    }


def build_lineage_evidence(
    case: dict[str, Any],
    observed_context: dict[str, Any],
    current_context: dict[str, Any],
    observed_matches: list[dict[str, Any]],
    current_matches: list[dict[str, Any]],
    observed_document_bytes: bytes,
    current_document_bytes: bytes,
) -> dict[str, Any] | None:
    if case["identity_truth"] != "wrong_target":
        return None
    case_id = case["case_id"]
    if len(observed_matches) != 2:
        raise ProbeError(f"{case_id}: wrong-target lineage requires exactly two observed exact-target matches")
    if len(current_matches) != 1:
        raise ProbeError(f"{case_id}: wrong-target lineage requires exactly one current exact-target match")
    duplicate_0 = observed_matches[0]
    duplicate_1 = observed_matches[1]
    span0 = duplicate_0["descriptor"]["span"]
    span1 = duplicate_1["descriptor"]["span"]
    a = span0["byte_start"]
    b = span0["byte_end"]
    c = span1["byte_start"]
    d = span1["byte_end"]
    if not (a < b <= c < d):
        raise ProbeError(f"{case_id}: duplicate ordering must satisfy a < b <= c < d")
    if duplicate_0["target_bytes"] != duplicate_1["target_bytes"]:
        raise ProbeError(f"{case_id}: observed duplicates must have byte-identical target content")
    if b - a != d - c:
        raise ProbeError(f"{case_id}: observed duplicates must have equal target byte lengths")
    observed_target_ordinal = case["observed_target_duplicate_ordinal"]
    survivor_ordinal = case["survivor_duplicate_ordinal"]
    observed_target_descriptor = observed_context["descriptor"]
    expected_observed_descriptor = duplicate_0["descriptor"] if observed_target_ordinal == 0 else duplicate_1["descriptor"]
    if observed_target_descriptor != expected_observed_descriptor:
        raise ProbeError(f"{case_id}: observed target selector does not resolve the required duplicate ordinal")
    current_target_descriptor = current_context["descriptor"]
    current_match_descriptor = current_matches[0]["descriptor"]
    if current_target_descriptor != current_match_descriptor:
        raise ProbeError(f"{case_id}: live current target must resolve from the lone current exact-target match")
    if case_id == "block-backward-survivor-same-locator-duplicate-substitution":
        insertion_bytes = case["backward_prefix_insertion_bytes"].encode("utf-8")
        if len(insertion_bytes) != c - a:
            raise ProbeError(f"{case_id}: len(I) must equal c - a for the backward-survivor reconstruction")
        if contains_subsequence(insertion_bytes, duplicate_0["target_bytes"]):
            raise ProbeError(f"{case_id}: backward_prefix_insertion_bytes must contain zero exact target-byte matches")
        reconstructed_current_document = observed_document_bytes[0:a] + insertion_bytes + observed_document_bytes[a:b] + observed_document_bytes[d:]
        expected_current_descriptor = duplicate_1["descriptor"]
    else:
        reconstructed_current_document = observed_document_bytes[0:a] + observed_document_bytes[c:]
        expected_current_descriptor = duplicate_0["descriptor"]
    reconstructed_sha256 = sha256_hex(reconstructed_current_document)
    if reconstructed_sha256 != case["require_reconstructed_current_document_sha256"]:
        raise ProbeError(f"{case_id}: reconstructed current document SHA-256 does not match the runner-owned contract")
    if current_document_bytes != reconstructed_current_document:
        raise ProbeError(f"{case_id}: current document bytes do not equal the required lineage reconstruction")
    if current_target_descriptor != expected_current_descriptor:
        raise ProbeError(f"{case_id}: live current target descriptor does not satisfy the same-locator lineage rule")
    return {
        "duplicate_ordinals": {
            "observed_target_duplicate_ordinal": observed_target_ordinal,
            "survivor_duplicate_ordinal": survivor_ordinal,
        },
        "duplicate_spans": {
            "duplicate_0": span0,
            "duplicate_1": span1,
        },
        "reconstruction_family": EXPECTED_RECONSTRUCTION_FAMILY[case_id],
        "reconstructed_current_document_sha256": reconstructed_sha256,
        "same-locator": True,
    }


def build_candidate_results(
    case: dict[str, Any],
    observed_context: dict[str, Any],
    current_context: dict[str, Any],
) -> dict[str, Any]:
    results: dict[str, Any] = {}
    for candidate_name in EXPECTED_CANDIDATE_ORDER:
        observed_preimage = token_preimage(candidate_name, observed_context)
        current_preimage = token_preimage(candidate_name, current_context)
        decision = "accept" if observed_preimage == current_preimage else "reject"
        credit = derive_credit(case["identity_truth"], decision)
        results[candidate_name] = {
            "credit": expect_choice(credit, f"{case['case_id']}.{candidate_name}.credit", EXPECTED_CREDITS),
            "current_token_sha256": sha256_hex(current_preimage),
            "decision": expect_choice(
                decision,
                f"{case['case_id']}.{candidate_name}.decision",
                EXPECTED_CANDIDATE_DECISIONS,
            ),
            "observed_token_sha256": sha256_hex(observed_preimage),
            "token_byte_equality": observed_preimage == current_preimage,
        }
    return results


def build_candidate_summary(case_reports: list[dict[str, Any]]) -> dict[str, Any]:
    summary: dict[str, Any] = {}
    for candidate_name in EXPECTED_CANDIDATE_ORDER:
        accepts = 0
        rejects = 0
        wrong_identity_accepts = 0
        same_target_rejects = 0
        neighbor_false_conflicts = 0
        for case_report in case_reports:
            result = case_report["candidate_results"][candidate_name]
            if result["decision"] == "accept":
                accepts += 1
            else:
                rejects += 1
            if result["credit"] == "wrong_identity":
                wrong_identity_accepts += 1
            if result["credit"] == "false_conflict":
                same_target_rejects += 1
                if case_report["case_class"] in {
                    "preceding_neighbor_edit_inside_prefix_window",
                    "following_neighbor_edit_inside_suffix_window",
                }:
                    neighbor_false_conflicts += 1
        if wrong_identity_accepts:
            graduation_verdict = "demoted_wrong_identity"
        elif same_target_rejects:
            graduation_verdict = "demoted_same_target_reject"
        else:
            graduation_verdict = "graduates"
        expect_choice(
            graduation_verdict,
            f"candidate_summary.{candidate_name}.graduation_verdict",
            EXPECTED_GRADUATION_VERDICTS,
        )
        summary[candidate_name] = {
            "accepts": accepts,
            "disposition": "promote" if graduation_verdict == "graduates" else "demote",
            "graduation_verdict": graduation_verdict,
            "neighbor_false_conflicts": neighbor_false_conflicts,
            "rejects": rejects,
            "same_target_rejects": same_target_rejects,
            "wrong_identity_accepts": wrong_identity_accepts,
        }
    return summary


def build_aggregate_verdict(candidate_summary: dict[str, Any]) -> dict[str, Any]:
    candidate_verdicts = {
        candidate_name: candidate_summary[candidate_name]["graduation_verdict"]
        for candidate_name in EXPECTED_CANDIDATE_ORDER
    }
    graduating_candidates = [
        candidate_name
        for candidate_name in EXPECTED_CANDIDATE_ORDER
        if candidate_summary[candidate_name]["disposition"] == "promote"
    ]
    demoted_candidates = [
        candidate_name
        for candidate_name in EXPECTED_CANDIDATE_ORDER
        if candidate_summary[candidate_name]["disposition"] == "demote"
    ]
    if not graduating_candidates:
        verdict = "no_bounded_context_candidate_graduates"
        selected_candidate = None
    elif len(graduating_candidates) == 1:
        verdict = "single_candidate_graduates"
        selected_candidate = graduating_candidates[0]
    else:
        verdict = "multiple_candidates_graduate"
        selected_candidate = None
    expect_choice(verdict, "aggregate_verdict.verdict", EXPECTED_AGGREGATE_VERDICTS)
    return {
        "candidate_verdicts": candidate_verdicts,
        "demoted_candidates": demoted_candidates,
        "graduating_candidates": graduating_candidates,
        "selected_candidate": selected_candidate,
        "verdict": verdict,
    }


def descriptor_evidence_record(context: dict[str, Any]) -> dict[str, Any]:
    return {
        "block_domain_start": context["block_domain_start"],
        "descriptor": context["descriptor"],
        "following_boundary_state": context["following_boundary_state"],
        "preceding_boundary_state": context["preceding_boundary_state"],
        "prefix_hits_bof": context["prefix_hits_bof"],
        "suffix_hits_eof": context["suffix_hits_eof"],
    }


def build_report_command_vector(md_binary: Path, case_id: str, role: str) -> list[str]:
    if role not in {"observed", "current"}:
        raise ProbeError(f"unsupported report role: {role}")
    logical_path = (
        CANONICAL_OBSERVED_CASE_PATH_TEMPLATE.format(case_id=case_id)
        if role == "observed"
        else CANONICAL_CURRENT_CASE_PATH_TEMPLATE.format(case_id=case_id)
    )
    return [
        repo_relative_path_text(md_binary),
        "blocks",
        logical_path,
        "--json",
    ]


def token_preimage(candidate_name: str, context: dict[str, Any]) -> bytes:
    payload = bytearray()
    payload.extend(length_prefixed(TOKEN_DOMAIN_LABEL))
    payload.extend(length_prefixed(b"1"))
    payload.extend(length_prefixed(candidate_name.encode("ascii")))
    payload.extend(length_prefixed(b"block"))
    for field_name in EXPECTED_CANDIDATE_PAYLOAD_FIELDS[candidate_name]:
        payload.extend(length_prefixed(candidate_field_bytes(context, field_name)))
    return bytes(payload)


def candidate_field_bytes(context: dict[str, Any], field_name: str) -> bytes:
    if field_name == "target_bytes":
        return context["target_bytes"]
    if field_name == "preceding_boundary_state":
        state = context["preceding_boundary_state"]
        if state not in EXPECTED_PRECEDING_BOUNDARY_STATES:
            raise ProbeError(f"unsupported preceding boundary state: {state!r}")
        return boundary_state_bytes(state)
    if field_name == "following_boundary_state":
        state = context["following_boundary_state"]
        if state not in EXPECTED_FOLLOWING_BOUNDARY_STATES:
            raise ProbeError(f"unsupported following boundary state: {state!r}")
        return boundary_state_bytes(state)
    if field_name == "preceding_block_bytes":
        return context["preceding_block_bytes"]
    if field_name == "following_block_bytes":
        return context["following_block_bytes"]
    if field_name == "prefix_window_bytes":
        return context["prefix_window_bytes"]
    if field_name == "suffix_window_bytes":
        return context["suffix_window_bytes"]
    if field_name == "prefix_hits_bof":
        return boolean_payload(context["prefix_hits_bof"])
    if field_name == "suffix_hits_eof":
        return boolean_payload(context["suffix_hits_eof"])
    raise ProbeError(f"unsupported candidate payload field: {field_name}")


def run_md_command(
    md_binary: Path,
    command: list[str],
    workspace: Path,
    where: str,
) -> dict[str, Any]:
    if command != [str(md_binary), "blocks", command[2], "--json"]:
        raise ProbeError(f"{where}: child invocation deviated from the only authorized argv shape")
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


def relevant_intervals(context: dict[str, Any]) -> dict[str, tuple[int, int]]:
    descriptor_span = context["descriptor"]["span"]
    intervals = {
        "target": interval_from_span(descriptor_span),
        "prefix_window": (
            descriptor_span["byte_start"] - len(context["prefix_window_bytes"]),
            descriptor_span["byte_start"],
        ),
        "suffix_window": (
            descriptor_span["byte_end"],
            descriptor_span["byte_end"] + len(context["suffix_window_bytes"]),
        ),
    }
    if context["preceding_descriptor"] is not None:
        intervals["preceding"] = interval_from_span(context["preceding_descriptor"]["span"])
    if context["following_descriptor"] is not None:
        intervals["following"] = interval_from_span(context["following_descriptor"]["span"])
    return intervals


def difference_bounds(observed_bytes: bytes, current_bytes: bytes) -> dict[str, int] | None:
    if observed_bytes == current_bytes:
        return None
    prefix = 0
    max_prefix = min(len(observed_bytes), len(current_bytes))
    while prefix < max_prefix and observed_bytes[prefix] == current_bytes[prefix]:
        prefix += 1
    observed_suffix = len(observed_bytes)
    current_suffix = len(current_bytes)
    while observed_suffix > prefix and current_suffix > prefix:
        if observed_bytes[observed_suffix - 1] != current_bytes[current_suffix - 1]:
            break
        observed_suffix -= 1
        current_suffix -= 1
    return {
        "observed_start": prefix,
        "observed_end": observed_suffix,
        "current_start": prefix,
        "current_end": current_suffix,
    }


def interval_from_span(span: dict[str, int]) -> tuple[int, int]:
    return (span["byte_start"], span["byte_end"])


def interval_contains(outer: tuple[int, int], inner: tuple[int, int]) -> bool:
    return outer[0] <= inner[0] and inner[1] <= outer[1]


def intervals_overlap(left: tuple[int, int], right: tuple[int, int]) -> bool:
    return left[0] < right[1] and right[0] < left[1]


def boundary_state_bytes(state: str) -> bytes:
    return state.encode("ascii")


def boolean_payload(value: bool) -> bytes:
    return b"\x01" if value else b"\x00"


def contains_subsequence(haystack: bytes, needle: bytes) -> bool:
    if needle == b"":
        return True
    return haystack.find(needle) >= 0


def equality_relation(left: bytes, right: bytes) -> str:
    return "equal" if left == right else "different"


def derive_credit(identity_truth: str, decision: str) -> str:
    if identity_truth == "same_target":
        return "correct" if decision == "accept" else "false_conflict"
    return "correct" if decision == "reject" else "wrong_identity"


def payload_record(payload: bytes) -> dict[str, Any]:
    return {
        "byte_length": len(payload),
        "sha256": sha256_hex(payload),
        "utf8": decode_utf8(payload, "payload"),
    }


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


def order_report_value(value: Any) -> Any:
    if isinstance(value, dict):
        ordered: dict[str, Any] = {}
        if set(value.keys()) == set(EXPECTED_CANDIDATE_ORDER):
            key_order = EXPECTED_CANDIDATE_ORDER
        else:
            key_order = tuple(sorted(value.keys()))
        for key in key_order:
            ordered[key] = order_report_value(value[key])
        return ordered
    if isinstance(value, list):
        return [order_report_value(item) for item in value]
    return value


def manifest_semantic_sha256_hex(manifest_map: dict[str, Any]) -> str:
    canonical_manifest = json.dumps(
        manifest_map,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return sha256_hex(canonical_manifest)


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def length_prefixed(value: bytes) -> bytes:
    return struct.pack(">Q", len(value)) + value


def verify_check_file(check_path: Path, report_bytes: bytes) -> None:
    resolved = resolve_repo_local_path(check_path, "--check path", strict=True)
    try:
        existing = resolved.read_bytes()
    except OSError as exc:
        raise ProbeError(f"failed to read --check file: {check_path}") from exc
    if existing != report_bytes:
        raise ProbeError(f"--check mismatch: {check_path}")


def atomic_write(output_path: Path, report_bytes: bytes) -> None:
    resolved = resolve_repo_local_output_path(output_path)
    parent = resolved.parent
    if not parent.exists():
        raise ProbeError(f"--output parent directory does not exist: {parent}")
    temp_name: str | None = None
    try:
        with NamedTemporaryFile(
            dir=str(parent),
            prefix=f".{resolved.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temp_name = handle.name
            handle.write(report_bytes)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, resolved)
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
        raise ProbeError(f"failed to write temporary case document: {path.name}") from exc


def resolve_repo_local_path(path_arg: Path, where: str, *, strict: bool) -> Path:
    candidate = path_arg if path_arg.is_absolute() else REPO_ROOT / path_arg
    try:
        resolved = candidate.resolve(strict=strict)
    except FileNotFoundError as exc:
        raise ProbeError(f"{where} not found: {path_arg}") from exc
    except OSError as exc:
        raise ProbeError(f"failed to resolve {where}: {path_arg}") from exc
    try:
        resolved.relative_to(REPO_ROOT)
    except ValueError as exc:
        raise ProbeError(f"{where} must resolve inside the repository root: {path_arg}") from exc
    return resolved


def resolve_repo_local_output_path(output_path: Path) -> Path:
    parent_arg = output_path.parent if output_path.parent != Path("") else Path(".")
    parent = resolve_repo_local_path(parent_arg, "--output parent", strict=True)
    return parent / output_path.name


def repo_relative_path_text(path: Path) -> str:
    try:
        relative = path.relative_to(REPO_ROOT)
    except ValueError as exc:
        raise ProbeError(f"reported binary path must stay inside the repository root: {path}") from exc
    text = relative.as_posix()
    if text.startswith("./") or text.startswith("../") or text == "":
        raise ProbeError(f"reported binary path is not a canonical repository-relative path: {text!r}")
    return text


def slice_target_bytes(document_bytes: bytes, span: dict[str, int], where: str) -> bytes:
    start = span["byte_start"]
    end = span["byte_end"]
    if end < start:
        raise ProbeError(f"{where}: span.byte_end must be >= span.byte_start")
    if end > len(document_bytes):
        raise ProbeError(f"{where}: span.byte_end {end} exceeds document byte length {len(document_bytes)}")
    target_bytes = document_bytes[start:end]
    decode_utf8(target_bytes, f"{where}.target_bytes")
    return target_bytes


def normalize_span(value: Any, where: str) -> dict[str, int]:
    span = expect_mapping(value, where)
    exact_key_set(span, SPAN_KEYS, where)
    line_start = expect_positive_int(span.get("line_start"), f"{where}.line_start")
    line_end = expect_positive_int(span.get("line_end"), f"{where}.line_end")
    byte_start = expect_nonnegative_int(span.get("byte_start"), f"{where}.byte_start")
    byte_end = expect_nonnegative_int(span.get("byte_end"), f"{where}.byte_end")
    if line_end < line_start:
        raise ProbeError(f"{where}.line_end must be >= line_start")
    if byte_end < byte_start:
        raise ProbeError(f"{where}.byte_end must be >= byte_start")
    return {
        "byte_end": byte_end,
        "byte_start": byte_start,
        "line_end": line_end,
        "line_start": line_start,
    }


def exact_key_set(mapping: dict[str, Any], expected_keys: set[str], where: str) -> None:
    actual_keys = set(mapping.keys())
    if actual_keys != expected_keys:
        missing = sorted(expected_keys - actual_keys)
        extra = sorted(actual_keys - expected_keys)
        raise ProbeError(f"{where}: unexpected key set; missing={missing}, extra={extra}")


def decode_utf8(payload: bytes, where: str) -> str:
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ProbeError(f"{where}: invalid UTF-8") from exc


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


def expect_exact_string(value: Any, where: str, expected: str) -> str:
    text = expect_string(value, where)
    if text != expected:
        raise ProbeError(f"{where} must be {expected!r}, got {text!r}")
    return text


def expect_sha256_string(value: Any, where: str) -> str:
    text = expect_string(value, where)
    if SHA256_PATTERN.fullmatch(text) is None:
        raise ProbeError(f"{where} must be exactly 64 lowercase hexadecimal characters")
    return text


def expect_choice(value: Any, where: str, choices: tuple[str, ...]) -> str:
    text = expect_string(value, where)
    if text not in choices:
        raise ProbeError(f"{where}: invalid value {text!r}; allowed values: {choices!r}")
    return text


def expect_bool(value: Any, where: str) -> bool:
    if not isinstance(value, bool):
        raise ProbeError(f"{where} must be a boolean")
    return value


def expect_nonnegative_int(value: Any, where: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ProbeError(f"{where} must be a nonnegative integer")
    return value


def expect_positive_int(value: Any, where: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        raise ProbeError(f"{where} must be a positive integer")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
