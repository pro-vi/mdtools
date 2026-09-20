"""Maintained operator entry points, using public fixtures and synthetic argv only."""
from __future__ import annotations

from dataclasses import replace
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys

import pytest

from bench import harness
from bench.command_policy import CliCondition
from bench.manifest import CampaignConfig
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
