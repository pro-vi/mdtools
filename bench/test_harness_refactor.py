"""Behavioral comparisons against an immutable source baseline, without a provider."""
from __future__ import annotations

import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile

import pytest

if __name__ != "__main__":
    from bench.test_command_policy import cli_pins


BASELINE = "572abb0a42c81f3382036a3bd58220def40ffb51"
REPO = Path(__file__).resolve().parent.parent


@pytest.fixture(scope="session")
def baseline_source(tmp_path_factory: pytest.TempPathFactory) -> Path:
    root = tmp_path_factory.mktemp("harness-baseline").resolve()
    paths = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", BASELINE, "bench"], cwd=REPO).decode().splitlines()
    selected = [path for path in paths if Path(path).parent == Path("bench") and
                (path.endswith(".py") or path == "bench/requirements.lock")]
    payload = subprocess.check_output(["git", "archive", BASELINE, "--", *selected], cwd=REPO)
    with tarfile.open(fileobj=io.BytesIO(payload)) as archive:
        for member in archive.getmembers():
            assert member.isdir() or member.isfile()
            assert not Path(member.name).is_absolute() and ".." not in Path(member.name).parts
        archive.extractall(root)
    return root


def source_probe(source: Path, output: Path, mode: str, payload: object) -> object:
    command = [sys.executable, "-I", str(Path(__file__).resolve()), str(source), str(output), mode, json.dumps(payload)]
    completed = subprocess.run(command, env={"PATH": "/usr/bin:/bin"}, capture_output=True, timeout=30, check=True)
    return json.loads(completed.stdout)


@pytest.mark.parametrize("case", ["pass", "wrong", "no-tool", "limit", "denied", "truncated", "duplicate"])
def test_attempt_behavior_matches_baseline(baseline_source: Path, tmp_path: Path, case: str) -> None:
    before = source_probe(baseline_source, tmp_path / "before", "attempt", case)
    after = source_probe(REPO, tmp_path / "after", "attempt", case)
    assert after == before
    assert after["grade"]["kind"] == ("pass" if case in ("pass", "duplicate") else "fail" if case in ("wrong", "no-tool") else "not_run")


def test_full_help_prompt_matches_baseline(baseline_source: Path, tmp_path: Path, cli_pins: dict) -> None:
    from bench.command_policy import resolve_toolkit, stage_condition
    from bench.trial_records import record_dict
    pins = [*cli_pins.values(), stage_condition(None, tmp_path / "stub", toolkit=resolve_toolkit())]
    for pin in pins:
        before = source_probe(baseline_source, tmp_path, "prompt", record_dict(pin))
        assert source_probe(REPO, tmp_path, "prompt", record_dict(pin)) == before


def test_archived_record_identity_round_trip(baseline_source: Path, tmp_path: Path) -> None:
    source_probe(baseline_source, tmp_path / "old", "attempt", "pass")
    from bench.harness import AttemptStore
    from bench.manifest import ExperimentSpec
    from bench.trial_records import decode_record_json, record_dict
    path = tmp_path / "old/receipt/experiment.json"
    raw = decode_record_json(path.read_bytes())
    spec = ExperimentSpec.from_dict(raw)
    assert record_dict(spec) == raw
    start, result = AttemptStore(tmp_path / "old/receipt").load()
    assert start.key.experiment_id == spec.identity and result.grade.kind == "pass"


def local_test_baseline_native_suite(baseline_source: Path) -> None:
    """Run preserved-CLI lifecycle controls against the archived implementation."""
    runner = os.environ.get("MDTOOLS_U5_RUNNER")
    pins = os.environ.get("MDTOOLS_U3_PIN_ROOT")
    assert runner and pins, "explicit local pins required"
    subprocess.run([sys.executable, "-m", "pytest", "-q", "--override-ini", "python_functions=local_test_*",
                    "bench/test_claude_containment.py"], cwd=baseline_source, timeout=120, check=True,
                   env={"PATH": "/usr/bin:/bin", "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
                        "MDTOOLS_U5_RUNNER": runner, "MDTOOLS_U3_PIN_ROOT": pins})


def probe(source: Path, output: Path, mode: str, payload: object) -> object:
    # This subprocess imports exactly one source tree; no module-cache sharing.
    sys.path.insert(0, str(source))
    from bench import harness
    from bench.command_policy import CliCondition, ConditionPin
    from bench.test_trial_records import synthetic_cli_events
    from bench.trial_records import record_dict
    policy = harness.StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False)
    task = harness.BenchTask("synthetic", "Write after.", ["input.md"], "answer.md", "file_contents", "synthetic", policy)
    if mode == "prompt":
        payload["condition"] = CliCondition(payload["condition"])
        return harness.build_prompt(task, condition=ConditionPin(**payload))
    fixtures, expected = output / "fixtures", output / "expected"
    fixtures.mkdir(parents=True)
    expected.mkdir()
    (fixtures / "input.md").write_bytes(b"before\n")
    (expected / "answer.md").write_bytes(b"after\n")
    events = synthetic_cli_events(denied=payload == "denied")
    if payload == "no-tool":
        events = [events[0], events[-1]]
    elif payload == "limit":
        events[-1].update(subtype="error_max_turns", terminal_reason="max_turns", is_error=True)
    elif payload == "truncated":
        events = events[:-1]
    elif payload == "duplicate":
        events.append(events[-1])
    script = ("from pathlib import Path; " + ("Path('input.md').write_bytes(b'after\\n'); " if payload in ("pass", "duplicate") else "") +
              "print(" + repr("\n".join(json.dumps(event) for event in events)) + ")")
    result = harness.run_agent(task, fixture_root=fixtures, expected_root=expected,
        command=[str(Path(sys.executable).resolve()), "-I", "-c", script], results_dir=output / "receipt", event_format="claude_stream")
    usage = record_dict(result.usage)
    usage.pop("elapsed_seconds")
    return {"execution": record_dict(result.execution), "grade": record_dict(result.grade),
            "usage": usage, "permission_fault": result.permission_fault, "evidence_complete": result.evidence_complete,
            "exit_code": result.exit_code, "artifacts": {name: (output / "receipt" / name).read_bytes().hex()
                for name in result.artifacts if name.startswith("artifacts/")}}


if __name__ == "__main__":
    print(json.dumps(probe(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], json.loads(sys.argv[4]))))
