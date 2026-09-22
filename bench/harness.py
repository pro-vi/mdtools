"""Offline fixed-task path selectively recovered from H (c933520).

Retains task fields, fresh staging, subprocess execution, final-file capture and
artifact writing. Excluded providers/Pi/multifile branches, dual scorers,
quarantine, old resume/report policy and eager configuration imports are removed.
The native real backend requires an explicit bounded phase grant.
"""

from __future__ import annotations

import argparse
from contextlib import ExitStack, nullcontext
from dataclasses import asdict, dataclass, replace
import fcntl
import hashlib
import json
import math
import os
import platform
from pathlib import Path, PurePosixPath
import signal
import selectors
import subprocess
import sys
import tempfile
import time
import uuid
from typing import BinaryIO, Callable, Literal, Mapping, Sequence, TypedDict

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from bench.command_policy import (CliCondition, ConditionPin, ClaudeRunner, CLAUDE_SHA256,
    CLAUDE_VERSION, DYLD_PROFILE, build_runner_command, native_profile, prepare_native_boundary,
    resolve_toolkit, stage_condition, tool_reference, verify_condition, ToolGuidance)
from bench.claude_shell import ContainmentError, OwnedShells, process_state
from bench.manifest import (CORE_TASK_IDS, CORPUS_PATHS, CampaignConfig, CampaignSpec,
    CorePreparationConsent, ExperimentSpec, LiveRunGrant, canonical_json, sha256_file, sha256_text, serial_schedule)
from bench.neutral_scorer import (GRADER_VERSION, SubmissionFormat, StructuralDiffPolicy,
    answer_instructions, answer_policy_snapshot, grade_submission, validate_expected, validate_policy)
from bench.trial_records import (SCHEMA, AttemptKey, AttemptStart, AttemptResult,
    ClaudeEvent, ExecutionOutcome, Grade, PermissionDenial, RecordIntegrityError,
    RunReceipt, ToolCall, ToolResult, Usage, admission_fault, decode_record_json,
    derive_trial_disposition, model_identity_fault, permission_fault, record_dict, require_index, require_quantity, require_text)


@dataclass(frozen=True)
class BenchTask:
    id: str
    description: str
    input_files: list[str]
    expected_output: str
    expected_artifact: str
    difficulty: str
    scorer: StructuralDiffPolicy
    expected_stdout: str | None = None
    support_files: list[str] | None = None


def public_task(task_id: str) -> BenchTask:
    """Project only the requested public row; no holdout rows enter Python."""
    if task_id not in ("T1", "T2", "T10", "T21"):
        raise ValueError("public packaging fixture is not allowlisted")
    import shutil
    jq = shutil.which("jq")
    if jq is None:
        raise RuntimeError("public-fixture projection requires jq")
    projected = subprocess.run([jq, "--arg", "id", task_id,
        '.[] | select(.id == $id) | {id,description,input_files,expected_output,expected_artifact,difficulty,scorer,expected_stdout,support_files}',
        str(Path(__file__).resolve().parent / "tasks/tasks.json")], check=True, capture_output=True)
    record = decode_record_json(projected.stdout)
    record["scorer"] = StructuralDiffPolicy(**record["scorer"])
    return BenchTask(**record)


@dataclass(frozen=True)
class ParsedAgentOutput:
    events: tuple[ClaudeEvent, ...]
    receipt: RunReceipt | None
    permission_fault: str | None
    integrity_fault: str | None
    usage: Usage


class ClaudeStreamDecoder:
    """Decode the pinned CLI's exercised envelopes; retain opaque IDs per attempt.

Callbacks are synchronous. The fault is latched before persistence/cleanup,
    and an exception in either callback cannot restore grade admission.
    """

    def __init__(self, *, backend: str = "synthetic",
                 requested_model: str | None = None,
                 on_raw: Callable[[dict[str, object]], None] | None = None,
                 on_fault: Callable[[str], None] | None = None) -> None:
        if backend not in ("synthetic", "claude_cli"):
            raise RecordIntegrityError("unknown decoder evidence backend")
        self.backend = backend
        if requested_model is not None:
            require_text(requested_model, "requested model")
        self.requested_model = requested_model
        self.on_raw = on_raw
        self.on_fault = on_fault
        self.permission_fault: str | None = None
        self.integrity_fault: str | None = None
        self.receipt: RunReceipt | None = None
        self.partial_usage = Usage()
        self._events: list[ClaudeEvent] = []
        self._calls: dict[str, ToolCall] = {}
        self._results: dict[str, ToolResult] = {}
        self._receipt_raw: str | None = None
        self._usage_raw: str | None = None
        self._buffer = b""

    @property
    def parsed(self) -> ParsedAgentOutput:
        return ParsedAgentOutput(tuple(self._events), self.receipt,
                                 self.permission_fault, self.integrity_fault,
                                 self.receipt.usage if self.receipt else self.partial_usage)

    def _fault(self, reason: str) -> None:
        if reason == "permission_denied":
            self.permission_fault = reason
        elif self.integrity_fault is None:
            self.integrity_fault = reason

    def accept(self, raw: object) -> None:
        terminal = type(raw) is dict and raw.get("type") == "result"
        previous = self.permission_fault or self.integrity_fault
        try:
            observed_fault = permission_fault(raw, terminal=terminal)
        except RecordIntegrityError:
            observed_fault = "receipt_integrity" if terminal else "trace_integrity"
        if observed_fault:
            self._fault(observed_fault)
        try:
            if type(raw) is not dict:
                raise RecordIntegrityError("provider event must be an object")
            if self.on_raw is not None:
                self.on_raw(raw)
        except (OSError, ValueError, TypeError) as exc:
            self._fault("trace_integrity")
            if self.on_fault is not None:
                self.on_fault(self.permission_fault or self.integrity_fault)
            if isinstance(exc, OSError):
                raise
            raise RecordIntegrityError("provider evidence could not be persisted") from exc
        if observed_fault and previous is None and self.on_fault is not None:
            self.on_fault(observed_fault)
        try:
            if terminal:
                # Valid quantities from a faulty receipt are partial evidence,
                # never grade admission or a second cumulative usage charge.
                canonical = canonical_json(raw)
                if self._usage_raw is None:
                    self.partial_usage = self._decode_usage(raw, complete=False)
                    self._usage_raw = canonical
                elif self._usage_raw != canonical:
                    raise RecordIntegrityError("conflicting cumulative usage receipt")
            if observed_fault in ("receipt_integrity", "trace_integrity"):
                raise RecordIntegrityError("invalid required permission metadata")
            if terminal:
                self._accept_receipt(raw)
            elif raw.get("type") in ("assistant", "user"):
                self._accept_message(raw)
            elif raw.get("type") == "system" and raw.get("subtype") == "permission_denied":
                self._check_reference(raw["tool_use_id"], raw["tool_name"])
            # Partial stream_event and irrelevant messages remain raw only.
        except (ValueError, TypeError, KeyError) as exc:
            self._fault("receipt_integrity" if terminal else "trace_integrity")
            if self.on_fault is not None:
                self.on_fault(self.permission_fault or self.integrity_fault)
            raise RecordIntegrityError("unsupported or conflicting required CLI event") from exc

    def feed(self, chunk: bytes) -> None:
        if type(chunk) is not bytes:
            raise RecordIntegrityError("CLI stream chunks must be bytes")
        self._buffer += chunk
        while b"\n" in self._buffer:
            line, self._buffer = self._buffer.split(b"\n", 1)
            if line:
                try:
                    self.accept(decode_record_json(line))
                except RecordIntegrityError:
                    self._fault("trace_integrity")
                    if self.on_fault is not None:
                        self.on_fault(self.permission_fault or self.integrity_fault)
                    raise

    def finish(self, *, require_receipt: bool = True) -> ParsedAgentOutput:
        if self._buffer:
            fragment, self._buffer = self._buffer, b""
            try:
                self.accept(decode_record_json(fragment))
            except RecordIntegrityError:
                self._fault("trace_integrity")
                raise
        if require_receipt and self.receipt is None and self.permission_fault is None:
            self._fault("trace_integrity")
            raise RecordIntegrityError("missing terminal CLI receipt")
        if require_receipt and self.permission_fault is None and set(self._calls) != set(self._results):
            self._fault("trace_integrity")
            raise RecordIntegrityError("missing completed tool result")
        return self.parsed

    def _check_reference(self, tool_id: str, tool_name: str | None = None) -> None:
        require_text(tool_id, "tool-use reference")
        call = self._calls.get(tool_id)
        if call is None or (tool_name is not None and call.tool_name != tool_name):
            raise RecordIntegrityError("unknown or mismatched tool-use reference")

    def _accept_message(self, raw: dict[str, object]) -> None:
        message = raw.get("message")
        if type(message) is not dict or type(message.get("content")) is not list:
            raise RecordIntegrityError("unsupported CLI message content")
        for block in message["content"]:
            if type(block) is not dict:
                raise RecordIntegrityError("unsupported CLI content block")
            if block.get("type") == "tool_use":
                if raw["type"] != "assistant" or type(block.get("input")) is not dict:
                    raise RecordIntegrityError("unsupported tool call")
                event = ToolCall(block.get("id"), block.get("name"), canonical_json(block["input"]))
                prior = self._calls.get(event.tool_use_id)
                if prior is not None:
                    if prior != event:
                        raise RecordIntegrityError("conflicting duplicate tool call")
                    continue
                if self.receipt is not None:
                    raise RecordIntegrityError("tool call after terminal receipt")
                self._calls[event.tool_use_id] = event
                self._events.append(event)
            elif block.get("type") == "tool_result":
                if raw["type"] != "user":
                    raise RecordIntegrityError("unsupported tool result")
                event = ToolResult(block.get("tool_use_id"), block.get("content"), block.get("is_error"))
                self._check_reference(event.tool_use_id)
                prior = self._results.get(event.tool_use_id)
                if prior is not None:
                    if prior != event:
                        raise RecordIntegrityError("conflicting duplicate tool result")
                    continue
                if self.receipt is not None:
                    raise RecordIntegrityError("tool result after terminal receipt")
                self._results[event.tool_use_id] = event
                self._events.append(event)

    def _accept_receipt(self, raw: dict[str, object]) -> None:
        canonical = canonical_json(raw)
        if self._receipt_raw is not None:
            if canonical != self._receipt_raw:
                raise RecordIntegrityError("conflicting terminal receipt")
            return
        for name in ("num_turns", "result_index"):
            require_index(raw.get(name), name)
        if type(raw.get("is_error")) is not bool:
            raise RecordIntegrityError("required receipt is_error must be boolean")
        subtype, terminal_reason = raw.get("subtype"), raw.get("terminal_reason")
        require_text(subtype, "receipt subtype")
        require_text(terminal_reason, "terminal reason")
        outcomes = {
            ("success", "completed", False): ExecutionOutcome("completed"),
            ("error_max_turns", "max_turns", True): ExecutionOutcome("budget_exhausted", "turn_limit"),
            ("error_max_budget_usd", "max_budget_usd", True): ExecutionOutcome("budget_exhausted", "cost_limit"),
            ("error_during_execution", "authentication_error", True): ExecutionOutcome("infrastructure_error", "authentication_error"),
            ("error_during_execution", "transport_error", True): ExecutionOutcome("infrastructure_error", "transient_transport"),
            ("error_during_execution", "startup_error", True): ExecutionOutcome("infrastructure_error", "transient_startup"),
        }
        execution = outcomes.get((subtype, terminal_reason, raw["is_error"]))
        if execution is None:
            raise RecordIntegrityError("unexercised/unsupported terminal outcome")
        if self.permission_fault:
            execution = ExecutionOutcome("infrastructure_error", "permission_denied")
        final_text = raw.get("result", "") if execution.kind != "completed" else raw.get("result")
        require_text(final_text, "terminal answer", nonempty=False)
        denials = tuple(PermissionDenial(d["tool_name"], d["tool_use_id"], canonical_json(d["tool_input"])) for d in raw["permission_denials"])
        for denial in denials:
            self._check_reference(denial.tool_use_id, denial.tool_name)
            if denial.tool_input_json != self._calls[denial.tool_use_id].input_json:
                raise RecordIntegrityError("denial input contradicts call")
        models = raw.get("modelUsage")
        if models is not None and (type(models) is not dict or len(models) > 1):
            raise RecordIntegrityError("unsupported/ambiguous observed model usage")
        observed_model = next(iter(models)) if models else None
        if observed_model is not None:
            require_text(observed_model, "observed model")
        identity_fault = model_identity_fault(backend=self.backend, execution=execution,
            requested_model=self.requested_model, observed_model=observed_model)
        if identity_fault and execution.kind == "completed":
            execution = ExecutionOutcome("infrastructure_error", identity_fault)
        usage = self._decode_usage(raw, complete=True)
        self.receipt = RunReceipt(execution, usage, final_text, observed_model, denials,
                                 raw["is_error"], subtype, terminal_reason)
        self._receipt_raw = canonical
        self._events.append(self.receipt)
        if identity_fault and self.on_fault is not None:
            self.on_fault(identity_fault)

    def _decode_usage(self, raw: dict[str, object], *, complete: bool) -> Usage:
        reported = raw.get("usage")
        if reported is not None and type(reported) is not dict:
            raise RecordIntegrityError("unsupported cumulative usage")
        reported = reported or {}
        require_quantity(raw.get("duration_ms"), "duration milliseconds")
        elapsed = raw["duration_ms"] / 1000 if raw.get("duration_ms") is not None else None
        measurements = {
            "input_tokens": reported.get("input_tokens"), "output_tokens": reported.get("output_tokens"),
            "cache_read_tokens": reported.get("cache_read_input_tokens"),
            "cache_creation_tokens": reported.get("cache_creation_input_tokens"),
            "estimated_usd": raw.get("total_cost_usd"), "elapsed_seconds": elapsed,
            "tool_output_bytes": sum(len(event.content.encode("utf-8")) for event in self._results.values()),
        }
        completeness = "complete" if complete and all(v is not None for v in measurements.values()) else "partial"
        return Usage(**measurements, source="synthetic_receipt" if self.backend == "synthetic" else "claude_terminal", completeness=completeness)


