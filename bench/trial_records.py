"""Closed CLI evaluation records and pure trial selection.

Raw provider messages and filesystem publication belong to harness.py. These
records describe one attempt; historical benchmark booleans are not accepted.
"""

from __future__ import annotations

from dataclasses import dataclass, fields, is_dataclass
import json
import math
from types import MappingProxyType
from typing import Mapping, Sequence, Union


SCHEMA = "mdtools.cli-eval/1"
CONDITIONS = ("no-md", "legacy", "current-compact")
BACKENDS = ("synthetic", "claude_cli")
INFRASTRUCTURE_REASONS = (
    "transient_transport", "transient_startup", "authentication_error",
    "configuration_error", "permission_denied", "receipt_integrity",
    "trace_integrity", "synthetic_process_exit", "synthetic_spawn_failed",
    "model_mismatch", "model_identity_unavailable",
)
RETRYABLE_REASONS = ("transient_transport", "transient_startup")
ADMISSION_FAULTS = ("permission_denied", "receipt_integrity", "trace_integrity",
                    "authentication_error", "configuration_error", "model_mismatch", "model_identity_unavailable")


class RecordIntegrityError(ValueError):
    """A required record/evidence contract cannot be established."""


def require_text(value: object, name: str, *, nonempty: bool = True) -> None:
    if type(value) is not str or (nonempty and not value):
        raise RecordIntegrityError(f"{name}: expected {'nonempty ' if nonempty else ''}string")


def require_index(value: object, name: str) -> None:
    if type(value) is not int or value < 0:
        raise RecordIntegrityError(f"{name}: expected nonnegative integer")


def require_quantity(value: object, name: str, *, integral: bool = False) -> None:
    if value is None:
        return
    if type(value) not in ((int,) if integral else (int, float)) or value < 0:
        raise RecordIntegrityError(f"{name}: expected nonnegative quantity or null")
    try:
        finite = math.isfinite(value)
    except OverflowError:
        finite = False
    if not finite:
        raise RecordIntegrityError(f"{name}: nonfinite quantity")


def require_digest(value: object, name: str) -> None:
    if type(value) is not str or len(value) != 64 or any(c not in "0123456789abcdef" for c in value):
        raise RecordIntegrityError(f"{name}: expected SHA256")


def require_artifact_path(value: object) -> None:
    require_text(value, "artifact path")
    if value.startswith("/") or "\\" in value or "\0" in value or any(p in ("", ".", "..") for p in value.split("/")):
        raise RecordIntegrityError("noncanonical artifact path")


def _closed(raw: object, names: Sequence[str]) -> dict[str, object]:
    if type(raw) is not dict or set(raw) != set(names):
        raise RecordIntegrityError("missing or unknown record fields")
    return raw


def _schema(raw: dict[str, object], record: str) -> None:
    if raw.get("schema") != SCHEMA or raw.get("record") != record:
        raise RecordIntegrityError("unknown record schema or variant")


def record_dict(value: object) -> object:
    """Serialize validated immutable records without exposing mutable internals."""
    if is_dataclass(value):
        return {field.name: record_dict(getattr(value, field.name)) for field in fields(value)}
    if isinstance(value, Mapping):
        return {key: record_dict(item) for key, item in value.items()}
    if isinstance(value, (tuple, list)):
        return [record_dict(item) for item in value]
    return value


def decode_record_json(payload: bytes | str) -> object:
    if type(payload) not in (bytes, str):
        raise RecordIntegrityError("record JSON must be bytes or string")
    def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
        decoded: dict[str, object] = {}
        for key, value in items:
            if key in decoded:
                raise RecordIntegrityError("duplicate JSON record key")
            decoded[key] = value
        return decoded
    def nonfinite(_: str) -> object:
        raise RecordIntegrityError("nonfinite JSON record number")
    def finite_float(text: str) -> float:
        quantity = float(text)
        if not math.isfinite(quantity):
            raise RecordIntegrityError("nonfinite JSON record number")
        return quantity
    try:
        return json.loads(payload, object_pairs_hook=pairs, parse_constant=nonfinite, parse_float=finite_float)
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise RecordIntegrityError("malformed JSON record") from exc


