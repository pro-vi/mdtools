#!/usr/bin/env python3
"""Run and verify the preregistered core-consumer topology probe."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
from dataclasses import dataclass
from pathlib import Path


BASELINE_COMMIT = "07eb509"
RESULT_START = "<!-- result-json:start -->"
RESULT_END = "<!-- result-json:end -->"
LANES = ("cli_preservation", "braid_adoption", "reader_readiness")


@dataclass(frozen=True)
class CommandResult:
    argv: list[str]
    returncode: int
    stdout: str
    stderr: str


def run(
    argv: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
    check: bool = False,
) -> CommandResult:
    completed = subprocess.run(
        argv,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    result = CommandResult(argv, completed.returncode, completed.stdout, completed.stderr)
    if check and completed.returncode != 0:
        raise RuntimeError(
            f"command failed ({completed.returncode}): {' '.join(argv)}\n{completed.stderr}"
        )
    return result


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cargo_env(target: Path) -> dict[str, str]:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    env["CARGO_NET_OFFLINE"] = "true"
    return env


def extract_baseline(repo: Path, destination: Path) -> None:
    archive = subprocess.run(
        ["git", "archive", "--format=tar", BASELINE_COMMIT],
        cwd=repo,
        capture_output=True,
        check=False,
    )
    if archive.returncode != 0:
        raise RuntimeError(
            f"git archive failed ({archive.returncode}): "
            f"{archive.stderr.decode('utf-8', errors='replace')}"
        )
    archive_path = destination.parent / "baseline.tar"
    archive_path.write_bytes(archive.stdout)
    with tarfile.open(archive_path) as bundle:
        bundle.extractall(destination, filter="data")


def build_binary(source: Path, target: Path) -> Path:
    run(["cargo", "build", "--release", "--locked"], cwd=source, env=cargo_env(target), check=True)
    return target / "release" / "md"


def invoke(binary: Path, args: list[str], cwd: Path) -> CommandResult:
    return run([str(binary), *args], cwd=cwd)


def cli_lane(repo: Path, workspace: Path) -> dict[str, object]:
    baseline = workspace / "baseline"
    current = repo
    baseline.mkdir()
    extract_baseline(repo, baseline)
    baseline_binary = build_binary(baseline, workspace / "baseline-target")
    current_binary = build_binary(current, workspace / "current-target")

    cases = [
        ["schema", "--json"],
        ["outline", "--json", "tests/fixtures/basic.md"],
        ["section", "Introduction", "tests/fixtures/basic.md", "--json"],
        ["tasks", "tests/fixtures/progress_example.md", "--json"],
        ["task", "9.0", "tests/fixtures/progress_example.md", "--json"],
    ]
    comparisons = []
    for args in cases:
        before = invoke(baseline_binary, args, baseline)
        after = invoke(current_binary, args, current)
        comparisons.append(
            {
                "args": args,
                "match": (
                    before.returncode,
                    before.stdout,
                    before.stderr,
                )
                == (after.returncode, after.stdout, after.stderr),
                "baseline_exit": before.returncode,
                "candidate_exit": after.returncode,
            }
        )

    baseline_mutation = workspace / "baseline-mutation"
    current_mutation = workspace / "current-mutation"
    baseline_mutation.mkdir()
    current_mutation.mkdir()
    fixture = repo / "tests/fixtures/progress_example.md"
    shutil.copy2(fixture, baseline_mutation / "progress.md")
    shutil.copy2(fixture, current_mutation / "progress.md")
    mutation_args = ["set-task", "9.0", "progress.md", "--status", "pending", "--json", "-i"]
    before_mutation = invoke(baseline_binary, mutation_args, baseline_mutation)
    after_mutation = invoke(current_binary, mutation_args, current_mutation)
    mutation_match = (
        before_mutation.returncode,
        before_mutation.stdout,
        before_mutation.stderr,
        (baseline_mutation / "progress.md").read_bytes(),
    ) == (
        after_mutation.returncode,
        after_mutation.stdout,
        after_mutation.stderr,
        (current_mutation / "progress.md").read_bytes(),
    )

    suite = run(
        ["cargo", "test", "--quiet", "--locked"],
        cwd=repo,
        env=cargo_env(workspace / "suite-target"),
    )
    passed = all(item["match"] for item in comparisons) and mutation_match and suite.returncode == 0
    return {
        "verdict": "pass" if passed else "fail",
        "comparisons": comparisons,
        "mutation_match": mutation_match,
        "suite_exit": suite.returncode,
        "baseline_binary_sha256": sha256(baseline_binary),
        "candidate_binary_sha256": sha256(current_binary),
    }


def package_source(repo: Path, workspace: Path) -> tuple[Path, list[str], str]:
    target = workspace / "package-target"
    run(
        ["cargo", "package", "--allow-dirty", "--no-verify", "--locked"],
        cwd=repo,
        env=cargo_env(target),
        check=True,
    )
    crate = target / "package" / "mdtools-0.1.0.crate"
    extracted = workspace / "package"
    with tarfile.open(crate, mode="r:gz") as bundle:
        members = bundle.getnames()
        bundle.extractall(extracted, filter="data")
    return extracted / "mdtools-0.1.0", members, sha256(crate)


def write_consumer(root: Path, name: str, dependency: Path, main_rs: str) -> None:
    (root / "src").mkdir(parents=True)
    (root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{name}"',
                'version = "0.1.0"',
                'edition = "2021"',
                "",
                "[dependencies]",
                f'mdtools = {{ path = {json.dumps(str(dependency))}, default-features = false }}',
                "",
            ]
        ),
        encoding="utf-8",
    )
    (root / "src/main.rs").write_text(main_rs, encoding="utf-8")


def consumer_lanes(repo: Path, workspace: Path) -> tuple[dict[str, object], dict[str, object]]:
    package, members, package_sha256 = package_source(repo, workspace)
    forbidden = (
        "/bench/runs/",
        "/bench/search/",
        "/bench/probes/",
        "/probes/",
        "/.inbox/",
        "/.loop/",
        "/docs/plans/",
    )
    package_clean = not any(any(marker in member for marker in forbidden) for member in members)

    braid = workspace / "braid-consumer"
    write_consumer(
        braid,
        "braid-consumer",
        package,
        r'''use mdtools::model::TaskStatus;
use mdtools::parser::ParsedDocument;
use mdtools::task::{self, TaskQuery};

#[derive(Debug, PartialEq)]
enum WorkflowStatus { Pending, Done }

#[derive(Debug, PartialEq)]
struct ProvenanceSpan { start: u32, end: u32 }

fn main() {
    let source = "# Phase\n\n- [ ] task\n".to_string();
    let document = ParsedDocument::parse(source).unwrap();
    let task = task::tasks(&document, &TaskQuery::default()).unwrap().remove(0);
    let status = match task.status {
        TaskStatus::Pending => WorkflowStatus::Pending,
        TaskStatus::Done => WorkflowStatus::Done,
    };
    let provenance = Some(ProvenanceSpan { start: task.span.byte_start, end: task.span.byte_end });
    assert_eq!(status, WorkflowStatus::Pending);
    assert!(provenance.is_some());
}
''',
    )
    braid_run = run(
        ["cargo", "run", "--quiet", "--offline"],
        cwd=braid,
        env=cargo_env(workspace / "braid-target"),
    )
    braid_verdict = "pass" if braid_run.returncode == 0 and package_clean else "fail"

    reader = workspace / "reader-consumer"
    write_consumer(
        reader,
        "reader-consumer",
        package,
        r'''use std::str::FromStr;
use mdtools::model::{HeadingMatchMode, SectionSelector, SectionSelectorKind, TaskStatus};
use mdtools::parser::ParsedDocument;
use mdtools::section::SectionIndex;
use mdtools::task::{self, SetTaskEdit, TaskLoc, TaskQuery};

fn main() {
    let source = "# Tasks\n\n- [ ] first\n".to_string();
    let document = ParsedDocument::parse(source.clone()).unwrap();
    let outline = SectionIndex::new(&document).outline();
    let selector = SectionSelector { kind: SectionSelectorKind::HeadingText, heading_text: Some("Tasks".into()), occurrence: None, match_mode: HeadingMatchMode::Exact };
    assert!(SectionIndex::new(&document).resolve(&selector).is_ok());
    let first = task::tasks(&document, &TaskQuery::default()).unwrap().remove(0);
    let outcome = task::set_task(&document, &SetTaskEdit { loc: TaskLoc::from_str(&first.loc).unwrap(), status: TaskStatus::Done, expect_etag: Some(first.etag) }).unwrap();
    assert_eq!(outline.len(), 1);
    assert_eq!(document.source, source);
    assert_eq!(outcome.content, "# Tasks\n\n- [x] first\n");
}
''',
    )
    reader_run = run(
        ["cargo", "run", "--quiet", "--offline"],
        cwd=reader,
        env=cargo_env(workspace / "reader-target"),
    )
    tree = run(
        ["cargo", "tree", "-e", "normal", "--offline"],
        cwd=reader,
        env=cargo_env(workspace / "reader-target"),
    )
    cli_deps = [line for line in tree.stdout.splitlines() if "clap " in line or "walkdir " in line]
    reader_passed = reader_run.returncode == 0 and tree.returncode == 0 and not cli_deps and package_clean

    return (
        {
            "verdict": braid_verdict,
            "consumer_exit": braid_run.returncode,
            "package_clean": package_clean,
            "package_file_count": len(members),
            "package_sha256": package_sha256,
        },
        {
            "verdict": "pass" if reader_passed else "fail",
            "consumer_exit": reader_run.returncode,
            "tree_exit": tree.returncode,
            "cli_only_dependencies": cli_deps,
            "package_clean": package_clean,
            "package_sha256": package_sha256,
        },
    )


def overall(lanes: dict[str, dict[str, object]]) -> str:
    verdicts = [str(lanes[name]["verdict"]) for name in LANES]
    if "fail" in verdicts:
        return "foundation_rejected"
    if "inconclusive" in verdicts:
        return "foundation_inconclusive"
    if "partial" in verdicts:
        return "foundation_narrowed"
    return "foundation_validated"


def render_results(repo: Path, payload: dict[str, object]) -> None:
    path = repo / "probes/core_consumer_topology/RESULTS.md"
    lane_lines = "\n".join(
        f"- `{name}`: **{payload['lanes'][name]['verdict']}**" for name in LANES
    )
    path.write_text(
        f"""# Core Consumer Topology Results

