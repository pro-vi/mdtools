from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DRIFT_SPECS_PATH = Path("probes/multifile/drift_specs.json")


@dataclass(frozen=True)
class DriftSpec:
    task: str
    target: str
    old: str
    new: str


@dataclass(frozen=True)
class DriftProof:
    enabled: bool
    observed_read: bool
    drift_fired: bool
    mutation_after_drift: bool
    invalid_reasons: tuple[str, ...]
    target_mutations: int
    fired_outcome: str | None = None

    @property
    def valid(self) -> bool:
        return (
            self.enabled
            and self.observed_read
            and self.drift_fired
            and self.mutation_after_drift
            and not self.invalid_reasons
        )


def load_drift_specs(path: Path = DRIFT_SPECS_PATH) -> dict[str, DriftSpec]:
    raw = json.loads(path.read_text())
    return {
        task: DriftSpec(
            task=task,
            target=str(spec["target"]),
            old=str(spec["old"]),
            new=str(spec["new"]),
        )
        for task, spec in raw.items()
    }


def drift_task_from_input_files(input_files: list[str]) -> str | None:
    for input_file in input_files:
        parent = Path(input_file).parent.name
        if parent.startswith("mf"):
            return parent
    return None


def summarize_drift_proof(events: list[dict[str, Any]]) -> DriftProof:
    enabled = False
    observed_read = False
    drift_fired = False
    mutation_after_drift = False
    invalid_reasons: list[str] = []
    target_mutations = 0
    fired_outcome: str | None = None

    for event in events:
        event_name = event.get("event")
        if event_name == "multifile_drift_config":
            details = event.get("details")
            enabled = enabled or (isinstance(details, dict) and details.get("enabled") is True)
        elif event_name == "multifile_drift_observed_read":
            observed_read = True
        elif event_name == "multifile_drift_fired":
            drift_fired = True
            details = event.get("details")
            if isinstance(details, dict) and isinstance(details.get("outcome"), str):
                fired_outcome = details["outcome"]
        elif event_name == "multifile_drift_target_mutation":
            target_mutations += 1
            details = event.get("details")
            after_drift = isinstance(details, dict) and details.get("afterDrift") is True
            after_read = isinstance(details, dict) and details.get("afterObservedRead") is True
            if after_read and after_drift:
                mutation_after_drift = True
        elif event_name == "multifile_drift_invalid":
            reason = event.get("reason")
            invalid_reasons.append(str(reason) if reason else "invalid drift order")
        elif event_name == "multifile_drift_error":
            reason = event.get("reason")
            invalid_reasons.append(str(reason) if reason else "drift injection error")

    return DriftProof(
        enabled=enabled,
        observed_read=observed_read,
        drift_fired=drift_fired,
        mutation_after_drift=mutation_after_drift,
        invalid_reasons=tuple(invalid_reasons),
        target_mutations=target_mutations,
        fired_outcome=fired_outcome,
    )


def format_drift_proof(proof: DriftProof) -> str:
    status = "OK" if proof.valid else "INVALID"
    parts = [
        f"multifile_drift_proof: {status}",
        f"enabled={proof.enabled}",
        f"observed_read={proof.observed_read}",
        f"drift_fired={proof.drift_fired}",
        f"mutation_after_drift={proof.mutation_after_drift}",
        f"target_mutations={proof.target_mutations}",
    ]
    if proof.fired_outcome:
        parts.append(f"outcome={proof.fired_outcome}")
    if proof.invalid_reasons:
        parts.append("reasons=" + "; ".join(proof.invalid_reasons))
    return " ".join(parts)