@dataclass(frozen=True)
class AttemptKey:
    experiment_id: str
    task_id: str
    condition: str
    repetition: int
    ordinal: int

    def __post_init__(self) -> None:
        require_digest(self.experiment_id, "experiment identity")
        require_text(self.task_id, "task id")
        if self.condition not in CONDITIONS:
            raise RecordIntegrityError("unknown condition")
        require_index(self.repetition, "repetition")
        require_index(self.ordinal, "attempt ordinal")

    @property
    def trial(self) -> tuple[str, str, str, int]:
        return self.experiment_id, self.task_id, self.condition, self.repetition

    @classmethod
    def from_dict(cls, raw: object) -> AttemptKey:
        return cls(**_closed(raw, ("experiment_id", "task_id", "condition", "repetition", "ordinal")))


@dataclass(frozen=True)
class ExecutionOutcome:
    kind: str
    reason: str | None = None

    def __post_init__(self) -> None:
        allowed = {"completed": (None,), "infrastructure_error": INFRASTRUCTURE_REASONS,
                   "timed_out": ("deadline",), "budget_exhausted": ("turn_limit", "cost_limit"),
                   "interrupted": ("operator_interrupt",)}
        if type(self.kind) is not str or self.kind not in allowed or self.reason not in allowed[self.kind]:
            raise RecordIntegrityError("unknown execution outcome/reason")

    @classmethod
    def from_dict(cls, raw: object) -> ExecutionOutcome:
        return cls(**_closed(raw, ("kind", "reason")))


@dataclass(frozen=True)
class Grade:
    kind: str
    reason: str

    def __post_init__(self) -> None:
        if self.kind not in ("pass", "fail", "unavailable", "not_run"):
            raise RecordIntegrityError("unknown grade kind")
        require_text(self.reason, "grade reason")

    @classmethod
    def from_dict(cls, raw: object) -> Grade:
        return cls(**_closed(raw, ("kind", "reason")))


@dataclass(frozen=True)
class Usage:
    input_tokens: int | None = None
    output_tokens: int | None = None
    cache_read_tokens: int | None = None
    cache_creation_tokens: int | None = None
    estimated_usd: float | None = None
    elapsed_seconds: float | None = None
    tool_output_bytes: int | None = None
    source: str = "unavailable"
    completeness: str = "unknown"

    def __post_init__(self) -> None:
        for name in ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens", "tool_output_bytes"):
            require_quantity(getattr(self, name), name, integral=True)
        for name in ("estimated_usd", "elapsed_seconds"):
            require_quantity(getattr(self, name), name)
        if self.source not in ("unavailable", "controller", "synthetic_receipt", "claude_terminal") or self.completeness not in ("unknown", "partial", "complete"):
            raise RecordIntegrityError("unknown usage source/completeness")
        supplied = [getattr(self, name) is not None for name in ("input_tokens", "output_tokens", "cache_read_tokens", "cache_creation_tokens", "estimated_usd", "elapsed_seconds", "tool_output_bytes")]
        if (self.completeness == "complete" and not all(supplied)) or (self.completeness == "unknown" and any(supplied)) or (self.source == "unavailable" and any(supplied)):
            raise RecordIntegrityError("usage completeness contradicts supplied measurements")

    @classmethod
    def from_dict(cls, raw: object) -> Usage:
        return cls(**_closed(raw, tuple(field.name for field in fields(cls))))


@dataclass(frozen=True)
class PermissionDenial:
    tool_name: str
    tool_use_id: str
    tool_input_json: str | None = None

    def __post_init__(self) -> None:
        require_text(self.tool_name, "denied tool name")
        require_text(self.tool_use_id, "denied tool-use id")
        if self.tool_input_json is not None:
            require_text(self.tool_input_json, "denied input")
            if type(decode_record_json(self.tool_input_json)) is not dict:
                raise RecordIntegrityError("denied input must be an object")

    @classmethod
    def from_dict(cls, raw: object) -> PermissionDenial:
        return cls(**_closed(raw, ("tool_name", "tool_use_id", "tool_input_json")))


