"""Native synthetic controls. Local CLI tests require an explicit preserved pin.

The scripted endpoint follows the pre-build permission probe's public response
envelopes. Model text and usage are invented; no real provider is contacted.
"""
from __future__ import annotations

from contextlib import contextmanager
from dataclasses import replace
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import socket
import shlex
import subprocess
import sys
import threading
import time
from typing import Iterator

import pytest

from bench import harness, command_policy
from bench.claude_shell import ContainmentError, OwnedShells, process_state
from bench.command_policy import (CliCondition, ClaudeRunner, NativeBoundary,
    prepare_native_boundary, resolve_toolkit, CLAUDE_FLAGS, CLAUDE_SHA256, CLAUDE_VERSION)
from bench.test_command_policy import cli_pins
from bench.test_harness_run_artifacts import synthetic_task
from bench.trial_records import record_dict


NATIVE = pytest.mark.skipif(sys.platform != "darwin", reason="macOS native boundary; not portable containment evidence")


def boundary_at(root: Path, condition=None) -> NativeBoundary:
    root.mkdir(mode=0o700)
    for name in ("fixtures", "scratch"):
        (root / name).mkdir(mode=0o700)
    return prepare_native_boundary(root, condition, toolkit=resolve_toolkit())


def shell(boundary: NativeBoundary, script: str, *, stdin: bytes = b"") -> subprocess.CompletedProcess:
    boundary.verify()
    return subprocess.run([str(boundary.launcher), "-c", "-l", script],
        cwd=boundary.workspace / "fixtures", env={"PATH": "/usr/bin:/bin", "U5_SYNTHETIC_PRIVATE": "not-a-secret"},
        input=stdin, capture_output=True, timeout=10)


@NATIVE
def test_native_files_stdin_quotes_environment_and_toolkit(tmp_path: Path) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt")
    script = ('test -z "${U5_SYNTHETIC_PRIVATE+x}" && test -z "${HOME+x}" && '
              'test -z "${ANTHROPIC_API_KEY+x}" && value="$(printf quoted)" && '
              'printf "%s\\n" "$value" | grep quoted > output && cat && mktemp "$TMPDIR/example.XXXXXX"')
    result = shell(boundary, script, stdin=b"input with 'quotes'\n")
    assert result.returncode == 0, result.stderr
    assert result.stdout.startswith(b"input with 'quotes'\n")
    assert (boundary.workspace / "fixtures/output").read_bytes() == b"quoted\n"
    assert Path(result.stdout.splitlines()[-1].decode()).parent == boundary.workspace / "scratch"
    assert shell(boundary, "exit 23").returncode == 23
    stub = shell(boundary, "md read missing.md")
    assert stub.returncode == 1 and b"unavailable here" in stub.stderr
    assert shell(boundary, "md read missing.md").stdout == b""


