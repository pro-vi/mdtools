"""Maintained operator entry points, using public fixtures and synthetic argv only."""
from __future__ import annotations

from dataclasses import asdict, replace
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys

import pytest

from bench import harness
from bench.command_policy import CliCondition
from bench.manifest import CORE_TASK_IDS, CORPUS_PATHS, CampaignConfig, CorePreparationConsent
from bench.trial_records import RecordIntegrityError, decode_record_json, record_dict


@pytest.fixture
def config(tmp_path: Path, cli_pins: dict) -> CampaignConfig:
    pin_root = tmp_path.resolve() / "condition-receipts"
    for name, condition in (("legacy", CliCondition.LEGACY), ("current", CliCondition.CURRENT_COMPACT)):
        directory = pin_root / name
        directory.mkdir(parents=True)
        (directory / "pin.json").write_text(json.dumps(record_dict(cli_pins[condition])))
    return CampaignConfig(task_ids=("T1",), pin_root=str(pin_root),
        source_root=str(Path(harness.__file__).resolve().parent.parent),
        output_root=str(tmp_path.resolve() / "operator"),
        command=(str(Path(sys.executable).resolve()), "-I", "-c", "print('[]')"),
        claude_executable=None, endpoint=None, model=None, effort=None, thinking_policy=None,
        guidance="full_help", repetitions=1, seed=1729, timeout_seconds=10,
        max_turns=12, retry_allowance=0, attempt_usd=0.25, campaign_usd=2.25)


def test_config_roundtrip_and_subprocess_prepare_run(config: CampaignConfig, tmp_path: Path) -> None:
    raw = record_dict(config)
    assert CampaignConfig.from_dict(raw) == config
    path = tmp_path / "config.json"
    path.write_text(json.dumps(raw))
    entry = [sys.executable, "-I", harness.__file__]
    prepared = subprocess.run([*entry, "--prepare-config", str(path)], capture_output=True, timeout=40)
    assert prepared.returncode == 0, prepared.stderr.decode() + prepared.stdout.decode()
    proposal = decode_record_json(prepared.stdout)
    assert proposal["authorization"] == "not granted"
    assert not (Path(config.output_root) / "campaign" / "attempts").exists()
    run = subprocess.run([*entry, "--run-config", str(path)], capture_output=True, timeout=40)
    assert run.returncode == 0, run.stderr.decode() + run.stdout.decode()
    summary = decode_record_json(run.stdout)
    assert summary["complete"] and summary["grade_counts"] == {"fail": 3}
    resumed = subprocess.run([*entry, "--run-config", str(path)], capture_output=True, timeout=40)
    assert resumed.returncode == 0 and decode_record_json(resumed.stdout) == summary


@pytest.mark.parametrize("field,value", [("task_ids", ["T14"]), ("guidance", "other"),
    ("repetitions", True), ("timeout_seconds", 0), ("unknown", 1)])
def test_config_rejects_invalid_input_before_reads(config: CampaignConfig, field: str, value: object) -> None:
    raw = record_dict(config)
    raw[field] = value
    with pytest.raises((ValueError, RecordIntegrityError)):
        CampaignConfig.from_dict(raw)
    assert not Path(config.output_root).exists()


def test_core_config_accepts_only_complete_scope(config: CampaignConfig) -> None:
    core = replace(config, task_ids=tuple(f"T{n}" for n in range(1, 25)), repetitions=5)
    assert core.core_study
    assert CampaignConfig.from_dict(record_dict(core)) == core
    with pytest.raises(RecordIntegrityError):
        replace(core, task_ids=core.task_ids[:-1])
    with pytest.raises(RecordIntegrityError):
        replace(core, repetitions=1)