def permission_fault(raw: object, *, terminal: bool = False) -> str | None:
    """One classifier for required terminal metadata and early control faults.

This deliberately never examines prose, tool errors, exit codes or raw success.
Provider extras stay raw; the fields needed to prove denial must be valid.
"""
    if type(raw) is not dict:
        raise RecordIntegrityError("provider event must be an object")
    if terminal:
        denials = raw.get("permission_denials")
        if type(denials) is not list:
            raise RecordIntegrityError("required permission_denials must be an array")
        for denial in denials:
            if type(denial) is not dict or type(denial.get("tool_input")) is not dict:
                raise RecordIntegrityError("malformed terminal permission denial")
            require_text(denial.get("tool_name"), "denied tool name")
            require_text(denial.get("tool_use_id"), "denied tool-use id")
        return "permission_denied" if denials else None
    if raw.get("type") == "system" and raw.get("subtype") == "permission_denied":
        require_text(raw.get("tool_name"), "denied tool name")
        require_text(raw.get("tool_use_id"), "denied tool-use id")
        return "permission_denied"
    return None


@dataclass(frozen=True)
class ToolCall:
    tool_use_id: str
    tool_name: str
    input_json: str
    kind: str = "tool_call"

    def __post_init__(self) -> None:
        if self.kind != "tool_call":
            raise RecordIntegrityError("unknown tool call variant")
        require_text(self.tool_use_id, "tool-use id")
        require_text(self.tool_name, "tool name")
        require_text(self.input_json, "tool input JSON")
        if type(decode_record_json(self.input_json)) is not dict:
            raise RecordIntegrityError("tool input must be an object")


@dataclass(frozen=True)
class ToolResult:
    tool_use_id: str
    content: str
    is_error: bool
    kind: str = "tool_result"

    def __post_init__(self) -> None:
        if self.kind != "tool_result":
            raise RecordIntegrityError("unknown tool result variant")
        require_text(self.tool_use_id, "result tool-use id")
        require_text(self.content, "tool result content", nonempty=False)
        if type(self.is_error) is not bool:
            raise RecordIntegrityError("tool error must be boolean")


@dataclass(frozen=True)
class RunReceipt:
    execution: ExecutionOutcome
    usage: Usage
    final_text: str
    observed_model: str | None
    permission_denials: tuple[PermissionDenial, ...]
    raw_is_error: bool
    raw_subtype: str
    raw_terminal_reason: str
    kind: str = "run_receipt"

    def __post_init__(self) -> None:
        if self.kind != "run_receipt" or not isinstance(self.execution, ExecutionOutcome) or not isinstance(self.usage, Usage):
            raise RecordIntegrityError("invalid run receipt")
        require_text(self.final_text, "terminal text", nonempty=False)
        if self.observed_model is not None:
            require_text(self.observed_model, "observed model")
        if type(self.permission_denials) is not tuple or any(not isinstance(d, PermissionDenial) for d in self.permission_denials):
            raise RecordIntegrityError("invalid normalized permission denials")
        if any(d.tool_input_json is None for d in self.permission_denials):
            raise RecordIntegrityError("terminal denial requires tool input")
        if type(self.raw_is_error) is not bool:
            raise RecordIntegrityError("raw receipt error must be boolean")
        require_text(self.raw_subtype, "raw receipt subtype")
        require_text(self.raw_terminal_reason, "raw terminal reason")
        if self.permission_denials and self.execution != ExecutionOutcome("infrastructure_error", "permission_denied"):
            raise RecordIntegrityError("receipt denial cannot establish completion")


ClaudeEvent = Union[ToolCall, ToolResult, RunReceipt]


