from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable

try:
    from bench.command_policy import GuardEvent, classify_command_kind
except ModuleNotFoundError:  # pragma: no cover - direct script use
    from command_policy import GuardEvent, classify_command_kind


@dataclass
class PiAuditCounters:
    tool_calls: int = 0
    tool_results: int = 0
    tool_errors: int = 0
    bytes_observation: int = 0
    blocked: int = 0
    policy_violations: int = 0
    mutations: int = 0
    requeried: bool = False
    model: str | None = None
    thinking_level: str | None = None
    bash_commands: list[str] = field(default_factory=list)


def load_pi_audit_events(path: Path) -> list[dict[str, Any]]:
    """Load pi-audit.v1 JSONL events, ignoring malformed lines."""
    if not path.exists():
        return []

    events: list[dict[str, Any]] = []
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(event, dict):
            events.append(event)
    return events


def summarize_pi_audit_events(
    events: Iterable[dict[str, Any]],
    *,
    guard_events: Iterable[GuardEvent] = (),
) -> PiAuditCounters:
    """Map reusable Pi audit events plus optional mdtools guard events to benchmark counters.

    The Pi audit schema is intentionally generic. This adapter is where mdtools-specific
    notions like query/mutation/re-query are derived.
    """
    counters = PiAuditCounters()
    call_sequence: list[str] = []
    seen_tool_calls: set[str] = set()

    for event in events:
        event_name = event.get("event")
        tool_call_id = event.get("toolCallId")
        tool_name = event.get("toolName")

        if event.get("decision") == "block":
            counters.blocked += 1

        if event_name == "tool_call":
            # A toolCallId is stable across transcript/live projections. If absent,
            # fall back to object identity by counting it as unique.
            dedupe_key = str(tool_call_id) if tool_call_id else f"<missing>:{id(event)}"
            if dedupe_key not in seen_tool_calls:
                seen_tool_calls.add(dedupe_key)
                counters.tool_calls += 1

            command = _command_from_event(event)
            if tool_name == "bash" and command:
                counters.bash_commands.append(command)
                kind = classify_command_kind(command)
                if kind:
                    call_sequence.append(kind)

        elif event_name == "tool_result":
            counters.tool_results += 1
            counters.bytes_observation += _int_value(event.get("outputBytes"))
        elif event_name == "tool_error":
            counters.tool_errors += 1
            counters.bytes_observation += _int_value(event.get("outputBytes"))
        elif event_name == "user_bash":
            command = _command_from_event(event)
            if command:
                counters.bash_commands.append(command)
        elif event_name == "model_change":
            model = _model_from_event(event)
            if model:
                counters.model = model
        elif event_name == "thinking_level_change":
            thinking_level = _thinking_level_from_event(event)
            if thinking_level:
                counters.thinking_level = thinking_level

    guard_sequence: list[str] = []
    for guard_event in guard_events:
        if guard_event.decision != "allow":
            counters.policy_violations += 1
            continue
        kind = guard_event.kind
        if kind:
            guard_sequence.append(kind)

    # Prefer the shell guard when available: it sees inner commands inside a bash string.
    effective_sequence = guard_sequence or call_sequence
    counters.mutations = sum(1 for kind in effective_sequence if kind == "mutation")
    counters.requeried = _has_query_after_mutation(effective_sequence)
    return counters


def _command_from_event(event: dict[str, Any]) -> str | None:
    input_value = event.get("input")
    if not isinstance(input_value, dict):
        return None
    command = input_value.get("command") or input_value.get("cmd")
    return command if isinstance(command, str) and command.strip() else None


def _int_value(value: Any) -> int:
    return value if isinstance(value, int) and value > 0 else 0


def _model_from_event(event: dict[str, Any]) -> str | None:
    details = event.get("details")
    if not isinstance(details, dict):
        return None

    provider = details.get("provider")
    model_id = details.get("modelId") or details.get("id") or details.get("model")
    if not isinstance(model_id, str) or not model_id.strip():
        return None
    model_id = model_id.strip()

    if isinstance(provider, str) and provider.strip():
        provider = provider.strip()
        if not model_id.startswith(f"{provider}/"):
            return f"{provider}/{model_id}"
    return model_id


def _thinking_level_from_event(event: dict[str, Any]) -> str | None:
    details = event.get("details")
    if not isinstance(details, dict):
        return None
    level = details.get("thinkingLevel") or details.get("level")
    return level.strip() if isinstance(level, str) and level.strip() else None


def _has_query_after_mutation(sequence: Iterable[str]) -> bool:
    saw_mutation = False
    for kind in sequence:
        if kind == "mutation":
            saw_mutation = True
        elif kind == "query" and saw_mutation:
            return True
    return False