def _relative_path(reference: str) -> Path:
    if not isinstance(reference, str) or not reference or "\\" in reference or "\0" in reference:
        raise ValueError("artifact reference must be a nonempty relative POSIX path")
    parts = reference.split("/")
    if any(part in ("", ".", "..") for part in parts) or PurePosixPath(reference).is_absolute():
        raise ValueError("artifact traversal or noncanonical path")
    return Path(*parts)


def _assert_no_symlinks(path: Path) -> None:
    for component in (*reversed(path.parents), path):
        if component.is_symlink():
            raise ValueError("symlink component in artifact path")


def _root(path: Path) -> Path:
    path = Path(os.path.abspath(path))
    _assert_no_symlinks(path)
    return path


def _read_source(root: Path, reference: str) -> bytes:
    path = root / _relative_path(reference)
    _assert_no_symlinks(path)
    if not path.is_file():
        raise ValueError("missing or nonregular source object")
    return path.read_bytes()


def _overlaps(left: Path, right: Path) -> bool:
    return left == right or left in right.parents or right in left.parents


def build_prompt(task: BenchTask, *, condition: ConditionPin | None = None,
                 guidance: ToolGuidance = ToolGuidance.FULL_HELP) -> str:
    """Task text and relative references only; never preload input or expected bytes."""
    guidance = ToolGuidance(guidance)
    contract = (f"TASK: {task.description}\nINPUT FILES: {json.dumps(task.input_files)}\n"
            f"SUPPORT FILES: {json.dumps(task.support_files or [])}\n"
            f"OUTPUT: {answer_instructions(task.scorer, artifact=task.expected_artifact)}\n")
    return contract if condition is None else contract + "\nTOOLS:\n" + tool_reference(condition, guidance=guidance) + "\n"


@dataclass(frozen=True)
class ReadRequest:
    """Controller-supplied frozen synthetic/public bytes; not an agent proxy."""
    document: bytes
    argv: tuple[str, ...]
    stdin: bytes = b""

    def __post_init__(self) -> None:
        if not isinstance(self.document, bytes) or not isinstance(self.stdin, bytes) or not isinstance(self.argv, tuple) or any(not isinstance(arg, str) or "\0" in arg for arg in self.argv):
            raise ValueError("replay requires bytes and exact tuple argv")


class StreamCapture(TypedDict):
    bytes: int
    sha256: str
    tokens: int | None
    unavailable: str | None


class TokenizerIdentity(TypedDict):
    encoding: str
    implementation: str
    special_tokens: str | None


class CliInvocation(TypedDict):
    argv: list[str]
    stdin_sha256: str
    exit_code: int
    streams: dict[str, StreamCapture]
    tokenizer: TokenizerIdentity | None
    provider_usage: bool
    billed_tokens: bool


class ReplayRecord(TypedDict):
    schema: str
    binary_sha256: str
    fixture_sha256: str
    request_sha256: str
    category: str
    invocations: list[CliInvocation]
    agent_trial: bool
    provider_usage: bool


class ExampleRecord(TypedDict):
    condition: str
    binary_sha256: str
    invocations: list[CliInvocation]
    committed_mutations: int
    live_isolation_verified: bool


def _capture_cli(pin: ConditionPin, argv: Sequence[str], *, cwd: Path,
                 stdin: bytes, output: Path, measure: bool = False) -> CliInvocation:
    verify_condition(pin)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    effective = [str(Path(pin.executable).resolve(strict=True)), *argv]
    usage_path = output / "usage.jsonl"
    env = {"PATH": "/usr/bin:/bin", "TMPDIR": str(cwd), "LC_ALL": "C"}
    if measure:
        env.update(MD_USAGE_LOG=str(usage_path), MD_USAGE_CALLER="benchmark")
    completed = subprocess.run(effective, cwd=cwd, input=stdin, env=env,
        capture_output=True, timeout=30)
    streams: dict[str, StreamCapture] = {}
    for name, payload in (("stdout", completed.stdout), ("stderr", completed.stderr)):
        _write_private(output / f"{name}.bin", payload)
        streams[name] = {"bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest(),
                         "tokens": None, "unavailable": "measurement_not_supplied"}
    tokenizer = None
    if usage_path.exists():
        rows = [json.loads(line) for line in usage_path.read_bytes().splitlines()]
        if len(rows) != 1:
            raise ValueError("replay usage must contain exactly one invocation")
        usage = rows[0]
        if usage.get("schema") != "md-usage.v1" or usage.get("surface") != "cli_streams" or type(usage.get("exit")) is not int or usage["exit"] != completed.returncode:
            raise ValueError("invalid replay measurement receipt")
        tokenizer = usage.get("tokenizer")
        if not isinstance(tokenizer, str) or not isinstance(usage.get("tokenizer_implementation"), str):
            raise ValueError("missing named tokenizer")
        for name in streams:
            measured = usage.get(name)
            if not isinstance(measured, dict) or type(measured.get("bytes")) is not int or measured["bytes"] != streams[name]["bytes"]:
                raise ValueError("measured stream byte mismatch")
            tokens = measured.get("tokens")
            if tokens is not None and (type(tokens) is not int or tokens < 0):
                raise ValueError("invalid tokenizer count")
            if tokens is None and not isinstance(measured.get("unavailable"), str):
                raise ValueError("missing measurement unavailable reason")
            streams[name].update(tokens=tokens, unavailable=measured.get("unavailable"))
        tokenizer = {"encoding": tokenizer, "implementation": usage["tokenizer_implementation"],
                     "special_tokens": usage.get("special_tokens")}
    record: CliInvocation = {"argv": effective, "stdin_sha256": hashlib.sha256(stdin).hexdigest(),
        "exit_code": completed.returncode, "streams": streams, "tokenizer": tokenizer,
        "provider_usage": False, "billed_tokens": False}
    _write_private(output / "invocation.json", canonical_json(record).encode())
    return record


def replay_read_pair(pin: ConditionPin, compact: ReadRequest, full: ReadRequest,
                     *, output: Path) -> ReplayRecord:
    """Replay one identical read request, differing only by global --json."""
    commands = verify_condition(pin)
    if pin.condition != CliCondition.CURRENT_COMPACT:
        raise ValueError("replay requires the pinned current binary")
    if compact != full:
        raise ValueError("nonidentical replay input/request")
    if not compact.argv or compact.argv[0] not in {item.name for item in commands if item.kind == "query"}:
        raise ValueError("replay requires a producer read command")
    if compact.argv[0] not in ("map", "query", "read") or "--json" in compact.argv:
        raise ValueError("unsupported replay request/presentation flag")
    # The controller owns this synthetic basename; no external fixture paths are
    # accepted. Only the presentation flag is inserted into the effective argv.
    if len(compact.argv) < 2 or compact.argv[1] != 'document with "quotes".md':
        raise ValueError("replay must reference the frozen controller document")
    if compact.argv[0] == "map":
        if len(compact.argv) != 2 or compact.stdin:
            raise ValueError("unsupported map replay arguments")
    elif len(compact.argv) != 4 or compact.argv[2] not in (("--query", "--from") if compact.argv[0] == "query" else ("--query", "--address", "--from")):
        raise ValueError("unsupported replay input flags")
    elif compact.argv[2] == "--from" and compact.argv[3] != "-":
        raise ValueError("replay disallows external input files")
    output = _root(output)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    document = output / compact.argv[1]
    _write_private(document, compact.document)
    records: list[CliInvocation] = []
    for name, argv in (("compact", compact.argv), ("full", ("--json", *full.argv))):
        if document.read_bytes() != compact.document:
            raise ValueError("replay fixture changed before invocation")
        records.append(_capture_cli(pin, argv, cwd=output, stdin=compact.stdin,
                                   output=output / name, measure=True))
        if document.read_bytes() != compact.document:
            raise ValueError("read replay changed fixture bytes")
    if records[0]["exit_code"] != records[1]["exit_code"]:
        raise ValueError("replay exits differ")
    if compact.argv[0] == "read" and records[0]["exit_code"] == 0:
        typed = json.loads((output / "full/stdout.bin").read_bytes())
        content = typed.get("source", typed.get("markdown", typed.get("raw")))
        if typed.get("type") == "frontmatter" and content is None:
            content = ""
        if not isinstance(content, str):
            raise ValueError("read replay content shape unsupported; no equivalence claim")
        if content.encode() != (output / "compact/stdout.bin").read_bytes():
            raise ValueError("compact/full requested content mismatch")
    record: ReplayRecord = {"schema": "mdtools.cli-eval.replay/1", "binary_sha256": pin.binary_sha256,
        "fixture_sha256": hashlib.sha256(compact.document).hexdigest(),
        "request_sha256": sha256_text(canonical_json({"argv": compact.argv, "stdin_sha256": hashlib.sha256(compact.stdin).hexdigest()})),
        "category": "successful_read" if records[0]["exit_code"] == 0 else "error_output",
        "invocations": records, "agent_trial": False, "provider_usage": False}
    _write_private(output / "replay.json", canonical_json(record).encode())
    return record


def exercise_cli_examples(pin: ConditionPin, *, output: Path) -> ExampleRecord:
    """Execute the documented controller recipes on disposable synthetic Markdown."""
    commands = verify_condition(pin)
    output = _root(output)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    filename = 'document with "quotes".md'
    document = output / filename
    original = b'# Heading\n\nBefore\n'
    _write_private(document, original)
    invocations: list[CliInvocation] = []
    def run(argv: Sequence[str], stdin: bytes = b"") -> CliInvocation:
        record = _capture_cli(pin, argv, cwd=output, stdin=stdin,
            output=output / f"example-{len(invocations):02d}")
        invocations.append(record)
        return record
    if pin.condition == CliCondition.NO_MD:
        if run(["--help"])["exit_code"] != 1:
            raise ValueError("unavailable-md stub did not exit unavailable")
    elif pin.condition == CliCondition.LEGACY:
        # Recipe names must exist in this binary's producer schema.
        if not {"schema", "outline", "section"}.issubset({item.name for item in commands}):
            raise ValueError("legacy examples require absent producer command")
        for argv in (["--json", "schema"], ["outline", filename], ["--json", "outline", filename], ["section", "Heading", filename]):
            if run(argv)["exit_code"] != 0:
                raise ValueError("legacy example failed")
    else:
        if not {"schema", "map", "query", "read", "patch"}.issubset({item.name for item in commands}):
            raise ValueError("current examples require absent producer command")
        query = canonical_json({"type": "section", "text": "Heading", "match_mode": "exact"})
        for argv, stdin in ((["schema"], b""), (["map", filename], b""),
            (["query", filename, "--query", query], b""),
            (["read", filename, "--query", query], b""),
            (["read", filename, "--address", '{"kind":"preamble"}'], b""),
            (["read", filename, "--from", "-"], b'{"kind":"preamble"}'),
            (["query", filename, "--from", "-"], query.encode()),
            (["--json", "read", filename, "--query", query], b"")):
            if run(argv, stdin)["exit_code"] != 0:
                raise ValueError("current discovery/read example failed")
        if run(["query", filename, "--query", '{"type":"kind","kind":"block"}'])["exit_code"] != 0:
            raise ValueError("guard preparation query failed")
        overview = json.loads((output / f"example-{len(invocations)-1:02d}/stdout.bin").read_bytes())
        address = next(item["target"]["address"] for item in overview if item["type"] == "target" and item["target"]["summary"].get("kind") == "paragraph")
        run(["--json", "read", filename, "--address", canonical_json(address)])
        snapshot = json.loads((output / f"example-{len(invocations)-1:02d}/stdout.bin").read_bytes())["snapshot"]
        patch = {"base_revision": snapshot["revision"], "operations": [{"op": "replace_block", "target": {
            "address": address["block"], "revision": snapshot["revision"], "guard": {
                "span": snapshot["guard"]["span"], "etag": snapshot["guard"]["etag"]}}, "markdown": "After\n"}]}
        payload = canonical_json(patch)
        if run(["--json", "patch", filename, "--patch", payload])["exit_code"] != 0 or document.read_bytes() != original:
            raise ValueError("preview mutated or failed")
        stale = json.loads(payload)
        # A different observed selection's valid etag cannot authorize this
        # block, even though the base revision and block span are still current.
        section_snapshot = json.loads((output / "example-07/stdout.bin").read_bytes())["snapshot"]
        stale["operations"][0]["target"]["guard"]["etag"] = section_snapshot["guard"]["etag"]
        if run(["--json", "patch", filename, "--patch", canonical_json(stale), "--in-place"])["exit_code"] == 0 or document.read_bytes() != original:
            raise ValueError("stale etag accepted or mutated")
        if run(["patch", filename, "--from", "-", "--in-place"], payload.encode())["exit_code"] != 0 or document.read_bytes() == original:
            raise ValueError("guarded commit failed")
        committed = document.read_bytes()
        if run(["patch", filename, "--patch", payload, "--in-place"])["exit_code"] == 0 or document.read_bytes() != committed:
            raise ValueError("stale patch accepted or mutated")
        if run(["read", filename, "--address", canonical_json(address)])["exit_code"] != 0:
            raise ValueError("committed reread failed")
        if (output / f"example-{len(invocations)-1:02d}/stdout.bin").read_bytes() != b"After":
            raise ValueError("reread does not contain committed content")
    record: ExampleRecord = {"condition": pin.condition.value, "binary_sha256": pin.binary_sha256,
        "invocations": invocations, "committed_mutations": int(document.read_bytes() != original),
        "live_isolation_verified": False}
    _write_private(output / "examples.json", canonical_json(record).encode())
    return record


