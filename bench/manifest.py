"""Hash primitives recovered/renamed from H's v3_manifest.py (c933520).

Old thresholds, headline eligibility and quarantine rules are not recovered.
Content identities stay separate from executable/fixture locators. A validated
specification does not supply a live run grant or prove native containment.
"""

from __future__ import annotations

from dataclasses import dataclass, fields, field
import hashlib
import json
from pathlib import Path
from types import MappingProxyType
from typing import Mapping

from bench.trial_records import (BACKENDS, SCHEMA, RecordIntegrityError,
    record_dict, require_digest, require_artifact_path, require_quantity, require_text)


def sha256_file(path: str | Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def canonical_json(value: object) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False)


@dataclass(frozen=True)
class ExperimentSpec:
    backend: str
    task_sha256: str
    prompt_sha256: str
    input_sha256: Mapping[str, str]
    expected_sha256: str
    command: tuple[str, ...]
    executable_sha256: str
    dependency_lock_sha256: str
    harness_sha256: str
    grader_sha256: str
    condition_sha256: str | None = None
    toolkit_sha256: Mapping[str, str] = field(default_factory=dict)
    support_sha256: Mapping[str, str] = field(default_factory=dict)
    expected_stdout_sha256: str | None = None
    answer_policy_sha256: str | None = None
    grader_version: str = "independent-source/1"
    requested_model: str | None = None
    effort: str | None = None
    thinking_policy: str | None = None
    runner_version: str = "synthetic-argv/1"
    runner_source_sha256: str | None = None
    runner_pin_sha256: str | None = None
    runner_configuration_sha256: str | None = None
    launcher_sha256: str | None = None
    profile_sha256: str | None = None
    limits: Mapping[str, float | int] = field(default_factory=lambda: {"timeout_seconds": 10, "max_turns": 30})
    retry_allowance: int = 1
    statistical_settings: Mapping[str, int] = field(default_factory=lambda: {"seed": 1729, "reps": 10000})
    interpreter: str = "python/3.9.6"
    platform_identity: str = "synthetic-portable"
    locators: Mapping[str, str] = field(default_factory=dict)
    schema: str = SCHEMA
    record: str = "experiment_spec"

    def __post_init__(self) -> None:
        if self.backend not in BACKENDS or self.schema != SCHEMA or self.record != "experiment_spec":
            raise RecordIntegrityError("unknown experiment backend/schema")
        for name in ("task_sha256", "prompt_sha256", "expected_sha256", "executable_sha256",
                     "dependency_lock_sha256", "harness_sha256", "grader_sha256"):
            require_digest(getattr(self, name), name)
        for name in ("condition_sha256", "expected_stdout_sha256", "answer_policy_sha256", "runner_source_sha256",
                     "runner_pin_sha256", "runner_configuration_sha256", "launcher_sha256", "profile_sha256"):
            digest = getattr(self, name)
            if digest is not None:
                require_digest(digest, name)
        for name in ("input_sha256", "support_sha256", "toolkit_sha256"):
            mapping = getattr(self, name)
            if not isinstance(mapping, Mapping):
                raise RecordIntegrityError("experiment digests require mappings")
            frozen = dict(mapping)
            for reference, digest in frozen.items():
                require_artifact_path(reference)
                require_digest(digest, name)
            object.__setattr__(self, name, MappingProxyType(frozen))
        if not self.input_sha256 or set(self.input_sha256) & set(self.support_sha256):
            raise RecordIntegrityError("missing or overlapping experiment inputs")
        if type(self.command) is not tuple or not self.command:
            raise RecordIntegrityError("experiment requires exact command tuple")
        for arg in self.command:
            require_text(arg, "command argument", nonempty=False)
            if "\0" in arg:
                raise RecordIntegrityError("NUL in experiment command")
        if not Path(self.command[0]).is_absolute():
            raise RecordIntegrityError("runner locator must be absolute")
        if type(self.retry_allowance) is not int or self.retry_allowance not in (0, 1):
            raise RecordIntegrityError("experiment retry allowance must be zero or one")
        if not isinstance(self.limits, Mapping) or set(self.limits) != {"timeout_seconds", "max_turns"}:
            raise RecordIntegrityError("closed experiment limits required")
        for name, quantity in self.limits.items():
            require_quantity(quantity, name, integral=name == "max_turns")
            if quantity is None or quantity <= 0:
                raise RecordIntegrityError("positive experiment limits required")
        if not isinstance(self.statistical_settings, Mapping) or set(self.statistical_settings) != {"seed", "reps"}:
            raise RecordIntegrityError("closed statistical settings required")
        for name, quantity in self.statistical_settings.items():
            require_quantity(quantity, name, integral=True)
            if quantity is None or (name == "reps" and quantity == 0):
                raise RecordIntegrityError("invalid statistical settings")
        for name in ("limits", "statistical_settings", "locators"):
            mapping = getattr(self, name)
            if not isinstance(mapping, Mapping):
                raise RecordIntegrityError("experiment requires locator/settings mapping")
            object.__setattr__(self, name, MappingProxyType(dict(mapping)))
        for name, locator in self.locators.items():
            require_text(name, "locator name")
            require_text(locator, "locator")
        for name in ("grader_version", "runner_version", "interpreter", "platform_identity"):
            require_text(getattr(self, name), name)
        for name in ("requested_model", "effort", "thinking_policy"):
            if getattr(self, name) is not None:
                require_text(getattr(self, name), name)
        if self.answer_policy_sha256 is None:
            raise RecordIntegrityError("complete answer policy identity required")
        if self.backend == "claude_cli" and any(getattr(self, name) is None for name in
            ("requested_model", "thinking_policy", "runner_source_sha256", "runner_pin_sha256", "runner_configuration_sha256", "launcher_sha256", "profile_sha256", "condition_sha256")):
            raise RecordIntegrityError("complete live runner/condition identity required")

    @property
    def identity(self) -> str:
        content = record_dict(self)
        content.pop("locators")
        # Executable bytes bind the runner; argv after argv[0] still binds its
        # behavior. Relocating the same regular executable is identity-neutral.
        content["command"] = ["@runner", *self.command[1:]]
        return sha256_text(canonical_json(content))

    @classmethod
    def from_dict(cls, raw: object) -> ExperimentSpec:
        if type(raw) is not dict or set(raw) != {f.name for f in fields(cls)} or type(raw.get("command")) is not list:
            raise RecordIntegrityError("missing/unknown experiment fields")
        payload = dict(raw)
        payload["command"] = tuple(payload["command"])
        return cls(**payload)

    def assert_same_identity(self, other: ExperimentSpec) -> None:
        if not isinstance(other, ExperimentSpec) or self.identity != other.identity:
            raise RecordIntegrityError("changed experiment content; new experiment required")
