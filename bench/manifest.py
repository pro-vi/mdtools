"""Hash primitives recovered/renamed from H's v3_manifest.py (c933520).

Old thresholds, headline eligibility and quarantine rules are not recovered.
Content identities stay separate from executable/fixture locators. A validated
specification does not supply a live run grant or prove native containment.
"""

from __future__ import annotations

from dataclasses import dataclass, fields, field, replace
from functools import cached_property
import hashlib
import json
import random
from pathlib import Path
from types import MappingProxyType
from typing import Mapping

from bench.trial_records import (BACKENDS, CONDITIONS, SCHEMA, AttemptKey, RecordIntegrityError,
    record_dict, require_digest, require_artifact_path, require_quantity, require_text)


def sha256_file(path: str | Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def canonical_json(value: object) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False)


@dataclass(frozen=True)
class CampaignConfig:
    """Operator inputs, not frozen evidence or permission to contact a provider."""
    task_ids: tuple[str, ...]
    pin_root: str
    source_root: str
    output_root: str
    command: tuple[str, ...]
    claude_executable: str | None
    endpoint: str | None
    model: str | None
    effort: str | None
    thinking_policy: str | None
    guidance: str
    repetitions: int
    seed: int
    timeout_seconds: float
    max_turns: int
    retry_allowance: int
    attempt_usd: float
    campaign_usd: float

    def __post_init__(self) -> None:
        from bench.command_policy import ClaudeRunner, ToolGuidance
        ToolGuidance(self.guidance)
        if (type(self.task_ids) is not tuple or not self.task_ids or
            any(type(task) is not str or task not in ("T1", "T2", "T10", "T21", "cli-canary") for task in self.task_ids) or
            len(set(self.task_ids)) != len(self.task_ids) or
            ("cli-canary" in self.task_ids and self.task_ids != ("cli-canary",))):
            raise RecordIntegrityError("unsupported configured task scope")
        for name in ("pin_root", "source_root", "output_root"):
            value = getattr(self, name)
            require_text(value, name)
            if not Path(value).is_absolute():
                raise RecordIntegrityError("config paths must be absolute")
        for name in ("repetitions", "seed", "timeout_seconds", "max_turns", "attempt_usd", "campaign_usd", "retry_allowance"):
            value = getattr(self, name)
            require_quantity(value, name, integral=name in ("repetitions", "seed", "max_turns", "retry_allowance"))
            if value is None or (name not in ("seed", "retry_allowance") and value <= 0):
                raise RecordIntegrityError("invalid configuration quantity")
        if self.retry_allowance not in (0, 1) or self.attempt_usd > self.campaign_usd:
            raise RecordIntegrityError("invalid retry or cost configuration")
        if type(self.command) is not tuple:
            raise RecordIntegrityError("command requires an argv tuple")
        if self.claude_executable is None:
            if not self.command or any(value is not None for value in (self.endpoint, self.model, self.effort, self.thinking_policy)):
                raise RecordIntegrityError("synthetic argv cannot contain Claude configuration")
            for argument in self.command:
                require_text(argument, "command argument", nonempty=False)
                if "\0" in argument:
                    raise RecordIntegrityError("NUL in command")
        else:
            if self.command or type(self.claude_executable) is not str or not Path(self.claude_executable).is_absolute():
                raise RecordIntegrityError("Claude config requires an explicit executable and no argv")
            ClaudeRunner(self.claude_executable, self.endpoint, self.model, self.effort,
                self.thinking_policy, self.max_turns, self.attempt_usd)

    @classmethod
    def from_dict(cls, raw: object) -> CampaignConfig:
        if type(raw) is not dict or set(raw) != {entry.name for entry in fields(cls)}:
            raise RecordIntegrityError("missing/unknown campaign config fields")
        payload = dict(raw)
        for name in ("task_ids", "command"):
            if type(payload[name]) is not list:
                raise RecordIntegrityError("config scope and argv must be arrays")
            payload[name] = tuple(payload[name])
        return cls(**payload)


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