def _validate_task(task: BenchTask) -> None:
    if not isinstance(task.id, str) or not task.id or not isinstance(task.description, str):
        raise ValueError("task requires a string identity and description")
    if not isinstance(task.input_files, list) or not task.input_files or any(not isinstance(ref, str) for ref in task.input_files):
        raise ValueError("task input_files must be a nonempty list of paths")
    if task.support_files is not None and (not isinstance(task.support_files, list) or any(not isinstance(ref, str) for ref in task.support_files)):
        raise ValueError("task support_files must be a list of paths or null")
    if not isinstance(task.scorer, StructuralDiffPolicy):
        raise ValueError("task scorer must be a StructuralDiffPolicy")
    if not isinstance(task.difficulty, str):
        raise ValueError("task difficulty must be a string")


def _write_private(path: Path, content: bytes) -> None:
    # pathlib's parents=True applies mode only to the last directory. Create
    # each missing component explicitly so nested captured artifacts stay private.
    for directory in (*reversed(path.parent.parents), path.parent):
        if not directory.exists():
            directory.mkdir(mode=0o700)
    with path.open("xb") as handle:
        os.chmod(path, 0o600)
        handle.write(content)
        handle.flush()
        os.fsync(handle.fileno())
    _fsync_directory(path.parent)


def _fsync_directory(directory: Path) -> None:
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


class AttemptStore:
    """Private attempt evidence with one exclusive, recoverable finalizer.

The atomic publication is an exclusive hard link to a flushed candidate. Its
    referenced manifest/files precede it. A crash before publication leaves a
    durable unfinished start; no fabricated terminal status is synthesized.
    """

    def __init__(self, directory: Path) -> None:
        self.directory = _root(directory)

    def start(self, start: AttemptStart) -> None:
        if not isinstance(start, AttemptStart):
            raise RecordIntegrityError("start requires validated identity/reservation")
        # Refuse a reused directory, including one left by a failed write.
        for parent in (*reversed(self.directory.parent.parents), self.directory.parent):
            if not parent.exists():
                parent.mkdir(mode=0o700)
        self.directory.mkdir(mode=0o700, exist_ok=False)
        _fsync_directory(self.directory.parent)
        _write_private(self.directory / "started.json", (canonical_json(record_dict(start)) + "\n").encode())
        _write_private(self.directory / "events.jsonl", b"")
        _write_private(self.directory / ".attempt.lock", b"")

    def _locked_file(self) -> BinaryIO:
        lock = self.directory / ".attempt.lock"
        _assert_no_symlinks(lock)
        handle = lock.open("r+b")
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BaseException:
            handle.close()
            raise
        return handle

    def append_event(self, raw: dict[str, object]) -> None:
        encoded = (canonical_json(raw) + "\n").encode("utf-8")
        with self._locked_file():
            if (self.directory / "result.json").exists():
                raise RecordIntegrityError("closed attempt cannot accept new evidence")
            path = self.directory / "events.jsonl"
            _assert_no_symlinks(path)
            with path.open("ab") as handle:
                handle.write(encoded)
                handle.flush()
                os.fsync(handle.fileno())

    def _read_start(self) -> AttemptStart:
        return AttemptStart.from_dict(decode_record_json(self._read_evidence("started.json")))

    def _read_evidence(self, reference: str) -> bytes:
        try:
            return _read_source(self.directory, reference)
        except (ValueError, OSError) as exc:
            raise RecordIntegrityError("missing, unreadable or unsafe attempt evidence") from exc

    def persisted_fault(self) -> str | None:
        """Resume admission checks flushed events even without any result file."""
        self._read_start()
        payload = self._read_evidence("events.jsonl")
        fault: str | None = None
        for line in payload.split(b"\n"):
            if not line:
                continue
            terminal = False
            try:
                raw = decode_record_json(line)
                terminal = type(raw) is dict and raw.get("type") == "result"
                observed = permission_fault(raw, terminal=terminal)
            except RecordIntegrityError:
                observed = "receipt_integrity" if terminal else "trace_integrity"
            # A subsequent empty receipt or malformed event cannot clear denial.
            if observed == "permission_denied":
                return observed
            fault = fault or observed
        if (self.directory / "result.json").exists():
            result = AttemptResult.from_dict(decode_record_json(self._read_evidence("result.json")))
            return result.permission_fault or admission_fault(result) or fault
        return fault

    def assert_admission(self) -> None:
        fault = self.persisted_fault()
        if fault:
            raise RecordIntegrityError(f"campaign admission held: {fault}")

    def _assert_artifacts(self, artifacts: dict[str, str]) -> None:
        for reference, digest in artifacts.items():
            if hashlib.sha256(self._read_evidence(reference)).hexdigest() != digest:
                raise RecordIntegrityError("artifact digest mismatch")

    def submission_format(self, grade: Grade, artifacts: Mapping[str, str]) -> SubmissionFormat | None:
        """Verify the policy and syntax evidence without regrading saved work."""
        reference = "artifacts/submission_format.json"
        # Older low-level synthetic records did not require experiment.json.
        if not (self.directory / "experiment.json").exists():
            if reference in artifacts:
                raise RecordIntegrityError("submission format lacks frozen experiment")
            return None
        spec = ExperimentSpec.from_dict(decode_record_json(self._read_evidence("experiment.json")))
        if spec.grader_version != GRADER_VERSION:
            return None
        if grade.kind not in ("pass", "fail"):
            if reference in artifacts:
                raise RecordIntegrityError("ungraded attempt has submission format")
            return None
        if reference not in artifacts:
            raise RecordIntegrityError("graded attempt lacks submission format")
        try:
            payload = self._read_evidence(reference)
            if hashlib.sha256(payload).hexdigest() != artifacts[reference]:
                raise ValueError("submission format artifact digest mismatch")
            observed = SubmissionFormat.from_dict(decode_record_json(payload))
            if sha256_text(canonical_json(observed.answer_policy)) != spec.answer_policy_sha256:
                raise ValueError("submission policy contradicts experiment")
            final_text = (self._read_evidence("artifacts/final_submission.bin")
                if observed.answer_policy["artifact"] != "file_contents" else b"")
            if (observed.answer_policy["artifact"] != "file_contents" and
                hashlib.sha256(final_text).hexdigest() != artifacts.get("artifacts/final_submission.bin")):
                raise ValueError("final submission artifact digest mismatch")
            observed.assert_matches(final_text)
        except ValueError as exc:
            raise RecordIntegrityError(str(exc)) from exc
        return observed

    def _assert_trace(self, start: AttemptStart, execution: ExecutionOutcome,
                      grade: Grade, permission_reason: str | None, usage: Usage,
                      observed_model: str | None, artifacts: dict[str, str]) -> None:
        payload = self._read_evidence("events.jsonl")
        if not payload and start.backend == "synthetic":
            return  # The explicit synthetic text transport has no CLI envelope.
        decoder = ClaudeStreamDecoder(backend=start.backend, requested_model=start.requested_model)
        try:
            decoder.feed(payload)
            parsed = decoder.finish(require_receipt=execution.kind == "completed")
        except RecordIntegrityError:
            if execution.kind == "completed" or grade.kind in ("pass", "fail"):
                raise
            parsed = decoder.parsed  # Retain known fields from partial traces.
        if parsed.permission_fault and permission_reason != "permission_denied":
            raise RecordIntegrityError("result erased trace permission fault")
        if execution.kind == "completed" and (parsed.receipt is None or parsed.receipt.execution.kind != "completed" or parsed.integrity_fault):
            raise RecordIntegrityError("trace cannot establish normal completion")
        expected_usage = parsed.usage
        # These quantities come from the cumulative receipt/observed tool trace.
        # Elapsed seconds belong to the controller's independent monotonic clock.
        for name in ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens", "estimated_usd"):
            if getattr(usage, name) != getattr(expected_usage, name):
                raise RecordIntegrityError("attempt usage contradicts terminal receipt")
        if expected_usage.source != "unavailable" and usage.source != expected_usage.source:
            raise RecordIntegrityError("attempt usage source contradicts terminal receipt")
        if expected_usage.source != "unavailable" and usage.tool_output_bytes != expected_usage.tool_output_bytes:
            raise RecordIntegrityError("attempt tool bytes contradict terminal trace")
        if parsed.receipt is not None:
            if observed_model != parsed.receipt.observed_model:
                raise RecordIntegrityError("attempt model contradicts terminal receipt")
            reference = "artifacts/final_submission.bin"
            if reference in artifacts and self._read_evidence(reference) != parsed.receipt.final_text.encode("utf-8"):
                raise RecordIntegrityError("final submission contradicts terminal receipt")

    def load(self) -> tuple[AttemptStart, AttemptResult | None]:
        start = self._read_start()
        if not (self.directory / "result.json").exists():
            return start, None
        result = AttemptResult.from_dict(decode_record_json(self._read_evidence("result.json")))
        if result.key != start.key or result.backend != start.backend or result.requested_model != start.requested_model:
            raise RecordIntegrityError("result/start identity mismatch")
        manifest_bytes = self._read_evidence("artifact_manifest.json")
        if hashlib.sha256(manifest_bytes).hexdigest() != result.artifact_manifest_sha256:
            raise RecordIntegrityError("artifact manifest digest mismatch")
        manifest = decode_record_json(manifest_bytes)
        if type(manifest) is not dict or set(manifest) != {"schema", "record", "key", "artifacts", "evidence_complete"} or type(manifest["evidence_complete"]) is not bool or type(manifest["artifacts"]) is not dict:
            raise RecordIntegrityError("invalid closed artifact manifest")
        if AttemptKey.from_dict(manifest["key"]) != start.key:
            raise RecordIntegrityError("artifact manifest key mismatch")
        if manifest != {"schema": SCHEMA, "record": "artifact_manifest", "key": record_dict(start.key),
                        "artifacts": dict(result.artifacts), "evidence_complete": result.evidence_complete}:
            raise RecordIntegrityError("artifact manifest contradicts result")
        self._assert_artifacts(dict(result.artifacts))
        self.submission_format(result.grade, result.artifacts)
        self._assert_trace(start, result.execution, result.grade, result.permission_fault,
                           result.usage, result.observed_model, dict(result.artifacts))
        if self.persisted_fault() == "permission_denied" and result.permission_fault != "permission_denied":
            raise RecordIntegrityError("final record erased persisted permission fault")
        return start, result

    def finalize(self, *, execution: ExecutionOutcome, grade: Grade, usage: Usage,
                 artifacts: dict[str, str], evidence_complete: bool,
                 permission_fault: str | None = None, observed_model: str | None = None,
                 exit_code: int | None = None) -> AttemptResult:
        with self._locked_file():
            start = self._read_start()
            persisted = self.persisted_fault()
            if persisted == "permission_denied":
                permission_fault = persisted
                if execution.kind not in ("timed_out", "interrupted"):
                    execution = ExecutionOutcome("infrastructure_error", persisted)
                grade = Grade("not_run", persisted)
            elif persisted and execution.kind == "completed":
                raise RecordIntegrityError("required trace fault forbids completion")
            identity_fault = model_identity_fault(backend=start.backend, execution=execution,
                requested_model=start.requested_model, observed_model=observed_model)
            if identity_fault and execution.kind == "completed":
                execution = ExecutionOutcome("infrastructure_error", identity_fault)
                grade = Grade("not_run", identity_fault)
            recorded = dict(artifacts)
            recorded["events.jsonl"] = sha256_file(self.directory / "events.jsonl")
            self._assert_artifacts(recorded)
            self.submission_format(grade, recorded)
            self._assert_trace(start, execution, grade, permission_fault, usage, observed_model, recorded)
            manifest_bytes = (canonical_json({"schema": SCHEMA, "record": "artifact_manifest",
                "key": record_dict(start.key), "artifacts": recorded,
                "evidence_complete": evidence_complete}) + "\n").encode()
            result = AttemptResult(start.key, start.backend, execution, grade, usage, recorded,
                evidence_complete, hashlib.sha256(manifest_bytes).hexdigest(), permission_fault,
                start.requested_model, observed_model, exit_code)
            encoded = (canonical_json(record_dict(result)) + "\n").encode()
            if (self.directory / "result.json").exists():
                _, prior = self.load()
                if self._read_evidence("result.json") != encoded:
                    raise RecordIntegrityError("conflicting immutable finalization")
                return prior
            manifest = self.directory / "artifact_manifest.json"
            if manifest.exists():
                if self._read_evidence("artifact_manifest.json") != manifest_bytes:
                    raise RecordIntegrityError("conflicting prepared artifact manifest")
            else:
                _write_private(manifest, manifest_bytes)
            candidate = self.directory / (".result-" + uuid.uuid4().hex + ".json")
            _write_private(candidate, encoded)
            # link() is exclusive, atomic, and never overwrites a rival result.
            os.link(candidate, self.directory / "result.json")
            _fsync_directory(self.directory)
            candidate.unlink()  # Only this finalizer's own candidate is removed.
            _fsync_directory(self.directory)
            return result


