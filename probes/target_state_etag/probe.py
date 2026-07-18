"""Static contract surface for the target-state etag probe.

This file remains source-only in the current run. It describes the local CLI
shape for a deterministic manifest-driven runner without performing repository
mutation, network access, environment lookup, or implicit persistence.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

SCRIPT_DIR = Path(__file__).resolve().parent
MANIFEST_PATH = SCRIPT_DIR / "cases.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Static contract surface for the target-state etag probe."
    )
    parser.add_argument(
        "--md-binary",
        type=Path,
        required=True,
        help="Path to the md binary that would supply projection authority later.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Explicit result path for later local execution; unused in this static run.",
    )
    parser.add_argument(
        "--check",
        type=Path,
        help="Explicit check artifact path for later local execution; unused in this static run.",
    )
    return parser.parse_args(argv)


def build_plan(args: argparse.Namespace) -> dict[str, object]:
    with TemporaryDirectory(prefix="target-state-etag-") as tempdir:
        return {
            "mode": "static-contract",
            "md_binary": str(args.md_binary),
            "cases_path": str(MANIFEST_PATH),
            "output_path": str(args.output) if args.output else None,
            "check_path": str(args.check) if args.check else None,
            "ephemeral_workspace": tempdir,
            "subprocess_contract": "later local execution must remain shell=False",
            "manifest_policy": "read only the repo-local deterministic cases manifest relative to this script",
            "output_policy": "later local execution requires an explicit --output path and atomic write if practical",
            "failure_policy": [
                "Treat nonzero md exit status as a hard probe error.",
                "Treat invalid JSON and missing projection descriptors as hard probe errors.",
                "Treat out-of-bounds spans over exact document bytes as hard probe errors.",
            ],
            "notes": [
                "No probe execution occurs in this static run.",
                "No result files are written in this static run.",
                "Descriptors must come from md --json projection surfaces only.",
            ],
        }


def _subprocess_contract_example(cmd: list[str], *, check: bool) -> None:
    """Reference shape only: subprocess.run(cmd, shell=False, check=check)."""
    _ = (cmd, check)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    plan = build_plan(args)
    json.dump(plan, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