def event_from_dict(raw: object) -> ClaudeEvent:
    if type(raw) is not dict:
        raise RecordIntegrityError("invalid normalized event")
    kind = raw.get("kind")
    if kind == "tool_call":
        return ToolCall(**_closed(raw, tuple(f.name for f in fields(ToolCall))))
    if kind == "tool_result":
        return ToolResult(**_closed(raw, tuple(f.name for f in fields(ToolResult))))
    if kind == "run_receipt":
        payload = dict(_closed(raw, tuple(f.name for f in fields(RunReceipt))))
        payload["execution"] = ExecutionOutcome.from_dict(payload["execution"])
        payload["usage"] = Usage.from_dict(payload["usage"])
        if type(payload["permission_denials"]) is not list:
            raise RecordIntegrityError("invalid normalized denial array")
        payload["permission_denials"] = tuple(PermissionDenial.from_dict(d) for d in payload["permission_denials"])
        return RunReceipt(**payload)
    raise RecordIntegrityError("unknown normalized event kind")


@dataclass(frozen=True)
class AttemptStart:
    key: AttemptKey
    backend: str
    reservation_usd: float | None = None
    requested_model: str | None = None
    schema: str = SCHEMA
    record: str = "attempt_start"

    def __post_init__(self) -> None:
        if self.schema != SCHEMA or self.record != "attempt_start" or not isinstance(self.key, AttemptKey) or self.backend not in BACKENDS:
            raise RecordIntegrityError("invalid attempt start")
        require_quantity(self.reservation_usd, "reservation USD")
        if self.requested_model is not None:
            require_text(self.requested_model, "requested model")
        if self.backend == "claude_cli" and (self.reservation_usd is None or self.requested_model is None):
            raise RecordIntegrityError("live start requires model and reservation")

    @classmethod
    def from_dict(cls, raw: object) -> AttemptStart:
        payload = dict(_closed(raw, tuple(f.name for f in fields(cls))))
        _schema(payload, "attempt_start")
        payload["key"] = AttemptKey.from_dict(payload["key"])
        return cls(**payload)