def _stop_owned_group(process: subprocess.Popen[bytes]) -> None:
    """Stop only the session/group created for this synthetic subprocess."""
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    except PermissionError:
        # macOS can return EPERM for a group whose last process just exited.
        # Do not suppress a real permission failure while any member remains.
        process.poll()  # Reap only this Popen's exited child before inspection.
        observed = subprocess.run(["/bin/ps", "-axo", "pid=,pgid=,stat="], capture_output=True, check=True)
        members = [line.split() for line in observed.stdout.splitlines()]
        if any(len(columns) == 3 and columns[1] == str(process.pid).encode() and not columns[2].startswith(b"Z") for columns in members):
            raise


@dataclass(frozen=True)
class CapturedProcess:
    execution: ExecutionOutcome
    stdout: bytes
    stderr: bytes
    exit_code: int
    elapsed_seconds: float


def capture_process(process: subprocess.Popen[bytes], *, prompt: bytes,
                    timeout_seconds: float, decoder: ClaudeStreamDecoder | None = None,
                    stop_owned: Callable[[], None] | None = None) -> CapturedProcess:
    """Capture one owned lifecycle; interruption at any boundary stops/reaps it.

    U5 supplies cleanup of separately registered Bash groups through stop_owned.
    The Popen group is always stopped independently, even if that callback fails.
    Cleanup/reap failures propagate; they cannot produce a finalized success.
    """
    if process.stdin is None or process.stdout is None or process.stderr is None:
        raise RecordIntegrityError("capture requires all three process pipes")
    if type(prompt) is not bytes or type(timeout_seconds) not in (int, float) or not math.isfinite(timeout_seconds) or timeout_seconds <= 0:
        raise RecordIntegrityError("capture requires prompt bytes and a positive finite deadline")
    started = time.monotonic()
    deadline = started + timeout_seconds
    execution = ExecutionOutcome("completed")
    output: dict[str, bytearray] = {"stdout": bytearray(), "stderr": bytearray()}
    previous_fault = decoder.on_fault if decoder is not None else None

    def cleanup() -> None:
        try:
            if stop_owned is not None:
                stop_owned()
        finally:
            _stop_owned_group(process)

    def close_for_fault(reason: str) -> None:
        nonlocal execution
        if execution.kind == "completed":
            execution = ExecutionOutcome("infrastructure_error", reason)
        cleanup()
        if previous_fault is not None:
            previous_fault(reason)

    decode_open = True
    offset = 0
    interrupted = False
    try:
        if decoder is not None:
            decoder.on_fault = close_for_fault
        with selectors.DefaultSelector() as selector:
            for name, pipe in (("stdout", process.stdout), ("stderr", process.stderr)):
                os.set_blocking(pipe.fileno(), False)
                selector.register(pipe, selectors.EVENT_READ, name)
            os.set_blocking(process.stdin.fileno(), False)
            if prompt:
                selector.register(process.stdin, selectors.EVENT_WRITE, "stdin")
            else:
                process.stdin.close()
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0 and execution.kind == "completed":
                    execution = ExecutionOutcome("timed_out", "deadline")
                    cleanup()
                ready = selector.select(max(0, min(0.05, remaining)) if execution.kind == "completed" else 0.05)
                for selected, _ in ready:
                    pipe, name = selected.fileobj, selected.data
                    if name == "stdin":
                        try:
                            offset += os.write(pipe.fileno(), prompt[offset:offset + 65536])
                        except BrokenPipeError:
                            offset = len(prompt)
                        if offset == len(prompt):
                            selector.unregister(pipe)
                            pipe.close()
                        continue
                    chunk = os.read(pipe.fileno(), 65536)
                    if not chunk:
                        selector.unregister(pipe)
                        continue
                    output[name].extend(chunk)
                    if name == "stdout" and decoder is not None and decode_open:
                        try:
                            decoder.feed(chunk)
                        except (RecordIntegrityError, OSError):
                            decode_open = False
                            close_for_fault(decoder.permission_fault or decoder.integrity_fault or "trace_integrity")
            try:
                process.wait(timeout=max(0.01, deadline - time.monotonic()))
            except subprocess.TimeoutExpired:
                if execution.kind == "completed":
                    execution = ExecutionOutcome("timed_out", "deadline")
                cleanup()
                process.wait(timeout=max(0.01, deadline - time.monotonic()))
        if decoder is not None and decode_open:
            try:
                decoder.finish(require_receipt=execution.kind == "completed")
            except (RecordIntegrityError, OSError):
                close_for_fault(decoder.permission_fault or decoder.integrity_fault or "trace_integrity")
        if execution.kind == "completed":
            if decoder is not None:
                if decoder.permission_fault or decoder.integrity_fault:
                    execution = ExecutionOutcome("infrastructure_error", decoder.permission_fault or decoder.integrity_fault)
                elif decoder.receipt is not None:
                    execution = decoder.receipt.execution
            # CLI limit/auth/error receipts normally accompany nonzero exit.
            # Preserve their validated operational class; a success receipt
            # cannot turn a contradictory nonzero exit into normal completion.
            if execution.kind == "completed" and process.returncode != 0:
                execution = ExecutionOutcome("infrastructure_error", "synthetic_process_exit")
    except KeyboardInterrupt:
        interrupted = True
        if execution.kind != "timed_out":
            execution = ExecutionOutcome("interrupted", "operator_interrupt")
    finally:
        try:
            try:
                cleanup()
            except KeyboardInterrupt:
                interrupted = True
                if execution.kind != "timed_out":
                    execution = ExecutionOutcome("interrupted", "operator_interrupt")
                cleanup()  # Finish the interrupted idempotent ownership cleanup.
        finally:
            try:
                try:
                    process.wait(timeout=max(0.01, deadline - time.monotonic()))
                except KeyboardInterrupt:
                    interrupted = True
                    if execution.kind != "timed_out":
                        execution = ExecutionOutcome("interrupted", "operator_interrupt")
                    cleanup()
                    process.wait(timeout=max(0.01, deadline - time.monotonic()))
            finally:
                if decoder is not None:
                    decoder.on_fault = previous_fault
    if interrupted:
        # Retain available pipe bytes after owned writers are stopped/reaped.
        # They remain partial raw evidence and never re-enter grade admission.
        try:
            for name, pipe in (("stdout", process.stdout), ("stderr", process.stderr)):
                os.set_blocking(pipe.fileno(), False)
                while True:
                    try:
                        chunk = os.read(pipe.fileno(), 65536)
                    except BlockingIOError:
                        break
                    if not chunk:
                        break
                    output[name].extend(chunk)
        except KeyboardInterrupt:
            if execution.kind != "timed_out":
                execution = ExecutionOutcome("interrupted", "operator_interrupt")
    return CapturedProcess(execution, bytes(output["stdout"]), bytes(output["stderr"]),
                           process.returncode, time.monotonic() - started)


@dataclass(frozen=True)
class PreparedAgent:
    spec: ExperimentSpec
    argv: tuple[str, ...]
    event_format: str
    sources: dict[str, bytes]
    expected: bytes
    expected_stdout: bytes | None
    prompt: str
    toolkit: dict[str, str]
    results_dir: Path


def harness_source_identity() -> str:
    root = Path(__file__).resolve().parent
    return sha256_text(canonical_json({name: sha256_file(root / name) for name in (
        "harness.py", "manifest.py", "trial_records.py", "agg_util.py", "stats.py", "report.py", "command_policy.py",
        "prompt_comparison.py")}))


def prepare_agent(task: BenchTask, *, fixture_root: Path, expected_root: Path,
              command: Sequence[str], results_dir: Path, runner: str = "synthetic",
              timeout_seconds: float = 10, condition: ConditionPin | None = None,
              event_format: str = "text",
              retry_allowance: int = 1,
              claude: ClaudeRunner | None = None,
              guidance: ToolGuidance = ToolGuidance.FULL_HELP) -> PreparedAgent:
    """Validate and hash every behavior input without creating state or spawning."""
    guidance = ToolGuidance(guidance)
    if claude is not None:
        if not isinstance(claude, ClaudeRunner) or command or runner != "synthetic":
            raise ValueError("configured Claude requires no competing argv")
        claude.verify()
        argv, event_format = claude.command(), "claude_stream"
    else:
        argv = build_runner_command(command, runner=runner)
    if event_format not in ("text", "claude_stream"):
        raise RecordIntegrityError("unsupported synthetic event format")
    _validate_task(task)
    validate_policy(task.scorer, artifact=task.expected_artifact)
    if task.expected_stdout is not None and not isinstance(task.expected_stdout, str):
        raise ValueError("expected_stdout must be a string or null")
    expected_stdout = task.expected_stdout.encode("utf-8") if task.expected_stdout is not None else None
    if isinstance(timeout_seconds, bool) or not isinstance(timeout_seconds, (int, float)) or not math.isfinite(timeout_seconds) or timeout_seconds <= 0:
        raise ValueError("timeout must be a positive finite number")
    fixture_root, expected_root, results_dir = map(_root, (fixture_root, expected_root, results_dir))
    if any((_overlaps(fixture_root, expected_root), _overlaps(results_dir, fixture_root), _overlaps(results_dir, expected_root))):
        raise ValueError("fixture, expected and result roots must be disjoint")
    references = [*task.input_files, *(task.support_files or [])]
    relative_paths = [_relative_path(reference) for reference in references]
    if len(set(relative_paths)) != len(relative_paths) or any(
        left in right.parents for left in relative_paths for right in relative_paths if left != right
    ):
        raise ValueError("staging collision")
    sources = {reference: _read_source(fixture_root, reference) for reference in references}
    expected = _read_source(expected_root, task.expected_output)
    validate_expected(task.scorer, expected, artifact=task.expected_artifact, expected_stdout=expected_stdout)
    expected_stat = (expected_root / task.expected_output).stat()
    if any((fixture_root / reference).stat().st_ino == expected_stat.st_ino and
           (fixture_root / reference).stat().st_dev == expected_stat.st_dev for reference in references):
        raise ValueError("expected source aliases a worker input")
    toolkit = resolve_toolkit() if condition is not None or claude is not None else {}
    prompt = build_prompt(task, condition=condition, guidance=guidance)
    spec = ExperimentSpec(claude.backend if claude else "synthetic", sha256_text(canonical_json(asdict(task))),
        sha256_text(prompt), {name: hashlib.sha256(sources[name]).hexdigest() for name in task.input_files},
        hashlib.sha256(expected).hexdigest(), tuple(argv), sha256_file(argv[0]),
        sha256_file(Path(__file__).with_name("requirements.lock")),
        harness_source_identity(), sha256_file(Path(__file__).with_name("neutral_scorer.py")),
        condition.content_identity if condition is not None else None,
        {name: sha256_file(path) for name, path in toolkit.items()},
        support_sha256={name: hashlib.sha256(sources[name]).hexdigest() for name in task.support_files or []},
        expected_stdout_sha256=hashlib.sha256(expected_stdout).hexdigest() if expected_stdout is not None else None,
        answer_policy_sha256=sha256_text(canonical_json(answer_policy_snapshot(task.scorer, task.expected_artifact))),
        grader_version=GRADER_VERSION,
        requested_model=claude.model if claude else None,
        effort=claude.effort if claude else None,
        thinking_policy=claude.thinking_policy if claude else None,
        runner_version=CLAUDE_VERSION if claude else "synthetic-argv/1",
        runner_source_sha256=CLAUDE_SHA256 if claude else None,
        runner_pin_sha256=CLAUDE_SHA256 if claude else None,
        launcher_sha256=sha256_file(Path(__file__).with_name("claude_shell.py")) if claude else None,
        profile_sha256=sha256_text(native_profile(Path("/@attempt"), toolkit)) if claude else None,
        runner_configuration_sha256=sha256_text(canonical_json({"event_format": event_format,
            "environment": ("local-scripted-cleared/1" if claude.backend == "synthetic" else "native-parent-auth-child-cleared/1") if claude else "synthetic-cleared/1",
            "dyld_sha256": sha256_file(DYLD_PROFILE) if claude else None})),
        limits={"timeout_seconds": timeout_seconds, "max_turns": claude.max_turns if claude else 30}, retry_allowance=retry_allowance,
        platform_identity=platform.platform() if claude else "synthetic-portable",
        interpreter="python/" + ".".join(str(n) for n in sys.version_info[:3]),
        locators={"fixture_root": str(fixture_root), "expected_root": str(expected_root),
                  "runner": argv[0], **(claude.provenance_locators() if claude else {}),
                  **{f"toolkit/{name}": path for name, path in toolkit.items()}})
    return PreparedAgent(spec, tuple(argv), event_format, sources, expected, expected_stdout,
                         prompt, toolkit, results_dir)