@dataclass(frozen=True)
class CampaignSpec:
    """Checked aggregate of one model's task/condition specifications.

    Schedule entries name trials, never attempts. Relocation is neutral only
    where the constituent ExperimentSpec already declares a locator.
    """

    members: Mapping[str, ExperimentSpec]
    schedule: tuple[tuple[str, str, int], ...]
    prefix_length: int = 0
    grant_usd: float | None = None
    reservation_usd: float | None = None
    core_study: bool = False
    schema: str = SCHEMA
    record: str = "campaign_spec"

    def __post_init__(self) -> None:
        if self.schema != SCHEMA or self.record != "campaign_spec" or type(self.core_study) is not bool:
            raise RecordIntegrityError("unknown campaign schema")
        if not isinstance(self.members, Mapping) or not self.members:
            raise RecordIntegrityError("campaign requires checked members")
        members = dict(self.members)
        models = set()
        statistics = set()
        for name, spec in members.items():
            require_artifact_path(name)
            if not isinstance(spec, ExperimentSpec):
                raise RecordIntegrityError("campaign member requires ExperimentSpec")
            models.add((spec.backend, spec.requested_model, spec.effort, spec.thinking_policy))
            statistics.add(canonical_json(record_dict(spec.statistical_settings)))
        if len(models) != 1:
            raise RecordIntegrityError("campaign cannot pool models/configurations")
        if len(statistics) != 1:
            raise RecordIntegrityError("campaign requires common statistical settings")
        if type(self.schedule) is not tuple or not self.schedule or any(type(entry) is not tuple for entry in self.schedule):
            raise RecordIntegrityError("empty or malformed campaign schedule")
        used = set()
        for entry in self.schedule:
            if type(entry) is not tuple or len(entry) != 3:
                raise RecordIntegrityError("malformed scheduled trial")
            task, condition, repetition = entry
            AttemptKey("0" * 64, task, condition, repetition, 0)
            used.add(self.member_name(task, condition))
        if len(set(self.schedule)) != len(self.schedule):
            raise RecordIntegrityError("duplicate campaign schedule")
        if used != set(members):
            raise RecordIntegrityError("schedule/member mismatch")
        tasks = {entry[0] for entry in self.schedule}
        repetitions = {entry[2] for entry in self.schedule}
        if sorted(repetitions) != list(range(len(repetitions))) or set(self.schedule) != {
                (task, condition, repetition) for task in tasks for condition in CONDITIONS for repetition in repetitions}:
            raise RecordIntegrityError("campaign requires complete task/condition/repetition schedule")
        if any(len({(entry[0], entry[2]) for entry in self.schedule[index:index + 3]}) != 1
               for index in range(0, len(self.schedule), 3)):
            raise RecordIntegrityError("condition order must stay inside serial trial blocks")
        if type(self.prefix_length) is not int or not 0 <= self.prefix_length <= len(self.schedule):
            raise RecordIntegrityError("invalid prefix length")
        for name in ("grant_usd", "reservation_usd"):
            require_quantity(getattr(self, name), name)
        if (self.grant_usd is None) != (self.reservation_usd is None):
            raise RecordIntegrityError("budget requires grant and per-attempt reservation")
        if self.core_study:
            expected = {(f"T{n}", condition, repetition) for n in range(1, 25)
                        for condition in CONDITIONS for repetition in range(5)}
            prefix = {(task, condition, 0) for task in ("T14", "T23") for condition in CONDITIONS}
            if set(self.schedule) != expected or self.prefix_length != 6 or set(self.schedule[:6]) != prefix:
                raise RecordIntegrityError("core study requires all 360 trials and six-entry prefix")
        object.__setattr__(self, "members", MappingProxyType(members))

    @staticmethod
    def member_name(task_id: str, condition: str) -> str:
        require_artifact_path(task_id)
        return task_id + "/" + condition

    @cached_property
    def identity(self) -> str:
        return sha256_text(canonical_json({"schema": self.schema, "record": self.record,
            "members": {name: spec.identity for name, spec in self.members.items()},
            "schedule": self.schedule, "prefix_length": self.prefix_length,
            "grant_usd": self.grant_usd, "reservation_usd": self.reservation_usd,
            "core_study": self.core_study}))

    def retry_allowance(self, trial: tuple[str, str, str, int]) -> int:
        self.assert_trial(trial)
        entry = trial[1:]
        return 0 if entry in self.schedule[:self.prefix_length] else self.members[self.member_name(*entry[:2])].retry_allowance

    def assert_trial(self, trial: tuple[str, str, str, int]) -> None:
        if type(trial) is not tuple or len(trial) != 4 or trial[0] != self.identity or trial[1:] not in self.schedule:
            raise RecordIntegrityError("foreign/unscheduled campaign trial")

    def assert_member(self, key: AttemptKey, spec: ExperimentSpec) -> None:
        self.assert_trial(key.trial)
        self.members[self.member_name(key.task_id, key.condition)].assert_same_identity(spec)

    @classmethod
    def from_dict(cls, raw: object) -> CampaignSpec:
        if type(raw) is not dict or set(raw) != {f.name for f in fields(cls)} or type(raw.get("members")) is not dict or type(raw.get("schedule")) is not list:
            raise RecordIntegrityError("missing/unknown campaign fields")
        payload = dict(raw)
        payload["members"] = {name: ExperimentSpec.from_dict(spec) for name, spec in payload["members"].items()}
        if any(type(entry) is not list for entry in payload["schedule"]):
            raise RecordIntegrityError("malformed campaign schedule")
        payload["schedule"] = tuple(tuple(entry) for entry in payload["schedule"])
        return cls(**payload)