@dataclass(frozen=True)
class AttemptResult:
    key: AttemptKey
    backend: str
    execution: ExecutionOutcome
    grade: Grade
    usage: Usage
    artifacts: Mapping[str, str]
    evidence_complete: bool
    artifact_manifest_sha256: str
    permission_fault: str | None = None
    requested_model: str | None = None
    observed_model: str | None = None
    exit_code: int | None = None
    schema: str = SCHEMA
    record: str = "attempt_result"

    def __post_init__(self) -> None:
        if self.schema != SCHEMA or self.record != "attempt_result" or not isinstance(self.key, AttemptKey) or self.backend not in BACKENDS:
            raise RecordIntegrityError("invalid attempt result schema/key/backend")
        if not isinstance(self.execution, ExecutionOutcome) or not isinstance(self.grade, Grade) or not isinstance(self.usage, Usage):
            raise RecordIntegrityError("attempt requires validated execution/grade/usage")
        if type(self.evidence_complete) is not bool or self.permission_fault not in (None, "permission_denied"):
            raise RecordIntegrityError("invalid evidence completeness or permission fault")
        require_digest(self.artifact_manifest_sha256, "artifact manifest digest")
        if not isinstance(self.artifacts, Mapping):
            raise RecordIntegrityError("artifacts must be a digest mapping")
        frozen = dict(self.artifacts)
        for path, digest in frozen.items():
            require_artifact_path(path)
            require_digest(digest, "artifact digest")
        object.__setattr__(self, "artifacts", MappingProxyType(frozen))
        for model in (self.requested_model, self.observed_model):
            if model is not None:
                require_text(model, "model identity")
        if self.exit_code is not None and type(self.exit_code) is not int:
            raise RecordIntegrityError("exit code must be integer or null")
        completed = self.execution.kind == "completed"
        if (completed and self.grade.kind == "not_run") or (not completed and self.grade.kind != "not_run"):
            raise RecordIntegrityError("grade contradicts execution")
        if self.grade.kind in ("pass", "fail") and (not self.evidence_complete or self.permission_fault):
            raise RecordIntegrityError("semantic grade requires complete conforming evidence")
        if self.grade.kind in ("pass", "fail") and (
            not {"artifacts/stdout.bin", "artifacts/stderr.bin", "events.jsonl"}.issubset(frozen) or
            not any(path == "artifacts/final_submission.bin" or path.startswith("artifacts/final/") for path in frozen)):
            raise RecordIntegrityError("semantic grade requires captured streams and final evidence")
        if self.permission_fault and self.grade.kind != "not_run":
            raise RecordIntegrityError("permission fault forbids grading")
        if self.execution.reason == "permission_denied" and self.permission_fault != "permission_denied":
            raise RecordIntegrityError("permission outcome requires durable fault")
        if self.backend == "synthetic" and self.usage.source == "claude_terminal":
            raise RecordIntegrityError("synthetic usage cannot be live evidence")
        if self.grade.kind in ("pass", "fail") and model_identity_fault(
            backend=self.backend, execution=self.execution, requested_model=self.requested_model,
            observed_model=self.observed_model):
            raise RecordIntegrityError("live grade requires matching actual model identity")

    @property
    def live_study_evidence(self) -> bool:
        return (self.backend == "claude_cli" and self.evidence_complete and
                self.execution.kind == "completed" and self.grade.kind in ("pass", "fail") and
                self.permission_fault is None and self.requested_model is not None and
                self.observed_model == self.requested_model and self.usage.source == "claude_terminal")

    @classmethod
    def from_dict(cls, raw: object) -> AttemptResult:
        payload = dict(_closed(raw, tuple(f.name for f in fields(cls))))
        _schema(payload, "attempt_result")
        payload["key"] = AttemptKey.from_dict(payload["key"])
        payload["execution"] = ExecutionOutcome.from_dict(payload["execution"])
        payload["grade"] = Grade.from_dict(payload["grade"])
        payload["usage"] = Usage.from_dict(payload["usage"])
        return cls(**payload)


@dataclass(frozen=True)
class TrialDisposition:
    kind: str
    selected: AttemptKey | None
    next_ordinal: int | None
    reason: str | None

    def __post_init__(self) -> None:
        if self.kind not in ("pending", "running", "retry_pending", "succeeded", "task_failed", "operational_failed", "ungradeable", "skipped", "invalid"):
            raise RecordIntegrityError("unknown trial disposition")
        if self.selected is not None and not isinstance(self.selected, AttemptKey):
            raise RecordIntegrityError("invalid selected attempt")
        if self.next_ordinal is not None:
            require_index(self.next_ordinal, "next ordinal")
        if self.reason is not None:
            require_text(self.reason, "disposition reason")
        if self.kind in ("pending", "retry_pending") and self.next_ordinal is None:
            raise RecordIntegrityError("pending disposition requires next ordinal")
        if self.kind not in ("pending", "retry_pending") and self.next_ordinal is not None:
            raise RecordIntegrityError("terminal/running disposition cannot admit an ordinal")
        if self.kind in ("succeeded", "task_failed", "operational_failed", "ungradeable", "running") and self.selected is None:
            raise RecordIntegrityError("selected disposition requires attempt")

    @property
    def workflow_success(self) -> bool:
        return self.kind == "succeeded"

    @classmethod
    def from_dict(cls, raw: object) -> TrialDisposition:
        payload = dict(_closed(raw, tuple(f.name for f in fields(cls))))
        if payload["selected"] is not None:
            payload["selected"] = AttemptKey.from_dict(payload["selected"])
        return cls(**payload)


def model_identity_fault(*, backend: str, execution: ExecutionOutcome,
                         requested_model: str | None, observed_model: str | None) -> str | None:
    """Missing model after completion is a fault; pre-model provider errors are not.

    Actual identifiers compare exactly to the frozen request. No alias, family
    or case conversion is inferred from an unknown provider response.
    """
    if backend != "claude_cli":
        return None
    if observed_model is None:
        return "model_identity_unavailable" if execution.kind == "completed" else None
    if requested_model is None or observed_model != requested_model:
        return "model_mismatch"
    return None


