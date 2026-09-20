"""One verified producer pair for the explicit benchmark test session."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil

import pytest

from bench.command_policy import CliCondition, ConditionPin, build_pinned_condition, verify_condition


@pytest.fixture(scope="session")
def cli_pins(tmp_path_factory: pytest.TempPathFactory) -> dict[CliCondition, ConditionPin]:
    root = os.environ.get("MDTOOLS_U3_PIN_ROOT")
    if root:
        pins = {}
        for condition, name in ((CliCondition.LEGACY, "legacy"), (CliCondition.CURRENT_COMPACT, "current")):
            payload = json.loads((Path(root) / name / "pin.json").read_bytes())
            payload["condition"] = CliCondition(payload["condition"])
            pins[condition] = ConditionPin(**payload)
            verify_condition(pins[condition])
        return pins
    cargo = shutil.which("cargo")
    if cargo is None and (Path.home() / ".cargo/bin/cargo").is_file():
        cargo = str(Path.home() / ".cargo/bin/cargo")
    if cargo is None:
        pytest.fail("pinned CLI integration requires cargo; no installed md fallback")
    output = tmp_path_factory.mktemp("pinned-cli").resolve()
    repo = Path(__file__).resolve().parent.parent
    return {condition: build_pinned_condition(repo, condition, output / condition.value, cargo=Path(cargo))
            for condition in (CliCondition.LEGACY, CliCondition.CURRENT_COMPACT)}
