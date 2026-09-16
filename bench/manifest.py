"""Hash primitives recovered/renamed from H's v3_manifest.py (c933520).

Old thresholds, headline eligibility and quarantine rules are not recovered.
This synthetic specification is a U1 mechanism receipt, not a frozen study spec.
"""

from __future__ import annotations

from dataclasses import dataclass, asdict, field
import hashlib
import json
from pathlib import Path


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
    input_sha256: dict[str, str]
    expected_sha256: str
    command: tuple[str, ...]
    executable_sha256: str
    dependency_lock_sha256: str
    harness_sha256: str
    grader_sha256: str
    condition_sha256: str | None = None
    toolkit_sha256: dict[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if self.backend != "synthetic":
            raise ValueError("U1 ExperimentSpec supports synthetic evidence only")

    @property
    def identity(self) -> str:
        return sha256_text(canonical_json(asdict(self)))