def _native_shell_conforms(*, initial: Sequence[dict[str, object]], tool_calls: Sequence[ToolCall],
                           registrations: Sequence[dict[str, object]], profile_sha256: str) -> bool:
    """Check shell evidence after OwnedShells validated registration ownership.

    Receipt permission/model validation and task grading remain separate owners.
    Claude starts the shell lazily, so a no-tool answer need not have an env probe.
    """
    return (len(initial) == 1 and initial[0].get("tools") == ["Bash"] and
        initial[0].get("permissionMode") == "dontAsk" and
        all(event.tool_name == "Bash" for event in tool_calls) and
        all(raw["profile_sha256"] == profile_sha256 for raw in registrations) and
        sum(any("&& eval " in arg for arg in raw["argv"]) for raw in registrations) == len(tool_calls) and
        (not tool_calls or any(raw["argv"] == ["-c", "env"] for raw in registrations)))


@dataclass(frozen=True)
class CampaignAdmission:
    kind: Literal["launch", "finished", "hold"]
    key: AttemptKey | None = None
    reason: str | None = None

    def __post_init__(self) -> None:
        if (self.kind not in ("launch", "finished", "hold") or
            (self.kind == "launch") != isinstance(self.key, AttemptKey) or
            (self.kind != "launch" and self.key is not None) or
            (self.kind == "hold") != (isinstance(self.reason, str) and bool(self.reason)) or
            (self.kind != "hold" and self.reason is not None)):
            raise RecordIntegrityError("incoherent campaign admission")


def campaign_admission(spec: CampaignSpec, starts: Sequence[AttemptStart], results: Sequence[AttemptResult],
                       faults: Sequence[str], *, live_grant: LiveRunGrant | None = None,
                       operator_stop: bool = False) -> CampaignAdmission:
    """Derive a decision from fresh checked records; this is not launch authority."""
    from bench.report import trial_views
    from bench.agg_util import can_reserve_estimated_usd, measurement_coverage
    views = trial_views(tuple((spec.identity, *entry) for entry in spec.schedule), starts, results, campaign=spec)
    reasons = sorted(set(faults) | {fault for result in results if (fault := admission_fault(result))})
    if reasons:
        return CampaignAdmission("hold", reason=reasons[0])
    if any(view.disposition.kind == "running" for view in views):
        return CampaignAdmission("hold", reason="unfinished_attempt")
    if live_grant is not None and live_grant.phase == "canary" and any(result.grade.kind == "fail" for result in results):
        return CampaignAdmission("hold", reason="canary_failed")
    if any(view.disposition.kind not in ("succeeded", "task_failed", "pending") for view in views[:spec.prefix_length]):
        return CampaignAdmission("hold", reason="incomplete_contract_prefix")
    budget = measurement_coverage(starts, results)["budget"]
    if spec.grant_usd is not None and budget["unresolved_attempts"]:
        return CampaignAdmission("hold", reason="unknown_estimated_cost")
    pending = next((view for view in views if view.disposition.kind in ("pending", "retry_pending")), None)
    if pending is None:
        return CampaignAdmission("finished")
    if operator_stop:
        return CampaignAdmission("hold", reason="operator_stop")
    if live_grant is not None and (len(starts) >= live_grant.max_attempts or any(
            result.usage.estimated_usd is not None and result.usage.estimated_usd > live_grant.attempt_usd for result in results)):
        return CampaignAdmission("hold", reason="live_attempt_limit")
    if spec.grant_usd is not None:
        if not can_reserve_estimated_usd(budget["known_estimated_usd"], spec.reservation_usd, spec.grant_usd):
            return CampaignAdmission("hold", reason="grant_exhausted")
    return CampaignAdmission("launch", key=AttemptKey(*pending.trial, pending.disposition.next_ordinal))


