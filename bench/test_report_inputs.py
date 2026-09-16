"""Synthetic subprocess/disk campaigns; never reads the task corpus or providers."""
from __future__ import annotations

from dataclasses import replace
import fcntl
import json
from pathlib import Path
import shutil
import subprocess
import sys

import pytest

from bench import harness
from bench.command_policy import ClaudeRunner, CliCondition, stage_condition, resolve_toolkit
from bench.harness import (AttemptStore, BenchTask, StructuralDiffPolicy, campaign_attempt_name,
                           freeze_campaign, run_campaign)
from bench.manifest import CampaignSpec, LiveRunGrant, canonical_json, serial_schedule, sha256_file
from bench.report import attempt_report, report_campaign
from bench.test_command_policy import cli_pins
from bench.test_trial_records import synthetic_cli_events, result as synthetic_result
from bench.trial_records import (AttemptKey, AttemptStart, ExecutionOutcome, Grade, Usage,
                                 RecordIntegrityError, record_dict)


@pytest.fixture
def campaign_case(tmp_path: Path, cli_pins: dict) -> tuple:
    root = tmp_path.resolve()
    fixtures, expected = root / "fixtures", root / "expected"
    fixtures.mkdir()
    expected.mkdir()
    (fixtures / "input.md").write_bytes(b"before\n")
    (expected / "answer.md").write_bytes(b"after\n")
    policy = StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False)
    # These IDs are synthetic, not corpus descriptions/inputs/expectations.
    tasks = [BenchTask(name, "Synthetic: write after.", ["input.md"], "answer.md", "file_contents", "synthetic", policy)
             for name in ("T14", "T23")]
    stub = stage_condition(None, root / "stub", toolkit=resolve_toolkit())
    conditions = {"no-md": stub, "legacy": cli_pins[CliCondition.LEGACY],
                  "current-compact": cli_pins[CliCondition.CURRENT_COMPACT]}
    events = synthetic_cli_events(final_text="synthetic finish")
    script = "from pathlib import Path; Path('input.md').write_bytes(b'after\\n'); print(" + repr("\n".join(json.dumps(event) for event in events)) + ")"
    command = (str(Path(sys.executable).resolve()), "-I", "-c", script)
    arguments = dict(fixture_root=fixtures, expected_root=expected, command=command,
                     conditions=conditions, event_format="claude_stream")
    spec = freeze_campaign(tasks, results_dir=root / "run", repetitions=2, **arguments)
    return root, tasks, spec, arguments


def publish_synthetic(spec: CampaignSpec, key: AttemptKey, path: Path, *,
                      execution: ExecutionOutcome = ExecutionOutcome("completed"),
                      grade: Grade | None = None, cost: float | None = 0.01,
                      denied: bool = False, crash: bool = False) -> object:
    store = AttemptStore(path)
    store.start(AttemptStart(key, "synthetic", reservation_usd=spec.reservation_usd))
    harness._write_private(path / "experiment.json", (canonical_json(record_dict(
        spec.members[spec.member_name(key.task_id, key.condition)])) + "\n").encode())
    if denied:
        store.append_event({"type": "system", "subtype": "permission_denied", "tool_name": "Bash", "tool_use_id": "synthetic-denial"})
    if crash:
        raise RuntimeError("synthetic crash after event")
    artifacts = {}
    for name in ("stdout.bin", "stderr.bin", "final_submission.bin"):
        harness._write_private(path / "artifacts" / name, b"synthetic")
        artifacts["artifacts/" + name] = sha256_file(path / "artifacts" / name)
    return store.finalize(execution=execution,
        grade=grade or (Grade("pass", "synthetic_match") if execution.kind == "completed" else Grade("not_run", execution.reason)),
        usage=Usage(estimated_usd=cost, source="synthetic_receipt" if cost is not None else "unavailable",
                    completeness="partial" if cost is not None else "unknown"),
        artifacts=artifacts, evidence_complete=True)