@pytest.fixture
def core_case(config: CampaignConfig, tmp_path: Path) -> tuple[CampaignConfig, CorePreparationConsent]:
    source = tmp_path.resolve() / "synthetic-source"
    for name in CORPUS_PATHS:
        (source / name).mkdir(parents=True)
    policy = harness.StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False)
    rows = []
    for task_id in CORE_TASK_IDS:
        artifact = "stdout_and_file" if task_id == "T14" else "stdout_text" if task_id == "T23" else "file_contents"
        task = harness.BenchTask(task_id, "Write after and submit [].", ["bench/inputs/input.md"],
            "bench/expected/text.md" if artifact == "stdout_text" else "bench/expected/file.md",
            artifact, "synthetic", policy, expected_stdout="[]\n" if artifact == "stdout_and_file" else None)
        rows.append(asdict(task))
    (source / "bench/tasks/tasks.json").write_text(json.dumps(rows))
    (source / "bench/inputs/input.md").write_bytes(b"before\n")
    (source / "bench/expected/file.md").write_bytes(b"after\n")
    (source / "bench/expected/text.md").write_bytes(b"[]\n")
    (source / "bench/holdout/task_ids.json").write_text('["T14", "T23"]')
    def git(*argv: str) -> str:
        return subprocess.run(["git", *argv], cwd=source, capture_output=True, check=True).stdout.decode().strip()
    git("init", "-q")
    git("add", "bench")
    git("-c", "core.hooksPath=/dev/null", "commit", "--no-gpg-sign", "-qm", "Synthetic corpus fixture")
    revision = git("rev-parse", "HEAD")
    core = replace(config, task_ids=CORE_TASK_IDS, repetitions=5, source_root=str(source),
        command=(str(Path(sys.executable).resolve()), "-I", "-c",
            "from pathlib import Path; Path('bench/inputs/input.md').write_bytes(b'after\\n'); print('[]')"))
    consent = CorePreparationConsent("Synthetic permission fixture; no real corpus authorized.", str(source), revision,
        {path: git("rev-parse", f"{revision}:{path}") for path in CORPUS_PATHS},
        CORE_TASK_IDS, (core.output_root,), True)
    return core, consent


