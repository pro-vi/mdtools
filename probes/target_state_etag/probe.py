"""Review-only scaffold for the target-state etag probe.

This file is intentionally inert in the current run. It describes the future
local CLI shape for an authorized probe runner without performing repository
mutation, network access, environment lookup, or implicit persistence.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from tempfile import TemporaryDirectory


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Static scaffold for the target-state etag probe."
    )
    parser.add_argument(
        "--md-binary",
        type=Path,
        required=True,
        help="Path to the md binary that would supply projection authority later.",
    )
    parser.add_argument(
        "--cases",
        type=Path,
        default=Path("probes/target_state_etag/cases.json"),
        help="Deterministic case manifest.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Reserved for a future explicit output path; ignored by this scaffold.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Reserved future validation flag; the current scaffold remains inert.",
    )
    return parser.parse_args(argv)


def build_plan(args: argparse.Namespace) -> dict[str, object]:
    with TemporaryDirectory(prefix="target-state-etag-") as tempdir:
        return {
            "mode": "review-only-scaffold",
            "md_binary": str(args.md_binary),
            "cases_path": str(args.cases),
            "output_path": str(args.output) if args.output else None,
            "check_requested": bool(args.check),
            "ephemeral_workspace": tempdir,
            "subprocess_contract": "future authorized execution must remain shell=False",
            "notes": [
                "No execution occurs in this scaffold.",
                "No result files are written in this scaffold.",
                "Descriptors must come from md --json projection surfaces only.",
            ],
        }


def _future_subprocess_contract(cmd: list[str], *, check: bool) -> None:
    """Placeholder only: subprocess.run(cmd, shell=False, check=check)."""
    _ = (cmd, check)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    plan = build_plan(args)
    print(json.dumps(plan, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