def run_agent(task: BenchTask, *, fixture_root: Path, expected_root: Path,
              command: Sequence[str], results_dir: Path, runner: str = "synthetic",
              timeout_seconds: float = 10, condition: ConditionPin | None = None,
              event_format: str = "text", attempt_key: AttemptKey | None = None,
              retry_allowance: int = 1,
              claude: ClaudeRunner | None = None,
              prior_attempts: Sequence[tuple[AttemptStart, AttemptResult]] = (),
              campaign: CampaignSpec | None = None,
              _campaign_lock: BinaryIO | None = None,
              live_grant: LiveRunGrant | None = None,
              prerequisite_bundle: Path | None = None,
              guidance: ToolGuidance = ToolGuidance.FULL_HELP) -> AttemptResult:
    """Run checked synthetic argv, retaining final evidence and immutable records.

    Campaign keys are admitted only after their constituent content identity
    matches the independently prepared task. A campaign is never a live grant.
    """
    prepared = prepare_agent(task, fixture_root=fixture_root, expected_root=expected_root,
        command=command, results_dir=results_dir, runner=runner, timeout_seconds=timeout_seconds,
        condition=condition, event_format=event_format, retry_allowance=retry_allowance,
        claude=claude, guidance=guidance)
    spec, argv, sources, expected, expected_stdout, prompt, toolkit, results_dir = (
        prepared.spec, prepared.argv, prepared.sources, prepared.expected, prepared.expected_stdout,
        prepared.prompt, prepared.toolkit, prepared.results_dir)
    event_format = prepared.event_format
    if spec.backend == "claude_cli":
        if campaign is None or not isinstance(live_grant, LiveRunGrant):
            raise RecordIntegrityError("real Claude requires a campaign-locked explicit scoped live grant")
        live_grant.assert_scope(campaign)
        _assert_live_prerequisite(live_grant, prerequisite_bundle, campaign)
    key = attempt_key or AttemptKey(spec.identity, task.id, condition.condition.value if condition else "no-md", 0, 0)
    if key.task_id != task.id or key.condition != (condition.condition.value if condition else "no-md"):
        raise RecordIntegrityError("attempt key contradicts frozen experiment")
    if campaign is not None:
        if not isinstance(campaign, CampaignSpec):
            raise RecordIntegrityError("campaign requires checked specification")
        campaign.assert_member(key, spec)
        campaign_directory = results_dir.parent.parent
        if _campaign_lock is None or results_dir.parent.name != "attempts" or results_dir.name != campaign_attempt_name(key):
            raise RecordIntegrityError("campaign launch requires controller lock and canonical attempt path")
        lock_stat = os.fstat(_campaign_lock.fileno())
        saved_stat = (campaign_directory / ".campaign.lock").stat()
        if (lock_stat.st_dev, lock_stat.st_ino) != (saved_stat.st_dev, saved_stat.st_ino):
            raise RecordIntegrityError("foreign campaign lock")
        fcntl.flock(_campaign_lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        saved, all_starts, all_results, faults = load_campaign_bundles((campaign_directory,))
        if saved.identity != campaign.identity:
            raise RecordIntegrityError("campaign records forbid further admission")
        action = campaign_admission(campaign, all_starts, all_results, faults, live_grant=live_grant)
        if action.kind != "launch" or action.key != key:
            raise RecordIntegrityError("campaign admission forbids this launch: " + (action.reason or "frozen_order"))
        effective_retry = campaign.retry_allowance(key.trial)
    else:
        if key.experiment_id != spec.identity:
            raise RecordIntegrityError("attempt key contradicts frozen experiment")
        effective_retry = retry_allowance
    prior_starts = [start for start, _ in prior_attempts]
    prior_results = [result for _, result in prior_attempts]
    admission = derive_trial_disposition(key.trial, starts=prior_starts, results=prior_results, retry_allowance=effective_retry)
    if admission.kind not in ("pending", "retry_pending") or admission.next_ordinal != key.ordinal:
        raise RecordIntegrityError("trial disposition forbids this attempt launch")
    store = AttemptStore(results_dir)
    store.start(AttemptStart(key, spec.backend, reservation_usd=campaign.reservation_usd if campaign else None,
                            requested_model=claude.model if claude else None))
    _write_private(results_dir / "experiment.json", (canonical_json(record_dict(spec)) + "\n").encode())
    artifacts: dict[str, str] = {}
    execution, exit_code = ExecutionOutcome("completed"), None
    grade = Grade("not_run", "not_started")
    evidence_complete = True
    decoder = ClaudeStreamDecoder(backend=spec.backend, on_raw=store.append_event, requested_model=spec.requested_model) if event_format == "claude_stream" else None
    usage = Usage()
    # Native workspaces stay private with the attempt, including on cleanup
    # failure. Never remove a directory that may still have an owned writer.
    workspace_context = nullcontext(str(results_dir / "workspace")) if claude else tempfile.TemporaryDirectory(prefix="mdtools_offline_")
    with workspace_context as temporary:
        workspace = Path(temporary).resolve()
        if claude:
            workspace.mkdir(mode=0o700)
        worker = workspace / "fixtures"
        worker.mkdir(mode=0o700)
        scratch = workspace / "scratch"
        scratch.mkdir(mode=0o700)
        for reference, content in sources.items():
            _write_private(worker / reference, content)
        child_env = {"PATH": "/usr/bin:/bin", "TMPDIR": str(scratch),
                     "LANG": "C.UTF-8", "LC_ALL": "C", "PYTHONNOUSERSITE": "1"}
        boundary = prepare_native_boundary(workspace, condition, toolkit=toolkit) if claude else None
        if boundary is not None:
            boundary.verify()
            child_env = boundary.parent_environment(claude)
            for name in ("boundary.json", "profile.sb", "shell.json", "bash-eval-launcher"):
                reference = "workspace/control/" + name
                artifacts[reference] = sha256_file(results_dir / reference)
        elif condition is not None:
            stage_condition(condition, workspace / "bin", toolkit=toolkit)
            child_env["PATH"] = str(workspace / "bin")
        try:
            process = subprocess.Popen(argv, cwd=worker, env=child_env, stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        except OSError:
            stdout, stderr = b"", b""
            execution = ExecutionOutcome("infrastructure_error", "synthetic_spawn_failed")
            evidence_complete = False
        else:
            with ExitStack() as pipes:
                for pipe in (process.stdin, process.stdout, process.stderr):
                    pipes.enter_context(pipe)
                owned = None
                if boundary is not None:
                    state = process_state(process.pid)
                    if state is None:
                        _stop_owned_group(process)
                        process.wait(timeout=5)
                        raise ContainmentError("Claude exited before ownership registration")
                    owned = OwnedShells(boundary.registry, state.identity)
                captured = capture_process(process, prompt=prompt.encode(), timeout_seconds=timeout_seconds,
                    decoder=decoder, stop_owned=owned.stop if owned else None)
                execution, stdout, stderr, exit_code = captured.execution, captured.stdout, captured.stderr, captured.exit_code
                usage = decoder.parsed.usage if decoder is not None and decoder.parsed.usage.source != "unavailable" else Usage(
                    elapsed_seconds=captured.elapsed_seconds, source="controller", completeness="partial")
                usage = replace(usage, elapsed_seconds=captured.elapsed_seconds,
                    completeness="partial" if usage.completeness == "unknown" else usage.completeness,
                    source="controller" if usage.source == "unavailable" else usage.source)
                if execution.kind in ("timed_out", "interrupted") or (decoder is not None and (decoder.receipt is None or decoder.integrity_fault)):
                    evidence_complete = False
                if boundary is not None:
                    registrations = owned.registrations()
                    tool_calls = [event for event in decoder.parsed.events if isinstance(event, ToolCall)]
                    raw_events = [decode_record_json(line) for line in (results_dir / "events.jsonl").read_bytes().splitlines()]
                    initial = [event for event in raw_events if event.get("type") == "system" and event.get("subtype") == "init"]
                    conforming = _native_shell_conforms(initial=initial, tool_calls=tool_calls,
                        registrations=registrations, profile_sha256=boundary.hashes[str(workspace / "control/profile.sb")])
                    if execution.kind == "completed" and (not conforming or decoder.receipt.observed_model != claude.model):
                        execution = ExecutionOutcome("infrastructure_error", "configuration_error")
                    for path in boundary.registry.iterdir():
                        if path.is_file():
                            reference = str(path.relative_to(results_dir))
                            artifacts[reference] = sha256_file(path)
        for name, content in (("stdout.bin", stdout), ("stderr.bin", stderr)):
            _write_private(results_dir / "artifacts" / name, content)
            artifacts[f"artifacts/{name}"] = hashlib.sha256(content).hexdigest()
        actual = None
        if task.expected_artifact in ("file_contents", "stdout_and_file"):
            try:
                actual = _read_source(worker, task.input_files[0])
            except ValueError:
                evidence_complete = False
                if execution.kind == "completed":
                    grade = Grade("unavailable", "capture_unavailable")
            else:
                final_reference = "artifacts/final/" + task.input_files[0]
                _write_private(results_dir / final_reference, actual)
                artifacts[final_reference] = hashlib.sha256(actual).hexdigest()
        final_text = stdout if decoder is None else (decoder.receipt.final_text.encode("utf-8") if decoder.receipt is not None else b"")
        if task.expected_artifact != "file_contents":
            final_reference = "artifacts/final_submission.bin"
            _write_private(results_dir / final_reference, final_text)
            artifacts[final_reference] = hashlib.sha256(final_text).hexdigest()
        if execution.kind == "completed" and evidence_complete:
            try:
                grade = grade_submission(task.scorer, artifact=task.expected_artifact,
                    final_text=final_text, actual_file=actual, expected=expected, expected_stdout=expected_stdout)
                if not isinstance(grade, Grade) or grade.kind not in ("pass", "fail"):
                    raise RecordIntegrityError("grader did not return an authoritative grade")
            except (ValueError, RuntimeError):
                grade = Grade("unavailable", "grader_unavailable")
        elif execution.kind != "completed":
            grade = Grade("not_run", execution.reason)
        if grade.kind in ("pass", "fail"):
            observed = SubmissionFormat.observe(task.scorer, task.expected_artifact, final_text)
            reference = "artifacts/submission_format.json"
            payload = (canonical_json(asdict(observed)) + "\n").encode()
            _write_private(results_dir / reference, payload)
            artifacts[reference] = hashlib.sha256(payload).hexdigest()
    result = store.finalize(execution=execution, grade=grade, usage=usage, artifacts=artifacts,
        evidence_complete=evidence_complete, permission_fault=decoder.permission_fault if decoder else None,
        observed_model=decoder.receipt.observed_model if decoder and decoder.receipt else None, exit_code=exit_code)
    from bench.report import attempt_report
    summary = attempt_report([*prior_starts, store.load()[0]], [*prior_results, result], retry_allowance=effective_retry)
    _write_private(results_dir / "report.json", (canonical_json(summary) + "\n").encode())
    return result


def campaign_attempt_name(key: AttemptKey) -> str:
    return sha256_text(canonical_json(key.trial)) + "-" + str(key.ordinal)


def freeze_campaign(tasks: Sequence[BenchTask], *, fixture_root: Path, expected_root: Path,
                    command: Sequence[str], results_dir: Path,
                    conditions: Mapping[str, ConditionPin], repetitions: int = 5,
                    seed: int = 1729, core_study: bool = False,
                    event_format: str = "text", timeout_seconds: float = 10,
                    retry_allowance: int = 1, grant_usd: float | None = None,
                    reservation_usd: float | None = None,
                    claude: ClaudeRunner | None = None,
                    guidance: ToolGuidance = ToolGuidance.FULL_HELP) -> CampaignSpec:
    """Freeze trusted synthetic task content and exact condition/schedule inputs.

    No corpus discovery, provider call or launch authority is provided here.
    Every supplied task must already be authorized to read.
    """
    if set(conditions) != {condition.value for condition in CliCondition}:
        raise RecordIntegrityError("campaign requires all three explicit conditions")
    if any(not isinstance(pin, ConditionPin) or pin.condition.value != name for name, pin in conditions.items()):
        raise RecordIntegrityError("condition pin contradicts campaign name")
    task_ids = tuple(task.id for task in tasks)
    schedule = serial_schedule(task_ids, repetitions=repetitions, seed=seed, core_study=core_study)
    members = {}
    for task in tasks:
        for condition, pin in conditions.items():
            prepared = prepare_agent(task, fixture_root=fixture_root, expected_root=expected_root,
                command=command, results_dir=results_dir, condition=pin, event_format=event_format,
                timeout_seconds=timeout_seconds, retry_allowance=retry_allowance, claude=claude, guidance=guidance)
            members[CampaignSpec.member_name(task.id, condition)] = prepared.spec
    return CampaignSpec(members, schedule, 6 if core_study else 0, grant_usd, reservation_usd, core_study)


def _load_campaign(directory: Path) -> CampaignSpec:
    spec = CampaignSpec.from_dict(decode_record_json(_read_source(directory, "campaign.json")))
    schedule = decode_record_json(_read_source(directory, "schedule.json"))
    if schedule != {"experiment_id": spec.identity, "schedule": record_dict(spec.schedule)}:
        raise RecordIntegrityError("frozen schedule contradicts campaign")
    return spec


def load_campaign_bundles(bundles: Sequence[Path]) -> tuple[CampaignSpec, list[AttemptStart], list[AttemptResult], list[str]]:
    """Reload all attempt artifacts; duplicates across bundles are invalid."""
    if not bundles:
        raise RecordIntegrityError("report requires campaign bundles")
    spec = None
    starts, results, faults = [], [], []
    for bundle in bundles:
        directory = _root(bundle)
        incoming = _load_campaign(directory)
        if spec is None:
            spec = incoming
        elif spec.identity != incoming.identity:
            raise RecordIntegrityError("foreign experiment bundle")
        attempts = directory / "attempts"
        _assert_no_symlinks(attempts)
        if not attempts.is_dir():
            raise RecordIntegrityError("missing campaign attempt directory")
        for path in sorted(attempts.iterdir()):
            _assert_no_symlinks(path)
            if not path.is_dir():
                raise RecordIntegrityError("unexpected campaign attempt entry")
            store = AttemptStore(path)
            start, result = store.load()
            spec.assert_trial(start.key.trial)
            if path.name != campaign_attempt_name(start.key) or start.reservation_usd != spec.reservation_usd:
                raise RecordIntegrityError("attempt path/reservation contradicts campaign")
            member = spec.members[spec.member_name(start.key.task_id, start.key.condition)]
            if start.backend != member.backend or start.requested_model != member.requested_model:
                raise RecordIntegrityError("attempt start contradicts campaign member")
            # A start may precede experiment.json when the controller crashes.
            # A finished result must prove the member identity on disk too.
            if (path / "experiment.json").exists() or result is not None:
                local = ExperimentSpec.from_dict(decode_record_json(_read_source(path, "experiment.json")))
                spec.assert_member(start.key, local)
            starts.append(start)
            if result is not None:
                results.append(result)
            fault = store.persisted_fault()
            if fault:
                faults.append(fault)
    from bench.report import trial_views
    trial_views(tuple((spec.identity, *entry) for entry in spec.schedule), starts, results, campaign=spec)
    return spec, starts, results, faults


def _assert_live_prerequisite(grant: LiveRunGrant, bundle: Path | None, spec: CampaignSpec) -> None:
    """A later grant cannot convert failed canaries into task-launch authority."""
    assert_prerequisite(spec, phase=grant.phase, bundle=bundle,
        prerequisite_experiment_id=grant.prerequisite_experiment_id)


def assert_prerequisite(spec: CampaignSpec, *, phase: str, bundle: Path | None,
                        prerequisite_experiment_id: str | None) -> None:
    """Validate evidence without constructing or supplying launch authority."""
    if phase not in ("canary", "public_pilot", "core_study", "prompt_comparison"):
        raise RecordIntegrityError("unknown prerequisite phase")
    if phase == "canary":
        if bundle is not None:
            raise RecordIntegrityError("canary phase cannot inherit another experiment")
        return
    if bundle is None:
        raise RecordIntegrityError("live phase requires independently reloaded prerequisite evidence")
    prior, starts, results, faults = load_campaign_bundles((bundle,))
    if prior.identity != prerequisite_experiment_id or faults:
        raise RecordIntegrityError("live prerequisite identity/fault mismatch")
    from bench.report import trial_views
    views = trial_views(tuple((prior.identity, *entry) for entry in prior.schedule), starts, results, campaign=prior)
    prior_ids = {entry[0] for entry in prior.schedule}
    if phase in ("public_pilot", "prompt_comparison"):
        if len(prior_ids) != 1 or prior_ids & set(CORE_TASK_IDS) or len(views) != 3 or len(starts) != 3 or any(
                member.retry_allowance != 0 for member in prior.members.values()) or any(not view.workflow_success for view in views):
            raise RecordIntegrityError("public pilot requires three successful zero-retry canaries")
    elif prior_ids != {"T1", "T2", "T10"} or len(views) != 9:
        raise RecordIntegrityError("core study requires the separate nine-trial public pilot")
    if any(view.disposition.kind not in ("succeeded", "task_failed", "operational_failed") for view in views) or any(
            result.usage.estimated_usd is None for result in results) or len(results) != len(starts):
        raise RecordIntegrityError("live prerequisite completion/cost is unresolved")
    for view in views:
        selected = next(result for result in view.attempts if result.key == view.disposition.selected)
        if not selected.live_study_evidence:
            raise RecordIntegrityError("synthetic prerequisite cannot authorize real task launches")
        current = next(member for name, member in spec.members.items() if name.endswith("/" + view.trial[2]))
        previous = prior.members[prior.member_name(view.trial[1], view.trial[2])]
        for field in ("requested_model", "effort", "thinking_policy", "runner_version", "runner_source_sha256",
                      "runner_pin_sha256", "runner_configuration_sha256", "launcher_sha256", "profile_sha256",
                      "condition_sha256", "toolkit_sha256", "dependency_lock_sha256", "harness_sha256", "grader_sha256"):
            if getattr(previous, field) != getattr(current, field):
                raise RecordIntegrityError("live prerequisite runner/model/condition content mismatch")
    prior_grant, _ = load_authorization_evidence(_root(bundle), prior)
    if prior_grant.phase != ("canary" if phase in ("public_pilot", "prompt_comparison") else "public_pilot"):
        raise RecordIntegrityError("live prerequisite requires its admitted phase grant")
    from bench.agg_util import can_reserve_estimated_usd, estimated_usd_total
    if len(starts) > prior_grant.max_attempts or not can_reserve_estimated_usd(float(estimated_usd_total(results)), 0, prior_grant.campaign_usd) or any(
            result.usage.estimated_usd > prior_grant.attempt_usd for result in results):
        raise RecordIntegrityError("live prerequisite exceeded its admitted grant")


def load_authorization_evidence(directory: Path, spec: CampaignSpec) -> tuple[LiveRunGrant, Path | None]:
    """Audit only: this file never supplies a resumed launch's user authority."""
    evidence = decode_record_json(_read_source(directory, "authorization-evidence.json"))
    if type(evidence) is not dict or set(evidence) != {"grant", "grant_id", "prerequisite_bundle"}:
        raise RecordIntegrityError("invalid authorization evidence")
    grant = LiveRunGrant.from_dict(evidence["grant"])
    if evidence["grant_id"] != grant.identity:
        raise RecordIntegrityError("authorization evidence grant digest mismatch")
    grant.assert_scope(spec)
    locator = evidence["prerequisite_bundle"]
    if locator is not None and (type(locator) is not str or not Path(locator).is_absolute()):
        raise RecordIntegrityError("invalid prerequisite evidence locator")
    if (grant.phase == "canary") != (locator is None):
        raise RecordIntegrityError("authorization evidence prerequisite/phase mismatch")
    return grant, Path(locator) if locator is not None else None


def run_campaign(spec: CampaignSpec, tasks: Sequence[BenchTask], *, fixture_root: Path,
                 expected_root: Path, command: Sequence[str], conditions: Mapping[str, ConditionPin],
                 results_dir: Path, event_format: str = "text", timeout_seconds: float = 10,
                 stop_after: int | None = None,
                 claude: ClaudeRunner | None = None,
                 live_grant: LiveRunGrant | None = None,
                 prerequisite_bundle: Path | None = None,
                 guidance: ToolGuidance = ToolGuidance.FULL_HELP,
                 _executor: Callable[[AttemptKey, Sequence[tuple[AttemptStart, AttemptResult]], Path], AttemptResult] | None = None) -> dict[str, object]:
    """Serial campaign under one whole-run filesystem lock and explicit admission.

    All current member content is checked before any continuation. A stopped
    campaign only resumes pending keys, never an unfinished attempt. The private
    executor hook exercises synthetic faults in tests; its publication is always
    reloaded and validated rather than trusted as evidence.
    """
    if not isinstance(spec, CampaignSpec):
        raise RecordIntegrityError("campaign requires checked specification")
    if next(iter(spec.members.values())).backend == "claude_cli":
        if not isinstance(live_grant, LiveRunGrant) or claude is None or claude.backend != "claude_cli" or _executor is not None:
            raise RecordIntegrityError("real campaign requires configured Claude and an explicit scoped grant")
        live_grant.assert_scope(spec)
        if claude.attempt_usd != live_grant.attempt_usd:
            raise RecordIntegrityError("configured Claude attempt cap contradicts live grant")
        _assert_live_prerequisite(live_grant, prerequisite_bundle, spec)
    elif live_grant is not None or prerequisite_bundle is not None:
        raise RecordIntegrityError("synthetic campaigns cannot consume live grants")
    if stop_after is not None and (type(stop_after) is not int or stop_after < 0):
        raise RecordIntegrityError("invalid stop count")
    by_task = {task.id: task for task in tasks}
    if len(by_task) != len(tasks) or set(by_task) != {entry[0] for entry in spec.schedule}:
        raise RecordIntegrityError("campaign task inventory changed")
    if set(conditions) != {condition.value for condition in CliCondition}:
        raise RecordIntegrityError("campaign condition inventory changed")
    directory = _root(results_dir)
    if not directory.exists():
        for parent in (*reversed(directory.parent.parents), directory.parent):
            if not parent.exists():
                parent.mkdir(mode=0o700)
        directory.mkdir(mode=0o700)
        _fsync_directory(directory.parent)
    _assert_no_symlinks(directory / ".campaign.lock")
    descriptor = os.open(directory / ".campaign.lock", os.O_RDWR | os.O_CREAT, 0o600)
    with os.fdopen(descriptor, "r+b") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        if (directory / "campaign.json").exists():
            if _load_campaign(directory).identity != spec.identity:
                raise RecordIntegrityError("changed campaign content; new experiment required")
        else:
            if any(path.name != ".campaign.lock" for path in directory.iterdir()):
                raise RecordIntegrityError("incomplete campaign initialization")
            _write_private(directory / "campaign.json", (canonical_json(record_dict(spec)) + "\n").encode())
            _write_private(directory / "schedule.json", (canonical_json({"experiment_id": spec.identity,
                "schedule": record_dict(spec.schedule)}) + "\n").encode())
            (directory / "attempts").mkdir(mode=0o700)
            _fsync_directory(directory)
        if live_grant is not None:
            evidence = (canonical_json({"grant": record_dict(live_grant), "grant_id": live_grant.identity,
                "prerequisite_bundle": str(_root(prerequisite_bundle)) if prerequisite_bundle is not None else None}) + "\n").encode()
            evidence_path = directory / "authorization-evidence.json"
            if evidence_path.exists():
                if _read_source(directory, "authorization-evidence.json") != evidence:
                    raise RecordIntegrityError("changed live authorization evidence; explicit new run required")
            else:
                _write_private(evidence_path, evidence)
        # Verify the entire input inventory, including later task inputs, now.
        for task_id, condition in dict.fromkeys((entry[0], entry[1]) for entry in spec.schedule):
            entry = (task_id, condition, 0)
            member = spec.members[spec.member_name(task_id, condition)]
            pin = conditions[condition]
            if not isinstance(pin, ConditionPin) or pin.condition.value != condition:
                raise RecordIntegrityError("campaign condition pin changed")
            prepared = prepare_agent(by_task[task_id], fixture_root=fixture_root, expected_root=expected_root,
                command=command, results_dir=directory, condition=pin, event_format=event_format,
                timeout_seconds=timeout_seconds, retry_allowance=member.retry_allowance, claude=claude, guidance=guidance)
            spec.assert_member(AttemptKey(spec.identity, *entry, 0), prepared.spec)
        from bench.report import report_campaign
        launched = 0
        hold = None
        while True:
            _, starts, results, faults = load_campaign_bundles((directory,))
            action = campaign_admission(spec, starts, results, faults, live_grant=live_grant,
                operator_stop=stop_after is not None and launched >= stop_after)
            if action.kind != "launch":
                hold = action.reason
                break
            key = action.key
            path = directory / "attempts" / campaign_attempt_name(key)
            prior = [(start, result) for start in starts for result in results
                     if start.key == result.key and start.key.trial == key.trial]
            member = spec.members[spec.member_name(key.task_id, key.condition)]
            if _executor is None:
                run_agent(by_task[key.task_id], fixture_root=fixture_root, expected_root=expected_root,
                    command=command, results_dir=path, condition=conditions[key.condition],
                    event_format=event_format, timeout_seconds=timeout_seconds,
                    retry_allowance=member.retry_allowance, attempt_key=key, prior_attempts=prior, campaign=spec,
                    _campaign_lock=lock, claude=claude, live_grant=live_grant, prerequisite_bundle=prerequisite_bundle,
                    guidance=guidance)
            else:
                _executor(key, prior, path)
            launched += 1
        summary = report_campaign((directory,))
        if hold:
            summary["admission_hold"] = hold
            summary["exit_code"] = 1
        # Reports are derived snapshots, never record/schedule authority.
        candidate = directory / (".report-" + uuid.uuid4().hex + ".json")
        _write_private(candidate, (canonical_json(summary) + "\n").encode())
        _assert_no_symlinks(directory / "report.json")
        os.replace(candidate, directory / "report.json")
        _fsync_directory(directory)
        return summary


def offline_exercise(results_dir: Path) -> AttemptResult:
    """One actual subprocess mutation; no task registry, provider or user settings."""
    with tempfile.TemporaryDirectory(prefix="mdtools_synthetic_") as temporary:
        root = Path(temporary).resolve()
        fixtures, expected = root / "inputs", root / "expected"
        _write_private(fixtures / "nested/input.md", b"# Before\n")
        _write_private(expected / "answer.md", b"# After\n")
        task = BenchTask("synthetic-u1", "Change the heading to After.", ["nested/input.md"],
            "answer.md", "file_contents", "synthetic",
            StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False))
        script = "from pathlib import Path; Path('nested/input.md').write_bytes(b'# After\\n'); print('finished')"
        return run_agent(task, fixture_root=fixtures, expected_root=expected,
            command=[str(Path(sys.executable).resolve()), "-I", "-c", script], results_dir=results_dir)


def load_condition_pins(pin_root: Path) -> dict[str, ConditionPin]:
    """Load both explicit producer receipts; never fall back to installed md."""
    conditions = {}
    for name, condition in (("legacy", CliCondition.LEGACY), ("current", CliCondition.CURRENT_COMPACT)):
        payload = decode_record_json(_read_source(_root(pin_root), name + "/pin.json"))
        if type(payload) is not dict:
            raise RecordIntegrityError("malformed condition pin")
        payload["condition"] = CliCondition(payload["condition"])
        pin = ConditionPin(**payload)
        verify_condition(pin)
        if pin.condition != condition:
            raise RecordIntegrityError("wrong explicit condition pin")
        conditions[condition.value] = pin
    return conditions


def offline_campaign(results_dir: Path, pin_root: Path, *, stop_after: int | None = None) -> dict[str, object]:
    """Disposable synthetic tasks and explicit verified pins; no corpus access."""
    conditions = load_condition_pins(pin_root)
    with tempfile.TemporaryDirectory(prefix="mdtools_synthetic_campaign_") as temporary:
        root = Path(temporary).resolve()
        fixtures, expected = root / "fixtures", root / "expected"
        _write_private(fixtures / "input.md", b"before\n")
        _write_private(expected / "answer.md", b"after\n")
        policy = StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False)
        tasks = [BenchTask(name, "Synthetic: write after.", ["input.md"], "answer.md", "file_contents", "synthetic", policy)
                 for name in ("synthetic-a", "synthetic-b")]
        conditions["no-md"] = stage_condition(None, root / "stub", toolkit=resolve_toolkit())
        command = [str(Path(sys.executable).resolve()), "-I", "-c",
                   "from pathlib import Path; Path('input.md').write_bytes(b'after\\n'); print('synthetic finish')"]
        args = dict(fixture_root=fixtures, expected_root=expected, command=command,
                    conditions=conditions, results_dir=results_dir)
        spec = freeze_campaign(tasks, repetitions=2, **args)
        return run_campaign(spec, tasks, stop_after=stop_after, **args)