def test_core_preparation_denied_before_source_read(core_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    core, consent = core_case
    def forbidden(*args: object, **kwargs: object) -> None:
        pytest.fail("source read occurred without matching consent")
    monkeypatch.setattr(harness, "_read_source", forbidden)
    for authority in (None, replace(consent, output_roots=(str(Path(core.output_root).parent / "other"),))):
        with pytest.raises(RecordIntegrityError, match="consent"):
            harness.configured_campaign(core, prepare_only=True, preparation_consent=authority)
    assert not Path(core.output_root).exists()


def test_core_preparation_preserves_full_schedule_and_prefix(core_case: tuple) -> None:
    core, consent = core_case
    assert CorePreparationConsent.from_dict(record_dict(consent)) == consent
    proposal = harness.configured_campaign(core, prepare_only=True, preparation_consent=consent)
    spec = harness.CampaignSpec.from_dict(proposal["spec"])
    assert spec.core_study and len(set(spec.schedule)) == 360 and spec.prefix_length == 6
    assert {entry[0] for entry in spec.schedule[:6]} == {"T14", "T23"}
    assert not (Path(core.output_root) / "campaign/attempts").exists()
    assert harness.configured_campaign(core, prepare_only=True, preparation_consent=consent) == proposal
    with pytest.raises(RecordIntegrityError, match="explicit controller-read consent"):
        harness.configured_campaign(core, prepare_only=True)


def test_core_preparation_rejects_changed_corpus_before_output(core_case: tuple) -> None:
    core, consent = core_case
    (Path(core.source_root) / "bench/expected/file.md").write_text("changed")
    with pytest.raises(RecordIntegrityError, match="identity check failed"):
        harness.configured_campaign(core, prepare_only=True, preparation_consent=consent)
    assert not Path(core.output_root).exists()


def test_preparation_consent_never_authorizes_launch(core_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    core, consent = core_case
    live = replace(core, command=(), claude_executable="/missing/claude",
        model="claude-haiku-4-5-20251001", thinking_policy="disabled")
    def forbidden(*args: object, **kwargs: object) -> None:
        pytest.fail("preparation or spawn occurred without launch grant")
    monkeypatch.setattr(harness, "prepare_campaign_config", forbidden)
    with pytest.raises(RecordIntegrityError, match="explicitly supplied grant"):
        harness.configured_campaign(live, preparation_consent=consent)


def test_live_core_preparation_requires_pilot_before_reads(core_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    core, consent = core_case
    live = replace(core, command=(), claude_executable="/missing/claude",
        model="claude-haiku-4-5-20251001", thinking_policy="disabled")
    def forbidden(*args: object, **kwargs: object) -> None:
        pytest.fail("core read occurred before pilot validation")
    monkeypatch.setattr(harness, "_read_source", forbidden)
    with pytest.raises(RecordIntegrityError, match="public pilot prerequisite"):
        harness.configured_campaign(live, prepare_only=True, preparation_consent=consent)


def test_core_worker_cannot_receive_another_tasks_expected_file(core_case: tuple) -> None:
    core, consent = core_case
    source = Path(core.source_root)
    registry = source / "bench/tasks/tasks.json"
    rows = json.loads(registry.read_bytes())
    rows[0]["support_files"] = ["bench/expected/text.md"]
    registry.write_text(json.dumps(rows))
    def git(*argv: str) -> str:
        return subprocess.run(["git", *argv], cwd=source, capture_output=True, check=True).stdout.decode().strip()
    git("add", "bench/tasks/tasks.json")
    git("-c", "core.hooksPath=/dev/null", "commit", "--no-gpg-sign", "-qm", "Synthetic invalid task reference")
    revision = git("rev-parse", "HEAD")
    consent = replace(consent, source_commit=revision,
        corpus_objects={path: git("rev-parse", f"{revision}:{path}") for path in CORPUS_PATHS})
    with pytest.raises(RecordIntegrityError, match="unsupported core task contract"):
        harness.configured_campaign(core, prepare_only=True, preparation_consent=consent)
    assert not Path(core.output_root).exists()


def test_core_prepare_validates_public_pilot_phase(core_case: tuple, monkeypatch: pytest.MonkeyPatch) -> None:
    core, consent = core_case
    proposal = harness.configured_campaign(core, prepare_only=True, preparation_consent=consent)
    spec = harness.CampaignSpec.from_dict(proposal["spec"])
    monkeypatch.setattr(harness, "_load_campaign", lambda root: spec)
    phases = []
    monkeypatch.setattr(harness, "assert_prerequisite", lambda spec, **kwargs: phases.append(kwargs["phase"]))
    harness.configured_campaign(core, prepare_only=True, preparation_consent=consent,
        prerequisite_bundle=Path(core.output_root))
    assert phases == ["core_study"]


def local_test_core_config_resume_and_independent_report(core_case: tuple, tmp_path: Path) -> None:
    core, consent = core_case
    config_path, consent_path = tmp_path / "core-config.json", tmp_path / "read-consent.json"
    config_path.write_text(json.dumps(record_dict(core)))
    consent_path.write_text(json.dumps(record_dict(consent)))
    entry = [sys.executable, "-I", harness.__file__]
    def call(mode: str, *extra: str) -> dict:
        completed = subprocess.run([*entry, mode, str(config_path), "--preparation-consent", str(consent_path), *extra],
            capture_output=True, timeout=1200)
        assert completed.returncode in (0, 1), completed.stdout.decode() + completed.stderr.decode()
        return decode_record_json(completed.stdout)
    proposal = call("--prepare-config")
    assert len(proposal["spec"]["schedule"]) == 360
    partial = call("--run-config", "--stop-after", "6")
    assert not partial["complete"]
    finished = call("--run-config")
    assert finished["complete"] and not finished["core_study_complete"]  # Synthetic evidence only.
    assert finished["grade_counts"] == {"pass": 360}
    attempts = Path(core.output_root) / "campaign/attempts"
    assert len(list(attempts.iterdir())) == 360
    assert call("--run-config") == finished
    regenerated = subprocess.run([*entry, "--report-bundle", str(Path(core.output_root) / "campaign")],
        capture_output=True, timeout=60)
    assert regenerated.returncode == 0, regenerated.stdout.decode() + regenerated.stderr.decode()
    assert decode_record_json(regenerated.stdout) == finished


def test_core_config_resume_preserves_prefix(core_case: tuple) -> None:
    core, consent = core_case
    harness.configured_campaign(core, prepare_only=True, preparation_consent=consent)
    partial = harness.configured_campaign(core, preparation_consent=consent, stop_after=6)
    assert not partial["complete"] and partial["grade_counts"] == {"pass": 6}
    continued = harness.configured_campaign(core, preparation_consent=consent, stop_after=1)
    assert continued["grade_counts"] == {"pass": 7}
    spec, starts, results, faults = harness.load_campaign_bundles((Path(core.output_root) / "campaign",))
    assert len(starts) == len(results) == 7 and not faults
    assert len({start.key for start in starts}) == 7
    assert {entry[0] for entry in spec.schedule[:6]} == {"T14", "T23"}


@pytest.mark.parametrize("field,value", [("prior_exposure_acknowledged", False), ("action", "launch"),
    ("source_commit", "HEAD"), ("task_ids", ("T14",)), ("approval_quote", ""), ("corpus_objects", {})])
def test_preparation_consent_rejects_incomplete_authority(core_case: tuple, field: str, value: object) -> None:
    _, consent = core_case
    with pytest.raises(RecordIntegrityError):
        replace(consent, **{field: value})


def test_config_cannot_supply_or_recover_consent(config: CampaignConfig) -> None:
    raw = record_dict(config)
    raw["grant"] = {}
    with pytest.raises(RecordIntegrityError):
        CampaignConfig.from_dict(raw)
    with pytest.raises(RecordIntegrityError):
        decode_record_json(b'{"guidance":"full_help","guidance":"discovery"}')


def test_live_config_without_grant_does_not_read_or_create(config: CampaignConfig) -> None:
    live = replace(config, command=(), claude_executable="/missing/claude",
        model="claude-haiku-4-5-20251001", thinking_policy="disabled")
    with pytest.raises(RecordIntegrityError, match="explicitly supplied grant"):
        harness.configured_campaign(live)
    assert not Path(config.output_root).exists()


def test_config_fixture_does_not_assume_producer_directory_names(tmp_path: Path, cli_pins: dict) -> None:
    replicas = {}
    for condition, pin in cli_pins.items():
        directory = tmp_path / "fresh-builds" / condition.value
        directory.mkdir(parents=True)
        executable = directory / "md"
        shutil.copy2(pin.executable, executable)
        replica = replace(pin, executable=str(executable))
        (directory / "pin.json").write_text(json.dumps(record_dict(replica)))
        replicas[condition] = replica
    configured = config.__wrapped__(tmp_path / "consumer", replicas)
    proposal = harness.configured_campaign(configured, prepare_only=True)
    assert proposal["authorization"] == "not granted"


def local_test_configured_native_canary(config: CampaignConfig, tmp_path: Path) -> None:
    from bench.test_claude_containment import scripted_provider
    executable = os.environ["MDTOOLS_U5_RUNNER"]
    with scripted_provider("printf 'ready\\n' > input.md") as (endpoint, observations):
        native = replace(config, task_ids=("cli-canary",), command=(),
            claude_executable=executable, endpoint=endpoint, model="claude-haiku-4-5-20251001",
            thinking_policy="disabled", timeout_seconds=20, guidance="discovery")
        path = tmp_path / "native-config.json"
        path.write_text(json.dumps(record_dict(native)))
        entry = [sys.executable, "-I", harness.__file__]
        for mode in ("--prepare-config", "--run-config"):
            run = subprocess.run([*entry, mode, str(path)], capture_output=True, timeout=60)
            assert run.returncode == 0, run.stdout.decode() + run.stderr.decode()
        report = decode_record_json(run.stdout)
        assert report["complete"] and report["grade_counts"] == {"pass": 3}
        assert observations