@NATIVE
def test_native_outside_symlinks_alternate_executables_network_and_signal(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    boundary = boundary_at(root / "attempt")
    outside = root / "expected-synthetic"
    outside.write_bytes(b"SYNTHETIC-EXPECTED")
    worker = boundary.workspace / "fixtures"
    (worker / "read-escape").symlink_to(outside)
    (worker / "write-escape").symlink_to(outside)
    for script in [f"cat '{outside}'", "cat read-escape", "printf wrong > write-escape",
                   f"cat '/System/Volumes/Data{outside}'", "/usr/bin/python3 -c pass",
                   "/usr/bin/env /usr/bin/python3 -c pass", "cat ../control/shell.json",
                   '/usr/bin/sandbox-exec -p "(version 1)(allow default)" /bin/bash -c true']:
        assert shell(boundary, script).returncode != 0, script
    assert outside.read_bytes() == b"SYNTHETIC-EXPECTED"
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.settimeout(0.1)
        assert shell(boundary, f"printf request > /dev/tcp/127.0.0.1/{listener.getsockname()[1]}").returncode != 0
        with pytest.raises(socket.timeout):
            listener.accept()
    with subprocess.Popen(["/bin/sleep", "30"], start_new_session=True) as sentinel:
        try:
            assert shell(boundary, f"kill -0 {sentinel.pid}").returncode != 0
            assert sentinel.poll() is None
        finally:
            sentinel.terminate()
            sentinel.wait(timeout=5)


@NATIVE
@pytest.mark.parametrize("condition", [CliCondition.LEGACY, CliCondition.CURRENT_COMPACT])
def test_native_selected_md_and_other_condition_denied(tmp_path: Path, cli_pins: dict, condition: CliCondition) -> None:
    root = tmp_path.resolve()
    boundary = boundary_at(root / "attempt", cli_pins[condition])
    version = shell(boundary, "md --version")
    assert version.returncode == 0 and version.stdout.startswith(b"md 0.")
    other = next(pin for name, pin in cli_pins.items() if name != condition)
    assert shell(boundary, f"'{other.executable}' --version").returncode != 0
    assert shell(boundary, f"cp '{boundary.workspace}/bin/md' ./copied-md && ./copied-md --version").returncode != 0


@NATIVE
def test_native_current_read_stdin_and_guarded_patch(tmp_path: Path, cli_pins: dict) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt", cli_pins[CliCondition.CURRENT_COMPACT])
    document = boundary.workspace / "fixtures/input.md"
    document.write_bytes(b"# Heading\n\nBefore\n")
    query = json.dumps({"type": "kind", "kind": "block"})
    queried = shell(boundary, "md query input.md --from -", stdin=query.encode())
    assert queried.returncode == 0, queried.stderr
    address = next(row["target"]["address"] for row in json.loads(queried.stdout)
                   if row["target"]["summary"].get("kind") == "paragraph")
    read = shell(boundary, "md --json read input.md --address " + shlex.quote(json.dumps(address)))
    assert read.returncode == 0, read.stderr
    snapshot = json.loads(read.stdout)["snapshot"]
    patch = {"base_revision": snapshot["revision"], "operations": [{"op": "replace_block", "target": {
        "address": address["block"], "revision": snapshot["revision"], "guard": {
            "span": snapshot["guard"]["span"], "etag": snapshot["guard"]["etag"]}}, "markdown": "After\n"}]}
    changed = shell(boundary, "md patch input.md --from - --in-place", stdin=json.dumps(patch).encode())
    assert changed.returncode == 0, changed.stderr
    assert document.read_bytes() == b"# Heading\n\nAfter\n"
    rejected = shell(boundary, "md patch input.md --from - --in-place", stdin=json.dumps(patch).encode())
    assert rejected.returncode == 4
    assert document.read_bytes() == b"# Heading\n\nAfter\n"


@NATIVE
def test_native_cleanup_owns_background_job_group_not_sentinel(tmp_path: Path) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt")
    marker = boundary.workspace / "scratch/background.pid"
    command = f'set -m; /bin/bash -c "while :; do :; done" & printf "%s" "$!" > "{marker}"; wait'
    script = (f"import subprocess,time; subprocess.Popen({[str(boundary.launcher), '-c', '-l', command]!r}, "
              f"cwd={str(boundary.workspace / 'fixtures')!r}, env={{'PATH':'/usr/bin:/bin'}}); time.sleep(30)")
    sentinel = subprocess.Popen(["/bin/sleep", "30"], start_new_session=True)
    parent = subprocess.Popen([sys.executable, "-I", "-c", script], start_new_session=True,
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    owned = OwnedShells(boundary.registry, process_state(parent.pid).identity)
    try:
        deadline = time.monotonic() + 5
        while not marker.exists() or not marker.read_bytes():
            assert time.monotonic() < deadline, "background process did not start"
            time.sleep(0.01)
        child_pid = int(marker.read_bytes())
        child = process_state(child_pid)
        assert child.group_id == child_pid and child.session_id != parent.pid
        captured = harness.capture_process(parent, prompt=b"", timeout_seconds=0.1, stop_owned=owned.stop)
        assert captured.execution.kind == "timed_out"
        assert process_state(child_pid) is None or process_state(child_pid).status == 5
        assert sentinel.poll() is None
        owned.stop()  # Repeated cleanup is harmless; admission stays closed.
        duplicate = boundary.registry / "shell-duplicate.json"
        duplicate.write_text(json.dumps(owned.registrations()[0]))
        with pytest.raises(ContainmentError, match="name mismatch"):
            owned.registrations()
        duplicate.unlink()  # This test's duplicate copy only; original retained.
        refused = subprocess.run([str(boundary.launcher), "-c", "true"], capture_output=True)
        assert refused.returncode == 78
    finally:
        if parent.poll() is None:
            owned.stop()
            parent.wait(timeout=5)
        for pipe in (parent.stdin, parent.stdout, parent.stderr):
            pipe.close()
        sentinel.terminate()
        sentinel.wait(timeout=5)


@NATIVE
@pytest.mark.parametrize("asset", ["profile.sb", "bash-eval-launcher", "shell.json"])
def test_changed_or_missing_boundary_fails_before_shell(tmp_path: Path, asset: str) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt")
    target = boundary.workspace / "control" / asset
    target.chmod(0o600)
    target.write_bytes(b"changed synthetic asset")
    with pytest.raises(ValueError, match="asset changed"):
        boundary.verify()


@pytest.mark.parametrize("endpoint", ["https://api.anthropic.com", "http://localhost:1234", "http://127.0.0.1", "http://user@127.0.0.1:1234", "http://127.0.0.1:1234/path"])
def test_scripted_endpoint_cannot_be_remote(endpoint: str) -> None:
    with pytest.raises(ValueError):
        ClaudeRunner("/not/launched", endpoint, "claude-sonnet-5", "high", "adaptive")


def test_model_configuration_has_no_haiku_effort_or_auth_bypass() -> None:
    runner = ClaudeRunner("/not/launched", "http://127.0.0.1:1234", "claude-haiku-4-5-20251001", None, "disabled")
    assert "--effort" not in runner.command()
    assert "--bare" not in CLAUDE_FLAGS and "--dangerously-skip-permissions" not in CLAUDE_FLAGS
    with pytest.raises(ValueError):
        replace(runner, effort="high")


def test_runner_provenance_does_not_require_pruned_source(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    source = str(root / "updater-pruned/claude")
    executable = str(root / "preserved-claude")
    runner = ClaudeRunner(executable, "http://127.0.0.1:1234", "claude-sonnet-5", "high", "adaptive")
    receipt = {"source_path": source, "executable_path": executable, "sha256": CLAUDE_SHA256,
               "version_output": CLAUDE_VERSION + " (Claude Code)"}
    (root / "pin.json").write_text(json.dumps(receipt))
    assert runner.provenance_locators()["runner_source"] == source
    assert not Path(source).exists()
    receipt["source_path"] = executable
    (root / "pin.json").write_text(json.dumps(receipt))
    with pytest.raises(ValueError, match="distinct"):
        runner.provenance_locators()


@NATIVE
def test_live_parent_preserves_invented_normal_auth_but_child_cannot_read_it(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt")
    # No process is launched with this invented HOME. This tests unchanged
    # inheritance, not redirecting a real application's home or credentials.
    invented = {"HOME": "/synthetic-normal-home", "CLAUDE_CONFIG_DIR": "/synthetic-normal-config",
        "ANTHROPIC_API_KEY": "synthetic-test-key", "CLAUDE_CODE_OAUTH_TOKEN": "synthetic-test-token",
        "UNRELATED_PRIVATE": "synthetic-must-not-inherit", "CLAUDE_CODE_SHELL_PREFIX": "synthetic-prefix"}
    monkeypatch.setattr(command_policy.os, "environ", invented)
    runner = ClaudeRunner("/not/launched", None, "claude-sonnet-5", "high", "adaptive")
    parent = boundary.parent_environment(runner)
    assert runner.backend == "claude_cli"
    assert parent["HOME"] == invented["HOME"] and parent["CLAUDE_CONFIG_DIR"] == invented["CLAUDE_CONFIG_DIR"]
    assert parent["ANTHROPIC_API_KEY"] == "synthetic-test-key"
    assert "UNRELATED_PRIVATE" not in parent and "CLAUDE_CODE_SHELL_PREFIX" not in parent
    child = json.loads((boundary.workspace / "control/shell.json").read_bytes())["environment"]
    assert not set(child) & {"HOME", "ANTHROPIC_API_KEY", "CLAUDE_CODE_OAUTH_TOKEN", "CLAUDE_CONFIG_DIR"}
    local = boundary.parent_environment(replace(runner, endpoint="http://127.0.0.1:1234"))
    assert local["ANTHROPIC_API_KEY"] == "synthetic-local-only" and "HOME" not in local


@contextmanager
def scripted_provider(command: str | None) -> Iterator[tuple[str, list[dict[str, object]]]]:
    observations: list[dict[str, object]] = []

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, format: str, *args: object) -> None:
            return

        def do_POST(self) -> None:
            request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            if not self.path.startswith("/v1/messages"):
                self.send_error(404)
                return
            results = [block for message in request.get("messages", []) if isinstance(message.get("content"), list)
                       for block in message["content"] if block.get("type") == "tool_result"]
            observations.append({"model": request.get("model"), "thinking": request.get("thinking"),
                "output_config": request.get("output_config"), "tools": [tool["name"] for tool in request.get("tools", [])],
                "tool_errors": [block.get("is_error", False) for block in results]})
            finished = command is None or bool(results)
            block = ({"type": "text", "text": "synthetic complete"} if finished else
                     {"type": "tool_use", "id": "opaque:local_probe", "name": "Bash",
                      "input": {"command": command, "description": "Synthetic native boundary control"}})
            model = request["model"]
            reason = "end_turn" if finished else "tool_use"
            usage = {"input_tokens": 1, "output_tokens": 1}
            if request.get("stream"):
                start_block = {"type": "text", "text": ""} if finished else {"type": "tool_use", "id": block["id"], "name": "Bash", "input": {}}
                delta = {"type": "text_delta", "text": block["text"]} if finished else {"type": "input_json_delta", "partial_json": json.dumps(block["input"])}
                events = [
                    ("message_start", {"type": "message_start", "message": {"id": "msg_synthetic", "type": "message", "role": "assistant", "model": model,
                        "content": [], "stop_reason": None, "stop_sequence": None, "usage": usage}}),
                    ("content_block_start", {"type": "content_block_start", "index": 0, "content_block": start_block}),
                    ("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": delta}),
                    ("content_block_stop", {"type": "content_block_stop", "index": 0}),
                    ("message_delta", {"type": "message_delta", "delta": {"stop_reason": reason, "stop_sequence": None}, "usage": {"output_tokens": 1}}),
                    ("message_stop", {"type": "message_stop"})]
                body = "".join("event: " + name + "\ndata: " + json.dumps(event) + "\n\n" for name, event in events).encode()
                content_type = "text/event-stream"
            else:
                body = json.dumps({"id": "msg_synthetic", "type": "message", "role": "assistant", "model": model,
                    "content": [block], "stop_reason": reason, "stop_sequence": None, "usage": usage}).encode()
                content_type = "application/json"
            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    with ThreadingHTTPServer(("127.0.0.1", 0), Provider) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            yield f"http://127.0.0.1:{server.server_port}", observations
        finally:
            server.shutdown()
            thread.join(timeout=5)


@pytest.mark.parametrize("expected_bytes,grade", [(b"before\n", "pass"), (b"after\n", "fail")])
def local_test_actual_cli_no_tool_response_is_graded(tmp_path: Path, expected_bytes: bytes, grade: str) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    (expected / "answer.md").write_bytes(expected_bytes)
    with scripted_provider(None) as (endpoint, observations):
        runner = ClaudeRunner(executable, endpoint, "claude-sonnet-5", "high", "adaptive")
        result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[],
            claude=runner, results_dir=root / "result", timeout_seconds=20)
    assert result.execution.kind == "completed", record_dict(result)
    assert result.grade.kind == grade and result.evidence_complete
    assert result.usage.tool_output_bytes == 0 and result.usage.input_tokens is not None
    assert observations and all(row["tools"] == ["Bash"] and row["tool_errors"] == [] for row in observations)
    assert (root / "result/artifacts/final/input.md").read_bytes() == b"before\n"
    assert harness.AttemptStore(root / "result").load()[1] == result


@pytest.mark.parametrize("model,effort,thinking", [("claude-sonnet-5", "high", "adaptive"), ("claude-haiku-4-5-20251001", None, "disabled")])
def local_test_actual_cli_model_configuration_and_native_path(tmp_path: Path, model: str, effort: str | None, thinking: str) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    with scripted_provider("printf 'after\\n' > input.md") as (endpoint, observations):
        runner = ClaudeRunner(executable, endpoint, model, effort, thinking)
        result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[],
            claude=runner, results_dir=root / "result", timeout_seconds=20)
    assert result.execution.kind == "completed", record_dict(result)
    assert result.grade.kind == "pass" and not result.live_study_evidence
    assert observations and all(row["tools"] == ["Bash"] and row["model"] == model for row in observations)
    assert observations[-1]["tool_errors"] == [False]
    if thinking == "disabled":
        assert all(row["thinking"] in (None, {"type": "disabled"}) for row in observations)
    else:
        assert all(row["thinking"] == {"type": "adaptive"} and row["output_config"]["effort"] == "high" for row in observations)
    assert harness.AttemptStore(root / "result").load()[1] == result


