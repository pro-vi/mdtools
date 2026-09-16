"""Adapt H's independent scorer intent with synthetic source/type controls."""

from __future__ import annotations

from dataclasses import replace
import ast
from pathlib import Path
import subprocess

import pytest

from bench import neutral_scorer as scorer
from bench.neutral_scorer import Comparison, StructuralDiffPolicy


def policy(kind: str = "normalized_text", **flags: object) -> StructuralDiffPolicy:
    return replace(StructuralDiffPolicy(kind, True, True, False, False, False, False, False), **flags)


def test_known_good_document_and_heading_rendering() -> None:
    sample = "# The `md` **tasks** [command](https://example.com)\n\nMulti\nLine Heading\n===========\n"
    assert scorer.neutral_heading_tree(sample) == [(1, "The md tasks command"), (1, "Multi Line Heading")]
    assert scorer.neutral_block_texts("# Title\n\n- first\n- second\n") == ["# Title\n", "- first\n- second\n"]


@pytest.mark.parametrize("actual,expected,flags,reason", [
    (b"# Wrong\n", b"# Right\n", {"compare_heading_tree": True}, "heading_tree"),
    (b"Title\n", b"# Title\n", {"compare_block_text": True}, "block_text"),
    (b"***\n", b"---\n", {"compare_block_text": True}, "block_text"),
    (b"quoted\n", b"> quoted\n", {"compare_block_text": True}, "block_text"),
    (b"- top\n- inner\n", b"- top\n  - inner\n", {"compare_block_text": True}, "block_text"),
    (b"hello\n", b"```\nhello\n```\n", {"compare_block_text": True}, "block_text"),
    (b"```js\nx = 1\n```\n", b"```python\nx = 1\n```\n", {"compare_block_text": True}, "block_text"),
    (b"```python\nx = 1\n```\n", b"  ```python\n  x = 1\n  ```\n", {"compare_block_text": True}, "block_text"),
    (b"body\n\n# H\n", b"# H\n\nbody\n", {"compare_block_order": True}, "block_order"),
    (b"- top\n- inner\n", b"- top\n  - inner\n", {"compare_block_order": True}, "block_order"),
    (b"[label](wrong)\n", b"[label](right)\n", {"compare_link_destinations": True}, "link_destinations"),
])
def test_source_negative_controls(actual: bytes, expected: bytes, flags: dict[str, object], reason: str) -> None:
    comparison = scorer.score_task(policy(**flags), actual, expected)
    assert comparison.kind == "fail"
    assert reason in comparison.reason


def test_fence_and_unicode_negative_controls() -> None:
    expected = "paragraph\u2028still same line\n\n# Later\n"
    actual = "paragraph\u2028still same line\n\n# Wrong\n"
    assert scorer.neutral_block_texts(expected)[-1] == "# Later\n"
    assert scorer.score_task(policy(compare_block_text=True), actual.encode(), expected.encode()).kind == "fail"
    assert scorer.neutral_heading_tree(expected) == [(1, "Later")]
    assert scorer.neutral_block_texts("a\u0085b\u2029c\n\n# Next\n")[-1] == "# Next\n"


def test_declared_normalization_only() -> None:
    declared = policy(compare_block_text=True, compare_heading_tree=True, compare_block_order=True)
    expected = b"# H\n\n```python\n  content\n```\n"
    actual = b"# H \r\n\r\n```python\r\n  content \t\r\n```\r\n"
    assert scorer.score_task(declared, actual, expected).kind == "pass"
    assert scorer.score_task(replace(declared, normalize_line_endings=False), actual, expected).kind == "fail"
    assert scorer.score_task(replace(declared, ignore_trailing_whitespace=False), actual, expected).kind == "fail"
    assert scorer.score_task(declared, expected.rstrip(b"\n"), expected).kind == "fail"
    assert scorer.score_task(declared, expected.replace(b"content", "content\u00a0".encode()), expected).kind == "fail"


def test_only_declared_dimensions_are_compared() -> None:
    assert scorer.score_task(policy(compare_heading_tree=True), b"# *Title*\n\nwrong body\n", b"# Title\n\nright body\n").kind == "pass"
    assert scorer.score_task(policy(compare_block_order=True), b"# Wrong\n\nwrong\n", b"# Right\n\nright\n").kind == "pass"
    assert scorer.score_task(policy(compare_link_destinations=True), b"[new](right)\n", b"[old](right)\n").kind == "pass"
    duplicate = b"# Repeat\n\n# Repeat\n"
    assert scorer.score_task(policy(compare_heading_tree=True), duplicate, duplicate).kind == "pass"
    assert scorer.score_task(policy(compare_heading_tree=True), b"# Repeat\n", duplicate).kind == "fail"