def serial_schedule(task_ids: tuple[str, ...], *, repetitions: int = 5,
                    seed: int = 1729, core_study: bool = False) -> tuple[tuple[str, str, int], ...]:
    """Freeze randomized condition order within task/repetition blocks."""
    if type(task_ids) is not tuple or not task_ids or any(type(task) is not str for task in task_ids) or len(set(task_ids)) != len(task_ids):
        raise RecordIntegrityError("unique task IDs required")
    if type(repetitions) is not int or repetitions <= 0 or type(seed) is not int or seed < 0:
        raise RecordIntegrityError("invalid schedule settings")
    for task_id in task_ids:
        require_artifact_path(task_id)
    blocks = [(task, repetition) for repetition in range(repetitions) for task in task_ids]
    if core_study:
        if set(task_ids) != {f"T{n}" for n in range(1, 25)} or repetitions != 5:
            raise RecordIntegrityError("core study needs 24 tasks and five repetitions")
        prefix = [("T14", 0), ("T23", 0)]
        blocks = prefix + [block for block in blocks if block not in prefix]
    rng = random.Random(seed)
    schedule = []
    for task, repetition in blocks:
        order = list(CONDITIONS)
        rng.shuffle(order)
        schedule.extend((task, condition, repetition) for condition in order)
    return tuple(schedule)


