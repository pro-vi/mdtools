from __future__ import annotations

import json
import os
import shlex
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class PiJsonTrace:
    stdout: str
    model: str | None = None
    thinking_level: str | None = None
    tool_calls: int = 0
    turns: int = 0
    bytes_observation: int = 0
    tool_outputs: list[str] = field(default_factory=list)
    text_outputs: list[str] = field(default_factory=list)
    runner_error: str | None = None


def default_audit_extension_path() -> Path:
    """Return the globally promoted Pi audit extension entrypoint."""
    override = os.environ.get("BENCH_PI_AUDIT_EXTENSION") or os.environ.get("PI_AUDIT_EXTENSION")
    if override:
        return Path(override).expanduser()

    agent_dir = Path(os.environ.get("PI_CODING_AGENT_DIR", "~/.pi/agent")).expanduser()
    return agent_dir / "extensions" / "audit" / "index.ts"


def build_pi_json_command(
    *,
    agent_cmd: str,
    model: str | None,
    audit_extension_path: Path,
    session_dir: Path,
    tools: tuple[str, ...] = ("bash",),
    thinking_level: str | None = None,
) -> list[str]:
    """Build a Pi JSON-mode command for benchmark execution.

    `agent_cmd` may be `pi` plus extra flags. If it names a different agent
    (e.g. the harness default `claude -p`), use plain `pi` so `--runner pi-json`
    works without also overriding `--agent`.
    """
    parts = shlex.split(agent_cmd) if agent_cmd.strip() else []
    cmd = parts if parts and Path(parts[0]).name == "pi" else ["pi"]

    cmd += [
        "--mode",
        "json",
        "--no-context-files",
        "--no-skills",
        "--no-prompt-templates",
        "--no-extensions",
        "-e",
        str(audit_extension_path),
        "--tools",
        ",".join(tools),
        "--session-dir",
        str(session_dir),
    ]
    if model:
        cmd += ["--model", model]
    if thinking_level:
        cmd += ["--thinking", thinking_level]
    return cmd


def parse_pi_json_output(raw_stdout: str) -> PiJsonTrace:
    """Parse `pi --mode json` JSONL into the harness' ParsedAgentOutput shape."""
    trace = PiJsonTrace(stdout=raw_stdout)
    assistant_text_by_message: list[str] = []
    errors: list[str] = []
    saw_tool_execution_end = False
    deferred_tool_results: list[str] = []

    for line in raw_stdout.splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(event, dict):
            continue

        event_type = event.get("type")
        if event_type == "turn_start":
            trace.turns += 1
        elif event_type == "tool_execution_start":
            trace.tool_calls += 1
        elif event_type == "tool_execution_end":
            saw_tool_execution_end = True
            output = _tool_result_text(event.get("result"))
            if output:
                trace.tool_outputs.append(output)
                trace.bytes_observation += len(output.encode())
            if event.get("isError") is True:
                errors.append(_compact_error("tool_error", event.get("toolName"), output))
        elif event_type == "message_end":
            message = event.get("message")
            if not isinstance(message, dict):
                continue
            role = message.get("role")
            if role == "assistant":
                if trace.model is None:
                    trace.model = _message_model(message)
                if trace.thinking_level is None and isinstance(message.get("thinkingLevel"), str):
                    trace.thinking_level = message["thinkingLevel"]
                text = _message_text(message)
                if text:
                    assistant_text_by_message.append(text)
                if message.get("stopReason") == "error":
                    errors.append(_compact_error("assistant_error", message.get("model"), message.get("errorMessage") or text))
            elif role == "toolResult":
                # Buffer toolResult message_ends. Pi versions that emit
                # tool_execution_end use that as the authoritative source;
                # versions that don't fall through to these buffered values.
                # Decided after the loop so multi-tool-call traces on the
                # message_end path are captured fully, not just the first.
                text = _message_text(message)
                if text:
                    deferred_tool_results.append(text)
        elif event_type == "extension_error":
            errors.append(_compact_error("extension_error", event.get("extensionPath"), event.get("error")))

    if not saw_tool_execution_end and deferred_tool_results:
        for text in deferred_tool_results:
            trace.tool_outputs.append(text)
            trace.bytes_observation += len(text.encode())

    trace.text_outputs = assistant_text_by_message
    combined = trace.tool_outputs + trace.text_outputs
    trace.stdout = "\n".join(combined) if combined else raw_stdout
    trace.runner_error = errors[0] if errors else None
    return trace


def _tool_result_text(result: Any) -> str:
    if not isinstance(result, dict):
        return ""
    return _content_text(result.get("content"))


def _message_text(message: dict[str, Any]) -> str:
    return _content_text(message.get("content"))


def _message_model(message: dict[str, Any]) -> str | None:
    model = message.get("model")
    if not isinstance(model, str) or not model.strip():
        return None
    model = model.strip()

    provider = message.get("provider")
    if isinstance(provider, str) and provider.strip():
        provider = provider.strip()
        if not model.startswith(f"{provider}/"):
            return f"{provider}/{model}"
    return model


def _content_text(content: Any) -> str:
    if isinstance(content, str):
        return content
    if not isinstance(content, list):
        return ""
    parts: list[str] = []
    for item in content:
        if isinstance(item, dict) and item.get("type") == "text" and isinstance(item.get("text"), str):
            parts.append(item["text"])
    return "\n".join(parts)


def _compact_error(kind: str, subject: Any, message: Any) -> str:
    subject_text = subject if isinstance(subject, str) and subject else "unknown"
    message_text = message if isinstance(message, str) and message else "unknown error"
    collapsed = " ".join(message_text.split())
    if len(collapsed) > 300:
        collapsed = collapsed[:300] + "…"
    return f"{kind}: {subject_text}: {collapsed}"
