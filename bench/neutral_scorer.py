"""Independent comparisons recovered from H (c933520), without md diagnostics.

Task-owned answer contracts and source views never invoke the treatment binary.
"""

from __future__ import annotations

from dataclasses import dataclass, fields
from decimal import Decimal, InvalidOperation
import json
import re
from typing import Literal

from markdown_it import MarkdownIt
from markdown_it.token import Token


def _render_inline_to_plaintext(children: list[Token] | None) -> str:
    """H's rendered heading text extraction, independent of CLI output."""
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
    """Extract ordered rendered headings without invoking the evaluated binary."""
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
class Comparison:
    kind: Literal["pass", "fail"]
    reason: str


def validate_policy(policy: StructuralDiffPolicy, *, artifact: str) -> None:
    if artifact not in ("file_contents", "json_envelope", "stdout_text", "stdout_and_file"):
        raise ValueError("unsupported artifact")
    if policy.kind not in ("raw_bytes", "normalized_text", "structural"):
        raise ValueError("unsupported scorer")
    for item in fields(policy):
        if item.name not in ("kind", "json_required_keys") and type(getattr(policy, item.name)) is not bool:
            raise ValueError(f"policy flag {item.name} must be a boolean")
    dimensions = (policy.compare_frontmatter_json, policy.compare_heading_tree,
                  policy.compare_block_order, policy.compare_link_destinations,
                  policy.compare_block_text)
    if policy.json_required_keys is not None:
        keys = policy.json_required_keys
        if not isinstance(keys, list) or any(type(key) is not str for key in keys) or len(set(keys)) != len(keys):
            raise ValueError("json_required_keys must be distinct string keys")
        if not policy.json_canonical:
            raise ValueError("required keys need json_canonical")
    if artifact == "json_envelope":
        if policy.kind != "structural" or policy.compare_block_order or policy.compare_block_text:
            raise ValueError("unsupported JSON policy dimensions")
        if policy.json_canonical:
            if any(dimensions):
                raise ValueError("unsupported mixed canonical/semantic JSON policy")
        elif sum(dimensions) != 1:
            raise ValueError("unsupported JSON projection combination")
    else:
        if policy.json_canonical or policy.json_required_keys is not None or policy.compare_frontmatter_json:
            raise ValueError("unsupported file/text policy dimensions")
        if policy.kind == "raw_bytes" and any(dimensions):
            raise ValueError("unsupported raw_bytes dimensions")
        if artifact == "stdout_text" and (any(dimensions) or policy.kind == "structural"):
            raise ValueError("unsupported stdout_text policy dimensions")
        if policy.kind == "structural" and not any(dimensions):
            raise ValueError("structural policy requires a declared dimension")


def _normalize(content: bytes, policy: StructuralDiffPolicy) -> bytes:
    if policy.normalize_line_endings:
        content = content.replace(b"\r\n", b"\n")
    if policy.ignore_trailing_whitespace:
        # Preserve final newlines and non-ASCII whitespace; only declared line tails.
        content = b"\n".join(line.rstrip(b" \t") for line in content.split(b"\n"))
    return content


def _decode_json(content: bytes) -> object:
    def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in items:
            if key in result:
                raise ValueError("duplicate_json_key")
            result[key] = value
        return result

    def finite_number(text: str) -> Decimal:
        try:
            number = Decimal(text)
        except InvalidOperation as exc:
            raise ValueError("unsupported_json_number") from exc
        if not number.is_finite():
            raise ValueError("nonfinite_json_number")
        return number

    def invalid_constant(text: str) -> object:
        raise ValueError("nonfinite_json_number")

    try:
        return json.loads(content.decode("utf-8"), object_pairs_hook=pairs,
                          parse_float=finite_number, parse_constant=invalid_constant)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ValueError("invalid_json") from exc


