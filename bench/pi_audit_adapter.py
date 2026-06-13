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
    # Per-tool counts for pi's NATIVE file tools (edit/write/read). The guarded bash shell
    # never sees these (they bypass it), so they're absent from the harness's guard-derived
    # tool_mix — the harness folds this in to make the md-vs-native-editor adoption split
    # computable on the native arm. Keyed by pi tool name (lowercase): edit/write/read.
    native_tool_mix: dict[str, int] = field(default_factory=dict)


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
    audit_calls: list[tuple[str, str | None]] = []   # ordered (toolName, command) per tool_call
    seen_tool_calls: set[str] = set()
    saw_native_tool = False

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
            audit_calls.append((tool_name, command))
            if tool_name == "bash" and command:
                counters.bash_commands.append(command)
                kind = classify_command_kind(command)
                if kind:
                    call_sequence.append(kind)
            elif tool_name in ("edit", "write"):
                # pi's native editor mutations bypass the guarded shell, so the guard never
                # sees them; record them here, else the native arm reports mutations=0 even
                # when the agent edited the file (the inverse of the claude-cli native arm,
                # which maps Edit/Write tool_use -> transcript_mutations).
                call_sequence.append("mutation")
                counters.native_tool_mix[tool_name] = counters.native_tool_mix.get(tool_name, 0) + 1
                saw_native_tool = True
            elif tool_name == "read":
                # native read == a structural query (mirrors the claude-cli arm, where Read
                # feeds the query-after-mutation re-query scan).
                call_sequence.append("query")
                counters.native_tool_mix[tool_name] = counters.native_tool_mix.get(tool_name, 0) + 1
                saw_native_tool = True

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

    guard_events = list(guard_events)
    guard_sequence: list[str] = []
    for guard_event in guard_events:
        if guard_event.decision != "allow":
            counters.policy_violations += 1
            continue
        kind = guard_event.kind
        if kind:
            guard_sequence.append(kind)

    # The guard is the only source that expands a compound bash string into its inner
    # commands (`grep ... && ./md set-task ...` -> query, mutation); classify_command_kind
    # on the whole string is coarse and can hide a mutation after a query. The audit stream
    # is the only source of pi's native edit/write/read (they bypass the guarded shell, so
    # the guard never sees them). Neither alone is complete when the two are mixed:
    #   - bash-only run: keep the guard view (finer), or coarse call_sequence if no guard log.
    #   - native tools present: MERGE both views in event order, so a bash mutation between a
    #     bash query and a native re-read still registers as a mutation and a requery.
    if saw_native_tool:
        effective_sequence = _merge_native_and_guard_sequence(audit_calls, guard_events)
    else:
        effective_sequence = guard_sequence or call_sequence
    counters.mutations = sum(1 for kind in effective_sequence if kind == "mutation")
    counters.requeried = _has_query_after_mutation(effective_sequence)
    return counters


def _merge_native_and_guard_sequence(
    audit_calls: list[tuple[str, str | None]],
    guard_events: Iterable[GuardEvent],
) -> list[str]:
    """Merge pi's native tool calls with the guard's granular bash view into one ordered
    query/mutation sequence. The guard expands a compound bash command into its inner
    commands (the coarse classify_command_kind on the whole string can't); the audit stream
    is the only source of native edit/write/read. Walk the audit tool-calls in order: a bash
    call splices in its guard kinds (matched by the guard's raw_command appearing in the
    call's command string, consumed in order); edit/write -> mutation, read -> query. A bash
    call with no matching guard rows (guard absent / shape differs) falls back to coarse
    classification; any unaligned guard kinds are appended so no bash mutation/query is lost."""
    allow_guard = [g for g in guard_events if g.decision == "allow"]
    gi = 0
    n = len(allow_guard)
    merged: list[str] = []
    for tool_name, command in audit_calls:
        if tool_name in ("edit", "write"):
            merged.append("mutation")
        elif tool_name == "read":
            merged.append("query")
        elif tool_name == "bash" and command:
            consumed = False
            while gi < n and allow_guard[gi].raw_command and allow_guard[gi].raw_command.strip() in command:
                kind = allow_guard[gi].kind
                if kind:
                    merged.append(kind)
                gi += 1
                consumed = True
            if not consumed:
                kind = classify_command_kind(command)
                if kind:
                    merged.append(kind)
    while gi < n:
        kind = allow_guard[gi].kind
        if kind:
            merged.append(kind)
        gi += 1
    return merged


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
