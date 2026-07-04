#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


SPECS_PATH = Path(__file__).with_name("drift_specs.json")


def load_specs() -> dict[str, dict[str, object]]:
    raw = json.loads(SPECS_PATH.read_text())
    return {
        task: {
            "target": Path(spec["target"]),
            "old": spec["old"],
            "new": spec["new"],
        }
        for task, spec in raw.items()
    }


def inject_once(root: Path, task: str) -> str:
    spec = load_specs()[task]
    state = root / f".{task}.drift-fired"
    target = root / spec["target"]
    if state.exists():
        return "already-fired"
    text = target.read_text()
    if spec["old"] not in text:
        if spec["new"] in text:
            state.write_text("already-drifted\n")
            return "already-drifted"
        raise SystemExit(f"{spec['target']}: drift anchor missing")
    target.write_text(text.replace(spec["old"], spec["new"], 1))
    state.write_text("fired\n")
    return "fired"


def main() -> None:
    specs = load_specs()
    parser = argparse.ArgumentParser(description="Apply a deterministic one-shot multifile drift.")
    parser.add_argument("--root", required=True, type=Path, help="Workdir root containing mf01/ or mf02/")
    parser.add_argument("--task", required=True, choices=sorted(specs), help="Drift fixture id")
    args = parser.parse_args()
    print(inject_once(args.root, args.task))


if __name__ == "__main__":
    main()