def _typed_equal(actual: object, expected: object) -> bool:
    # JSON has one numeric value domain. Booleans are a separate domain even
    # though Python's bool is an int subclass. Decimal avoids float rounding.
    if type(actual) in (int, Decimal) and type(expected) in (int, Decimal):
        return actual == expected
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict) and isinstance(actual, dict):
        return actual.keys() == expected.keys() and all(_typed_equal(actual[key], value) for key, value in expected.items())
    if isinstance(expected, list) and isinstance(actual, list):
        return len(actual) == len(expected) and all(_typed_equal(left, right) for left, right in zip(actual, expected))
    return actual == expected


def _project_keys(answer: object, keys: list[str]) -> object:
    if isinstance(answer, list):
        return [_project_keys(item, keys) for item in answer]
    if not isinstance(answer, dict) or any(key not in answer for key in keys):
        raise ValueError("missing_required_keys")
    return {key: answer[key] for key in keys}


def answer_instructions(policy: StructuralDiffPolicy, *, artifact: str) -> str:
    """Packaging only: this function cannot receive any expected answer values."""
    validate_policy(policy, artifact=artifact)
    if artifact == "file_contents":
        return "Modify the first input file in place."
    if artifact == "stdout_and_file":
        return "Modify the first input file in place AND submit the requested final text verbatim."
    if artifact == "stdout_text":
        return "Submit the requested final text verbatim."
    if policy.json_canonical:
        keys = f" Required keys in every result object: {json.dumps(policy.json_required_keys)}." if policy.json_required_keys else ""
        return "Submit the task's declared JSON object or array, preserving its fields and meaningful array order." + keys + " No prose or code fences."
    if policy.compare_heading_tree:
        return 'Submit an ordered JSON array of headings: [{"level": integer 1 through 6, "text": string}]. Use [] for no headings. No prose or code fences.'
    if policy.compare_frontmatter_json:
        return 'Submit JSON {"present": boolean, "format": string or null, "value": parsed JSON payload or null}. Preserve the frontmatter format: yaml/toml labels are case-insensitive; YAML and TOML remain distinct. Absent frontmatter requires false/null/null. No prose or code fences.'
    return 'Submit an ordered JSON array of links: [{"kind": string, "destination": string}]. Use [] for no links. No prose or code fences.'


def _semantic_answer(policy: StructuralDiffPolicy, answer: object, *, legacy_expected: bool) -> object:
    """Convert frozen legacy expectations only; submissions use the shared shape."""
    if policy.compare_heading_tree:
        if legacy_expected:
            if not isinstance(answer, dict) or not isinstance(answer.get("entries"), list):
                raise ValueError("missing_heading_entries")
            headings = []
            for entry in answer["entries"]:
                if not isinstance(entry, dict) or not isinstance(entry.get("heading"), dict):
                    raise ValueError("invalid_heading_entry")
                headings.append({key: entry["heading"][key] for key in ("level", "text") if key in entry["heading"]})
            answer = headings
        if not isinstance(answer, list):
            raise ValueError("invalid_heading_list")
        for heading in answer:
            if not isinstance(heading, dict) or set(heading) != {"level", "text"} or type(heading["level"]) is not int or not 1 <= heading["level"] <= 6 or type(heading["text"]) is not str:
                raise ValueError("invalid_heading_fields")
    elif policy.compare_frontmatter_json:
        if legacy_expected:
            if not isinstance(answer, dict) or type(answer.get("present")) is not bool:
                raise ValueError("invalid_frontmatter_presence")
            frontmatter = answer.get("frontmatter")
            if answer["present"]:
                if not isinstance(frontmatter, dict) or "data" not in frontmatter or "format" not in frontmatter:
                    raise ValueError("missing_frontmatter_fields")
                answer = {"present": True, "format": frontmatter["format"], "value": frontmatter["data"]}
            else:
                if frontmatter is not None:
                    raise ValueError("unexpected_absent_frontmatter")
                answer = {"present": False, "format": None, "value": None}
        if not isinstance(answer, dict) or set(answer) != {"present", "format", "value"} or type(answer["present"]) is not bool:
            raise ValueError("invalid_frontmatter_fields")
        if answer["present"]:
            if type(answer["format"]) is not str or not answer["format"]:
                raise ValueError("invalid_frontmatter_format")
            # Both pinned producers describe the same format with different
            # enum spelling. Normalize only the two known semantic labels, on
            # expected and submitted projections; preserve all raw evidence.
            if answer["format"].lower() in ("yaml", "toml"):
                answer = {**answer, "format": answer["format"].lower()}
        elif answer["format"] is not None or answer["value"] is not None:
            raise ValueError("unexpected_absent_frontmatter")
    else:
        if legacy_expected and isinstance(answer, dict):
            if "links" not in answer:
                raise ValueError("missing_links")
            answer = answer["links"]
        if not isinstance(answer, list):
            raise ValueError("invalid_link_list")
        links = []
        for link in answer:
            if not isinstance(link, dict) or any(type(link.get(key)) is not str for key in ("kind", "destination")):
                raise ValueError("invalid_link_fields")
            if not legacy_expected and set(link) != {"kind", "destination"}:
                raise ValueError("invalid_link_fields")
            links.append({key: link[key] for key in ("kind", "destination")})
        answer = links
    return answer


