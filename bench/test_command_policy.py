"""P policy recovery narrowed to pinned producers and offline CLI recipes."""

from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
import subprocess

import pytest

from bench.command_policy import (CliCondition, ORDINARY_TOOLS,
    UNAVAILABLE_MD, classify_md_argv, decode_schema,
    resolve_toolkit, stage_condition, tool_reference, verify_condition)
from bench.harness import ReadRequest, exercise_cli_examples, replay_read_pair


def test_toolkit_is_complete_and_resolved(tmp_path: Path) -> None:
    toolkit = resolve_toolkit()
    assert set(toolkit) == set((*ORDINARY_TOOLS, "bash", "sh"))
    assert all(str(Path(path).resolve()) == path for path in toolkit.values())
    pin = stage_condition(None, tmp_path.resolve() / "bin", toolkit=toolkit)
    assert Path(pin.executable).read_bytes() == UNAVAILABLE_MD
    assert verify_condition(pin) == ()
    result = subprocess.run([pin.executable, "read", "missing.md"], capture_output=True)
    assert result.returncode == 1 and b"unavailable" in result.stderr
    assert b"permission_denied" not in result.stderr
    scratch = tmp_path.resolve() / "scratch"
    scratch.mkdir()
    result = subprocess.run([toolkit["mktemp"], str(scratch / "example.XXXXXX")], env={"TMPDIR": str(scratch)}, capture_output=True, check=True)
    assert Path(result.stdout.decode().strip()).parent == scratch
    with pytest.raises(ValueError, match="toolkit names"):
        stage_condition(None, tmp_path / "other", toolkit={"cat": toolkit["cat"]})


@pytest.mark.parametrize("condition", [CliCondition.LEGACY, CliCondition.CURRENT_COMPACT])
def test_producer_identity_and_relocation(cli_pins: dict, condition: CliCondition, tmp_path: Path) -> None:
    pin = cli_pins[condition]
    commands = verify_condition(pin)
    assert commands and len({item.name for item in commands}) == len(commands)
    relocated = stage_condition(pin, tmp_path.resolve() / "bin", toolkit=resolve_toolkit())
    assert relocated.content_identity == pin.content_identity
    assert verify_condition(relocated) == commands
    reference = tool_reference(pin)
    assert all(f"md {item.name}" in reference for item in commands)
    assert all(name in reference for name in ORDINARY_TOOLS)
    assert classify_md_argv([pin.executable, "--json", commands[0].name], commands) == commands[0].kind
    assert classify_md_argv([pin.executable, "unknown"], commands) is None
    with pytest.raises(ValueError, match="binary digest"):
        verify_condition(replace(pin, binary_sha256="0" * 64))
    with pytest.raises(ValueError, match="schema digest"):
        verify_condition(replace(pin, schema_sha256="0" * 64))
    with pytest.raises(ValueError, match="source/build digest"):
        verify_condition(replace(pin, source_sha256="0" * 64))
    raw = json.loads(Path(pin.schema_path).read_bytes())
    raw["commands"] = []
    with pytest.raises(ValueError, match="no commands"):
        decode_schema(json.dumps(raw).encode(), condition)
    raw = json.loads(Path(pin.schema_path).read_bytes())
    raw["commands"].append(raw["commands"][0])
    with pytest.raises(ValueError, match="duplicate"):
        decode_schema(json.dumps(raw).encode(), condition)


def test_all_documented_examples(cli_pins: dict, tmp_path: Path) -> None:
    for condition, pin in cli_pins.items():
        record = exercise_cli_examples(pin, output=tmp_path.resolve() / condition.value)
        assert record["committed_mutations"] == int(condition == CliCondition.CURRENT_COMPACT)
    stub = stage_condition(None, tmp_path.resolve() / "stub-bin", toolkit=resolve_toolkit())
    assert exercise_cli_examples(stub, output=tmp_path.resolve() / "no-md")["committed_mutations"] == 0


@pytest.mark.parametrize("argv,document,stdin", [
    (("map", 'document with "quotes".md'), b"# Heading\n\nBefore\n", b""),
    (("query", 'document with "quotes".md', "--query", '{"type":"section","text":"A \\\"quote\\\"","match_mode":"exact"}'), b'# A "quote"\n\nBefore\n', b""),
    (("read", 'document with "quotes".md', "--query", '{"type":"section","text":"Heading","match_mode":"exact"}'), b"# Heading\n\nBefore\n", b""),
    (("read", 'document with "quotes".md', "--from", "-"), b"", b'{"kind":"preamble"}'),
    (("read", 'document with "quotes".md', "--query", '{"type":"section","text":"Absent","match_mode":"exact"}'), b"", b""),
])
def test_direct_replay_identity_and_measurement(cli_pins: dict, tmp_path: Path,
        argv: tuple[str, ...], document: bytes, stdin: bytes) -> None:
    pin = cli_pins[CliCondition.CURRENT_COMPACT]
    request = ReadRequest(document, argv, stdin)
    output = tmp_path.resolve() / "replay"
    record = replay_read_pair(pin, request, request, output=output)
    compact, full = record["invocations"]
    assert full["argv"] == [compact["argv"][0], "--json", *compact["argv"][1:]]
    assert compact["stdin_sha256"] == full["stdin_sha256"]
    assert record["binary_sha256"] == pin.binary_sha256
    assert not record["agent_trial"] and not record["provider_usage"]
    for invocation in record["invocations"]:
        assert invocation["tokenizer"]["encoding"] == "o200k_base"
        assert all(type(stream["tokens"]) is int for stream in invocation["streams"].values())
    if argv[0] == "read" and record["category"] == "successful_read":
        typed = json.loads((output / "full/stdout.bin").read_bytes())
        assert typed["markdown"].encode() == (output / "compact/stdout.bin").read_bytes()
    with pytest.raises(ValueError, match="nonidentical"):
        replay_read_pair(pin, request, replace(request, document=document + b"x"), output=tmp_path / "refused")
    assert not (tmp_path / "refused").exists()