def test_parser_line_boundaries_keep_source_line_endings() -> None:
    assert scorer.neutral_block_texts("# One\r\r# Two\r") == ["# One\r", "# Two\r"]
    assert scorer.neutral_block_texts("# One\r\n\r\n# Two\r\n") == ["# One\r\n", "# Two\r\n"]


def test_grading_never_invokes_treatment_binary(monkeypatch: pytest.MonkeyPatch) -> None:
    def forbidden(*args: object, **kwargs: object) -> object:
        raise AssertionError("grader invoked subprocess")
    monkeypatch.setattr(subprocess, "run", forbidden)
    monkeypatch.setattr(subprocess, "Popen", forbidden)
    assert scorer.score_task(policy(compare_heading_tree=True, compare_block_text=True), b"# Wrong\n", b"# Right\n").kind == "fail"
    source = ast.parse(Path(scorer.__file__).read_text())
    imports = [node for node in ast.walk(source) if isinstance(node, (ast.Import, ast.ImportFrom))]
    assert not any(isinstance(node, ast.Import) and any(alias.name in ("subprocess", "bench.harness") for alias in node.names) or isinstance(node, ast.ImportFrom) and node.module in ("subprocess", "bench.harness") for node in imports)


@pytest.mark.parametrize("artifact,declared", [
    ("unknown", policy()), ("multi_file_contents_any", policy()),
    ("file_contents", policy("raw_bytes", compare_heading_tree=True)),
    ("file_contents", policy(compare_frontmatter_json=True)),
    ("file_contents", policy("structural")),
    ("file_contents", policy(json_canonical=True)),
    ("stdout_text", policy(compare_block_text=True)),
    ("stdout_text", policy("structural", compare_heading_tree=True)),
    ("json_envelope", policy("structural", compare_block_text=True)),
    ("json_envelope", policy("structural", compare_heading_tree=True, compare_link_destinations=True)),
    ("json_envelope", policy("structural", json_canonical=True, compare_heading_tree=True)),
    ("json_envelope", policy("structural", json_canonical=True, json_required_keys=["a", "a"])),
    ("file_contents", policy(normalize_line_endings=1)),
])
def test_unsupported_policy_is_explicit(artifact: str, declared: StructuralDiffPolicy) -> None:
    with pytest.raises(ValueError):
        scorer.validate_policy(declared, artifact=artifact)


def family_cases() -> dict[str, dict[str, object]]:
    return {
        "file_contents": dict(policy=policy("raw_bytes"), artifact="file_contents", final_text=b"ignored", actual_file=b"file\n", expected=b"file\n"),
        "json_envelope": dict(policy=policy("structural", json_canonical=True), artifact="json_envelope", final_text=b'{"count":1}', actual_file=None, expected=b'{"count":1}'),
        "stdout_text": dict(policy=policy(), artifact="stdout_text", final_text=b"text\n", actual_file=None, expected=b"text\n"),
        "stdout_and_file": dict(policy=policy("raw_bytes"), artifact="stdout_and_file", final_text=b"text\n", actual_file=b"file\n", expected=b"file\n", expected_stdout=b"text\n"),
    }


def test_artifact_kind_dispatch_is_exact(monkeypatch: pytest.MonkeyPatch) -> None:
    for artifact, case in family_cases().items():
        calls = []
        with monkeypatch.context() as local:
            for family, name in (("file_contents", "score_task"), ("json_envelope", "grade_json"), ("stdout_text", "grade_stdout_text"), ("stdout_and_file", "grade_stdout_and_file")):
                def adapter(*args: object, family: str = family) -> Comparison:
                    calls.append(family)
                    return Comparison("pass", family)
                local.setattr(scorer, name, adapter)
            assert scorer.grade_submission(**case).reason == artifact
        assert calls == [artifact]


@pytest.mark.parametrize("adapter,affected", [("grade_stdout_text", "stdout_text"), ("grade_stdout_and_file", "stdout_and_file")])
@pytest.mark.parametrize("fault", ["exception", "wrong_grade"])
def test_text_family_adapters_do_not_interfere(monkeypatch: pytest.MonkeyPatch, adapter: str, affected: str, fault: str) -> None:
    cases = family_cases()
    original = {family: (scorer.answer_instructions(case["policy"], artifact=family), scorer.grade_submission(**case), dict(case)) for family, case in cases.items()}
    def broken(*args: object) -> Comparison:
        if fault == "exception":
            raise RuntimeError("synthetic adapter defect")
        return Comparison("fail", "injected_wrong_grade")
    monkeypatch.setattr(scorer, adapter, broken)
    for family, case in cases.items():
        if family == affected:
            if fault == "exception":
                with pytest.raises(RuntimeError):
                    scorer.grade_submission(**case)
            else:
                assert scorer.grade_submission(**case).kind == "fail"
        else:
            assert (scorer.answer_instructions(case["policy"], artifact=family), scorer.grade_submission(**case), dict(case)) == original[family]