def test_three_condition_subprocess_campaign_report_parity(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    summary = run_campaign(spec, tasks, results_dir=root / "run", **arguments)
    assert summary["complete"] and summary["exit_code"] == 0
    assert summary["backend"] == "synthetic" and summary["requested_model"] is None
    assert summary["effort"] is None and summary["thinking_policy"] is None
    assert summary["grade_counts"] == {"pass": 12}
    assert all(cell["successes"] == 4 for cell in summary["cells"].values())
    assert summary == report_campaign((root / "run",))
    assert summary == json.loads((root / "run/report.json").read_bytes())
    assert summary["coverage"]["estimated_usd"]["total"] == pytest.approx(0.12)
    assert len(summary["comparisons"]) == 3 and not summary["core_study_complete"]
    command = [sys.executable, "-I", str(Path(harness.__file__).with_name("report.py")), str(root / "run")]
    direct = subprocess.run(command, capture_output=True, timeout=30)
    assert direct.returncode == 0 and json.loads(direct.stdout) == summary
    assert all(result["workflow_success_difference"]["estimate"] == 0 for result in summary["comparisons"])


def test_attempt_report_scalars_follow_selected_retry_not_input_order() -> None:
    first = synthetic_result(execution=ExecutionOutcome("infrastructure_error", "transient_transport"))
    second = synthetic_result(ordinal=1, grade=Grade("pass", "synthetic_match"))
    starts = [AttemptStart(first.key, "synthetic"), AttemptStart(second.key, "synthetic")]
    ordered = attempt_report(starts, [first, second])
    reversed_records = attempt_report(starts[::-1], [second, first])
    assert reversed_records == ordered
    assert reversed_records["execution"]["kind"] == "completed"
    assert reversed_records["grade"]["kind"] == "pass"
    assert reversed_records["disposition"]["selected"]["ordinal"] == 1
    with pytest.raises(RecordIntegrityError, match="unfinished"):
        attempt_report(starts, [first])


def test_default_cli_existing_result_is_structured_and_non_destructive(tmp_path: Path) -> None:
    output = tmp_path.resolve() / "existing"
    output.mkdir()
    marker = output / "retained"
    marker.write_bytes(b"do not overwrite")
    completed = subprocess.run([sys.executable, "-I", harness.__file__, "--offline-exercise", "--results-dir", str(output)],
        env={"PATH": "/usr/bin:/bin"}, capture_output=True, timeout=10)
    assert completed.returncode == 2
    message = json.loads(completed.stdout)
    assert message["complete"] is False and message["exit_code"] == 2 and message["comparisons"] == []
    assert "exists" in message["fault"].lower() and "results-dir" in message["help"]
    assert b"Traceback" not in completed.stderr
    assert list(output.iterdir()) == [marker] and marker.read_bytes() == b"do not overwrite"


def test_interrupted_resumed_subprocess_campaign_matches_uninterrupted(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    interrupted = run_campaign(spec, tasks, results_dir=root / "run", stop_after=4, **arguments)
    assert interrupted["exit_code"] == 1
    before = {path: path.read_bytes() for path in (root / "run/attempts").glob("*/result.json")}
    resumed = run_campaign(spec, tasks, results_dir=root / "run", **arguments)
    assert all(path.read_bytes() == content for path, content in before.items())
    uninterrupted = run_campaign(spec, tasks, results_dir=root / "uninterrupted", **arguments)
    for key in ("cells", "execution_counts", "grade_counts", "comparisons", "failures", "trials"):
        assert resumed[key] == uninterrupted[key]
    for measurement in ("estimated_usd", "input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens", "tool_output_bytes"):
        assert resumed["coverage"][measurement] == uninterrupted["coverage"][measurement]


@pytest.mark.parametrize("mutation", ["input", "expected", "prompt", "support", "command", "limit", "statistics", "policy", "runner", "toolkit", "grader", "harness", "configuration"])
def test_resume_rejects_all_changed_content_before_spawn(campaign_case: tuple, mutation: str, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, spec, arguments = campaign_case
    run_campaign(spec, tasks, results_dir=root / "run", stop_after=2, **arguments)
    changed_tasks, changed_spec, changed_arguments = tasks, spec, dict(arguments)
    if mutation == "input":
        (arguments["fixture_root"] / "input.md").write_bytes(b"changed")
    elif mutation == "expected":
        (arguments["expected_root"] / "answer.md").write_bytes(b"changed")
    elif mutation == "prompt":
        changed_tasks = [replace(tasks[0], description="Changed synthetic prompt"), tasks[1]]
    elif mutation == "support":
        (arguments["fixture_root"] / "support.md").write_bytes(b"changed")
        changed_tasks = [replace(tasks[0], support_files=["support.md"]), tasks[1]]
    elif mutation == "command":
        changed_arguments["command"] = (*arguments["command"][:2], "-c", "print('changed')")
    elif mutation == "limit":
        changed_arguments["timeout_seconds"] = 9
    else:
        field = {"statistics": "statistical_settings", "policy": "answer_policy_sha256", "runner": "executable_sha256",
                 "toolkit": "toolkit_sha256", "grader": "grader_sha256", "harness": "harness_sha256", "configuration": "runner_configuration_sha256"}[mutation]
        value = {"seed": 1729, "reps": 9999} if mutation == "statistics" else {"synthetic-tool": "0" * 64} if mutation == "toolkit" else "0" * 64
        members = dict(spec.members)
        first = next(iter(members))
        if mutation == "statistics":
            members = {name: replace(member, **{field: value}) for name, member in members.items()}
        else:
            members[first] = replace(members[first], **{field: value})
        changed_spec = replace(spec, members=members)
    calls = []
    with pytest.raises((RecordIntegrityError, ValueError)):
        run_campaign(changed_spec, changed_tasks, results_dir=root / "run", _executor=lambda *args: calls.append(args), **changed_arguments)
    assert calls == []
    assert len(list((root / "run/attempts").glob("*/result.json"))) == 2


def test_campaign_rejects_inconsistent_statistical_settings(campaign_case: tuple) -> None:
    _, _, spec, _ = campaign_case
    members = dict(spec.members)
    first = next(iter(members))
    members[first] = replace(members[first], statistical_settings={"seed": 0, "reps": 10000})
    with pytest.raises(RecordIntegrityError, match="common statistical"):
        replace(spec, members=members)


def test_campaign_round_trip_relocation_and_frozen_block_validation(campaign_case: tuple) -> None:
    _, _, spec, _ = campaign_case
    decoded = CampaignSpec.from_dict(json.loads(canonical_json(record_dict(spec))))
    assert decoded.identity == spec.identity and decoded.schedule == spec.schedule
    relocated = replace(spec, members={name: replace(member, locators={"runner": "/synthetic/moved"}) for name, member in spec.members.items()})
    assert relocated.identity == spec.identity
    invalid = list(spec.schedule)
    invalid[0], invalid[3] = invalid[3], invalid[0]
    with pytest.raises(RecordIntegrityError, match="serial trial blocks"):
        replace(spec, schedule=tuple(invalid))


def test_report_source_mismatch_retains_recorded_descriptive_counts(campaign_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, spec, arguments = campaign_case
    original = run_campaign(spec, tasks, results_dir=root / "run", **arguments)
    monkeypatch.setattr(harness, "harness_source_identity", lambda: "0" * 64)
    diagnostic = report_campaign((root / "run",))
    assert diagnostic["faults"] == ["changed_reporting_source"] and diagnostic["comparisons"] == []
    assert diagnostic["grade_counts"] == original["grade_counts"] and diagnostic["coverage"] == original["coverage"]
    assert diagnostic["source_mismatches"]["harness_sha256"]["current"] == "0" * 64


def test_actual_cli_controller_stop_resume_and_all_report_paths(campaign_case: tuple) -> None:
    root, _, _, _ = campaign_case
    repo = Path(harness.__file__).resolve().parent.parent
    pin_root = root / "pins"
    # Pins come from explicit verified builds/receipts, never installed md.
    source_pins = campaign_case[3]["conditions"]
    for folder, condition in (("legacy", "legacy"), ("current", "current-compact")):
        (pin_root / folder).mkdir(parents=True)
        (pin_root / folder / "pin.json").write_text(canonical_json({**record_dict(source_pins[condition]), "condition": condition}))
    campaign_root = root / "controller"
    command = [sys.executable, "-I", str(repo / "bench/harness.py"), "--offline-campaign",
               "--pin-root", str(pin_root), "--results-dir", str(campaign_root)]
    partial = subprocess.run([*command, "--stop-after", "3"], capture_output=True, timeout=30)
    assert partial.returncode == 1, partial.stderr.decode()
    before = {path: path.read_bytes() for path in (campaign_root / "attempts").glob("*/result.json")}
    resumed = subprocess.run(command, capture_output=True, timeout=30)
    assert resumed.returncode == 0, resumed.stdout.decode() + resumed.stderr.decode()
    summary = json.loads(resumed.stdout)
    assert len(before) == 3 and all(path.read_bytes() == content for path, content in before.items())
    assert summary == report_campaign((campaign_root,))
    report = subprocess.run([sys.executable, "-I", str(repo / "bench/harness.py"), "--report-bundle", str(campaign_root)], capture_output=True, timeout=30)
    assert report.returncode == 0 and json.loads(report.stdout) == summary


def test_duplicate_bundle_and_missing_trials_are_diagnostic(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    partial = run_campaign(spec, tasks, results_dir=root / "run", stop_after=1, **arguments)
    assert len(partial["missing_or_unresolved_trials"]) == 11 and partial["comparisons"] == []
    shutil.copytree(root / "run", root / "copy")
    with pytest.raises(RecordIntegrityError, match="duplicate"):
        report_campaign((root / "run", root / "copy"))
    cli = subprocess.run([sys.executable, "-I", str(Path(harness.__file__).with_name("report.py")), str(root / "run"), str(root / "copy")], capture_output=True, timeout=10)
    assert cli.returncode == 2 and json.loads(cli.stdout)["comparisons"] == []


def test_campaign_lock_excludes_second_controller(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    run_campaign(spec, tasks, results_dir=root / "run", stop_after=0, **arguments)
    with (root / "run/.campaign.lock").open("r+b") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        with pytest.raises(BlockingIOError):
            run_campaign(spec, tasks, results_dir=root / "run", **arguments)


def test_nested_campaign_control_directories_are_private(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    directory = root / "new-parent" / "new-control" / "campaign"
    run_campaign(spec, tasks, results_dir=directory, stop_after=1, **arguments)
    assert all(path.stat().st_mode & 0o777 == 0o700 for path in (root / "new-parent", root / "new-parent/new-control", directory))
    assert all(path.stat().st_mode & 0o777 == 0o700 for path in directory.rglob("*") if path.is_dir())
    assert all(path.stat().st_mode & 0o777 == 0o600 for path in directory.rglob("*") if path.is_file())


def test_invalid_grader_holds_actual_subprocess_campaign(campaign_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, spec, arguments = campaign_case
    calls = []
    def invalid_grade(*args, **kwargs):
        calls.append(1)
        return object()
    monkeypatch.setattr(harness, "grade_submission", invalid_grade)
    summary = run_campaign(spec, tasks, results_dir=root / "run", **arguments)
    assert len(calls) == 1 and summary["faults"] == ["grader_unavailable"]
    assert summary["exit_code"] == 1 and summary["grade_counts"] == {"unavailable": 1}
    assert summary["coverage"]["estimated_usd"]["total"] == 0.01


@pytest.mark.parametrize("position", [0, 7])
def test_permission_event_survives_actual_controller_crash(campaign_case: tuple, position: int) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, prefix_length=6, grant_usd=1, reservation_usd=0.1)
    run_campaign(spec, tasks, results_dir=root / "run", stop_after=position, **arguments)
    key = AttemptKey(spec.identity, *spec.schedule[position], 0)
    path = root / "run/attempts" / campaign_attempt_name(key)
    repo = str(Path(harness.__file__).resolve().parent.parent)
    script = "\n".join([
        "import os, sys; from pathlib import Path; sys.path.insert(0, " + repr(repo) + ")",
        "from bench.harness import AttemptStore",
        "from bench.trial_records import AttemptKey, AttemptStart",
        "key = AttemptKey(**" + repr(record_dict(key)) + ")",
        "store = AttemptStore(Path(" + repr(str(path)) + "))",
        "store.start(AttemptStart(key, 'synthetic', reservation_usd=0.1))",
        "store.append_event({'type':'system','subtype':'permission_denied','tool_name':'Bash','tool_use_id':'synthetic-crash'})",
        "os._exit(81)",
    ])
    crashed = subprocess.run([sys.executable, "-I", "-c", script], capture_output=True, timeout=10)
    assert crashed.returncode == 81, crashed.stderr.decode()
    assert not (path / "result.json").exists() and not (path / "experiment.json").exists()
    calls = []
    resumed = run_campaign(spec, tasks, results_dir=root / "run", _executor=lambda *args: calls.append(args), **arguments)
    assert calls == [] and resumed["exit_code"] == 1 and "permission_denied" in resumed["faults"]
    assert resumed["coverage"]["budget"]["unresolved_reservation_usd"] == 0.1
    assert resumed["comparisons"] == []


@pytest.mark.parametrize("cost", [None, 0.0, 0.02])
def test_budget_hold_precedes_effect_and_retains_unknown_reservation(campaign_case: tuple, cost: float | None) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, grant_usd=0.02, reservation_usd=0.01)
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        return publish_synthetic(spec, key, path, cost=cost)
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    if cost is None:
        assert len(calls) == 1 and summary["admission_hold"] == "unknown_estimated_cost"
        assert summary["coverage"]["estimated_usd"]["total"] is None
        assert summary["coverage"]["budget"]["unresolved_reservation_usd"] == 0.01
    elif cost == 0:
        assert len(calls) == 12 and summary["exit_code"] == 0
        assert summary["coverage"]["estimated_usd"]["total"] == 0
    else:
        assert len(calls) == 1 and summary["admission_hold"] == "grant_exhausted"
        assert summary["coverage"]["estimated_usd"]["total"] == 0.02


def test_exact_decimal_budget_admits_last_authorized_reservation(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, grant_usd=0.3, reservation_usd=0.1)
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        return publish_synthetic(spec, key, path, cost=0.1)
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 3 and summary["admission_hold"] == "grant_exhausted"


def test_retry_is_ordinal_and_retains_all_attempted_costs(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        if len(calls) == 1:
            return publish_synthetic(spec, key, path, execution=ExecutionOutcome("infrastructure_error", "transient_transport"), cost=0.03)
        return publish_synthetic(spec, key, path, cost=0.01)
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 13 and calls[1].ordinal == 1 and calls[1].trial == calls[0].trial
    assert summary["cells"][calls[0].condition]["successes"] == 4
    assert summary["coverage"]["estimated_usd"]["total"] == pytest.approx(0.15)


@pytest.mark.parametrize("reason", ["authentication_error", "configuration_error", "model_mismatch", "receipt_integrity"])
def test_configuration_fault_stops_campaign_and_withholds_comparisons(campaign_case: tuple, reason: str) -> None:
    root, tasks, spec, arguments = campaign_case
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        return publish_synthetic(spec, key, path, execution=ExecutionOutcome("infrastructure_error", reason), cost=0.03)
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 1 and summary["exit_code"] == 1 and reason in summary["faults"]
    assert not summary["comparative_performance_eligible"] and summary["comparisons"] == []
    resumed = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 1 and resumed == summary


@pytest.mark.parametrize("position", [0, 7])
@pytest.mark.parametrize("crash", [False, True])
def test_permission_fault_survives_resume(campaign_case: tuple, position: int, crash: bool) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, prefix_length=6, grant_usd=1, reservation_usd=0.1)
    run_campaign(spec, tasks, results_dir=root / "run", stop_after=position, **arguments)
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        return publish_synthetic(spec, key, path, denied=True, crash=crash, cost=None,
            execution=ExecutionOutcome("infrastructure_error", "permission_denied"))
    if crash:
        with pytest.raises(RuntimeError, match="synthetic crash"):
            run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    else:
        summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
        assert summary["grade_counts"].get("not_run") == 1
    resumed = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 1 and resumed["exit_code"] == 1
    assert "permission_denied" in resumed["faults"] and resumed["comparisons"] == []
    assert resumed["coverage"]["budget"]["unresolved_reservation_usd"] == 0.1
    regenerated = report_campaign((root / "run",))
    for field in ("faults", "cells", "comparisons", "coverage", "exit_code"):
        assert regenerated[field] == resumed[field]


def test_core_prefix_keys_occur_once() -> None:
    schedule = serial_schedule(tuple(f"T{n}" for n in range(1, 25)), core_study=True)
    keys = {(task, condition, 0) for task in ("T14", "T23") for condition in ("no-md", "legacy", "current-compact")}
    assert len(schedule) == len(set(schedule)) == 360 and set(schedule[:6]) == keys
    assert not (set(schedule[6:]) & keys)
    assert serial_schedule(tuple(f"T{n}" for n in range(1, 25)), core_study=True) == schedule


@pytest.mark.parametrize("failure", ["unavailable", "contract_error", "unresolved_packaging", "incomplete"])
def test_core_prefix_holds_before_next_spawn(campaign_case: tuple, failure: str) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, prefix_length=6)
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        if failure == "incomplete":
            return publish_synthetic(spec, key, path, execution=ExecutionOutcome("timed_out", "deadline"))
        return publish_synthetic(spec, key, path, grade=Grade("unavailable", failure))
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    # An incomplete prefix cannot be moved past, even after the first timeout
    # becomes an ordinary terminal disposition.
    assert len(calls) == 1 and summary["exit_code"] == 1 and summary["comparisons"] == []
    run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 1


def test_incomplete_first_prefix_execution_holds_even_with_complete_evidence(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, prefix_length=6)
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        return publish_synthetic(spec, key, path, execution=ExecutionOutcome("budget_exhausted", "turn_limit"))
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert len(calls) == 1 and summary["admission_hold"] == "incomplete_contract_prefix"
    assert report_campaign((root / "run",))["faults"] == ["incomplete_contract_prefix"]


def test_core_prefix_resume_and_zero_retries(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, prefix_length=6)
    first = run_campaign(spec, tasks, results_dir=root / "run", stop_after=3, **arguments)
    before = set((root / "run/attempts").iterdir())
    calls = []
    def execute(key, prior, path):
        calls.append(key)
        return publish_synthetic(spec, key, path, execution=ExecutionOutcome("infrastructure_error", "transient_transport"))
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert before.issubset(set((root / "run/attempts").iterdir())) and len(calls) == 1
    assert all(key.ordinal == 0 for key in calls) and summary["exit_code"] == 1
    assert spec.retry_allowance((spec.identity, *spec.schedule[6])) == 1


def test_supported_wrong_answers_remain_reportable_and_empty_intersection(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    spec = replace(spec, prefix_length=6)
    def execute(key, prior, path):
        return publish_synthetic(spec, key, path, grade=Grade("fail", "synthetic_wrong"), cost=0)
    summary = run_campaign(spec, tasks, results_dir=root / "run", _executor=execute, **arguments)
    assert summary["exit_code"] == 0 and summary["grade_counts"] == {"fail": 12}
    assert all(cell["successes"] == 0 for cell in summary["cells"].values())
    assert all(comparison["successful_intersection_cost"]["delta"] is None for comparison in summary["comparisons"])


def test_family_local_loss_does_not_complete_core_study(campaign_case: tuple) -> None:
    root, tasks, spec, arguments = campaign_case
    # Metadata-only schedule construction; all 24 task bodies are synthetic.
    synthetic = [replace(tasks[0], id=f"T{n}") for n in range(1, 25)]
    core = freeze_campaign(synthetic, results_dir=root / "core", core_study=True, **arguments)
    summary = run_campaign(core, synthetic, results_dir=root / "core", stop_after=6, **arguments)
    assert not summary["core_study_complete"] and len(summary["missing_or_unresolved_trials"]) == 354
    assert summary["comparisons"] == []
    with pytest.raises(RecordIntegrityError, match="360"):
        replace(core, schedule=tuple(entry for entry in core.schedule if entry[0] not in ("T14", "T23")),
                members={name: member for name, member in core.members.items() if not name.startswith(("T14/", "T23/"))}, prefix_length=0)


@pytest.fixture
def live_case(campaign_case: tuple, monkeypatch: pytest.MonkeyPatch) -> tuple:
    root, tasks, _, arguments = campaign_case
    # Invented configuration with launch spies. No Claude/version/auth action.
    runner = ClaudeRunner("/usr/bin/false", None, "claude-sonnet-5", "high", "adaptive", 30, 0.1)
    monkeypatch.setattr(ClaudeRunner, "verify", lambda self: None)
    monkeypatch.setattr(ClaudeRunner, "provenance_locators", lambda self: {"runner": self.executable,
        "runner_source": str(root / "synthetic-pruned-source"), "runner_pin_receipt": str(root / "synthetic-pin.json"),
        "runner_preserved_at": "synthetic timestamp"})
    profile = root / "synthetic-runtime-profile.sb"
    profile.write_bytes(b"synthetic runtime profile")
    monkeypatch.setattr(harness, "DYLD_PROFILE", profile)
    tasks = [replace(tasks[0], id="synthetic-canary")]
    args = {**arguments, "command": (), "claude": runner}
    spec = freeze_campaign(tasks, results_dir=root / "live", repetitions=1, retry_allowance=0,
                           grant_usd=0.3, reservation_usd=0.1, **args)
    grant = LiveRunGrant(spec.identity, runner.model, runner.effort, runner.thinking_policy, "canary",
        ("synthetic-canary",), ("no-md", "legacy", "current-compact"), (0,), 10, 30, 0.1, 0.3, 3, False, None)
    return root, tasks, spec, args, grant


def test_valid_live_grant_reserves_before_auth_and_spawn_spy(live_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, spec, arguments, grant = live_case
    calls = []
    class SyntheticBoundary:
        def __init__(self, workspace):
            self.workspace = workspace
            for name in ("boundary.json", "profile.sb", "shell.json", "bash-eval-launcher"):
                harness._write_private(workspace / "control" / name, b"synthetic boundary")
        def verify(self):
            calls.append("boundary_verified")
        def parent_environment(self, runner):
            started = AttemptStore(self.workspace.parent).load()[0]
            assert started.reservation_usd == 0.1 and started.backend == "claude_cli"
            calls.append("auth_spy_after_reservation")
            return {"PATH": "/usr/bin:/bin"}
    monkeypatch.setattr(harness, "prepare_native_boundary", lambda workspace, *args, **kwargs: SyntheticBoundary(workspace))
    original_spawn = harness.subprocess.Popen
    def no_provider_spawn(*args, **kwargs):
        if args[0][0] != "/usr/bin/false":
            return original_spawn(*args, **kwargs)
        calls.append("spawn_spy")
        raise OSError("synthetic blocked spawn")
    monkeypatch.setattr(harness.subprocess, "Popen", no_provider_spawn)
    summary = run_campaign(spec, tasks, results_dir=root / "live", live_grant=grant, **arguments)
    assert calls == ["boundary_verified", "auth_spy_after_reservation", "spawn_spy"]
    assert summary["exit_code"] == 1 and summary["coverage"]["budget"]["unresolved_reservation_usd"] == 0.1


@pytest.mark.parametrize("fault", ["missing", "identity", "model", "limits", "attempt_budget", "phase"])
def test_live_grant_mismatch_stops_before_auth_or_spawn(live_case: tuple, monkeypatch: pytest.MonkeyPatch, fault: str) -> None:
    root, tasks, spec, arguments, grant = live_case
    if fault == "missing":
        grant = None
    elif fault == "identity":
        grant = replace(grant, experiment_id="0" * 64)
    elif fault == "model":
        grant = replace(grant, model="invented-other-model")
    elif fault == "limits":
        grant = replace(grant, max_turns=2)
    elif fault == "attempt_budget":
        grant = replace(grant, attempt_usd=0.2)
    elif fault == "phase":
        grant = replace(grant, phase="public_pilot", prerequisite_experiment_id="0" * 64)
    calls = []
    monkeypatch.setattr(harness, "prepare_native_boundary", lambda *args, **kwargs: calls.append("boundary"))
    monkeypatch.setattr(harness.subprocess, "Popen", lambda *args, **kwargs: calls.append("spawn"))
    with pytest.raises(RecordIntegrityError):
        run_campaign(spec, tasks, results_dir=root / "live", live_grant=grant, **arguments)
    assert calls == [] and not (root / "live").exists()


def test_live_single_task_cannot_launch_outside_campaign(live_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, _, arguments, _ = live_case
    calls = []
    monkeypatch.setattr(harness, "prepare_native_boundary", lambda *args, **kwargs: calls.append("boundary"))
    with pytest.raises(RecordIntegrityError, match="campaign-locked"):
        harness.run_agent(tasks[0], fixture_root=arguments["fixture_root"], expected_root=arguments["expected_root"],
            command=(), results_dir=root / "live", claude=arguments["claude"], condition=arguments["conditions"]["no-md"])
    assert calls == []


def test_live_pilot_cannot_inherit_missing_or_synthetic_canaries(live_case: tuple, campaign_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, _, arguments, grant = live_case
    public = [replace(tasks[0], id=name) for name in ("T1", "T2", "T10")]
    spec = freeze_campaign(public, results_dir=root / "live", repetitions=1, grant_usd=0.3, reservation_usd=0.1, **arguments)
    pilot_grant = replace(grant, experiment_id=spec.identity, phase="public_pilot", task_ids=("T1", "T2", "T10"), max_attempts=9,
                          prerequisite_experiment_id="0" * 64)
    calls = []
    monkeypatch.setattr(harness, "prepare_native_boundary", lambda *args, **kwargs: calls.append("boundary"))
    with pytest.raises(RecordIntegrityError, match="prerequisite"):
        run_campaign(spec, public, results_dir=root / "live", live_grant=pilot_grant, **arguments)
    _, synthetic_tasks, synthetic_spec, synthetic_args = campaign_case
    canaries = [replace(synthetic_tasks[0], id="synthetic-canary")]
    canary_spec = freeze_campaign(canaries, results_dir=root / "canary", repetitions=1, retry_allowance=0, **synthetic_args)
    run_campaign(canary_spec, canaries, results_dir=root / "canary", **synthetic_args)
    pilot_grant = replace(pilot_grant, prerequisite_experiment_id=canary_spec.identity)
    with pytest.raises(RecordIntegrityError, match="synthetic prerequisite"):
        run_campaign(spec, public, results_dir=root / "live", live_grant=pilot_grant, prerequisite_bundle=root / "canary", **arguments)
    assert calls == []


def test_live_grant_closed_core_scope_and_contract_acknowledgment(live_case: tuple) -> None:
    root, tasks, _, arguments, grant = live_case
    core_tasks = [replace(tasks[0], id=f"T{n}") for n in range(1, 25)]
    spec = freeze_campaign(core_tasks, results_dir=root / "core", core_study=True, grant_usd=40, reservation_usd=0.1, **arguments)
    core_grant = replace(grant, experiment_id=spec.identity, phase="core_study", task_ids=tuple(f"T{n}" for n in range(1, 25)),
                         repetitions=tuple(range(5)), campaign_usd=40, max_attempts=360, prerequisite_experiment_id="0" * 64)
    with pytest.raises(RecordIntegrityError, match="acknowledgment"):
        core_grant.assert_scope(spec)
    replace(core_grant, core_contract_risk_acknowledged=True).assert_scope(spec)
    with pytest.raises(RecordIntegrityError, match="scope"):
        replace(core_grant, task_ids=("T1", "T2", "T10"), core_contract_risk_acknowledged=True).assert_scope(spec)


def test_synthetic_live_shape_core_selection_retains_retry_cost(live_case: tuple) -> None:
    """Invented live-shaped records test selection only, not provider evidence."""
    root, tasks, _, arguments, _ = live_case
    core_tasks = [replace(tasks[0], id=f"T{n}") for n in range(1, 25)]
    spec = freeze_campaign(core_tasks, results_dir=root / "core", core_study=True, grant_usd=40, reservation_usd=0.1, **arguments)
    from bench.test_trial_records import result as shape_result
    from bench.report import summarize
    results = []
    for entry in spec.schedule:
        key = AttemptKey(spec.identity, *entry, 0)
        results.append(replace(shape_result(), key=key, backend="claude_cli", requested_model="claude-sonnet-5",
            observed_model="claude-sonnet-5", usage=Usage(0, 0, 0, 0, 0.01, 0, 0, "claude_terminal", "complete")))
    selected = results[6]
    failed = replace(selected, execution=ExecutionOutcome("infrastructure_error", "transient_transport"),
                     grade=Grade("not_run", "transient_transport"))
    results[6] = failed
    results.append(replace(selected, key=replace(selected.key, ordinal=1)))
    starts = [AttemptStart(result.key, result.backend, reservation_usd=0.1, requested_model=result.requested_model) for result in results]
    summary = summarize(tuple((spec.identity, *entry) for entry in spec.schedule), starts, results, campaign=spec)
    assert summary["core_study_complete"] and summary["grade_counts"] == {"pass": 360, "not_run": 1}
    assert summary["coverage"]["estimated_usd"]["total"] == pytest.approx(3.61)


def test_grant_artifact_has_closed_identity_and_is_not_resume_authority(live_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    root, tasks, spec, arguments, grant = live_case
    summary = run_campaign(spec, tasks, results_dir=root / "live", live_grant=grant, stop_after=0, **arguments)
    evidence = json.loads((root / "live/authorization-evidence.json").read_bytes())
    assert evidence["grant_id"] == grant.identity
    assert evidence["grant"] == record_dict(grant)
    assert summary["authorization_evidence_is_launch_authority"] is False
    with pytest.raises(RecordIntegrityError, match="explicit scoped grant"):
        run_campaign(spec, tasks, results_dir=root / "live", **arguments)
    (root / "live/authorization-evidence.json").write_text(canonical_json({**evidence, "grant_id": "0" * 64}))
    with pytest.raises(RecordIntegrityError, match="digest mismatch"):
        report_campaign((root / "live",))
    with pytest.raises(RecordIntegrityError, match="changed live authorization"):
        run_campaign(spec, tasks, results_dir=root / "live", live_grant=grant, **arguments)