def admission_fault(result: AttemptResult) -> str | None:
    if not isinstance(result, AttemptResult):
        raise RecordIntegrityError("admission requires a validated attempt")
    if result.permission_fault:
        return result.permission_fault
    if result.execution.reason in ADMISSION_FAULTS:
        return result.execution.reason
    identity_fault = model_identity_fault(backend=result.backend, execution=result.execution,
        requested_model=result.requested_model, observed_model=result.observed_model)
    if identity_fault:
        return identity_fault
    if result.grade.kind == "unavailable":
        return result.grade.reason
    if not result.evidence_complete:
        return "incomplete_evidence"
    return None


def derive_trial_disposition(trial: tuple[str, str, str, int], *, starts: Sequence[AttemptStart] = (),
                             results: Sequence[AttemptResult] = (), retry_allowance: int = 1,
                             skipped_reason: str | None = None) -> TrialDisposition:
    """Select by controller ordinal; permit at most one explicit transient retry.

Consumers supply frozen schedule membership as `trial`. Missing/unfinished
starts remain visible and cannot be converted into another launch opportunity.
"""
    if type(trial) is not tuple or len(trial) != 4:
        raise RecordIntegrityError("invalid scheduled trial key")
    AttemptKey(*trial, 0)
    if type(retry_allowance) is not int or retry_allowance not in (0, 1):
        raise RecordIntegrityError("retry allowance must be zero or one")
    if any(not isinstance(s, AttemptStart) or s.key.trial != trial for s in starts) or any(not isinstance(r, AttemptResult) or r.key.trial != trial for r in results):
        raise RecordIntegrityError("foreign/unvalidated attempt in trial")
    by_start = {s.key.ordinal: s for s in starts}
    by_result = {r.key.ordinal: r for r in results}
    if len(by_start) != len(starts) or len(by_result) != len(results):
        raise RecordIntegrityError("duplicate attempt record")
    if skipped_reason is not None:
        require_text(skipped_reason, "skipped reason")
        if starts or results:
            raise RecordIntegrityError("skipped trial has attempts")
        return TrialDisposition("skipped", None, None, skipped_reason)
    if not starts:
        if results:
            raise RecordIntegrityError("result without durable start")
        return TrialDisposition("pending", None, 0, None)
    if sorted(by_start) != list(range(len(by_start))) or set(by_result) - set(by_start):
        raise RecordIntegrityError("attempt ordinals/start evidence are incomplete")
    for ordinal, start in sorted(by_start.items()):
        result = by_result.get(ordinal)
        if result is None:
            if ordinal != len(by_start) - 1:
                raise RecordIntegrityError("new attempt after unfinished start")
            return TrialDisposition("running", start.key, None, "unfinished_attempt")
        if result.backend != start.backend or result.requested_model != start.requested_model:
            raise RecordIntegrityError("final result contradicts start")
        fault = admission_fault(result)
        if ordinal < len(by_start) - 1:
            if fault or result.execution.kind != "infrastructure_error" or result.execution.reason not in RETRYABLE_REASONS or ordinal >= retry_allowance:
                raise RecordIntegrityError("attempt launched after nonretryable outcome")
            continue
        if result.execution.kind == "completed":
            kind = {"pass": "succeeded", "fail": "task_failed", "unavailable": "ungradeable"}[result.grade.kind]
            return TrialDisposition(kind, result.key, None, result.grade.reason)
        if not fault and result.execution.kind == "infrastructure_error" and result.execution.reason in RETRYABLE_REASONS and ordinal < retry_allowance:
            return TrialDisposition("retry_pending", result.key, ordinal + 1, result.execution.reason)
        return TrialDisposition("operational_failed", result.key, None, fault or result.execution.reason)
    raise RecordIntegrityError("unreachable empty disposition")
