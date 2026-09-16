"""Independent comparisons recovered from H (c933520), without md diagnostics.

U1 admits only file byte/text comparisons. Structural/JSON/text-output families
remain unavailable until U2; an unsupported dimension never disappears silently.
"""

from __future__ import annotations

from dataclasses import dataclass, fields
from typing import Literal

from markdown_it import MarkdownIt
from markdown_it.token import Token


def _render_inline_to_plaintext(children: list[Token] | None) -> str:
    """H's heading text extraction, retained as a parser primitive for U2."""
    parts = []
    for child in children or []:
        if child.type in ("text", "code_inline"):
            parts.append(child.content)
        elif child.type == "image":
            parts.append(_render_inline_to_plaintext(child.children))
        elif child.type in ("softbreak", "hardbreak"):
            parts.append(" ")
    return "".join(parts)


def neutral_heading_tree(content: str) -> list[tuple[int, str]]:
    """Recovered independent heading parser; not yet an admitted U1 grade family."""
    tokens = MarkdownIt("commonmark", {"html": True}).enable(["table"]).parse(content)
    tree = []
    for index, token in enumerate(tokens):
        if token.type == "heading_open":
            text = _render_inline_to_plaintext(tokens[index + 1].children) if index + 1 < len(tokens) and tokens[index + 1].type == "inline" else ""
            tree.append((int(token.tag[1:]), text))
    return tree


@dataclass(frozen=True)
class StructuralDiffPolicy:
    kind: Literal["structural", "normalized_text", "raw_bytes"]
    normalize_line_endings: bool
    ignore_trailing_whitespace: bool
    compare_frontmatter_json: bool
    compare_heading_tree: bool
    compare_block_order: bool
    compare_link_destinations: bool
    compare_block_text: bool
    json_canonical: bool = False
    json_required_keys: list[str] | None = None


@dataclass(frozen=True)
class FileComparison:
    kind: Literal["pass", "fail"]
    reason: str


def validate_policy(policy: StructuralDiffPolicy, *, artifact: str) -> None:
    if artifact != "file_contents":
        raise ValueError("unsupported artifact for U1: expected file_contents")
    if policy.kind not in ("raw_bytes", "normalized_text"):
        raise ValueError("unsupported scorer for U1: structural grading requires U2")
    for item in fields(policy):
        if item.name not in ("kind", "json_required_keys") and type(getattr(policy, item.name)) is not bool:
            raise ValueError(f"policy flag {item.name} must be a boolean")
    if any((policy.compare_frontmatter_json, policy.compare_heading_tree,
            policy.compare_block_order, policy.compare_link_destinations,
            policy.compare_block_text, policy.json_canonical)) or policy.json_required_keys is not None:
        raise ValueError("unsupported scorer dimensions for U1")


def _normalize(content: bytes, policy: StructuralDiffPolicy) -> bytes:
    if policy.normalize_line_endings:
        content = content.replace(b"\r\n", b"\n")
    if policy.ignore_trailing_whitespace:
        # Preserve final newlines and non-ASCII whitespace; only declared line tails.
        content = b"\n".join(line.rstrip(b" \t") for line in content.split(b"\n"))
    return content


def validate_expected(policy: StructuralDiffPolicy, expected: bytes) -> None:
    """Reject an invalid controller expectation before worker execution."""
    if policy.kind == "normalized_text":
        try:
            expected.decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError("invalid_expected_utf8") from exc


def score_task(policy: StructuralDiffPolicy, actual: bytes, expected: bytes) -> FileComparison:
    """Compare controller-captured bytes; no binary, harness or subprocess input."""
    validate_policy(policy, artifact="file_contents")
    validate_expected(policy, expected)
    actual = _normalize(actual, policy)
    expected = _normalize(expected, policy)
    if policy.kind == "normalized_text":
        try:
            actual.decode("utf-8")
        except UnicodeDecodeError:
            return FileComparison("fail", "invalid_utf8")
    return FileComparison("pass", "file_match") if actual == expected else FileComparison("fail", "file_mismatch")
