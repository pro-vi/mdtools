"""H prompt-neutrality intent, narrowed to public/synthetic answer packaging."""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

import pytest

from bench import harness, neutral_scorer as scorer
from bench.test_harness_json import public_task
from bench.test_harness_run_artifacts import python_command, synthetic_task
from bench.test_neutral_scorer import family_cases, policy


def test_expected_answers_never_enter_worker_inputs(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    task = replace(task, description="Submit the computed answer.", expected_artifact="json_envelope", scorer=policy("structural", json_canonical=True))
    (expected / "answer.md").write_bytes(b'{"private-synthetic-canary":1}')
    prompt = harness.build_prompt(task)
    assert "private-synthetic-canary" not in prompt
    assert str(expected) not in prompt
    script = "from pathlib import Path; import sys; p=sys.stdin.read(); assert 'private-synthetic-canary' not in p; assert sorted(str(x) for x in Path('.').rglob('*')) == ['input.md']; sys.stdout.write('{\"private-synthetic-canary\":1}')"
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=python_command(script), results_dir=root / "receipt")
    assert result.comparison.kind == "pass"


@pytest.mark.parametrize("task_id", ["T1", "T2", "T10", "T21"])
def test_public_prompts_keep_description_and_condition_neutral_contract(task_id: str) -> None:
    task = public_task(task_id)
    prompt = harness.build_prompt(task)
    assert task.description in prompt
    assert task.expected_output not in prompt
    assert all(name not in prompt for name in ("legacy", "current-compact", "no-md", "md outline", "md frontmatter"))
    assert scorer.answer_instructions(task.scorer, artifact=task.expected_artifact) in prompt
    if task_id == "T21":
        assert "format" in prompt


def test_task_contract_preserves_frozen_corpus() -> None:
    import subprocess
    repo = Path(__file__).resolve().parent.parent
    # Hash/identity checks do not reveal any corpus row or holdout content.
    subprocess.run(["git", "diff", "--exit-code", "c933520", "--", "bench/tasks", "bench/inputs", "bench/expected"], cwd=repo, check=True, capture_output=True)


def test_all_conditions_receive_same_answer_contract() -> None:
    for family, case in family_cases().items():
        task = harness.BenchTask("synthetic", "Compute the requested result.", ["input.md"], "expected", family, "synthetic", case["policy"], expected_stdout="text\n" if family == "stdout_and_file" else None)
        prompts = [harness.build_prompt(task) for condition in ("no-md", "legacy", "current-compact")]
        assert len(set(prompts)) == 1
        assert scorer.grade_submission(**case).kind == "pass"
