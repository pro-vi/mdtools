"""H content hashing intent; U4 still owns final validated study identities."""

from dataclasses import replace

from bench.manifest import ExperimentSpec


def test_condition_and_toolkit_content_bind_synthetic_identity() -> None:
    spec = ExperimentSpec("synthetic", "task", "prompt", {"input.md": "input"},
        "expected", ("/synthetic/python",), "binary", "lock", "harness", "grader")
    bound = replace(spec, condition_sha256="condition", toolkit_sha256={"cat": "cat-bytes"})
    assert spec.identity != bound.identity
    assert bound.task_sha256 == spec.task_sha256
    assert bound.harness_sha256 == spec.harness_sha256
    assert bound.grader_sha256 == spec.grader_sha256
    assert replace(bound, toolkit_sha256={"cat": "changed"}).identity != bound.identity
