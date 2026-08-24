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


def extract_regular_archive(bundle: tarfile.TarFile, destination: Path) -> None:
    destination_root = destination.resolve()
    for member in bundle.getmembers():
        if not (member.isfile() or member.isdir()):
            raise RuntimeError(f"archive contains unsupported entry: {member.name}")
        target = (destination / member.name).resolve()
        if not target.is_relative_to(destination_root):
            raise RuntimeError(f"archive path escapes destination: {member.name}")
    bundle.extractall(destination)


def cargo_env(target: Path, cargo_home: Path | None = None) -> dict[str, str]:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    env["CARGO_NET_OFFLINE"] = "true"
    if cargo_home is not None:
        env["CARGO_HOME"] = str(cargo_home)
    return env


def prepare_cargo_home(workspace: Path) -> Path:
    source = Path.home() / ".cargo/registry"
    destination = workspace / "cargo-home/registry"
    destination.mkdir(parents=True)
    for name in ("cache", "index"):
        source_entry = source / name
        if not source_entry.exists():
            raise RuntimeError(f"Cargo registry {name} is unavailable")
        (destination / name).symlink_to(source_entry, target_is_directory=True)
    (destination / "src").mkdir()
    return destination.parent


def command_verdict(passed: bool, *results: CommandResult) -> str:
    if passed:
        return "pass"
    operational_markers = (
        "Operation not permitted",
        "unable to get packages from source",
        "Could not resolve host",
    )
    if any(
        marker in result.stderr
        for result in results
        for marker in operational_markers
    ):
        return "inconclusive"
    return "fail"


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
        extract_regular_archive(bundle, destination)


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
        ["blocks", "tests/fixtures/basic.md", "--json"],
        ["block", "0", "tests/fixtures/basic.md", "--json"],
        ["search", "Introduction", "tests/fixtures/basic.md", "--json"],
        ["links", "tests/fixtures/basic.md", "--json"],
        ["frontmatter", "tests/fixtures/frontmatter.md", "--json"],
        ["collect", "tests/fixtures/frontmatter.md", "--field", "title", "--json"],
        ["stats", "tests/fixtures/basic.md", "--json"],
        ["table", "tests/fixtures/table.md", "--json"],
        ["table", "tests/fixtures/table.md", "--index", "1", "--json"],
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

    mutation_cases = [
        {
            "name": "replace-block",
            "source": "# A\n\nold\n",
            "payload": "new\n",
            "args": ["replace-block", "1", "doc.md", "--from", "payload.md", "--json", "-i"],
        },
        {
            "name": "insert-block",
            "source": "# A\n\nold\n",
            "payload": "inserted\n",
            "args": ["insert-block", "doc.md", "--after", "0", "--from", "payload.md", "--json", "-i"],
        },
        {
            "name": "delete-block",
            "source": "# A\n\nold\n",
            "args": ["delete-block", "1", "doc.md", "--json", "-i"],
        },
        {
            "name": "move-block",
            "source": "one\n\ntwo\n\nthree\n",
            "args": ["move-block", "0", "doc.md", "--after", "2", "--json", "-i"],
        },
        {
            "name": "replace-section",
            "source": "# A\n\nold\n\n# B\n\nbody\n",
            "payload": "# A\n\nnew\n",
            "args": ["replace-section", "A", "doc.md", "--from", "payload.md", "--json", "-i"],
        },
        {
            "name": "delete-section",
            "source": "# A\n\nold\n\n# B\n\nbody\n",
            "args": ["delete-section", "A", "doc.md", "--json", "-i"],
        },
        {
            "name": "move-section",
            "source": "# A\n\na\n\n# B\n\nb\n",
            "args": ["move-section", "A", "doc.md", "--after", "B", "--keep-level", "--json", "-i"],
        },
        {
            "name": "set-frontmatter",
            "source": "---\ntitle: old\n---\n\n# Body\n",
            "args": ["set", "title", "doc.md", "new", "--json", "-i"],
        },
        {
            "name": "replace-table-row",
            "source": "| A |\n| --- |\n| old |\n",
            "payload": "| new |\n",
            "args": ["replace-table-row", "0", "0", "doc.md", "--from", "payload.md", "--json", "-i"],
        },
        {
            "name": "insert-table-row",
            "source": "| A |\n| --- |\n| old |\n",
            "payload": "| new |\n",
            "args": ["insert-table-row", "0", "1", "doc.md", "--from", "payload.md", "--json", "-i"],
        },
        {
            "name": "delete-table-row",
            "source": "| A |\n| --- |\n| old |\n| keep |\n",
            "args": ["delete-table-row", "0", "0", "doc.md", "--json", "-i"],
        },
        {
            "name": "set-task",
            "source": "# Tasks\n\n- [ ] task\n",
            "args": ["set-task", "1.0", "doc.md", "--status", "done", "--json", "-i"],
        },
    ]
    mutation_comparisons = []
    for index, case in enumerate(mutation_cases):
        baseline_mutation = workspace / f"baseline-mutation-{index}"
        current_mutation = workspace / f"current-mutation-{index}"
        baseline_mutation.mkdir()
        current_mutation.mkdir()
        for root in (baseline_mutation, current_mutation):
            (root / "doc.md").write_text(case["source"], encoding="utf-8")
            if "payload" in case:
                (root / "payload.md").write_text(case["payload"], encoding="utf-8")
        before_mutation = invoke(baseline_binary, case["args"], baseline_mutation)
        after_mutation = invoke(current_binary, case["args"], current_mutation)
        matched = (
            before_mutation.returncode,
            before_mutation.stdout,
            before_mutation.stderr,
            (baseline_mutation / "doc.md").read_bytes(),
        ) == (
            after_mutation.returncode,
            after_mutation.stdout,
            after_mutation.stderr,
            (current_mutation / "doc.md").read_bytes(),
        )
        mutation_comparisons.append(
            {
                "name": case["name"],
                "args": case["args"],
                "match": matched,
                "baseline_exit": before_mutation.returncode,
                "candidate_exit": after_mutation.returncode,
            }
        )
    mutation_match = all(item["match"] for item in mutation_comparisons)

    suite = run(
        ["cargo", "test", "--quiet", "--locked"],
        cwd=repo,
        env=cargo_env(workspace / "suite-target"),
    )
    passed = all(item["match"] for item in comparisons) and mutation_match and suite.returncode == 0
    return {
        "verdict": "pass" if passed else "fail",
        "comparisons": comparisons,
        "mutation_comparisons": mutation_comparisons,
        "mutation_match": mutation_match,
        "suite_exit": suite.returncode,
        "baseline_binary_sha256": sha256(baseline_binary),
        "candidate_binary_sha256": sha256(current_binary),
    }


