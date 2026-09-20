"""Synthetic paired comparisons; task IDs do not load corpus content."""
from __future__ import annotations

from dataclasses import replace
import json
import fcntl
import os
from pathlib import Path
import subprocess
import sys

import pytest

from bench import harness
from bench.command_policy import ToolGuidance
from bench.manifest import PromptComparisonSpec, canonical_json
from bench.test_report_inputs import campaign_case, live_case, publish_synthetic
from bench.test_campaign_config import config
from bench.trial_records import RecordIntegrityError, record_dict


@pytest.fixture
def comparison_case(campaign_case: tuple) -> tuple:
    root, original, _, arguments = campaign_case
    tasks = [replace(original[0], id=name) for name in ("T1", "T2", "T10")]
    campaigns = {guidance.value: harness.freeze_campaign(tasks, results_dir=root / guidance.value,
        repetitions=1, retry_allowance=0, guidance=guidance, **arguments) for guidance in ToolGuidance}
    spec = PromptComparisonSpec.create(campaigns, seed=1729)
    return root, tasks, arguments, spec


def test_paired_schedule_and_closed_roundtrip(comparison_case: tuple) -> None:
    _, _, _, spec = comparison_case
    assert len(spec.schedule) == 18
    assert PromptComparisonSpec.from_dict(record_dict(spec)).identity == spec.identity
    assert sum(entry[0] == "full_help" for entry in spec.schedule[::2]) in (4, 5)
    for index in range(0, 18, 2):
        left, right = spec.schedule[index:index + 2]
        assert left[1:] == right[1:] and left[0] != right[0]


@pytest.mark.parametrize("field,value", [("grader_sha256", "a" * 64),
    ("harness_sha256", "a" * 64), ("limits", {"timeout_seconds": 99, "max_turns": 30})])
def test_nonprompt_differences_are_rejected(comparison_case: tuple, field: str, value: object) -> None:
    _, _, _, spec = comparison_case
    child = spec.campaigns["discovery"]
    members = {name: replace(member, **{field: value}) for name, member in child.members.items()}
    with pytest.raises(RecordIntegrityError, match="non-prompt"):
        PromptComparisonSpec.create({**spec.campaigns, "discovery": replace(child, members=members)}, seed=spec.seed)


def test_extra_repeat_or_missing_half_is_not_a_comparison(comparison_case: tuple) -> None:
    _, _, _, spec = comparison_case
    with pytest.raises(RecordIntegrityError):
        replace(spec, schedule=spec.schedule[:-1])
    with pytest.raises(RecordIntegrityError):
        replace(spec, schedule=(*spec.schedule, spec.schedule[0]))
    malformed = (*spec.schedule[0][:3], False)
    with pytest.raises(RecordIntegrityError):
        replace(spec, schedule=(malformed, *spec.schedule[1:]))


def test_subprocess_pair_resume_and_independent_report(config, tmp_path: Path) -> None:
    from bench.prompt_comparison import load_comparison
    configured = replace(config, task_ids=("T1", "T2", "T10"))
    path = tmp_path / "config.json"
    path.write_text(json.dumps(record_dict(configured)))
    entry = [sys.executable, "-I", harness.__file__]
    prepare = subprocess.run([*entry, "--prepare-comparison", str(path)], capture_output=True, timeout=40)
    assert prepare.returncode == 0, prepare.stdout.decode() + prepare.stderr.decode()
    partial = subprocess.run([*entry, "--run-comparison", str(path), "--stop-after", "1"], capture_output=True, timeout=40)
    assert partial.returncode == 1, partial.stdout.decode() + partial.stderr.decode()
    root = Path(configured.output_root)
    first = load_comparison(root).schedule[0][0]
    before = {str(file): file.read_bytes() for file in (root / first / "campaign" / "attempts").rglob("*.json")}
    assert len(list((root / first / "campaign" / "attempts").iterdir())) == 1
    run = subprocess.run([*entry, "--run-comparison", str(path)], capture_output=True, timeout=90)
    assert run.returncode == 0, run.stdout.decode() + run.stderr.decode()
    report = json.loads(run.stdout)
    assert report["complete"] and report["planned_pairs"] == 9
    assert report["paired_success_discordance"] == {"both": 0, "neither": 9, "discovery_only": 0, "full_help_only": 0}
    assert report["successful_intersection"]["pairs"] == 0
    assert all(Path(name).read_bytes() == content for name, content in before.items())
    regenerated = subprocess.run([*entry, "--report-comparison", str(root)], capture_output=True, timeout=40)
    assert regenerated.returncode == 0 and json.loads(regenerated.stdout) == report
    assert json.loads((root / "comparison-report.json").read_bytes()) == report