@dataclass(frozen=True)
class LiveRunGrant:
    """Explicit operator-owned authority; an experiment spec is not consent.

    Every field is required. This record must only be constructed from an
    explicit scoped user grant, never inferred from saved results or defaults.
    """

    experiment_id: str
    model: str
    effort: str | None
    thinking_policy: str
    phase: str
    task_ids: tuple[str, ...]
    conditions: tuple[str, ...]
    repetitions: tuple[int, ...]
    timeout_seconds: float
    max_turns: int
    attempt_usd: float
    campaign_usd: float
    max_attempts: int
    core_contract_risk_acknowledged: bool
    prerequisite_experiment_id: str | None

    def __post_init__(self) -> None:
        require_digest(self.experiment_id, "granted experiment")
        require_text(self.model, "granted model")
        require_text(self.thinking_policy, "granted thinking policy")
        if self.effort is not None:
            require_text(self.effort, "granted effort")
        if self.phase not in ("canary", "public_pilot", "core_study", "prompt_comparison"):
            raise RecordIntegrityError("unknown live phase")
        if type(self.task_ids) is not tuple or not self.task_ids or any(type(task) is not str for task in self.task_ids) or len(set(self.task_ids)) != len(self.task_ids):
            raise RecordIntegrityError("explicit granted task IDs required")
        for task in self.task_ids:
            require_artifact_path(task)
        if type(self.conditions) is not tuple or len(self.conditions) != 3 or any(type(condition) is not str for condition in self.conditions) or set(self.conditions) != set(CONDITIONS):
            raise RecordIntegrityError("grant requires exactly three conditions")
        if type(self.repetitions) is not tuple or not self.repetitions:
            raise RecordIntegrityError("explicit granted repetitions required")
        for repetition in self.repetitions:
            if type(repetition) is not int or repetition < 0:
                raise RecordIntegrityError("invalid granted repetition")
        if len(set(self.repetitions)) != len(self.repetitions):
            raise RecordIntegrityError("duplicate granted repetition")
        for name in ("timeout_seconds", "max_turns", "attempt_usd", "campaign_usd", "max_attempts"):
            quantity = getattr(self, name)
            require_quantity(quantity, name, integral=name in ("max_turns", "max_attempts"))
            if quantity is None or quantity <= 0:
                raise RecordIntegrityError("positive explicit grant limits required")
        if self.attempt_usd > self.campaign_usd or type(self.core_contract_risk_acknowledged) is not bool:
            raise RecordIntegrityError("invalid live grant limits/acknowledgment")
        if self.prerequisite_experiment_id is not None:
            require_digest(self.prerequisite_experiment_id, "prerequisite experiment")
        if (self.phase == "canary") != (self.prerequisite_experiment_id is None):
            raise RecordIntegrityError("live phase requires its separate prerequisite experiment")

    def assert_scope(self, spec: CampaignSpec) -> None:
        if not isinstance(spec, CampaignSpec) or self.experiment_id != spec.identity:
            raise RecordIntegrityError("live grant does not match campaign identity")
        if any(member.backend != "claude_cli" or (member.requested_model, member.effort, member.thinking_policy) !=
               (self.model, self.effort, self.thinking_policy) or dict(member.limits) !=
               {"timeout_seconds": self.timeout_seconds, "max_turns": self.max_turns}
               for member in spec.members.values()):
            raise RecordIntegrityError("live grant model/configuration/limits mismatch")
        if spec.grant_usd != self.campaign_usd or spec.reservation_usd != self.attempt_usd:
            raise RecordIntegrityError("live grant budget/reservation mismatch")
        schedule = {(task, condition, repetition) for task in self.task_ids
                    for condition in self.conditions for repetition in self.repetitions}
        if set(spec.schedule) != schedule:
            raise RecordIntegrityError("live grant trial scope mismatch")
        core_ids = {f"T{n}" for n in range(1, 25)}
        if self.phase == "canary":
            if len(self.task_ids) != 1 or set(self.task_ids) & core_ids or self.repetitions != (0,) or self.max_attempts != 3 or spec.core_study or any(member.retry_allowance != 0 for member in spec.members.values()):
                raise RecordIntegrityError("canary grant requires three synthetic trials and zero retries")
        elif self.phase in ("public_pilot", "prompt_comparison"):
            if set(self.task_ids) != {"T1", "T2", "T10"} or self.repetitions != (0,) or spec.core_study:
                raise RecordIntegrityError("public pilot grant requires exactly nine T1/T2/T10 trials")
            if self.phase == "prompt_comparison" and (self.max_attempts != 9 or any(member.retry_allowance != 0 for member in spec.members.values())):
                raise RecordIntegrityError("prompt comparison requires nine trials and zero retries per profile")
        elif not spec.core_study or set(self.task_ids) != core_ids or self.repetitions != tuple(range(5)) or not self.core_contract_risk_acknowledged:
            raise RecordIntegrityError("core grant requires 360 trials and explicit T14/T23 contract-risk acknowledgment")
        maximum = len(spec.schedule) + sum(spec.retry_allowance((spec.identity, *entry)) for entry in spec.schedule)
        if self.max_attempts > maximum:
            raise RecordIntegrityError("grant attempts exceed frozen retry policy")

    @property
    def identity(self) -> str:
        return sha256_text(canonical_json(record_dict(self)))

    @classmethod
    def from_dict(cls, raw: object) -> LiveRunGrant:
        if type(raw) is not dict or set(raw) != {f.name for f in fields(cls)}:
            raise RecordIntegrityError("missing/unknown live grant evidence fields")
        payload = dict(raw)
        for name in ("task_ids", "conditions", "repetitions"):
            if type(payload[name]) is not list:
                raise RecordIntegrityError("malformed live grant scope evidence")
            payload[name] = tuple(payload[name])
        return cls(**payload)