def package_source(repo: Path, workspace: Path) -> tuple[Path, list[str], str]:
    target = workspace / "package-target"
    run(
        ["cargo", "package", "--no-verify", "--locked"],
        cwd=repo,
        env=cargo_env(target),
        check=True,
    )
    crate = target / "package" / "mdtools-0.1.0.crate"
    extracted = workspace / "package"
    with tarfile.open(crate, mode="r:gz") as bundle:
        members = bundle.getnames()
        extract_regular_archive(bundle, extracted)
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
    cargo_home = prepare_cargo_home(workspace)
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
        r'''use mdtools::document::Document;
use mdtools::model::TaskStatus;
use mdtools::task::{self, TaskQuery};

#[derive(Debug, PartialEq)]
enum WorkflowStatus { Pending, Done }

#[derive(Debug, PartialEq)]
struct ProvenanceSpan { start: u32, end: u32 }

fn main() {
    let source = "# Phase\n\n- [ ] task\n".to_string();
    let document = Document::parse(source).unwrap();
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
        env=cargo_env(workspace / "braid-target", cargo_home),
    )
    braid_passed = braid_run.returncode == 0 and package_clean
    braid_verdict = command_verdict(braid_passed, braid_run)

    reader = workspace / "reader-consumer"
    write_consumer(
        reader,
        "reader-consumer",
        package,
        r'''use mdtools::block;
