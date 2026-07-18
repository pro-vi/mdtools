"""Review-only scaffold for the target-state etag probe.

This file is intentionally inert in the current run. It describes the future
local CLI shape for an authorized probe runner without performing repository
mutation, network access, environment lookup, or implicit persistence.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

SCRIPT_DIR = Path(__file__).resolve().parent
DEFAULT_CASES_PATH = SCRIPT_DIR / "cases.json"


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
        default=DEFAULT_CASES_PATH,
        help="Deterministic case manifest. Defaults to the script-local manifest.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Reserved for a future explicit output path; ignored by this scaffold.",
    )
    parser.add_argument(
        "--check",
        type=Path,
        help="Reserved future explicit check file path; ignored by this scaffold.",
    )
    return parser.parse_args(argv)


def build_plan(args: argparse.Namespace) -> dict[str, object]:
    with TemporaryDirectory(prefix="target-state-etag-") as tempdir:
        return {
            "mode": "review-only-scaffold",
            "md_binary": str(args.md_binary),
            "cases_path": str(args.cases),
            "output_path": str(args.output) if args.output else None,
            "check_path": str(args.check) if args.check else None,
            "ephemeral_workspace": tempdir,
            "subprocess_contract": "future authorized execution must remain shell=False",
            "manifest_policy": "read only the repo-local cases manifest relative to this script",
            "output_policy": "future results require an explicit --output path and atomic write if practical",
            "failure_policy": [
                "Treat nonzero md exit status as a hard probe error.",
                "Treat invalid JSON and missing projection descriptors as hard probe errors.",
                "Treat out-of-bounds spans over exact document bytes as hard probe errors.",
            ],
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
    json.dump(plan, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