def expected_answer(policy: StructuralDiffPolicy, expected: bytes) -> object:
    """Controller-side projection; never used by prompt construction."""
    answer = _decode_json(expected)
    if policy.json_canonical:
        if not isinstance(answer, (dict, list)):
            raise ValueError("canonical_json_requires_object_or_array")
        return _project_keys(answer, policy.json_required_keys) if policy.json_required_keys else answer
    return _semantic_answer(policy, answer, legacy_expected=True)


def validate_expected(policy: StructuralDiffPolicy, expected: bytes, *, artifact: str = "file_contents",
                      expected_stdout: bytes | None = None) -> None:
    """Reject an invalid controller expectation before worker execution."""
    validate_policy(policy, artifact=artifact)
    if artifact == "json_envelope":
        try:
            expected_answer(policy, expected)
        except ValueError as exc:
            raise ValueError(f"invalid_expected_json:{exc}") from exc
    elif policy.kind != "raw_bytes" or artifact == "stdout_text":
        try:
            expected.decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError("invalid_expected_utf8") from exc
    if artifact == "stdout_and_file":
        if not isinstance(expected_stdout, bytes):
            raise ValueError("stdout_and_file requires expected_stdout")
        try:
            expected_stdout.decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError("invalid_expected_stdout_utf8") from exc
    elif expected_stdout is not None:
        raise ValueError("unexpected expected_stdout")


def _tokens(content: str) -> list[Token]:
    return MarkdownIt("commonmark", {"html": True}).enable(["table"]).parse(content)


def neutral_block_texts(content: str) -> list[str]:
    """Source slices for outer blocks, with the parser's CR/LF line boundaries.

    Unicode separators stay within a physical line. Keep fence markers, info,
    indentation and source line endings; no unrestricted strip/splitlines.
    """
    lines = re.findall(r"[^\r\n]*(?:\r\n|\r|\n|$)", content)
    texts = []
    for token in _tokens(content):
        if token.level == 0 and token.map is not None and (token.nesting == 1 or token.nesting == 0 and token.type != "inline"):
            start, end = token.map
            texts.append("".join(lines[start:end]))
    return texts


def neutral_block_order(content: str) -> list[tuple[str, str, int]]:
    return [(token.type, token.tag, token.level) for token in _tokens(content) if token.type != "inline"]


def neutral_link_destinations(content: str) -> list[tuple[str, str]]:
    def collect(tokens: list[Token]) -> list[tuple[str, str]]:
        links = []
        for token in tokens:
            if token.type == "link_open":
                links.append(("link", token.attrGet("href") or ""))
            elif token.type == "image":
                links.append(("image", token.attrGet("src") or ""))
            if token.children:
                links.extend(collect(token.children))
        return links
    return collect(_tokens(content))


