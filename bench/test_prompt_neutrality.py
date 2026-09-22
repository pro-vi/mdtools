"""H prompt-neutrality intent, narrowed to public/synthetic answer packaging."""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

import pytest

from bench import harness, neutral_scorer as scorer
from bench.test_harness_json import public_task
from bench.test_harness_run_artifacts import python_command, synthetic_task
from bench.test_neutral_scorer import family_cases, policy
from bench.command_policy import resolve_toolkit, stage_condition


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
    assert result.grade.kind == "pass"


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
    subprocess.run(["git", "diff", "--exit-code", "c933520", "--", "bench/tasks", "bench/inputs", "bench/expected",
        ":(exclude)bench/expected/t2_inserted.md"], cwd=repo, check=True, capture_output=True)


def test_all_conditions_receive_same_answer_contract(cli_pins: dict, tmp_path: Path) -> None:
    stub = stage_condition(None, tmp_path.resolve() / "stub", toolkit=resolve_toolkit())
    for family, case in family_cases().items():
        task = harness.BenchTask("synthetic", "Compute the requested result.", ["input.md"], "expected", family, "synthetic", case["policy"], expected_stdout="text\n" if family == "stdout_and_file" else None)
        prompts = [harness.build_prompt(task, condition=condition) for condition in (stub, *cli_pins.values())]
        contracts = [prompt.split("\nTOOLS:\n")[0] for prompt in prompts]
        assert len(set(contracts)) == 1
        assert contracts[0] == harness.build_prompt(task)
        assert scorer.grade_submission(**case).kind == "pass"


def test_discovery_guidance_preserves_contract_and_avoids_eager_help(cli_pins: dict, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    from bench import command_policy
    calls = []
    original = command_policy.subprocess.run
    def observe(argv, *args, **kwargs):
        calls.append(argv)
        return original(argv, *args, **kwargs)
    monkeypatch.setattr(command_policy.subprocess, "run", observe)
    task = public_task("T1")
    for pin in cli_pins.values():
        calls.clear()
        prompt = harness.build_prompt(task, condition=pin, guidance=command_policy.ToolGuidance.DISCOVERY)
        assert prompt.split("\nTOOLS:\n")[0] == harness.build_prompt(task)
        assert "md --help" in prompt and "md <command> --help" in prompt
        assert "Use raw JSON or one complete ```json fence; no prose." in prompt and "$TMPDIR" in prompt
        assert all("--help" not in argv for argv in calls)
        assert len(prompt) < len(harness.build_prompt(task, condition=pin))
    stub = stage_condition(None, tmp_path / "stub", toolkit=resolve_toolkit())
    assert harness.build_prompt(task, condition=stub, guidance=command_policy.ToolGuidance.DISCOVERY) == harness.build_prompt(task, condition=stub)


def test_unknown_guidance_is_rejected_before_execution(tmp_path: Path) -> None:
    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    with pytest.raises(ValueError):
        harness.prepare_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command("raise SystemExit(99)"), results_dir=tmp_path / "receipt", guidance="unknown")
    assert not (tmp_path / "receipt").exists()


def test_guidance_is_bound_by_prompt_identity(cli_pins: dict, tmp_path: Path) -> None:
    from bench.command_policy import ToolGuidance
    from bench.trial_records import RecordIntegrityError

    task, fixtures, expected = synthetic_task(tmp_path.resolve())
    stub = stage_condition(None, tmp_path / "stub", toolkit=resolve_toolkit())
    conditions = {pin.condition.value: pin for pin in (stub, *cli_pins.values())}
    arguments = dict(fixture_root=fixtures, expected_root=expected,
        command=python_command("raise SystemExit(99)"), conditions=conditions,
        results_dir=tmp_path / "campaign")
    control = harness.freeze_campaign([task], repetitions=1, **arguments)
    treatment = harness.freeze_campaign([task], repetitions=1,
        guidance=ToolGuidance.DISCOVERY, **arguments)
    assert control.identity != treatment.identity
    for name, member in control.members.items():
        other = treatment.members[name]
        if name.endswith("no-md"):
            assert member.identity == other.identity
        else:
            assert member.prompt_sha256 != other.prompt_sha256
            assert replace(member, prompt_sha256=other.prompt_sha256).identity == other.identity
    with pytest.raises(RecordIntegrityError, match="changed experiment content"):
        harness.run_campaign(treatment, [task], **arguments)
    assert not tuple((tmp_path / "campaign" / "attempts").iterdir())
    summary = harness.run_campaign(treatment, [task], stop_after=0,
        guidance=ToolGuidance.DISCOVERY, **arguments)
    assert summary["admission_hold"] == "operator_stop"