def local_test_private_parent_config_outside_shell_profile(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    config = root / "synthetic-parent-config"
    config.mkdir(mode=0o700)
    original = NativeBoundary.parent_environment
    def parent_environment(boundary: NativeBoundary, runner: ClaudeRunner) -> dict[str, str]:
        env = original(boundary, runner)
        # Still synthetic auth + an empty private config; no user's config or
        # credentials are read. Only the snapshot-write location differs.
        env["CLAUDE_CONFIG_DIR"] = str(config)
        return env
    monkeypatch.setattr(NativeBoundary, "parent_environment", parent_environment)
    with scripted_provider("printf 'after\\n' | grep after > input.md") as (endpoint, observations):
        runner = ClaudeRunner(executable, endpoint, "claude-sonnet-5", "high", "adaptive")
        result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[],
            claude=runner, results_dir=root / "result", timeout_seconds=20)
    assert result.execution.kind == "completed" and result.grade.kind == "pass", record_dict(result)
    assert observations[-1]["tool_errors"] == [False]
    assert not list(config.glob("shell-snapshots/*.sh"))


def local_test_actual_cli_permission_denial_never_grades(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    original_command = ClaudeRunner.command
    def denied_command(runner: ClaudeRunner) -> list[str]:
        argv = original_command(runner)
        index = argv.index("--allowedTools")
        del argv[index:index + 2]
        return argv
    monkeypatch.setattr(ClaudeRunner, "command", denied_command)
    grader_calls = []
    monkeypatch.setattr(harness, "grade_submission", lambda *args, **kwargs: grader_calls.append(kwargs))
    with scripted_provider("printf 'after\\n' > input.md") as (endpoint, observations):
        runner = ClaudeRunner(executable, endpoint, "claude-sonnet-5", "high", "adaptive")
        result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[],
            claude=runner, results_dir=root / "result", timeout_seconds=20)
    assert result.execution.reason == "permission_denied" and result.permission_fault == "permission_denied"
    assert result.grade.kind == "not_run" and grader_calls == []
    assert (root / "result/artifacts/final/input.md").read_bytes() == b"before\n"
    # This CLI emits the structured denial before a terminal receipt. Immediate
    # cleanup retains unknown usage, not an invented zero or a fabricated bill.
    assert result.usage.input_tokens is None and result.usage.completeness == "partial"
    assert observations and all(row["tools"] == ["Bash"] for row in observations)
    with pytest.raises(ValueError, match="permission_denied"):
        harness.AttemptStore(root / "result").assert_admission()