## Verdict

**{payload['overall']}**

{lane_lines}

## Evidence

- Candidate commit: `{payload['candidate_commit']}`
- Baseline commit: `{BASELINE_COMMIT}`
- The JSON below records the executed lane details and is checked without rerunning builds.

{RESULT_START}
```json
{json.dumps(payload, indent=2, sort_keys=True)}
```
{RESULT_END}
""",
        encoding="utf-8",
    )


def check_results(repo: Path) -> int:
    path = repo / "probes/core_consumer_topology/RESULTS.md"
    text = path.read_text(encoding="utf-8")
    start = text.index(RESULT_START) + len(RESULT_START)
    end = text.index(RESULT_END, start)
    fenced = text[start:end].strip()
    if not fenced.startswith("```json\n") or not fenced.endswith("```"):
        raise ValueError("result JSON fence is malformed")
    payload = json.loads(fenced[len("```json\n") : -len("```")])
    if set(payload["lanes"]) != set(LANES):
        raise ValueError("result lanes do not match the protocol")
    if payload["overall"] != overall(payload["lanes"]):
        raise ValueError("overall verdict does not match lane verdicts")
    print(payload["overall"])
    return 0


def execute(repo: Path) -> int:
    with tempfile.TemporaryDirectory(prefix="mdtools-core-consumer-") as raw_workspace:
        workspace = Path(raw_workspace)
        try:
            cli = cli_lane(repo, workspace)
            braid, reader = consumer_lanes(repo, workspace)
            lanes = {
                "cli_preservation": cli,
                "braid_adoption": braid,
                "reader_readiness": reader,
            }
        except (OSError, RuntimeError, tarfile.TarError) as error:
            lanes = {
                name: {"verdict": "inconclusive", "error": str(error)} for name in LANES
            }
        commit = run(["git", "rev-parse", "HEAD"], cwd=repo, check=True).stdout.strip()
        payload = {
            "schema_version": "core-consumer-topology.v1",
            "candidate_commit": commit,
            "baseline_commit": BASELINE_COMMIT,
            "lanes": lanes,
            "overall": overall(lanes),
        }
        render_results(repo, payload)
        print(payload["overall"])
        return 0 if payload["overall"] == "foundation_validated" else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    return check_results(repo) if args.check else execute(repo)


if __name__ == "__main__":
    raise SystemExit(main())