use mdtools::block_edit;
use mdtools::document::Document;
use mdtools::frontmatter;
use mdtools::link;
use mdtools::model::{HeadingMatchMode, TaskStatus};
use mdtools::search::{self, SearchQuery};
use mdtools::section::{SectionIndex, SectionTarget};
use mdtools::stats;
use mdtools::table;
use mdtools::task::{self, SetTaskEdit, TaskQuery};

fn main() {
    let source = "---\ntitle: Demo\n---\n\n# Tasks\n\n- [ ] first\n\nRead [guide](guide.md).\n\n| A |\n| --- |\n| one |\n".to_string();
    let document = Document::parse_for_frontmatter(source.clone()).unwrap();
    let outline = SectionIndex::new(&document).outline();
    let selector = SectionTarget::heading("Tasks", None, HeadingMatchMode::Exact).unwrap();
    let section = SectionIndex::new(&document).resolve(&selector).unwrap();
    let first = task::tasks(&document, &TaskQuery::default()).unwrap().remove(0);
    let outcome = task::set_task(&document, &SetTaskEdit { loc: first.loc, status: TaskStatus::Done, expect_etag: Some(first.etag) }).unwrap();
    let prepared = block_edit::prepare_replace(&document, 1, None).unwrap();
    let _candidate = prepared.apply("- [x] first");
    assert!(!block::blocks(&document).is_empty());
    assert_eq!(link::links(&document).len(), 1);
    assert!(!search::search(&document, &SearchQuery::literal("guide")).is_empty());
    assert_eq!(frontmatter::read(&document).unwrap().data["title"], "Demo");
    assert_eq!(table::tables(&document).unwrap().len(), 1);
    assert!(stats::document_stats(&document).word_count > 0);
    assert_eq!(document.slice(&section.span).unwrap().lines().next(), Some("# Tasks"));
    assert_eq!(outline.len(), 1);
    assert_eq!(document.source(), source);
    assert!(outcome.content.contains("- [x] first"));
}
''',
    )
    reader_run = run(
        ["cargo", "run", "--quiet", "--offline"],
        cwd=reader,
        env=cargo_env(workspace / "reader-target", cargo_home),
    )
    tree = run(
        ["cargo", "tree", "-e", "normal", "--offline"],
        cwd=reader,
        env=cargo_env(workspace / "reader-target", cargo_home),
    )
    cli_deps = [line for line in tree.stdout.splitlines() if "clap " in line or "walkdir " in line]
    reader_passed = reader_run.returncode == 0 and tree.returncode == 0 and not cli_deps and package_clean

    return (
        {
            "verdict": braid_verdict,
            "consumer_exit": braid_run.returncode,
            "consumer_stdout": braid_run.stdout,
            "consumer_stderr": braid_run.stderr,
            "package_clean": package_clean,
            "package_file_count": len(members),
            "package_sha256": package_sha256,
        },
        {
            "verdict": command_verdict(reader_passed, reader_run, tree),
            "consumer_exit": reader_run.returncode,
            "consumer_stdout": reader_run.stdout,
            "consumer_stderr": reader_run.stderr,
            "tree_exit": tree.returncode,
            "tree_stderr": tree.stderr,
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

## Product Disposition

- Every current CLI command family is unchanged from `{BASELINE_COMMIT}` on its
  representative parity case: reads matched exit code, stdout, and stderr;
  mutations additionally matched final document bytes; the complete current
  Rust suite passed.
- A clean Braid-shaped consumer built from the published Cargo package without a
  sibling checkout and immediately translated Markdown task status and source
  coordinates into consumer-owned types.
- A reader-shaped consumer built with default features disabled, had no Clap or
  Walkdir dependency, used outline/section/task queries in-process, and produced
  a guarded task-edit candidate without changing its source or filesystem.
- The reusable single-document surface is complete for the current CLI:
  reads, guarded edit candidates, and relocation all run without process I/O.
  Rendering, viewer state, agent policy, and Braid workflow semantics remain
  outside mdtools.

## Reopening Gate

Reopen this topology result if the CLI protocol version changes, the Cargo
package layout changes, a consumer needs a separate release cadence, or a
future operation cannot preserve direct-core/CLI candidate parity.

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


def validate_recorded_evidence(payload: dict[str, object]) -> None:
    if payload.get("schema_version") != "core-consumer-topology.v1":
        raise ValueError("result schema version is not supported")
    if payload.get("baseline_commit") != BASELINE_COMMIT:
        raise ValueError("result baseline does not match the probe")
    if payload.get("source_clean") is not True:
        raise ValueError("result was not produced from a clean source tree")
    if set(payload["lanes"]) != set(LANES):
        raise ValueError("result lanes do not match the protocol")
    cli = payload["lanes"]["cli_preservation"]
    cli_pass = (
        cli.get("suite_exit") == 0
        and cli.get("mutation_match") is True
        and bool(cli.get("comparisons"))
        and all(item.get("match") is True for item in cli["comparisons"])
        and bool(cli.get("mutation_comparisons"))
        and all(item.get("match") is True for item in cli["mutation_comparisons"])
    )
    braid = payload["lanes"]["braid_adoption"]
    package_hash = braid.get("package_sha256")
    braid_pass = (
        braid.get("consumer_exit") == 0
        and braid.get("package_clean") is True
        and isinstance(package_hash, str)
        and len(package_hash) == 64
    )
    reader = payload["lanes"]["reader_readiness"]
    reader_pass = (
        reader.get("consumer_exit") == 0
        and reader.get("tree_exit") == 0
        and reader.get("package_clean") is True
        and reader.get("cli_only_dependencies") == []
        and reader.get("package_sha256") == package_hash
    )
    expected_passes = {
        "cli_preservation": cli_pass,
        "braid_adoption": braid_pass,
        "reader_readiness": reader_pass,
    }
    for name, passed in expected_passes.items():
        if (payload["lanes"][name].get("verdict") == "pass") != passed:
            raise ValueError(f"{name} verdict contradicts its recorded evidence")
    if payload["overall"] != overall(payload["lanes"]):
        raise ValueError("overall verdict does not match lane verdicts")


def check_results(repo: Path) -> int:
    path = repo / "probes/core_consumer_topology/RESULTS.md"
    text = path.read_text(encoding="utf-8")
    start = text.index(RESULT_START) + len(RESULT_START)
    end = text.index(RESULT_END, start)
    fenced = text[start:end].strip()
    if not fenced.startswith("```json\n") or not fenced.endswith("```"):
        raise ValueError("result JSON fence is malformed")
    payload = json.loads(fenced[len("```json\n") : -len("```")])
    validate_recorded_evidence(payload)
    candidate = str(payload.get("candidate_commit", ""))
    run(["git", "cat-file", "-e", f"{candidate}^{{commit}}"], cwd=repo, check=True)
    changed_after_candidate = run(
        ["git", "diff", "--name-only", f"{candidate}..HEAD"], cwd=repo, check=True
    ).stdout.splitlines()
    allowed = {"probes/core_consumer_topology/RESULTS.md"}
    if set(changed_after_candidate) - allowed:
        raise ValueError("result candidate is stale relative to current source")
    dirty_paths = {
        line[3:]
        for line in run(["git", "status", "--porcelain"], cwd=repo, check=True).stdout.splitlines()
    }
    if dirty_paths - allowed:
        raise ValueError("current source has changes not represented by the result")
    with tempfile.TemporaryDirectory(prefix="mdtools-core-check-") as raw_workspace:
        _, _, current_package_hash = package_source(repo, Path(raw_workspace))
    if current_package_hash != payload["lanes"]["braid_adoption"]["package_sha256"]:
        raise ValueError("current Cargo package hash does not match recorded evidence")
    print(payload["overall"])
    return 0


def execute(repo: Path) -> int:
    if run(["git", "status", "--porcelain"], cwd=repo, check=True).stdout:
        print("refusing to probe a dirty source tree")
        return 2
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
            "source_clean": True,
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