def _core_git(source: Path, *arguments: str) -> bytes:
    completed = subprocess.run(["git", *arguments], cwd=source, capture_output=True, timeout=30)
    if completed.returncode != 0:
        raise RecordIntegrityError("core source Git identity check failed")
    return completed.stdout


def _read_core_source(source: Path, consent: CorePreparationConsent, reference: str) -> bytes:
    relative = _relative_path(reference)
    if not any(Path(prefix) in relative.parents for prefix in CORPUS_PATHS):
        raise RecordIntegrityError("core reference outside consented corpus")
    # Git blobs, not the index or a timestamp, bind each read to the consent.
    oid = _core_git(source, "rev-parse", f"{consent.source_commit}:{reference}").decode().strip()
    content = _read_source(source, reference)
    blob = b"blob " + str(len(content)).encode() + b"\0" + content
    if hashlib.sha1(blob).hexdigest() != oid:
        raise RecordIntegrityError("core source bytes changed after consent")
    return content


def _core_tasks(config: CampaignConfig, consent: CorePreparationConsent | None) -> list[BenchTask]:
    if not isinstance(consent, CorePreparationConsent):
        raise RecordIntegrityError("core preparation requires explicit controller-read consent")
    consent.assert_scope(config)
    source, output = _root(Path(config.source_root)), _root(Path(config.output_root))
    if output == source or output in source.parents or any(_overlaps(output, source / path) for path in CORPUS_PATHS):
        raise RecordIntegrityError("core destination overlaps consented source")
    if _core_git(source, "rev-parse", "--show-toplevel").decode().strip() != str(source):
        raise RecordIntegrityError("core source must be its repository root")
    if _core_git(source, "rev-parse", f"{consent.source_commit}^{{commit}}").decode().strip() != consent.source_commit:
        raise RecordIntegrityError("core source identity must name a commit")
    for path, oid in consent.corpus_objects.items():
        if _core_git(source, "rev-parse", f"{consent.source_commit}:{path}").decode().strip() != oid:
            raise RecordIntegrityError("core corpus identity contradicts consent")
    _core_git(source, "diff", "--quiet", consent.source_commit, "--", *CORPUS_PATHS)
    context = "registry"
    try:
        rows = decode_record_json(_read_core_source(source, consent, "bench/tasks/tasks.json"))
        if type(rows) is not list:
            raise ValueError("registry is not an array")
        tasks = {}
        for row in rows:
            context = "registry"
            if type(row) is not dict or type(row.get("id")) is not str:
                raise ValueError("invalid task identity")
            if row["id"] not in CORE_TASK_IDS:
                continue
            context = row["id"] + ": task_shape"
            if row["id"] in tasks:
                raise ValueError("duplicate task")
            selected = dict(row)
            selected["scorer"] = StructuralDiffPolicy(**selected["scorer"])
            task = BenchTask(**selected)
            _validate_task(task)
            context = task.id + ": answer_policy"
            validate_policy(task.scorer, artifact=task.expected_artifact)
            tasks[task.id] = task
        context = "registry: task_inventory"
        if set(tasks) != set(CORE_TASK_IDS):
            raise ValueError("incomplete core corpus")
        expected_paths = {task.expected_output for task in tasks.values()}
        for task in tasks.values():
            context = task.id + ": worker_references"
            if any(reference in expected_paths for reference in [*task.input_files, *(task.support_files or [])]):
                raise ValueError("worker reference aliases expected answer")
    except (ValueError, TypeError, KeyError, RecordIntegrityError):
        # Task text and expected values must never escape via parser errors.
        raise RecordIntegrityError("invalid or unsupported core task contract (" + context + ")") from None
    return [tasks[task_id] for task_id in config.task_ids]