@pytest.mark.parametrize("case,control,treatment", [("equal", 9, 9), ("improved", 3, 9), ("regressed", 6, 0)])
def test_paired_report_detects_planted_outcomes(comparison_case: tuple, case: str, control: int, treatment: int) -> None:
    from bench.prompt_comparison import report_comparison
    root, tasks, arguments, _ = comparison_case
    old_script = arguments["command"][-1]
    predicate = {"equal": "True", "improved": "'CLI condition:' not in prompt",
                 "regressed": "'CLI condition:' in prompt"}[case]
    script = "import sys; prompt=sys.stdin.read(); " + old_script.replace(
        "Path('input.md').write_bytes(b'after\\n')", f"Path('input.md').write_bytes(b'after\\n' if {predicate} else b'before\\n')")
    arguments = {**arguments, "command": (*arguments["command"][:-1], script)}
    campaigns = {guidance.value: harness.freeze_campaign(tasks, results_dir=root / guidance.value / "campaign",
        repetitions=1, retry_allowance=0, guidance=guidance, **arguments) for guidance in ToolGuidance}
    spec = PromptComparisonSpec.create(campaigns, seed=1729)
    harness._write_private(root / "comparison.json", canonical_json(record_dict(spec)).encode())
    for profile, child in campaigns.items():
        harness.run_campaign(child, tasks, results_dir=root / profile / "campaign",
            guidance=ToolGuidance(profile), **arguments)
    report = report_comparison(root)
    assert report["complete"] and report["success_difference"] == (treatment - control) / 9
    assert report["profiles"]["full_help"]["successes"] == control
    assert report["profiles"]["discovery"]["successes"] == treatment
    for profile in campaigns:
        measured = report["profiles"][profile]["measurements"]
        assert measured["total_tokens"]["known_trials"] == 9
        assert measured["total_tokens"]["total"] == sum(measured[name]["total"] for name in
            ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens"))


def test_changed_source_withholds_comparison(comparison_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    from bench.prompt_comparison import report_comparison
    root, _, _, spec = comparison_case
    harness._write_private(root / "comparison.json", canonical_json(record_dict(spec)).encode())
    monkeypatch.setattr(harness, "harness_source_identity", lambda: "a" * 64)
    report = report_comparison(root)
    assert not report["complete"] and report["success_difference"] is None
    assert report["faults"] == ["changed_reporting_source"]


def test_unknown_final_cost_withholds_paired_result(comparison_case: tuple) -> None:
    from bench.prompt_comparison import report_comparison
    root, tasks, arguments, original = comparison_case
    children = {profile: replace(child, grant_usd=1.0, reservation_usd=0.01)
                for profile, child in original.campaigns.items()}
    spec = PromptComparisonSpec.create(children, seed=original.seed)
    harness._write_private(root / "comparison.json", canonical_json(record_dict(spec)).encode())
    for profile, child in children.items():
        def execute(key, prior, path):
            last = (profile, key.task_id, key.condition, key.repetition) == spec.schedule[-1]
            return publish_synthetic(child, key, path, cost=None if last else 0.01)
        harness.run_campaign(child, tasks, results_dir=root / profile / "campaign",
            guidance=ToolGuidance(profile), _executor=execute, **arguments)
    report = report_comparison(root)
    assert not report["complete"] and report["success_difference"] is None
    assert report["successful_intersection"] is None
    invalid = report["profiles"][spec.schedule[-1][0]]["campaign_report"]
    assert "unknown_estimated_cost" in invalid["faults"]
    assert invalid["coverage"]["estimated_usd"]["known_total"] == pytest.approx(0.08)


def test_comparison_phase_is_separate_and_zero_retry(live_case: tuple) -> None:
    root, tasks, _, arguments, grant = live_case
    public = [replace(tasks[0], id=name) for name in ("T1", "T2", "T10")]
    spec = harness.freeze_campaign(public, results_dir=root / "public", repetitions=1,
        retry_allowance=0, grant_usd=0.9, reservation_usd=0.1, **arguments)
    consent = replace(grant, experiment_id=spec.identity, phase="prompt_comparison",
        task_ids=("T1", "T2", "T10"), campaign_usd=0.9, max_attempts=9,
        prerequisite_experiment_id=grant.experiment_id)
    consent.assert_scope(spec)
    retried = replace(spec, members={name: replace(member, retry_allowance=1) for name, member in spec.members.items()})
    with pytest.raises(RecordIntegrityError, match="zero retries"):
        replace(consent, experiment_id=retried.identity).assert_scope(retried)


def test_no_grant_changed_config_and_competing_lock_never_launch(config, monkeypatch: pytest.MonkeyPatch) -> None:
    from bench.prompt_comparison import prepare_comparison, run_comparison
    configured = replace(config, task_ids=("T1", "T2", "T10"))
    prepare_comparison(configured)
    root = Path(configured.output_root)
    live = replace(configured, command=(), claude_executable="/missing/claude",
        model="claude-haiku-4-5-20251001", thinking_policy="disabled")
    with pytest.raises(RecordIntegrityError, match="explicit grants"):
        run_comparison(live)
    with pytest.raises(RecordIntegrityError, match="changed comparison configuration"):
        run_comparison(replace(configured, timeout_seconds=11))
    with (root / ".comparison.lock").open("wb") as held:
        fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
        with pytest.raises(BlockingIOError):
            run_comparison(configured)
    assert not (root / "full_help/campaign").exists()
    assert not (root / "discovery/campaign").exists()


def local_test_native_paired_comparison(config, tmp_path: Path) -> None:
    from bench.test_claude_containment import scripted_provider
    executable = os.environ["MDTOOLS_U5_RUNNER"]
    with scripted_provider("true") as (endpoint, observations):
        native = replace(config, task_ids=("T1", "T2", "T10"), command=(),
            claude_executable=executable, endpoint=endpoint, model="claude-haiku-4-5-20251001",
            thinking_policy="disabled", timeout_seconds=20)
        path = tmp_path / "comparison-config.json"
        path.write_text(json.dumps(record_dict(native)))
        entry = [sys.executable, "-I", harness.__file__]
        for mode in ("--prepare-comparison", "--run-comparison"):
            run = subprocess.run([*entry, mode, str(path)], capture_output=True, timeout=180)
            assert run.returncode == 0, run.stdout.decode() + run.stderr.decode()
        report = json.loads(run.stdout)
        assert report["complete"] and report["planned_pairs"] == 9
        assert all(row["measurements"]["total_tokens"]["known_trials"] == 9 for row in report["profiles"].values())
        assert observations


def local_test_native_comparison_denial_holds_both_profiles(config, monkeypatch: pytest.MonkeyPatch) -> None:
    from bench.command_policy import ClaudeRunner
    from bench.prompt_comparison import prepare_comparison, run_comparison
    from bench.test_claude_containment import scripted_provider
    original = ClaudeRunner.command
    def denied(runner: ClaudeRunner) -> list[str]:
        command = original(runner)
        index = command.index("--allowedTools")
        del command[index:index + 2]
        return command
    monkeypatch.setattr(ClaudeRunner, "command", denied)
    with scripted_provider("printf 'after\\n' > input.md") as (endpoint, _):
        native = replace(config, task_ids=("T1", "T2", "T10"), command=(),
            claude_executable=os.environ["MDTOOLS_U5_RUNNER"], endpoint=endpoint,
            model="claude-haiku-4-5-20251001", thinking_policy="disabled", timeout_seconds=20)
        prepare_comparison(native)
        report = run_comparison(native)
    assert not report["complete"] and report["success_difference"] is None
    assert report["admission_hold"] == "permission_denied"
    root = Path(native.output_root)
    attempts = list(root.glob("*/campaign/attempts/*/started.json"))
    assert len(attempts) == 1
    _, result = harness.AttemptStore(attempts[0].parent).load()
    assert result.grade.kind == "not_run" and result.permission_fault == "permission_denied"
