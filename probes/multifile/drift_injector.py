#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


SPECS = {
    "mf01": {
        "target": Path("mf01/runbook.md"),
        "old": "Pager owner: Alex\n",
        "new": "Pager owner: Riley\n",
    },
    "mf02": {
        "target": Path("mf02/backend.md"),
        "old": "Owner: Blake\n",
        "new": "Owner: Casey\n",
    },
}


def inject_once(root: Path, task: str) -> str:
    spec = SPECS[task]
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
    parser = argparse.ArgumentParser(description="Apply a deterministic one-shot multifile drift.")
    parser.add_argument("--root", required=True, type=Path, help="Workdir root containing mf01/ or mf02/")
    parser.add_argument("--task", required=True, choices=sorted(SPECS), help="Drift fixture id")
    args = parser.parse_args()
    print(inject_once(args.root, args.task))


if __name__ == "__main__":
    main()