def local_test_actual_cli_timeout_cleans_background_and_preserves_sentinel(tmp_path: Path) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    command = 'set -m; /bin/bash -c "while :; do :; done" & printf "%s" "$!" > background.pid; wait'
    sentinel = subprocess.Popen(["/bin/sleep", "30"], start_new_session=True)
    try:
        with scripted_provider(command) as (endpoint, observations):
            runner = ClaudeRunner(executable, endpoint, "claude-sonnet-5", "high", "adaptive")
            result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[],
                claude=runner, results_dir=root / "result", timeout_seconds=10)
        assert result.execution.kind == "timed_out" and result.grade.kind == "not_run"
        assert not result.evidence_complete and observations
        child_pid = int((root / "result/workspace/fixtures/background.pid").read_bytes())
        remaining = process_state(child_pid)
        assert remaining is None or remaining.status == 5
        assert sentinel.poll() is None
    finally:
        sentinel.terminate()
        sentinel.wait(timeout=5)


def local_test_actual_cli_turn_limit_is_operational_not_semantic(tmp_path: Path) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    with scripted_provider("printf 'after\\n' > input.md") as (endpoint, observations):
        runner = ClaudeRunner(executable, endpoint, "claude-sonnet-5", "high", "adaptive", max_turns=1)
        result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[],
            claude=runner, results_dir=root / "result", timeout_seconds=20)
    assert result.execution.kind == "budget_exhausted" and result.execution.reason == "turn_limit", record_dict(result)
    assert result.grade.kind == "not_run" and observations