def score_task(policy: StructuralDiffPolicy, actual: bytes, expected: bytes) -> Comparison:
    """Compare controller-captured bytes; no binary, harness or subprocess input."""
    validate_policy(policy, artifact="file_contents")
    validate_expected(policy, expected)
    actual = _normalize(actual, policy)
    expected = _normalize(expected, policy)
    if policy.kind != "raw_bytes":
        try:
            actual.decode("utf-8")
        except UnicodeDecodeError:
            return Comparison("fail", "invalid_utf8")
    comparisons = []
    for enabled, name, compare in (
        (policy.compare_heading_tree, "heading_tree", neutral_heading_tree),
        (policy.compare_block_order, "block_order", neutral_block_order),
        (policy.compare_link_destinations, "link_destinations", neutral_link_destinations),
        (policy.compare_block_text, "block_text", neutral_block_texts),
    ):
        if enabled:
            comparisons.append((name, compare(actual.decode("utf-8")) == compare(expected.decode("utf-8"))))
    if comparisons:
        mismatches = [name for name, matches in comparisons if not matches]
        return Comparison("fail", ",".join(mismatches) + "_mismatch") if mismatches else Comparison("pass", "declared_dimensions_match")
    return Comparison("pass", "file_match") if actual == expected else Comparison("fail", "file_mismatch")


def grade_json(policy: StructuralDiffPolicy, final_text: bytes, expected: bytes) -> Comparison:
    validate_expected(policy, expected, artifact="json_envelope")
    expected_value = expected_answer(policy, expected)
    try:
        actual_value = _decode_json(final_text)
        if policy.json_canonical:
            if not isinstance(actual_value, (dict, list)):
                raise ValueError("canonical_json_requires_object_or_array")
            if policy.json_required_keys:
                actual_value = _project_keys(actual_value, policy.json_required_keys)
        else:
            actual_value = _semantic_answer(policy, actual_value, legacy_expected=False)
    except ValueError as exc:
        return Comparison("fail", f"invalid_submission:{exc}")
    return Comparison("pass", "json_match") if _typed_equal(actual_value, expected_value) else Comparison("fail", "json_mismatch")


def grade_stdout_text(policy: StructuralDiffPolicy, final_text: bytes, expected: bytes) -> Comparison:
    validate_expected(policy, expected, artifact="stdout_text")
    try:
        final_text.decode("utf-8")
    except UnicodeDecodeError:
        return Comparison("fail", "invalid_submission_utf8")
    return Comparison("pass", "stdout_match") if _normalize(final_text, policy) == _normalize(expected, policy) else Comparison("fail", "stdout_mismatch")


def grade_stdout_and_file(policy: StructuralDiffPolicy, final_text: bytes, actual_file: bytes,
                          expected: bytes, expected_stdout: bytes) -> Comparison:
    validate_expected(policy, expected, artifact="stdout_and_file", expected_stdout=expected_stdout)
    try:
        final_text.decode("utf-8")
    except UnicodeDecodeError:
        return Comparison("fail", "invalid_submission_utf8")
    file_comparison = score_task(policy, actual_file, expected)
    text_matches = _normalize(final_text, policy) == _normalize(expected_stdout, policy)
    if file_comparison.kind == "pass" and text_matches:
        return Comparison("pass", "stdout_and_file_match")
    return Comparison("fail", "stdout_and_file_mismatch:" + (file_comparison.reason if file_comparison.kind == "fail" else "stdout_mismatch"))


def grade_submission(policy: StructuralDiffPolicy, *, artifact: str, final_text: bytes,
                     actual_file: bytes | None, expected: bytes,
                     expected_stdout: bytes | None = None) -> Comparison:
    """Exact-kind dispatcher. Missing file capture is a controller error."""
    validate_expected(policy, expected, artifact=artifact, expected_stdout=expected_stdout)
    if artifact == "json_envelope":
        return grade_json(policy, final_text, expected)
    if artifact == "stdout_text":
        return grade_stdout_text(policy, final_text, expected)
    if actual_file is None:
        raise ValueError("capture_unavailable")
    if artifact == "stdout_and_file":
        # validate_expected established bytes; do not coerce a missing value.
        if expected_stdout is None:
            raise ValueError("missing_expected_stdout")
        return grade_stdout_and_file(policy, final_text, actual_file, expected, expected_stdout)
    return score_task(policy, actual_file, expected)