def test_replay_rejects_mutation_and_forced_json(cli_pins: dict, tmp_path: Path) -> None:
    pin = cli_pins[CliCondition.CURRENT_COMPACT]
    for argv in (("patch", 'document with "quotes".md'), ("read", 'document with "quotes".md', "--json")):
        request = ReadRequest(b"", argv)
        with pytest.raises(ValueError):
            replay_read_pair(pin, request, request, output=tmp_path / "rejected")
    assert not (tmp_path / "rejected").exists()
    request = ReadRequest(b"", ("read", 'document with "quotes".md', "--from", "/synthetic/outside.json"))
    with pytest.raises(ValueError, match="external input files"):
        replay_read_pair(pin, request, request, output=tmp_path / "outside")
    assert not (tmp_path / "outside").exists()


def test_same_bytes_via_symlink_retains_identity(cli_pins: dict, tmp_path: Path) -> None:
    pin = cli_pins[CliCondition.CURRENT_COMPACT]
    linked = tmp_path.resolve() / "selected-md"
    linked.symlink_to(pin.executable)
    relocated = replace(pin, executable=str(linked))
    assert relocated.content_identity == pin.content_identity
    assert verify_condition(relocated) == verify_condition(pin)


@pytest.mark.parametrize("task_id", ["T1", "T2", "T10"])
def test_public_frozen_document_replay(cli_pins: dict, tmp_path: Path, task_id: str) -> None:
    from bench.test_harness_json import public_task
    task = public_task(task_id)  # Explicit non-holdout ID projection only.
    repo = Path(__file__).resolve().parent.parent
    document = (repo / task.input_files[0]).read_bytes()
    request = ReadRequest(document, ("read", 'document with "quotes".md', "--address", '{"kind":"document"}'))
    record = replay_read_pair(cli_pins[CliCondition.CURRENT_COMPACT], request, request, output=tmp_path.resolve() / task_id)
    assert record["category"] == "successful_read"
    assert (tmp_path / task_id / "compact/stdout.bin").read_bytes() == document


def test_unavailable_measurement_is_not_zero(cli_pins: dict, tmp_path: Path) -> None:
    from bench.harness import _capture_cli
    pin = cli_pins[CliCondition.CURRENT_COMPACT]
    record = _capture_cli(pin, ["schema"], cwd=tmp_path.resolve(), stdin=b"", output=tmp_path.resolve() / "capture")
    assert record["tokenizer"] is None
    assert all(stream["tokens"] is None and stream["unavailable"] == "measurement_not_supplied" for stream in record["streams"].values())


def test_selected_cli_reaches_actual_synthetic_worker(cli_pins: dict, tmp_path: Path) -> None:
    from bench.harness import run_agent
    from bench.test_harness_run_artifacts import python_command, synthetic_task
    root = tmp_path.resolve()
    task, fixtures, expected = synthetic_task(root)
    stub = stage_condition(None, root / "stub", toolkit=resolve_toolkit())
    for index, pin in enumerate((stub, *cli_pins.values())):
        script = ("from pathlib import Path; import subprocess; "
            "p=subprocess.run(['md','--json','schema'],capture_output=True); "
            f"assert p.returncode == {int(pin.condition == CliCondition.NO_MD)}; "
            "Path('input.md').write_bytes(b'after\\n')")
        result = run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=python_command(script), condition=pin, results_dir=root / f"receipt-{index}")
        assert result.grade.kind == "pass"
        stored = json.loads((root / f"receipt-{index}/experiment.json").read_bytes())
        assert stored["condition_sha256"] == pin.content_identity
        assert set(stored["toolkit_sha256"]) == set((*ORDINARY_TOOLS, "bash", "sh"))


def test_native_frontmatter_format_spelling_is_semantic(cli_pins: dict, tmp_path: Path) -> None:
    from bench.harness import _capture_cli
    from bench.test_harness_json import public_task
    from bench.neutral_scorer import grade_json
    task = public_task("T21")
    repo = Path(__file__).resolve().parent.parent
    expected = (repo / task.expected_output).read_bytes()
    observed = []
    for condition, pin in cli_pins.items():
        output = tmp_path.resolve() / condition.value
        argv = (["--json", "frontmatter", task.input_files[0]] if condition == CliCondition.LEGACY else
                ["--json", "read", task.input_files[0], "--address", '{"kind":"frontmatter"}'])
        assert _capture_cli(pin, argv, cwd=repo, stdin=b"", output=output)["exit_code"] == 0
        native = json.loads((output / "stdout.bin").read_bytes())
        payload = native["frontmatter"] if condition == CliCondition.LEGACY else native
        observed.append(payload["format"])
        projected = {"present": native["present"], "format": payload["format"], "value": payload["data"]}
        raw = json.dumps(projected).encode()
        assert grade_json(task.scorer, raw, expected).kind == "pass"
        projected["format"] = "toml"
        assert grade_json(task.scorer, json.dumps(projected).encode(), expected).kind == "fail"
        assert json.loads((output / "stdout.bin").read_bytes()) == native
    assert observed == ["Yaml", "yaml"]