@pytest.mark.parametrize("condition", [CliCondition.LEGACY, CliCondition.CURRENT_COMPACT])
def local_test_actual_cli_selected_md_and_forbidden_access(tmp_path: Path, cli_pins: dict, condition: CliCondition) -> None:
    executable = os.environ.get("MDTOOLS_U5_RUNNER")
    assert executable, "explicit preserved runner required; no PATH fallback"
    root = tmp_path.resolve()
    task, inputs, expected = synthetic_task(root)
    forbidden = expected / "answer.md"
    other = next(pin for name, pin in cli_pins.items() if name != condition)
    help_command = command_policy.verify_condition(cli_pins[condition])[0].name
    command = (f"if cat '{forbidden}'; then exit 90; fi; "
               f"if '{other.executable}' --version; then exit 91; fi; "
               f"md --help > /dev/null && md {help_command} --help > /dev/null && "
               "md --version | grep '^md ' && printf 'after\\n' > input.md")
    with scripted_provider(command) as (endpoint, observations):
        runner = ClaudeRunner(executable, endpoint, "claude-sonnet-5", "high", "adaptive")
        result = harness.run_agent(task, fixture_root=inputs, expected_root=expected, command=[], condition=cli_pins[condition],
            claude=runner, results_dir=root / "result", timeout_seconds=20,
            guidance=command_policy.ToolGuidance.DISCOVERY)
    assert result.execution.kind == "completed" and result.grade.kind == "pass", record_dict(result)
    assert observations[-1]["tool_errors"] == [False]
    assert forbidden.read_bytes() == b"after\n"