def paired_prompt_schedule(campaign: CampaignSpec, seed: int) -> tuple[tuple[str, str, str, int], ...]:
    """Keep child order intact and counterbalance adjacent profile pairs."""
    if type(seed) is not int or seed < 0:
        raise RecordIntegrityError("invalid comparison seed")
    rng = random.Random(seed)
    conditions = list(CONDITIONS)
    rng.shuffle(conditions)
    offset = rng.randrange(2)
    first = [0] * len(campaign.schedule)
    for condition_index, condition in enumerate(conditions):
        positions = [index for index, entry in enumerate(campaign.schedule) if entry[1] == condition]
        order = [(index + offset + condition_index) % 2 for index in range(len(positions))]
        rng.shuffle(order)
        for index, leading in zip(positions, order):
            first[index] = leading
    profiles = ("full_help", "discovery")
    return tuple((profiles[side], *entry) for entry, leading in zip(campaign.schedule, first)
                 for side in (leading, 1 - leading))


@dataclass(frozen=True)
class PromptComparisonSpec:
    """A public diagnostic links campaigns; it never merges their identities."""
    campaigns: Mapping[str, CampaignSpec]
    seed: int
    schedule: tuple[tuple[str, str, str, int], ...]
    analysis: str = "paired-public-diagnostic/1"

    def __post_init__(self) -> None:
        if not isinstance(self.campaigns, Mapping) or set(self.campaigns) != {"full_help", "discovery"} or any(
                not isinstance(child, CampaignSpec) for child in self.campaigns.values()):
            raise RecordIntegrityError("comparison requires exactly two profile campaigns")
        if type(self.schedule) is not tuple or any(type(entry) is not tuple or len(entry) != 4 for entry in self.schedule):
            raise RecordIntegrityError("invalid paired schedule shape")
        for profile, task, condition, repetition in self.schedule:
            if profile not in self.campaigns:
                raise RecordIntegrityError("unknown scheduled profile")
            AttemptKey(self.campaigns[profile].identity, task, condition, repetition, 0)
        left, right = self.campaigns["full_help"], self.campaigns["discovery"]
        required = {(task, condition, 0) for task in ("T1", "T2", "T10") for condition in CONDITIONS}
        if any(child.core_study or child.prefix_length != 0 or set(child.schedule) != required or
               any(member.retry_allowance != 0 for member in child.members.values()) for child in (left, right)):
            raise RecordIntegrityError("comparison requires nine public zero-retry trials per profile")
        if (left.schedule, left.grant_usd, left.reservation_usd) != (right.schedule, right.grant_usd, right.reservation_usd):
            raise RecordIntegrityError("non-prompt campaign differences")
        for name, member in left.members.items():
            other = right.members[name]
            if replace(member, prompt_sha256=other.prompt_sha256).identity != other.identity:
                raise RecordIntegrityError("non-prompt member differences")
            if name.endswith("/no-md"):
                if member.prompt_sha256 != other.prompt_sha256:
                    raise RecordIntegrityError("no-md control prompts differ")
            elif member.prompt_sha256 == other.prompt_sha256:
                raise RecordIntegrityError("md guidance treatment is absent")
        if self.analysis != "paired-public-diagnostic/1" or self.schedule != paired_prompt_schedule(left, self.seed):
            raise RecordIntegrityError("changed comparison schedule or analysis")
        object.__setattr__(self, "campaigns", MappingProxyType(dict(self.campaigns)))

    @classmethod
    def create(cls, campaigns: Mapping[str, CampaignSpec], *, seed: int) -> PromptComparisonSpec:
        if "full_help" not in campaigns:
            raise RecordIntegrityError("missing full-help control")
        return cls(campaigns, seed, paired_prompt_schedule(campaigns["full_help"], seed))

    @property
    def identity(self) -> str:
        return sha256_text(canonical_json({"campaigns": {name: child.identity for name, child in self.campaigns.items()},
            "seed": self.seed, "schedule": self.schedule, "analysis": self.analysis}))

    @classmethod
    def from_dict(cls, raw: object) -> PromptComparisonSpec:
        if type(raw) is not dict or set(raw) != {entry.name for entry in fields(cls)} or type(raw["campaigns"]) is not dict or type(raw["schedule"]) is not list:
            raise RecordIntegrityError("missing/unknown comparison fields")
        if any(type(entry) is not list for entry in raw["schedule"]):
            raise RecordIntegrityError("invalid comparison schedule entries")
        return cls({name: CampaignSpec.from_dict(child) for name, child in raw["campaigns"].items()},
            raw["seed"], tuple(tuple(entry) for entry in raw["schedule"]), raw["analysis"])