def prepare_campaign_config(config: CampaignConfig, *, preparation_consent: CorePreparationConsent | None = None) -> tuple[CampaignSpec, list[BenchTask], dict[str, ConditionPin], ClaudeRunner | None]:
    """Package authorized inputs and freeze a proposal; never launch a model."""
    core_tasks = _core_tasks(config, preparation_consent) if config.core_study else None
    if not config.core_study and preparation_consent is not None:
        raise RecordIntegrityError("core consent cannot change public preparation scope")
    root, source = _root(Path(config.output_root)), _root(Path(config.source_root))
    fixtures, expected = root / "inputs", root / "expected"
    if config.task_ids == ("cli-canary",):
        tasks = [BenchTask("cli-canary", "Replace input.md with exactly: ready followed by a newline.",
            ["input.md"], "answer.md", "file_contents", "synthetic",
            StructuralDiffPolicy("raw_bytes", False, False, False, False, False, False, False))]
        packaged = {fixtures / "input.md": b"before\n", expected / "answer.md": b"ready\n"}
    else:
        tasks = core_tasks if core_tasks is not None else [public_task(name) for name in config.task_ids]
        if any(_overlaps(root, source / reference) for task in tasks
               for reference in [*task.input_files, *(task.support_files or []), task.expected_output]):
            raise RecordIntegrityError("operator output overlaps a task source")
        def read(reference: str) -> bytes:
            return (_read_core_source(source, preparation_consent, reference) if core_tasks is not None
                    else _read_source(source, reference))
        packaged = {fixtures / reference: read(reference)
            for task in tasks for reference in [*task.input_files, *(task.support_files or [])]}
        packaged.update({expected / task.expected_output: read(task.expected_output) for task in tasks})
    if preparation_consent is not None:
        consent_path = root / "core-preparation-consent.json"
        content = (canonical_json(record_dict(preparation_consent)) + "\n").encode()
        if consent_path.exists():
            if _read_source(root, consent_path.name) != content:
                raise RecordIntegrityError("changed core preparation consent")
        else:
            _write_private(consent_path, content)
    for path, content in packaged.items():
        if path.exists():
            if _read_source(path.parent, path.name) != content:
                raise RecordIntegrityError("changed packaged task bytes; new output directory required")
        else:
            _write_private(path, content)
    conditions = load_condition_pins(Path(config.pin_root))
    stub = root / "stub"
    if not stub.exists():
        conditions["no-md"] = stage_condition(None, stub, toolkit=resolve_toolkit())
    else:
        digest = sha256_file(stub / "md")
        conditions["no-md"] = ConditionPin(CliCondition.NO_MD, str(stub / "md"), None, digest, digest, digest, None, None)
        verify_condition(conditions["no-md"])
    runner = None if config.claude_executable is None else ClaudeRunner(config.claude_executable,
        config.endpoint, config.model, config.effort, config.thinking_policy, config.max_turns, config.attempt_usd)
    spec = freeze_campaign(tasks, fixture_root=fixtures, expected_root=expected, command=config.command,
        results_dir=root / "campaign", conditions=conditions, repetitions=config.repetitions, seed=config.seed,
        core_study=config.core_study,
        timeout_seconds=config.timeout_seconds, retry_allowance=config.retry_allowance,
        grant_usd=config.campaign_usd if runner else None, reservation_usd=config.attempt_usd if runner else None,
        claude=runner, guidance=ToolGuidance(config.guidance))
    return spec, tasks, conditions, runner


def configured_campaign(config: CampaignConfig, *, prepare_only: bool = False,
                        live_grant: LiveRunGrant | None = None, prerequisite_bundle: Path | None = None,
                        preparation_consent: CorePreparationConsent | None = None,
                        stop_after: int | None = None) -> dict[str, object]:
    """One maintained operator route; saved proposals never supply permission."""
    if not prepare_only and config.claude_executable is not None and config.endpoint is None and live_grant is None:
        raise RecordIntegrityError("live run requires an explicitly supplied grant")
    if config.core_study and config.claude_executable is not None and config.endpoint is None:
        if not isinstance(preparation_consent, CorePreparationConsent):
            raise RecordIntegrityError("core preparation requires explicit controller-read consent")
        preparation_consent.assert_scope(config)
        if prerequisite_bundle is None:
            raise RecordIntegrityError("live core preparation requires its public pilot prerequisite")
        prior = _load_campaign(_root(prerequisite_bundle))
        assert_prerequisite(prior, phase="core_study", bundle=prerequisite_bundle,
            prerequisite_experiment_id=prior.identity)
        for member in prior.members.values():
            if ((member.requested_model, member.effort, member.thinking_policy) !=
                (config.model, config.effort, config.thinking_policy) or
                member.harness_sha256 != harness_source_identity() or
                member.grader_sha256 != sha256_file(Path(__file__).with_name("neutral_scorer.py"))):
                raise RecordIntegrityError("core preparation pilot model/source mismatch")
    spec, tasks, conditions, runner = prepare_campaign_config(config, preparation_consent=preparation_consent)
    if prepare_only and prerequisite_bundle is not None:
        prior = _load_campaign(_root(prerequisite_bundle))
        assert_prerequisite(spec, phase="core_study" if config.core_study else "canary" if config.task_ids == ("cli-canary",) else "public_pilot",
            bundle=prerequisite_bundle, prerequisite_experiment_id=prior.identity)
    root = _root(Path(config.output_root))
    proposal_path = root / "proposal.json"
    proposal = {"authorization": "not granted", "identity": spec.identity, "spec": record_dict(spec)}
    if proposal_path.exists():
        saved = decode_record_json(_read_source(root, "proposal.json"))
        if type(saved) is not dict or set(saved) != set(proposal) or saved["authorization"] != "not granted" or saved["identity"] != spec.identity or CampaignSpec.from_dict(saved["spec"]).identity != spec.identity:
            raise RecordIntegrityError("configured campaign differs from frozen proposal")
    elif prepare_only:
        _write_private(proposal_path, (canonical_json(proposal) + "\n").encode())
    else:
        raise RecordIntegrityError("prepare a frozen proposal before running")
    if prepare_only:
        return proposal
    if runner is not None and runner.backend == "claude_cli":
        if not isinstance(live_grant, LiveRunGrant):
            raise RecordIntegrityError("explicit scoped live grant required")
        live_grant.assert_scope(spec)
        _assert_live_prerequisite(live_grant, prerequisite_bundle, spec)
        auth = subprocess.run([runner.executable, "auth", "status", "--json"],
            capture_output=True, check=True, timeout=15)
        state = decode_record_json(auth.stdout)
        if type(state) is not dict or not (state.get("loggedIn") is True and state.get("authMethod") == "claude.ai" and state.get("subscriptionType") == "max"):
            raise RecordIntegrityError("configured live route requires Max subscription authentication")
    return run_campaign(spec, tasks, fixture_root=root / "inputs", expected_root=root / "expected",
        command=config.command, conditions=conditions, results_dir=root / "campaign", claude=runner,
        timeout_seconds=config.timeout_seconds, guidance=ToolGuidance(config.guidance),
        live_grant=live_grant, prerequisite_bundle=prerequisite_bundle, stop_after=stop_after)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Offline synthetic CLI; the real API requires an explicit scoped grant.")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--offline-exercise", action="store_true", help="Explicitly select the default synthetic exercise")
    mode.add_argument("--offline-campaign", action="store_true", help="Serial synthetic comparison with explicit verified pins")
    mode.add_argument("--report-bundle", action="append", type=Path, help="Regenerate an offline campaign report without launching")
    mode.add_argument("--prepare-config", type=Path, help="Freeze a campaign proposal without provider calls")
    mode.add_argument("--run-config", type=Path, help="Run/resume a prepared campaign; live runs require --grant")
    mode.add_argument("--prepare-comparison", type=Path, help="Freeze the 18-trial public prompt comparison")
    mode.add_argument("--run-comparison", type=Path, help="Run/resume the frozen paired comparison")
    mode.add_argument("--report-comparison", type=Path, help="Regenerate a paired report without launching")
    parser.add_argument("--grant", type=Path, help="Explicit scoped launch consent, never loaded from a result bundle")
    parser.add_argument("--preparation-consent", type=Path, help="Explicit controller-read permission for the complete core corpus; never launch permission")
    parser.add_argument("--prerequisite-bundle", type=Path, help="Independently checked earlier campaign")
    parser.add_argument("--pin-root", type=Path, help="Explicit legacy/current pin receipts; no installed md fallback")
    parser.add_argument("--stop-after", type=int, help="Stop synthetic campaign after this many new trials; resume with identical inputs")
    parser.add_argument("--results-dir", type=Path, help="New controller-owned result directory")
    args = parser.parse_args(argv)
    if args.prepare_comparison or args.run_comparison or args.report_comparison:
        from bench.prompt_comparison import prepare_comparison, run_comparison, report_comparison
        if args.preparation_consent is not None or args.results_dir is not None or args.pin_root is not None or ((args.prepare_comparison or args.report_comparison) and
                (args.grant is not None or args.prerequisite_bundle is not None or args.stop_after is not None)):
            parser.error("comparison paths come from config; only run accepts grants, prerequisite or stop count")
        try:
            if args.report_comparison:
                summary = report_comparison(args.report_comparison)
            else:
                path = args.prepare_comparison or args.run_comparison
                config = CampaignConfig.from_dict(decode_record_json(_read_source(_root(path.parent), path.name)))
                if args.prepare_comparison:
                    spec = prepare_comparison(config)
                    summary = {"authorization": "not granted", "comparison_id": spec.identity, "trials": len(spec.schedule)}
                else:
                    grants = None
                    if args.grant:
                        raw = decode_record_json(_read_source(_root(args.grant.parent), args.grant.name))
                        if type(raw) is not dict or set(raw) != {"full_help", "discovery"}:
                            raise RecordIntegrityError("expected explicit grants keyed by both profiles")
                        grants = {name: LiveRunGrant.from_dict(record) for name, record in raw.items()}
                    summary = run_comparison(config, grants=grants,
                        prerequisite_bundle=args.prerequisite_bundle, stop_after=args.stop_after)
        except (RecordIntegrityError, OSError, ValueError, TypeError, KeyError) as exc:
            print(canonical_json({"complete": False, "exit_code": 2, "fault": str(exc)}))
            return 2
        print(canonical_json(summary))
        return summary.get("exit_code", 0)
    if args.prepare_config or args.run_config:
        if args.results_dir is not None or args.pin_root is not None or (args.prepare_config and (args.grant or args.stop_after is not None)):
            parser.error("configured modes use only declared paths; prepare cannot consume a grant or stop count")
        try:
            path = args.prepare_config or args.run_config
            config = CampaignConfig.from_dict(decode_record_json(_read_source(_root(path.parent), path.name)))
            grant = None if args.grant is None else LiveRunGrant.from_dict(decode_record_json(
                _read_source(_root(args.grant.parent), args.grant.name)))
            consent = None if args.preparation_consent is None else CorePreparationConsent.from_dict(decode_record_json(
                _read_source(_root(args.preparation_consent.parent), args.preparation_consent.name)))
            summary = configured_campaign(config, prepare_only=args.prepare_config is not None,
                live_grant=grant, prerequisite_bundle=args.prerequisite_bundle, stop_after=args.stop_after,
                preparation_consent=consent)
        except (RecordIntegrityError, OSError, ValueError, TypeError, KeyError) as exc:
            print(canonical_json({"complete": False, "exit_code": 2, "fault": str(exc)}))
            return 2
        print(canonical_json(summary))
        return summary.get("exit_code", 0)
    if args.grant is not None or args.prerequisite_bundle is not None or args.preparation_consent is not None:
        parser.error("grant and prerequisite require a configured campaign")
    if args.report_bundle:
        from bench.report import main as report_main
        return report_main([str(path) for path in args.report_bundle])
    if args.results_dir is None:
        parent = Path(tempfile.mkdtemp(prefix="mdtools_offline_receipt_")).resolve()
        results_dir = parent / "result"
    else:
        results_dir = args.results_dir
    if args.offline_campaign:
        if args.pin_root is None:
            parser.error("--offline-campaign requires --pin-root")
        try:
            summary = offline_campaign(results_dir, args.pin_root, stop_after=args.stop_after)
        except (RecordIntegrityError, OSError, ValueError) as exc:
            print(canonical_json({"complete": False, "exit_code": 2, "comparisons": [], "fault": str(exc)}))
            return 2
        print(canonical_json(summary))
        return summary["exit_code"]
    if args.pin_root is not None or args.stop_after is not None:
        parser.error("--pin-root and --stop-after require --offline-campaign")
    try:
        result = offline_exercise(results_dir)
    except (RecordIntegrityError, OSError, ValueError) as exc:
        print(canonical_json({"complete": False, "exit_code": 2, "comparisons": [], "fault": str(exc),
            "help": "Use a new --results-dir for an offline exercise; existing evidence is not overwritten."}))
        return 2
    print(canonical_json({"backend": result.backend, "execution": record_dict(result.execution),
        "grade": record_dict(result.grade), "results_dir": str(results_dir)}))
    # A measured task failure is a valid evaluation result, not a controller error.
    return 0 if result.execution.kind == "completed" and result.grade.kind in ("pass", "fail") else 1


if __name__ == "__main__":
    raise SystemExit(main())