@NATIVE
def test_launcher_missing_profile_rejects_and_unknown_argv_never_runs(tmp_path: Path) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt")
    unknown = subprocess.run([str(boundary.launcher), "--rcfile", "/not/read"], capture_output=True)
    assert unknown.returncode == 78
    profile = boundary.workspace / "control/profile.sb"
    profile.rename(profile.with_suffix(".retained"))
    refused = subprocess.run([str(boundary.launcher), "-c", "printf bad > created"],
        cwd=boundary.workspace / "fixtures", capture_output=True)
    assert refused.returncode == 78 and not (boundary.workspace / "fixtures/created").exists()
    with pytest.raises((OSError, ValueError)):
        boundary.verify()


@NATIVE
def test_foreign_registration_cannot_be_owned(tmp_path: Path) -> None:
    boundary = boundary_at(tmp_path.resolve() / "attempt")
    assert shell(boundary, "true").returncode == 0
    with subprocess.Popen(["/bin/sleep", "30"], start_new_session=True) as parent:
        try:
            owned = OwnedShells(boundary.registry, process_state(parent.pid).identity)
            with pytest.raises(ContainmentError, match="foreign"):
                owned.registrations()
            assert parent.poll() is None
        finally:
            parent.terminate()
            parent.wait(timeout=5)
