#!/usr/bin/env python3
"""mdtools benchmark harness.

Runs the benchmark task corpus in unix, mdtools, and hybrid modes,
scores results using dual scorers (independent markdown-it-py + md
binary), and produces a benchmark report with byte-cost metrics.

Usage:
    python bench/harness.py                      # dry run: validate scorer
    python bench/harness.py --run                 # agent track: N runs per task×mode
    python bench/harness.py --run --mode mdtools   # agent track: mdtools only
    python bench/harness.py --run -N 5            # 5 runs per task×mode
    python bench/harness.py --agent "claude -p"   # specify agent command
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, asdict, field, fields
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal

REPO_ROOT = Path(__file__).resolve().parent.parent

try:
    from bench.command_policy import (
        _md_ablation_stub,
        classify_command_kind,
        classify_command_verb,
        create_restricted_shell_env,
        load_guard_events,
        md_workdir_must_be_stub,
    )
    from bench.neutral_scorer import neutral_block_texts, neutral_heading_tree
    from bench.multifile_drift import (
        DRIFT_SPECS_PATH,
        drift_task_from_input_files,
        format_drift_proof,
        summarize_drift_proof,
    )
    from bench.oai_loop import LoopError, resolve_oai_model, run_oai_loop
    from bench.pi_audit_adapter import load_pi_audit_events, summarize_pi_audit_events
    from bench.pi_runner import build_pi_json_command, default_audit_extension_path, parse_pi_json_output
    from bench.v3_manifest import (
        MANIFEST_PATH,
        SCORER_VERSION,
        ManifestConformanceError,
        current_prompt_template_sha256,
        load_manifest,
        manifest_hash,
        sha256_file,
        validate_headline_run_request,
    )
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from bench.command_policy import (
        _md_ablation_stub,
        classify_command_kind,
        classify_command_verb,
        create_restricted_shell_env,
        load_guard_events,
        md_workdir_must_be_stub,
    )
    from bench.neutral_scorer import neutral_block_texts, neutral_heading_tree
    from bench.multifile_drift import (
        DRIFT_SPECS_PATH,
        drift_task_from_input_files,
        format_drift_proof,
        summarize_drift_proof,
    )
    from bench.oai_loop import LoopError, resolve_oai_model, run_oai_loop
    from bench.pi_audit_adapter import load_pi_audit_events, summarize_pi_audit_events
    from bench.pi_runner import build_pi_json_command, default_audit_extension_path, parse_pi_json_output
    from bench.v3_manifest import (
        MANIFEST_PATH,
        SCORER_VERSION,
        ManifestConformanceError,
        current_prompt_template_sha256,
        load_manifest,
        manifest_hash,
        sha256_file,
        validate_headline_run_request,
    )

# ── Types ─────────────────────────────────────────────────────

BenchMode = Literal[
    "unix", "mdtools", "hybrid", "hybrid-no-md",
    "native", "native+md", "native+md-no-md",  # native-rooted arm (claude-cli only; FRAC-194)
]
ScorerKind = Literal["structural", "normalized_text", "raw_bytes"]


@dataclass
class StructuralDiffPolicy:
    kind: ScorerKind
    normalize_line_endings: bool
    ignore_trailing_whitespace: bool
    compare_frontmatter_json: bool
    compare_heading_tree: bool
    compare_block_order: bool
    compare_link_destinations: bool
    compare_block_text: bool
    json_canonical: bool = False
    json_required_keys: list[str] | None = None


@dataclass
class BenchTask:
    id: str
    description: str
    input_files: list[str]
    expected_output: str
    expected_artifact: str
    difficulty: str
    scorer: StructuralDiffPolicy
    expected_stdout: str | None = None
    support_files: list[str] | None = None


@dataclass
class BenchResult:
    task_id: str
    mode: BenchMode
    correct: bool | None
    correct_neutral: bool = True  # Independent scorer agreement
    run_index: int | None = None
    correct_diagnostic: bool | None = None
    verdict: str = "pass"
    quarantine: dict[str, object] | None = None
    model: str | None = None
    thinking_level: str | None = None
    bytes_prompt: int = 0
    bytes_output: int = 0
    bytes_observation: int = 0   # total tool-result content agent had to read
    tool_calls: int = 0
    turns: int = 0
    tokens_in: int = 0           # prompt tokens from runner usage (0 if runner gives none)
    tokens_out: int = 0          # completion tokens from runner usage (0 if runner gives none)
    mutations: int = 0           # write tool calls (set-task, replace-*, insert-*, delete-*)
    tool_mix: dict[str, int] = field(default_factory=dict)  # per-verb tool-choice counts, e.g. {"md outline": 2, "sed": 1}
    md_probe_count: int = 0      # hybrid-no-md: times the agent invoked the soft md stub (clean-ablation gate)
    policy_violations: int = 0   # denied commands recorded by the guarded shell executor
    requeried: bool = False      # did agent re-read structure after a mutation?
    invalid_responses: int = 0   # oai-loop parse_action_text failures (action-format adherence)
    unique_invalid_responses: int = 0  # distinct assistant texts among the invalid_responses (deterministic-lock signal)
    elapsed_seconds: float = 0.0
    diff_report: str = ""
    runner_error: str | None = None


@dataclass
class ParsedAgentOutput:
    stdout: str
    model: str | None = None
    thinking_level: str | None = None
    tool_calls: int = 0
    turns: int = 0
    bytes_observation: int = 0
    tokens_in: int = 0
    tokens_out: int = 0
    tool_outputs: list[str] = field(default_factory=list)
    text_outputs: list[str] = field(default_factory=list)
    runner_error: str | None = None
    # claude-cli per-verb tool use parsed from the transcript (U4/U7): native
    # Read/Edit/Write PLUS classified Bash verbs (md/sed/grep/...). The Bash guard
    # does not fire for claude-cli's Bash tool (it doesn't source BASH_ENV), so for
    # claude-cli this transcript parse is the only adoption + mutation signal.
    transcript_tool_mix: dict[str, int] = field(default_factory=dict)
    transcript_mutations: int = 0
    # ordered query/mutation trajectory parsed from the transcript (FRAC-194 #4) — the
    # guard's call_sequence is empty for claude-cli, so requery detection (a query AFTER
    # a mutation) needs this. Native Edit/Write are mutations; Read and query verbs are
    # queries. Folded into the run's call_sequence in the claude-cli branch.
    transcript_call_sequence: list[str] = field(default_factory=list)


# ── Tool inventories ──────────────────────────────────────────

UNIX_TOOLS = ["cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp"]
MDTOOLS_TOOLS = [
    "md outline", "md blocks", "md block", "md section",
    "md replace-section", "md delete-section", "md replace-block",
    "md replace-table-row", "md delete-table-row",
    "md insert-block", "md delete-block",
    "md search", "md links", "md frontmatter", "md collect", "md stats",
    "md table", "md set", "md tasks", "md set-task", "cat", "jq",
]


# ── System prompts ────────────────────────────────────────────

SYSTEM_PROMPT_TEMPLATE = """\
You are a benchmark agent. Complete the task using ONLY the allowed tools.
Do NOT use any other commands. Do NOT install packages. Do NOT use python or node.

TASK: {task_description}
INPUT FILE: {input_file}
OUTPUT: {output_instruction}

{tool_docs}

The `md` binary is available at `./md` in the working directory.

Work step by step. When done, {completion_instruction}
"""

MDTOOLS_DOCS = """\
TOOL REFERENCE — md (markdown-aware CLI):

  md outline <FILE> [--json]           Show heading hierarchy with line spans
  md blocks <FILE> [--json]            List all top-level blocks with index, kind, span, preview
  md block <INDEX> <FILE> [--json]     Read a single block's full content by 0-based index
  md section <SELECTOR> <FILE> [--json] [--ignore-case] [--occurrence N]
                                       Read a section by heading text. Use ":preamble" for content before first heading.
                                       Duplicate headings require --occurrence (1-based).
  md search <QUERY> <FILE> [--json] [--ignore-case] [--kind <KIND>...]
                                       Search block content. Filter by kind: heading, paragraph, list, code-fence, etc.
  md replace-block <INDEX> <FILE> [-i] [--json] [--from PATH]
                                       Replace block at INDEX. Reads from --from or stdin.
  md replace-section <SELECTOR> <FILE> [-i] [--json] [--ignore-case] [--occurrence N] [--from PATH]
                                       Replace section. Reads from --from or stdin.
  md replace-table-row <TABLE_BLOCK_INDEX> <ROW_INDEX> <FILE> [-i] [--json] [--from PATH] [--expect-etag ETAG]
                                       Replace one table data row. Read the table etag with `md table --json` first.
  md delete-table-row <TABLE_BLOCK_INDEX> <ROW_INDEX> <FILE> [-i] [--json] [--expect-etag ETAG]
                                       Delete one table data row. Selector-only mutation; no stdin or --from.
  md insert-block <FILE> [-i] --before <INDEX> | --after <INDEX> | --at-start | --at-end [--from PATH]
                                       Insert a new block. Reads from --from or stdin.
  md delete-block <INDEX> <FILE> [-i]  Delete block at INDEX.
  md delete-section <SELECTOR> <FILE> [-i] [--json] [--ignore-case] [--occurrence N]
                                       Delete an entire section (heading + content).
  md move-section — move a heading and its entire section body as one atomic operation.
      Use this whenever a task asks to move, relocate, nest, reorder, or place a
      section/heading somewhere else. Prefer over manually extracting, deleting, and
      re-inserting. Do NOT use delete-section + insert-block for section relocation.
      Canonical forms:
        md move-section --into "DEST" "SOURCE" FILE -i   # SOURCE becomes last child of DEST
        md move-section --after "DEST" "SOURCE" FILE -i  # SOURCE becomes next sibling after DEST
        md move-section --before "DEST" "SOURCE" FILE -i # SOURCE becomes prev sibling before DEST
      Mapping from task wording:
        "move X under Y" / "move X into Y" / "nest X inside Y"  →  --into "Y"
        "move X after Y" / "place X after Y"                    →  --after "Y"
        "move X before Y" / "place X above Y"                   →  --before "Y"
      --auto-level (default): adjust SOURCE heading levels to fit destination hierarchy.
      --keep-level: preserve SOURCE heading levels exactly (use only when task says so).
      --ignore-case: case-insensitive heading matching.
      --source-occurrence N / --dest-occurrence N: disambiguate duplicate heading names.
  md links <FILE> [--json]             List all links with kind, destination, source block
  md frontmatter <FILE> [--json]       Read YAML/TOML frontmatter as JSON
  md collect <FILE|DIR>... [--json]    Aggregate frontmatter into one ordered table; missing fields stay blank/null
  md stats <FILE> [--json]             Word/heading/block/link/section/line counts
  md table <FILE> [--json] [--index N] [--select COLS] [--where FILTER]
                                       List tables or read/project a specific table.
  md set <KEY> <FILE> <VALUE> [-i] [--json]
                                       Set a frontmatter field by dot-path. Use --delete to remove a field.
  md tasks <FILE> [--json] [--status pending|done] [-r]
                                       List GFM checkbox task items with loc, status, depth,
                                       nearest heading, and summary text. Loc is a dot-path
                                       like "9.0" or "14.4.0" for nested tasks.
  md set-task <LOC> <FILE> [-i] [--json] --status done|pending
                                       Set checkbox state of a task item by loc (from md tasks).
                                       1-byte mutation, idempotent. Use with -i to write in-place.
  cat <FILE>                           Read raw file contents
  jq <FILTER> [FILE]                   Process JSON (pipe from md --json commands)

EXAMPLES:
  md outline doc.md --json                          # heading tree as JSON
  md blocks doc.md                                  # list blocks: index, kind, lines, preview
  md replace-block 3 doc.md -i --from new.md  # replace block 3 from file
  md replace-section "Old" doc.md -i --from new.md
  md insert-block --after 2 doc.md -i --from new.md
  md move-section --into "API Reference" "Auth" doc.md -i   # make Auth a subsection of API Reference
  md move-section --after "Installation" "Setup" doc.md -i  # move Setup as sibling after Installation
  md delete-section "Notes" doc.md -i                       # delete entire section
  md search "method" doc.md --kind paragraph --json # find "method" in paragraphs only
  md table report.md --select Feature,Status         # project table columns as TSV
  md collect vault/ -r --field title,status         # aggregate frontmatter rows as TSV/JSON with path-first headers
  md set release.channel doc.md stable -i            # set YAML/TOML frontmatter field
  md tasks doc.md --status pending --json           # list pending task items
  md set-task 5.1 doc.md -i --status done           # mark task at loc 5.1 as done
"""

UNIX_DOCS = """\
ALLOWED TOOLS (standard POSIX):
  cat, grep, sed, awk, head, tail, wc, tee, mv, cp
  Shell: pipes (|), redirection (>, >>), temp files (mktemp)

Do NOT use: python, perl, ruby, node, or any other scripting language.
"""

# HYBRID_DOCS is a LEAN standalone literal (not MDTOOLS_DOCS) because the hybrid
# agent also has unix tools and re-reads this prompt every turn — a verbose md
# reference is O(turns) in token cost (cache_read/creation) without changing
# behavior. Keep every md subcommand discoverable but terse; keep md the obvious
# choice for STRUCTURAL ops (so md adds attributable lift, not neutered).
HYBRID_DOCS = """\
TOOLS — you have BOTH `md` (a markdown-aware CLI) and standard POSIX tools.

`md` subcommands (most take --json; pipe into jq for filtering):
  md outline F                     heading tree with line spans
  md blocks F  /  md block N F      list top-level blocks  /  read block N (0-based)
  md section "H" F                 read a section by heading (":preamble" = before 1st heading; --occurrence N for duplicate headings)
  md search Q F [--kind paragraph|heading|list|code-fence]
  md tasks F                       list GFM checkbox tasks with loc (e.g. 9.0, 14.4.0)
  md set-task LOC F -i --status done|pending      toggle a checkbox by loc
  md frontmatter F  /  md set KEY F VAL -i         read / set YAML-or-TOML frontmatter (dot-path; --delete removes)
  md collect F... [-r] [--field FIELDS]            aggregate frontmatter rows across multiple files/dirs into one table
  md table F [--select COLS] [--where "Col=val"]  /  md links F  /  md stats F
  md replace-table-row TABLE ROW F -i --from PATH [--expect-etag ETAG]
  md delete-table-row TABLE ROW F -i [--expect-etag ETAG]
  md replace-block N F -i --from PATH              replace block N
  md replace-section "H" F -i --from PATH          replace a section's body
  md insert-block F -i --after N|--before N|--at-start|--at-end --from PATH
  md delete-block N F -i  /  md delete-section "H" F -i
  md move-section --into|--after|--before "DEST" "SRC" F -i   atomic heading+body relocate

POSIX (also available): cat, grep, sed, awk, head, tail, wc, tee, mv, cp; pipes |, redirection >, >>; mktemp; jq.
Do NOT use: python, perl, ruby, node, or any other scripting language.

Choose the best tool for each step: `md` handles structural markdown operations
(sections, blocks, GFM tasks, tables, frontmatter, section moves) and POSIX tools
handle plain line/text work. If `md` is unavailable, the POSIX tools cover the
same tasks — fall back to them cleanly rather than retrying `md`.
"""

# NATIVE_DOCS / NATIVE_MD_DOCS — the native-rooted arm (claude-cli only; FRAC-194).
# The agent ALSO has its native Read/Edit/Write tools (enabled in _build_agent_cmd);
# these prompts advertise the shell/md surface and name the native tools as a
# co-equal option, so tool choice is free ("any tool at hand"). NATIVE_MD_DOCS is
# byte-identical for native+md and native+md-no-md — same anti-gaming discipline as
# HYBRID_DOCS: the clean ablation must not be distinguishable from the prompt.
NATIVE_DOCS = """\
TOOLS — you have your native file tools (Read, Edit, Write) for reading and editing
files directly, plus standard POSIX shell tools:
  cat, grep, sed, awk, head, tail, wc, tee, mv, cp; pipes (|), redirection (>, >>); mktemp.

Do NOT use: python, perl, ruby, node, or any other scripting language.

Choose the best tool for each step: your native Read/Edit/Write handle reading and
editing files; POSIX tools handle search and plain line/text work.
"""

NATIVE_MD_DOCS = """\
TOOLS — you have your native file tools (Read, Edit, Write), `md` (a markdown-aware CLI), and standard POSIX tools.

`md` subcommands (most take --json; pipe into jq for filtering):
  md outline F                     heading tree with line spans
  md blocks F  /  md block N F      list top-level blocks  /  read block N (0-based)
  md section "H" F                 read a section by heading (":preamble" = before 1st heading; --occurrence N for duplicate headings)
  md search Q F [--kind paragraph|heading|list|code-fence]
  md tasks F                       list GFM checkbox tasks with loc (e.g. 9.0, 14.4.0)
  md set-task LOC F -i --status done|pending      toggle a checkbox by loc
  md frontmatter F  /  md set KEY F VAL -i         read / set YAML-or-TOML frontmatter (dot-path; --delete removes)
  md collect F... [-r] [--field FIELDS]            aggregate frontmatter rows across multiple files/dirs into one table
  md table F [--select COLS] [--where "Col=val"]  /  md links F  /  md stats F
  md replace-table-row TABLE ROW F -i --from PATH [--expect-etag ETAG]
  md delete-table-row TABLE ROW F -i [--expect-etag ETAG]
  md replace-block N F -i --from PATH              replace block N
  md replace-section "H" F -i --from PATH          replace a section's body
  md insert-block F -i --after N|--before N|--at-start|--at-end --from PATH
  md delete-block N F -i  /  md delete-section "H" F -i
  md move-section --into|--after|--before "DEST" "SRC" F -i   atomic heading+body relocate

POSIX (also available): cat, grep, sed, awk, head, tail, wc, tee, mv, cp; pipes |, redirection >, >>; mktemp; jq.
Native file tools (also available): Read, Edit, Write — read and edit files directly.
Do NOT use: python, perl, ruby, node, or any other scripting language.

Choose the best tool for each step: your native Read/Edit/Write handle reading and
editing files, `md` handles structural markdown operations (sections, blocks, GFM
tasks, tables, frontmatter, section moves), and POSIX tools handle plain line/text
work. If `md` is unavailable, your native tools and the POSIX tools cover the same
tasks — fall back to them cleanly rather than retrying `md`.
"""


def build_prompt(task: BenchTask, mode: BenchMode, workdir: str) -> str:
    if mode == "mdtools":
        tool_docs = MDTOOLS_DOCS
    elif mode in ("hybrid", "hybrid-no-md"):
        # hybrid-no-md gets the SAME prompt as hybrid (md advertised); md is just
        # absent from its toolset, so the only difference is md availability.
        tool_docs = HYBRID_DOCS
    elif mode == "native":
        tool_docs = NATIVE_DOCS
    elif mode in ("native+md", "native+md-no-md"):
        # native+md-no-md gets the SAME (byte-identical) prompt as native+md; md is
        # just the soft stub, so the only difference is md availability.
        tool_docs = NATIVE_MD_DOCS
    else:
        tool_docs = UNIX_DOCS

    # Determine input reference for prompt
    if len(task.input_files) > 1:
        inp_dir = os.path.basename(os.path.dirname(task.input_files[0]))
        if inp_dir and inp_dir != "inputs":
            input_file = os.path.join(workdir, inp_dir) + "/"
        else:
            input_file = " ".join(
                copied_input_path(f, workdir) for f in task.input_files
            )
    else:
        input_file = copied_input_path(task.input_files[0], workdir)

    if task.expected_artifact == "json_envelope":
        output_instruction = "Print the result as JSON to stdout."
        completion_instruction = "print the JSON result and stop."
    elif task.expected_artifact == "stdout_text":
        output_instruction = "Print the requested text result to stdout."
        completion_instruction = "print the final text result and stop."
    elif task.expected_artifact == "file_contents":
        output_instruction = f"Modify the file {input_file} in place."
        completion_instruction = "confirm the file has been modified and stop."
    elif task.expected_artifact == "stdout_and_file":
        output_instruction = (
            f"If you make a change, modify {input_file} in place and print the loc. "
            f"If you cannot make a safe change, print exactly AMBIGUOUS (nothing else) and do not modify the file."
        )
        completion_instruction = "print your result and stop."
    elif task.expected_artifact == "multi_file_contents_any":
        output_instruction = f"Modify the provided files under {input_file} in place."
        completion_instruction = "confirm the files have been handled and stop."
    else:
        output_instruction = "Print the result to stdout."
        completion_instruction = "print the result and stop."

    return SYSTEM_PROMPT_TEMPLATE.format(
        task_description=task.description,
        input_file=input_file,
        output_instruction=output_instruction,
        tool_docs=tool_docs,
        completion_instruction=completion_instruction,
    )


def copied_input_path(input_path: str, workdir: str) -> str:
    """Return the path where run_agent copies an input file inside workdir."""
    return os.path.join(workdir, copied_input_relpath(input_path))


def copied_input_relpath(input_path: str) -> Path:
    """Return the relative path used when an input file is copied into the workdir."""
    inp_dir = os.path.dirname(input_path)
    inp_parent = os.path.basename(inp_dir) if inp_dir else ""
    if inp_parent and inp_parent != "inputs":
        return Path(inp_parent) / os.path.basename(input_path)
    return Path(os.path.basename(input_path))


def copy_final_input_snapshot(task: BenchTask, actual_root: Path, run_log_dir: Path) -> None:
    """Preserve final benchmark input files before the temp workdir is removed."""
    final_dir = run_log_dir / "final"
    for input_path in task.input_files:
        rel = copied_input_relpath(input_path)
        actual_path = actual_root / rel
        if not actual_path.exists():
            continue
        dest = final_dir / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(actual_path, dest)


# ── Dual scorer ───────────────────────────────────────────────

def score_task(
    task: BenchTask,
    actual: str,
    expected: str,
    md_binary: str = "md",
) -> tuple[bool | None, bool | None, str]:
    """Score with independent primary authority plus an md diagnostic when available.

    Returns (correct_primary, correct_diagnostic, diff_report). A divergent
    primary/diagnostic pair is turned into a durable quarantine by run_agent.
    """
    policy = task.scorer
    report = []

    if policy.kind == "raw_bytes":
        # Normalize before raw comparison if requested
        a, e = actual, expected
        if policy.normalize_line_endings:
            a = a.replace("\r\n", "\n")
            e = e.replace("\r\n", "\n")
        if policy.ignore_trailing_whitespace:
            a = "\n".join(line.rstrip() for line in a.split("\n")).rstrip()
            e = "\n".join(line.rstrip() for line in e.split("\n")).rstrip()
        ok_primary = a == e
        if ok_primary:
            report.append("exact: OK")
        else:
            report.append(f"raw_bytes: MISMATCH ({len(e)}b expected, {len(a)}b actual)")
        return ok_primary, ok_primary, "\n".join(report)

    # Normalize
    a, e = actual, expected
    if policy.normalize_line_endings:
        a = a.replace("\r\n", "\n")
        e = e.replace("\r\n", "\n")
    if policy.ignore_trailing_whitespace:
        a = "\n".join(line.rstrip() for line in a.split("\n"))
        e = "\n".join(line.rstrip() for line in e.split("\n"))

    if policy.kind == "structural" and policy.json_canonical:
        ok_primary, ok_diagnostic = score_json_canonical(policy, a, e, report)
        return ok_primary, ok_diagnostic, "\n".join(report)

    if policy.kind == "structural" and task.expected_artifact == "json_envelope":
        ok_primary, ok_diagnostic = score_structural_json(policy, a, e, report)
        return ok_primary, ok_diagnostic, "\n".join(report)

    if policy.kind == "normalized_text":
        ok_primary = score_normalized_text_neutral(policy, a, e, report)
        try:
            ok_diagnostic = score_normalized_text_md(policy, a, e, md_binary, report)
        except Exception as exc:  # noqa: BLE001 - diagnostic scorer failures block publication
            ok_diagnostic = None
            report.append(f"diagnostic_md_error: {type(exc).__name__}: {exc}")
        if ok_primary != ok_diagnostic:
            report.append(f"SCORER DIVERGENCE: primary={ok_primary} diagnostic_md={ok_diagnostic}")
        return ok_primary, ok_diagnostic, "\n".join(report)

    ok = a.strip() == e.strip()
    return ok, ok, "\n".join(report)


def _multifile_expected_alternatives(expected_output: str) -> list[Path]:
    root = Path(expected_output)
    if not root.exists():
        return [root]
    children = sorted(path for path in root.iterdir() if path.is_dir())
    return children if children else [root]


def _score_multifile_against_dir(
    task: BenchTask,
    *,
    actual_root: Path,
    expected_root: Path,
    md_binary: str,
) -> tuple[bool | None, bool | None, str]:
    report_parts: list[str] = []
    primary_results: list[bool | None] = []
    diagnostic_results: list[bool | None] = []
    for input_path in task.input_files:
        rel = copied_input_relpath(input_path)
        actual_path = actual_root / rel
        expected_path = expected_root / rel
        if not expected_path.exists():
            report_parts.append(f"{rel}: expected file missing in {expected_root}")
            primary_results.append(False)
            diagnostic_results.append(False)
            continue
        try:
            actual = actual_path.read_text()
        except FileNotFoundError:
            actual = ""
        expected = expected_path.read_text()
        ok_primary, ok_diagnostic, file_report = score_task(task, actual, expected, md_binary)
        primary_results.append(ok_primary)
        diagnostic_results.append(ok_diagnostic)
        if ok_primary and ok_diagnostic:
            report_parts.append(f"{rel}: OK")
        else:
            report_parts.append(f"{rel}: {file_report}")
    primary = all(result is True for result in primary_results)
    diagnostic = all(result is True for result in diagnostic_results)
    return primary, diagnostic, "\n".join(report_parts)


def score_multifile_any(
    task: BenchTask,
    *,
    actual_root: Path,
    expected_output: str,
    md_binary: str,
) -> tuple[bool | None, bool | None, str]:
    """Pass when all input files match any expected alternative directory."""
    reports: list[str] = []
    for alternative in _multifile_expected_alternatives(expected_output):
        ok_primary, ok_diagnostic, report = _score_multifile_against_dir(
            task,
            actual_root=actual_root,
            expected_root=alternative,
            md_binary=md_binary,
        )
        if ok_primary and ok_diagnostic:
            return True, True, f"multi_file_contents_any: matched {alternative.name}"
        reports.append(f"[{alternative.name}]\n{report}")
    return (
        False,
        False,
        "multi_file_contents_any: no alternative matched\n" + "\n".join(reports),
    )


def _canonicalize_json(obj):
    """Recursively sort object keys for canonical comparison."""
    if isinstance(obj, dict):
        return {k: _canonicalize_json(v) for k, v in sorted(obj.items())}
    if isinstance(obj, list):
        return [_canonicalize_json(item) for item in obj]
    return obj


def _project_keys(obj, keys: list[str]):
    """Project an object or array of objects to only the specified keys."""
    if isinstance(obj, list):
        return [_project_keys(item, keys) for item in obj]
    if isinstance(obj, dict):
        return {k: obj[k] for k in keys if k in obj}
    return obj


def score_json_canonical(
    policy: StructuralDiffPolicy,
    actual_str: str,
    expected_str: str,
    report: list[str],
) -> tuple[bool, bool]:
    """Score JSON output with a canonical invariant comparison."""
    try:
        actual = json.loads(actual_str)
        expected = json.loads(expected_str)
    except json.JSONDecodeError as exc:
        report.append(f"JSON parse error: {exc}")
        return False, False

    if policy.json_required_keys:
        actual = _project_keys(actual, policy.json_required_keys)
        expected = _project_keys(expected, policy.json_required_keys)

    actual_c = _canonicalize_json(actual)
    expected_c = _canonicalize_json(expected)

    if actual_c == expected_c:
        report.append("json_canonical: OK")
        return True, True

    # Report first mismatch
    a_str = json.dumps(actual_c, indent=2, ensure_ascii=False)
    e_str = json.dumps(expected_c, indent=2, ensure_ascii=False)
    # Find first differing line
    a_lines = a_str.splitlines()
    e_lines = e_str.splitlines()
    for i, (al, el) in enumerate(zip(a_lines, e_lines)):
        if al != el:
            report.append(f"json_canonical: MISMATCH at line {i+1}")
            report.append(f"  expected: {el}")
            report.append(f"  actual:   {al}")
            break
    else:
        report.append(f"json_canonical: MISMATCH (length: expected {len(e_lines)} lines, actual {len(a_lines)} lines)")

    return False, False


def score_structural_json(
    policy: StructuralDiffPolicy,
    actual_str: str,
    expected_str: str,
    report: list[str],
) -> tuple[bool, bool]:
    """Score JSON envelope using invariant checks independent of the md binary."""
    try:
        actual_json = json.loads(actual_str)
        expected_json = json.loads(expected_str)
    except json.JSONDecodeError as exc:
        report.append(f"JSON parse error: {exc}")
        return False, False

    # F3 closure: when compare_link_destinations is the sole structural check,
    # accept a top-level JSON array as equivalent to {"links": [...]}. The task
    # description is intentionally tool-neutral and unix-mode agents reasonably
    # emit a list directly; the structural scorer must not mode-contaminate.
    only_link_destinations = (
        policy.compare_link_destinations
        and not policy.compare_heading_tree
        and not policy.compare_frontmatter_json
        and not policy.compare_block_order
        and not policy.compare_block_text
    )
    if only_link_destinations:
        if isinstance(actual_json, list):
            actual_json = {"links": actual_json}
        if isinstance(expected_json, list):
            expected_json = {"links": expected_json}

    if not isinstance(actual_json, dict) or not isinstance(expected_json, dict):
        report.append(
            "json_envelope: MISMATCH expected top-level JSON object "
            f"(actual={type(actual_json).__name__}, expected={type(expected_json).__name__})"
        )
        return False, False

    ok_primary = True
    ok_diagnostic = True

    if policy.compare_heading_tree:
        # JSON envelope invariant: compare the emitted heading tree.
        a_tree = [(e["heading"]["level"], e["heading"]["text"]) for e in actual_json.get("entries", [])]
        e_tree = [(e["heading"]["level"], e["heading"]["text"]) for e in expected_json.get("entries", [])]
        if a_tree != e_tree:
            ok_primary = False
            ok_diagnostic = False
            report.append("heading_tree [primary]: MISMATCH")
        else:
            report.append("heading_tree [primary]: OK")

    if policy.compare_frontmatter_json:
        actual_present = actual_json.get("present")
        expected_present = expected_json.get("present")
        actual_fm = actual_json.get("frontmatter") or {}
        expected_fm = expected_json.get("frontmatter") or {}
        actual_data = _canonicalize_json(actual_fm.get("data"))
        expected_data = _canonicalize_json(expected_fm.get("data"))
        actual_format = actual_fm.get("format")
        expected_format = expected_fm.get("format")

        if (
            actual_present != expected_present
            or actual_format != expected_format
            or actual_data != expected_data
        ):
            ok_primary = False
            ok_diagnostic = False
            report.append("frontmatter_json: MISMATCH")
        else:
            report.append("frontmatter_json: OK")

    if policy.compare_link_destinations:
        actual_links = [
            (link.get("kind"), link.get("destination"))
            for link in actual_json.get("links", [])
        ]
        expected_links = [
            (link.get("kind"), link.get("destination"))
            for link in expected_json.get("links", [])
        ]

        if actual_links != expected_links:
            ok_primary = False
            ok_diagnostic = False
            report.append(f"link_destinations: MISMATCH {expected_links} vs {actual_links}")
        else:
            report.append("link_destinations: OK")

    return ok_primary, ok_diagnostic


def score_normalized_text_md(
    policy: StructuralDiffPolicy,
    actual: str,
    expected: str,
    md_binary: str,
    report: list[str],
) -> bool:
    """Score using md binary (may be circular)."""
    correct = True

    if policy.compare_heading_tree:
        a = _md_heading_tree(actual, md_binary)
        e = _md_heading_tree(expected, md_binary)
        if a != e:
            correct = False
            report.append(f"heading_tree [md]: MISMATCH {e} vs {a}")
        else:
            report.append("heading_tree [md]: OK")

    if policy.compare_block_order:
        a = _md_block_order(actual, md_binary)
        e = _md_block_order(expected, md_binary)
        if a != e:
            correct = False
            report.append(f"block_order [md]: MISMATCH {e} vs {a}")
        else:
            report.append("block_order [md]: OK")

    if policy.compare_block_text:
        a = _md_block_texts(actual, md_binary)
        e = _md_block_texts(expected, md_binary)
        if a != e:
            correct = False
            report.append("block_text [md]: MISMATCH")
            for i, (at, et) in enumerate(zip(a, e)):
                if at != et:
                    report.append(f"  block {i}: {et[:60]!r} vs {at[:60]!r}")
                    break
        else:
            report.append("block_text [md]: OK")

    return correct


def score_normalized_text_neutral(
    policy: StructuralDiffPolicy,
    actual: str,
    expected: str,
    report: list[str],
) -> bool:
    """Score using markdown-it-py (independent, not circular)."""
    correct = True

    if policy.compare_heading_tree:
        a = neutral_heading_tree(actual)
        e = neutral_heading_tree(expected)
        if a != e:
            correct = False
            report.append(f"heading_tree [neutral]: MISMATCH {e} vs {a}")
        else:
            report.append("heading_tree [neutral]: OK")

    if policy.compare_block_text:
        a = neutral_block_texts(actual)
        e = neutral_block_texts(expected)
        if a != e:
            correct = False
            report.append("block_text [neutral]: MISMATCH")
            for i, (at, et) in enumerate(zip(a, e)):
                if at != et:
                    report.append(f"  block {i}: {et[:60]!r} vs {at[:60]!r}")
                    break
            if len(a) != len(e):
                report.append(f"  count: {len(e)} vs {len(a)}")
        else:
            report.append("block_text [neutral]: OK")

    return correct


# ── md binary helpers ─────────────────────────────────────────

def _md_run(content: str, md_binary: str, args: list[str]) -> str | None:
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
        f.write(content)
        f.flush()
        try:
            r = subprocess.run(
                [md_binary] + args + [f.name, "--json"],
                capture_output=True, text=True, timeout=10,
            )
            return r.stdout if r.returncode == 0 else None
        finally:
            os.unlink(f.name)


def _md_heading_tree(content: str, md_binary: str) -> list[tuple[int, str]]:
    out = _md_run(content, md_binary, ["outline"])
    if not out:
        return []
    data = json.loads(out)
    return [(e["heading"]["level"], e["heading"]["text"]) for e in data.get("entries", [])]


def _md_block_order(content: str, md_binary: str) -> list[str]:
    out = _md_run(content, md_binary, ["blocks"])
    if not out:
        return []
    return [b["kind"] for b in json.loads(out).get("blocks", [])]


def _md_block_texts(content: str, md_binary: str) -> list[str]:
    out = _md_run(content, md_binary, ["blocks"])
    if not out:
        return []
    data = json.loads(out)
    content_bytes = content.encode("utf-8")
    return [content_bytes[b["span"]["byte_start"]:b["span"]["byte_end"]].decode("utf-8").strip()
            for b in data.get("blocks", [])]


# ── Harness ───────────────────────────────────────────────────

def load_tasks(tasks_path: str = "bench/tasks/tasks.json") -> list[BenchTask]:
    with open(tasks_path) as f:
        raw = json.load(f)
    tasks = []
    for t in raw:
        scorer_data = dict(t["scorer"])
        scorer = StructuralDiffPolicy(**scorer_data)
        tasks.append(BenchTask(
            id=t["id"], description=t["description"],
            input_files=t["input_files"], expected_output=t["expected_output"],
            expected_artifact=t["expected_artifact"], difficulty=t["difficulty"],
            scorer=scorer,
            expected_stdout=t.get("expected_stdout"),
            support_files=t.get("support_files"),
        ))
    return tasks


def load_task_ids(task_ids_path: str) -> list[str]:
    with open(task_ids_path) as f:
        raw = json.load(f)

    if not isinstance(raw, list) or not all(isinstance(task_id, str) and task_id for task_id in raw):
        raise ValueError(f"{task_ids_path} must be a JSON array of non-empty task IDs")

    task_ids: list[str] = []
    seen: set[str] = set()
    for task_id in raw:
        if task_id in seen:
            raise ValueError(f"duplicate task ID in {task_ids_path}: {task_id}")
        seen.add(task_id)
        task_ids.append(task_id)
    return task_ids


def select_tasks(tasks: list[BenchTask], task_ids: list[str]) -> list[BenchTask]:
    by_id = {task.id: task for task in tasks}
    missing = [task_id for task_id in task_ids if task_id not in by_id]
    if missing:
        raise ValueError(f"unknown task IDs in selection: {', '.join(missing)}")
    return [by_id[task_id] for task_id in task_ids]


# ── Holdout immutability guard (L1 closure) ─────────────────────
# A task in the holdout split must not change without a holdout-repair
# exception path being followed (ledger entry + holdout_version bump).
# This guard fingerprints the canonical task JSON entry and the bytes of
# every input/expected/support file. Any divergence between the live
# tasks file and bench/holdout/fingerprints.json is a holdout-immutability
# breach (P0 oracle-trustworthiness disturbance).

def _sha256_file(path: str) -> str:
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def _sha256_task_json(task_entry: dict) -> str:
    canonical = json.dumps(task_entry, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def compute_task_fingerprint(task_entry: dict) -> dict:
    """Fingerprint a raw task entry (the dict from tasks.json) plus its referenced files."""
    fp = {
        "task_json_sha256": _sha256_task_json(task_entry),
        "input_files_sha256": {p: _sha256_file(p) for p in task_entry["input_files"]},
        "expected_output_sha256": _sha256_file(task_entry["expected_output"]),
    }
    if task_entry.get("support_files"):
        fp["support_files_sha256"] = {p: _sha256_file(p) for p in task_entry["support_files"]}
    return fp


def load_holdout_fingerprints(path: str = "bench/holdout/fingerprints.json") -> dict:
    with open(path) as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must be a JSON object")
    if "holdout_version" not in data or not isinstance(data["holdout_version"], int):
        raise ValueError(f"{path} missing integer 'holdout_version'")
    if "fingerprints" not in data or not isinstance(data["fingerprints"], dict):
        raise ValueError(f"{path} missing object 'fingerprints'")
    return data


def check_holdout_integrity(
    tasks_path: str = "bench/tasks/tasks.json",
    holdout_ids_path: str = "bench/holdout/task_ids.json",
    fingerprints_path: str = "bench/holdout/fingerprints.json",
) -> str | None:
    """Runtime wrapper around verify_holdout_fingerprints for harness invocation.

    Returns None if no drift was detected or if the holdout split is not
    configured in this checkout (either holdout_ids_path or fingerprints_path
    missing). Returns the breach message string when drift is detected, so
    the caller can surface it via parser.error or equivalent.

    Skipped silently for missing files because mdtools forks may opt out of
    the holdout split entirely; the cheap-channel unit test still pins live
    repo integrity. The runtime check exists to convert L1 from procedural
    to mechanical for any harness invocation that runs against a configured
    holdout split.
    """
    if not (os.path.exists(holdout_ids_path) and os.path.exists(fingerprints_path)):
        return None
    try:
        verify_holdout_fingerprints(
            tasks_path=tasks_path,
            holdout_ids_path=holdout_ids_path,
            fingerprints_path=fingerprints_path,
        )
    except ValueError as exc:
        return str(exc)
    return None


def read_holdout_version(
    fingerprints_path: str = "bench/holdout/fingerprints.json",
) -> int | None:
    """Return the integer ``holdout_version`` from the fingerprints manifest.

    Returns None if the manifest file is absent (fork compat) or malformed,
    so callers can stamp the value onto run.json metadata when present and
    leave it null otherwise. The spec's holdout-repair exception path
    requires bundles to carry the version under which they were produced;
    this helper provides the single authoritative read used at run start.
    """
    if not os.path.exists(fingerprints_path):
        return None
    try:
        manifest = load_holdout_fingerprints(fingerprints_path)
    except (ValueError, OSError):
        return None
    return int(manifest["holdout_version"])


def verify_holdout_fingerprints(
    tasks_path: str = "bench/tasks/tasks.json",
    holdout_ids_path: str = "bench/holdout/task_ids.json",
    fingerprints_path: str = "bench/holdout/fingerprints.json",
) -> None:
    """Raise ValueError if any holdout task drifted from the recorded fingerprint.

    To legitimately mutate a holdout task, follow the holdout-repair exception
    path in the frontier-loop spec: file a P0 ledger entry, bump
    `holdout_version` in fingerprints.json, regenerate fingerprints, and mark
    prior holdout results non-comparable in bench/RESULTS.md.
    """
    holdout_ids = load_task_ids(holdout_ids_path)
    manifest = load_holdout_fingerprints(fingerprints_path)
    expected = manifest["fingerprints"]

    with open(tasks_path) as fh:
        raw_tasks = json.load(fh)
    by_id = {t["id"]: t for t in raw_tasks}

    drift: list[str] = []
    for tid in holdout_ids:
        if tid not in by_id:
            drift.append(f"{tid}: holdout task missing from {tasks_path}")
            continue
        if tid not in expected:
            drift.append(f"{tid}: no fingerprint baseline in {fingerprints_path}")
            continue
        live = compute_task_fingerprint(by_id[tid])
        baseline = expected[tid]
        if live != baseline:
            diffs = []
            for key in sorted(set(live) | set(baseline)):
                if live.get(key) != baseline.get(key):
                    diffs.append(key)
            drift.append(f"{tid}: fingerprint drift in fields {diffs}")

    extra = sorted(set(expected) - set(holdout_ids))
    if extra:
        drift.append(
            f"fingerprint baseline contains task IDs not in holdout split: {extra}"
        )

    if drift:
        raise ValueError(
            "holdout-immutability breach (holdout_version="
            f"{manifest['holdout_version']}): "
            + "; ".join(drift)
            + " — follow the holdout-repair exception path before reporting holdout results."
        )


def _iso_timestamp(timestamp: float) -> str:
    return datetime.fromtimestamp(timestamp, tz=timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def aggregate_results(results: list[BenchResult]) -> dict[str, float | int]:
    total = len(results)
    if total == 0:
        return {
            "runs": 0,
            "pass_count": 0,
            "pass_rate": 0.0,
            "neutral_pass_count": 0,
            "neutral_pass_rate": 0.0,
            "avg_elapsed_seconds": 0.0,
            "avg_tool_calls": 0.0,
            "avg_bytes_output": 0.0,
            "avg_bytes_observation": 0.0,
            "avg_mutations": 0.0,
            "avg_policy_violations": 0.0,
            "avg_invalid_responses": 0.0,
            "avg_unique_invalid_responses": 0.0,
            "requery_rate": 0.0,
        }

    return {
        "runs": total,
        "pass_count": sum(1 for result in results if result.correct),
        "pass_rate": sum(1 for result in results if result.correct) / total,
        "neutral_pass_count": sum(1 for result in results if result.correct_neutral),
        "neutral_pass_rate": sum(1 for result in results if result.correct_neutral) / total,
        "avg_elapsed_seconds": sum(result.elapsed_seconds for result in results) / total,
        "avg_tool_calls": sum(result.tool_calls for result in results) / total,
        "avg_bytes_output": sum(result.bytes_output for result in results) / total,
        "avg_bytes_observation": sum(result.bytes_observation for result in results) / total,
        "avg_mutations": sum(result.mutations for result in results) / total,
        "avg_policy_violations": sum(result.policy_violations for result in results) / total,
        "avg_invalid_responses": sum(result.invalid_responses for result in results) / total,
        "avg_unique_invalid_responses": sum(result.unique_invalid_responses for result in results) / total,
        "requery_rate": sum(1 for result in results if result.requeried) / total,
    }


def build_run_metadata(
    *,
    run_kind: str,
    tasks_path: str,
    task_ids_path: str | None,
    selected_task_ids: list[str],
    modes: list[BenchMode],
    md_binary: str,
    runner: str | None,
    executor: str | None,
    model: str | None,
    runs_per_task: int,
    results: list[BenchResult],
    started_at: float,
    finished_at: float,
    thinking_level: str | None = None,
    holdout_version: int | None = None,
    temperature_policy: str | None = None,
) -> dict[str, object]:
    resolved_model = model
    if resolved_model is None:
        observed_models = {
            result.model
            for result in results
            if isinstance(result.model, str) and result.model.strip()
        }
        if len(observed_models) == 1:
            resolved_model = next(iter(observed_models))

    resolved_thinking_level = thinking_level
    if resolved_thinking_level is None:
        observed_thinking_levels = {
            result.thinking_level
            for result in results
            if isinstance(result.thinking_level, str) and result.thinking_level.strip()
        }
        if len(observed_thinking_levels) == 1:
            resolved_thinking_level = next(iter(observed_thinking_levels))

    by_mode: dict[str, dict[str, float | int]] = {}
    for mode in modes:
        mode_results = [result for result in results if result.mode == mode]
        if mode_results:
            by_mode[mode] = aggregate_results(mode_results)

    if temperature_policy is None:
        temperature_policy = _temperature_policy(runner, resolved_model, resolved_thinking_level)

    return {
        "schema_version": 1,
        "kind": run_kind,
        "started_at": _iso_timestamp(started_at),
        "finished_at": _iso_timestamp(finished_at),
        "tasks_path": tasks_path,
        "task_ids_path": task_ids_path,
        "selected_task_ids": selected_task_ids,
        "modes": modes,
        "md_binary": md_binary,
        "runner": runner,
        "executor": executor,
        "model": resolved_model,
        "requested_model": model,
        "thinking_level": resolved_thinking_level,
        "requested_thinking_level": thinking_level,
        "temperature_policy": temperature_policy,
        "task_file_sha256": sha256_file(tasks_path),
        "prompt_template_sha256": current_prompt_template_sha256(),
        "scorer_version": SCORER_VERSION,
        "manifest_hash": manifest_hash(MANIFEST_PATH) if MANIFEST_PATH.exists() else None,
        "runs_per_task": runs_per_task,
        "trials_per_cell": runs_per_task,
        "holdout_version": holdout_version,
        "aggregates": {
            "overall": aggregate_results(results),
            "by_mode": by_mode,
        },
    }


def _temperature_policy(
    runner: str | None,
    model: str | None,
    thinking_level: str | None,
) -> str | None:
    if runner == "oai-loop":
        if model and "qwen" in model.lower():
            return "temperature=0; chat_template_kwargs.enable_thinking=false"
        return "temperature=0"
    if runner == "claude-cli":
        return "provider default (temperature not exposed by claude-cli)"
    if runner == "pi-json":
        if thinking_level:
            return f"provider default via pi-json; thinking={thinking_level}"
        return "provider default via pi-json"
    return None


def _write_atomic(path: Path, content: str) -> None:
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(content)
    tmp.replace(path)


def write_run_artifacts(
    results_dir: str,
    *,
    metadata: dict[str, object],
    results: list[BenchResult],
    selected_task_ids: list[str],
) -> None:
    output_dir = Path(results_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    _write_atomic(output_dir / "run.json", json.dumps(metadata, indent=2) + "\n")
    _write_atomic(output_dir / "results.json", json.dumps([asdict(result) for result in results], indent=2) + "\n")
    _write_atomic(output_dir / "task_ids.json", json.dumps(selected_task_ids, indent=2) + "\n")


def load_resume_results(
    results_dir: str,
    *,
    selected_task_ids: list[str],
    modes: list[BenchMode],
    runs_per_task: int,
    runner: str,
    executor: str,
    model: str | None,
    tasks_path: str | Path = "bench/tasks/tasks.json",
    thinking_level: str | None = None,
) -> list[BenchResult]:
    output_dir = Path(results_dir)
    results_path = output_dir / "results.json"
    run_path = output_dir / "run.json"
    if not results_path.exists():
        return []
    if not run_path.exists():
        raise ValueError("--resume found results.json but no run.json")

    run_metadata = json.loads(run_path.read_text())
    expected_metadata = {
        "kind": "agent-track",
        "selected_task_ids": selected_task_ids,
        "modes": list(modes),
        "trials_per_cell": runs_per_task,
        "runner": runner,
        "executor": executor,
        "task_file_sha256": sha256_file(tasks_path),
        "prompt_template_sha256": current_prompt_template_sha256(),
        "scorer_version": SCORER_VERSION,
    }
    if "requested_model" in run_metadata:
        expected_metadata["requested_model"] = model
    elif model is not None:
        expected_metadata["model"] = model
    if "requested_thinking_level" in run_metadata:
        expected_metadata["requested_thinking_level"] = thinking_level
    elif thinking_level is not None:
        expected_metadata["thinking_level"] = thinking_level
    for key, expected in expected_metadata.items():
        actual = run_metadata.get(key)
        if actual != expected:
            raise ValueError(
                f"--resume bundle metadata mismatch for {key}: "
                f"expected {expected!r}, found {actual!r}"
            )

    raw_results = json.loads(results_path.read_text())
    if not isinstance(raw_results, list):
        raise ValueError("--resume results.json must contain a list")

    allowed_fields = {field_.name for field_ in fields(BenchResult)}
    selected = set(selected_task_ids)
    allowed_modes = set(modes)
    seen: set[tuple[str, str, int]] = set()
    results: list[BenchResult] = []
    for index, raw_row in enumerate(raw_results):
        if not isinstance(raw_row, dict):
            raise ValueError(f"--resume results row {index} must be an object")
        row = {key: value for key, value in raw_row.items() if key in allowed_fields}
        try:
            result = BenchResult(**row)
        except TypeError as exc:
            raise ValueError(f"--resume results row {index} is invalid: {exc}") from exc
        if result.mode not in allowed_modes:
            raise ValueError(f"--resume row {index} has unexpected mode {result.mode!r}")
        if result.task_id not in selected:
            raise ValueError(f"--resume row {index} has unexpected task_id {result.task_id!r}")
        if result.run_index is None or not 0 <= result.run_index < runs_per_task:
            raise ValueError(f"--resume row {index} has invalid run_index {result.run_index!r}")
        key = (result.mode, result.task_id, result.run_index)
        if key in seen:
            mode, task_id, run_index = key
            raise ValueError(
                f"--resume duplicate result for mode={mode} task_id={task_id} "
                f"run_index={run_index}"
            )
        seen.add(key)
        results.append(result)
    return results


def _verdict_fields(
    *,
    task_id: str,
    mode: BenchMode,
    primary: bool | None,
    diagnostic: bool | None,
) -> tuple[bool | None, str, dict[str, object] | None]:
    if diagnostic is None or primary != diagnostic:
        return (
            None,
            "divergent",
            {
                "task_id": task_id,
                "mode": mode,
                "reason": "scorer_divergence",
                "primary": primary,
                "diagnostic": diagnostic,
            },
        )
    return primary, "pass" if primary else "fail", None


def dry_run(tasks: list[BenchTask], md_binary: str) -> list[BenchResult]:
    """Validate scorer authority: expected vs itself should pass primary and diagnostic paths."""
    results = []
    for task in tasks:
        expected = (
            ""
            if task.expected_artifact == "multi_file_contents_any"
            else Path(task.expected_output).read_text()
        )

        if task.expected_artifact == "stdout_and_file":
            # For safe-fail tasks: expected_output is the unchanged input file,
            # expected_stdout is the text that should appear on stdout.
            # Dry-run: verify file identity passes and stdout matches itself.
            ok_file_primary, ok_file_diagnostic, file_report = score_task(task, expected, expected, md_binary)
            ok_stdout = task.expected_stdout is not None
            report = file_report
            if ok_stdout:
                report += "\nstdout_check: OK (dry-run)"
            ok_primary = bool(ok_file_primary and ok_stdout)
            ok_diagnostic = bool(ok_file_diagnostic and ok_stdout)
        elif task.expected_artifact == "multi_file_contents_any":
            alternatives = _multifile_expected_alternatives(task.expected_output)
            ok_primary, ok_diagnostic, report = _score_multifile_against_dir(
                task,
                actual_root=alternatives[0],
                expected_root=alternatives[0],
                md_binary=md_binary,
            )
        else:
            ok_primary, ok_diagnostic, report = score_task(task, expected, expected, md_binary)

        correct, verdict, quarantine = _verdict_fields(
            task_id=task.id,
            mode="mdtools",
            primary=ok_primary,
            diagnostic=ok_diagnostic,
        )

        results.append(BenchResult(
            task_id=task.id, mode="mdtools",
            correct=correct,
            correct_neutral=bool(ok_primary),
            correct_diagnostic=ok_diagnostic,
            verdict=verdict,
            quarantine=quarantine,
            run_index=0,
            diff_report=report,
        ))
    return results


def _build_agent_cmd(
    agent_cmd: str,
    mode: BenchMode,
    model: str | None = None,
    max_turns: int = 30,
) -> list[str]:
    """Build the full agent subprocess command with proper flags."""
    parts = agent_cmd.split()
    # If agent is claude (Claude Code), add flags for non-interactive tool use
    if Path(parts[0]).name == "claude":
        cmd = [parts[0], "-p"]
        if model:
            cmd += ["--model", model]
        # U2 (FRAC-194): native* modes additionally expose Claude Code's native
        # file tools (Read/Edit/Write) — the realistic frontier alternative to `md`.
        # These are built-ins, NOT MCP/CLAUDE.md/slash-sourced, so they compose with
        # the isolation flags below: additive-only, no contamination path reopened.
        toolset = "Bash Read Edit Write" if mode in ("native", "native+md", "native+md-no-md") else "Bash"
        cmd += ["--tools", toolset]
        cmd += ["--allowedTools", toolset]
        cmd += ["--dangerously-skip-permissions"]
        cmd += ["--max-turns", str(max_turns)]
        cmd += ["--no-session-persistence"]
        # bench-v2 ISOLATION (PR#10 Codex P1). The previous `--settings ""` did NOT
        # isolate: it discovered ~94 slash-commands/skills, 5 agents, 6 MCP servers +
        # CLAUDE.md (~32k input tokens) into EVERY cell, and the mdtools CLAUDE.md
        # (which documents `md`) leaked into the unix/hybrid-no-md baselines — a
        # mode-isolation breach. These flags isolate WITHOUT zeroing usage (unlike
        # `--bare`, which kills the token cost axis): no slash-commands/skills, strict
        # (empty) MCP, no user/project/local settings or hooks, no custom agents.
        # Combined with the clean temp cwd (cwd=workdir, no CLAUDE.md ancestor) a
        # trivial prompt drops ~32k -> ~3.5k input tokens, usage intact, md-tool docs
        # NOT in context. The remaining 5 agents are built-ins (no contamination).
        cmd += ["--disable-slash-commands", "--strict-mcp-config"]
        cmd += ["--setting-sources", ""]
        cmd += ["--agents", "{}"]
        cmd += ["--output-format", "json"]
        return cmd
    return parts


NATIVE_MODES = ("native", "native+md", "native+md-no-md")

# Runners that expose first-class file Read/Edit/Write tools — the "native editor" the
# native* arm measures md against. claude-cli ships Read/Edit/Write; pi-json exposes pi's
# built-in read/edit/write via `--tools` (2026-06-13: pi-json native arm). oai-loop is
# intentionally ABSENT — it drives the model through a single Bash action protocol
# (oai_loop.py: only {"type":"bash"}/{"type":"final"} actions) and has no native editor to
# expose. Allowlist (not `!= "claude-cli"`) so a future runner stays blocked from native*
# until it's proven to expose a native editor — fail-closed, same discipline as MD_REAL_MODES.
NATIVE_CAPABLE_RUNNERS = ("claude-cli", "pi-json")

# pi (`--runner pi-json`) tool allowlists, passed to `pi --tools` (lowercase, comma-joined
# by build_pi_json_command). native* modes expose pi's native editor; every other mode is
# Bash-only — mirroring the claude-cli toolset switch (the "Bash Read Edit Write" line in
# _build_agent_cmd). All THREE native modes get the SAME tuple: the native / native+md /
# native+md-no-md distinction is whether ./md is the real binary or the stub (MD_REAL_MODES
# + the no-md preflight), enforced at the FILESYSTEM, never via the tool list.
PI_NATIVE_TOOLS = ("read", "bash", "edit", "write")
PI_BASH_ONLY_TOOLS = ("bash",)


def _pi_tools_for_mode(mode: str) -> tuple[str, ...]:
    return PI_NATIVE_TOOLS if mode in NATIVE_MODES else PI_BASH_ONLY_TOOLS


def _pi_agent_bin_dir() -> Path:
    """The bin dir pi's getShellEnv() PREPENDS to PATH at spawn time (pi dist/utils/shell.js
    getBinDir): <$PI_CODING_AGENT_DIR or ~/.pi/agent>/bin. The no-md preflight must probe
    this dir too for runner==pi-json: pi resolves bare commands with it on PATH, but the
    harness child_env does NOT include it — so a real `md` landing there would be invisible
    to the preflight yet reachable to pi (the ./md-bypass family, pi-PATH axis). Resolved
    exactly as pi_runner.default_audit_extension_path resolves the agent dir."""
    agent_dir = Path(os.environ.get("PI_CODING_AGENT_DIR", "~/.pi/agent")).expanduser()
    return agent_dir / "bin"


def native_runner_error(modes: list[str], runner: str) -> str | None:
    """native* modes need a runner with first-class file Read/Edit/Write tools — the native
    editor the arm measures md against: claude-cli, or pi-json (pi's read/edit/write via
    `--tools`). oai-loop drives a single Bash action protocol and has no native editor, so it
    stays rejected. Return an actionable error for the first offending mode, else None."""
    offending = [m for m in modes if m in NATIVE_MODES]
    if offending and runner not in NATIVE_CAPABLE_RUNNERS:
        return (
            f"--mode {offending[0]} requires a runner with native file tools "
            f"(claude-cli or pi-json) — got --runner {runner}. oai-loop drives a single "
            f"Bash action protocol and has no Read/Edit/Write to expose"
        )
    return None


def _summarize_runner_error(code: str | None, message: str | None) -> str | None:
    clean_code = code.strip() if isinstance(code, str) and code.strip() else None
    clean_message = None
    if isinstance(message, str):
        collapsed = " ".join(message.split())
        clean_message = collapsed if collapsed else None

    if clean_code and clean_message:
        return f"{clean_code}: {clean_message}"
    return clean_code or clean_message


def _normalize_runner_model(value: object) -> str | None:
    if not isinstance(value, str):
        return None
    model = value.strip()
    if not model or model == "<synthetic>":
        return None
    return model


def _requeried_from_sequence(call_sequence: list[str]) -> bool:
    """A 'query' AFTER any 'mutation' in the ordered trajectory = the agent re-read
    structure post-edit (the re-query pattern). Shared by the POSIX-guard arm (which
    builds call_sequence from guard events) and the claude-cli arm (which builds it from
    the transcript, FRAC-194 #4) so requery is computed identically for both."""
    saw_mutation = False
    for kind in call_sequence:
        if kind == "mutation":
            saw_mutation = True
        elif kind == "query" and saw_mutation:
            return True
    return False


def _prepend_workdir_to_path(child_env: dict, workdir: str) -> None:
    """Prepend the workdir to PATH so a BARE `md` call resolves to the workdir copy
    (the stub for md-free/ablation modes, the real binary for native+md) — identical to
    `./md`. FRAC-194 #8 (the PATH-axis recurrence of the ./md-bypass family): the native
    arm runs on claude-cli, whose Bash tool never sources BASH_ENV, so the guard's PATH
    restriction (export PATH=$BENCH_RESTRICTED_PATH) never fires and the agent keeps the
    full system PATH — including the real ~/.local/bin/md. NATIVE_MD_DOCS advertises bare
    `md`, so without this a stub mode (native, native+md-no-md) silently resolves bare
    `md` to the REAL binary, bypassing the ./md stub and invalidating the clean ablation.
    For guarded runners the guard overrides PATH to .bench-bin, so this is a no-op there."""
    child_env["PATH"] = os.path.abspath(workdir) + os.pathsep + child_env.get("PATH", "")


class NoMdLeakError(RuntimeError):
    """A no-md/ablation mode could reach a WORKING md before the task ran — the
    clean-ablation invariant is broken (the ./md-bypass family). The preflight raises
    this so we fail CLOSED (abort, before the billed agent call) instead of silently
    collecting contaminated data — the durable guard against this family's 5 recurrences."""


def _assert_no_md_reachable(child_env: dict, workdir: str,
                            extra_path_dirs: tuple[str, ...] = ()) -> None:
    """Preflight fail-closed proof for md-free/ablation modes (FRAC-194 #8 hardening):
    BEFORE the (billed) agent runs, verify neither bare `md` NOR `./md` resolves to a
    working binary in the agent's exact env. Runs via /bin/sh -c which — like claude-cli's
    Bash tool — does NOT source BASH_ENV, so it proves md is unreachable even with the
    guard absent: the strongest guarantee, and the only one that holds for the guard-blind
    native arm. The probe log is redirected to /dev/null so these proof calls don't
    inflate md_probe_count. Raises NoMdLeakError if any form answers (exit 0).

    extra_path_dirs are PREPENDED to the probe PATH so the proof covers a PATH the agent's
    runner will use but child_env omits — pi-json prepends its own bin dir at spawn time
    (getShellEnv; see _pi_agent_bin_dir), so without this the preflight could pass while pi
    resolves a real md on its own PATH (the ./md-bypass family, pi-PATH axis)."""
    probe_env = dict(child_env)
    probe_env["BENCH_MD_PROBE_LOG"] = os.devnull   # don't let proof calls touch the real probe log
    if extra_path_dirs:
        probe_env["PATH"] = os.pathsep.join([*extra_path_dirs, probe_env.get("PATH", "")])
    for invocation in ("md --version", "./md --version"):
        proc = subprocess.run(["/bin/sh", "-c", invocation], cwd=workdir, env=probe_env,
                              text=True, capture_output=True)
        if proc.returncode == 0:   # the stub exits non-zero; a real md --version exits 0
            raise NoMdLeakError(
                f"no-md preflight FAILED in {workdir!r}: `{invocation}` ran a working md "
                f"(exit 0) — real md is reachable, the clean ablation would be invalid. "
                f"stdout={proc.stdout.strip()[:200]!r}")


def parse_agent_output(raw_stdout: str) -> ParsedAgentOutput:
    """Extract combined stdout, tool stats, and runner failures from agent output."""
    parsed = ParsedAgentOutput(stdout=raw_stdout)

    try:
        messages = json.loads(raw_stdout)
    except (json.JSONDecodeError, TypeError):
        parsed.tool_calls = raw_stdout.count('"tool_use"') + raw_stdout.count("$ ")
        return parsed

    if not isinstance(messages, list):
        parsed.tool_calls = raw_stdout.count('"tool_use"') + raw_stdout.count("$ ")
        return parsed

    error_code: str | None = None
    error_message: str | None = None

    for msg in messages:
        if not isinstance(msg, dict):
            continue

        msg_type = msg.get("type")
        if msg_type == "system" and msg.get("subtype") == "init" and parsed.model is None:
            parsed.model = _normalize_runner_model(msg.get("model"))
        elif msg_type == "result":
            parsed.turns = msg.get("num_turns", 0)
            usage = msg.get("usage") or {}
            parsed.tokens_in = (
                int(usage.get("input_tokens") or 0)
                + int(usage.get("cache_creation_input_tokens") or 0)
                + int(usage.get("cache_read_input_tokens") or 0)
            )
            parsed.tokens_out = int(usage.get("output_tokens") or 0)
            result_text = msg.get("result", "")
            if isinstance(result_text, str):
                parsed.text_outputs.append(result_text)
            if msg.get("is_error"):
                if isinstance(msg.get("api_error_status"), str) and not error_code:
                    error_code = msg["api_error_status"]
                if isinstance(result_text, str) and result_text.strip() and not error_message:
                    error_message = result_text
        elif msg_type == "assistant":
            if parsed.model is None:
                parsed.model = _normalize_runner_model(msg.get("message", {}).get("model"))
            if isinstance(msg.get("error"), str) and msg["error"].strip() and not error_code:
                error_code = msg["error"]

            content = msg.get("message", {}).get("content", [])
            if isinstance(content, list):
                text_blocks: list[str] = []
                for block in content:
                    if not isinstance(block, dict):
                        continue
                    if block.get("type") == "tool_use":
                        parsed.tool_calls += 1
                        # U4/U7: per-verb adoption parsed from the transcript — the
                        # Bash guard never fires for claude-cli's Bash tool. Native
                        # Read/Edit/Write are counted by tool name; Bash commands are
                        # classified by verb (md tasks / sed / ...) and kind (mutation).
                        name = block.get("name")
                        if name in ("Read", "Edit", "Write"):
                            parsed.transcript_tool_mix[name] = parsed.transcript_tool_mix.get(name, 0) + 1
                            # native Edit/Write mutate the file (Read does not); count
                            # them as mutations so native-arm mutation/requery metrics
                            # aren't 0 for a real edit. (FRAC-194 review #4.)
                            if name == "Read":
                                parsed.transcript_call_sequence.append("query")
                            else:
                                parsed.transcript_mutations += 1
                                parsed.transcript_call_sequence.append("mutation")
                        elif name == "Bash":
                            cmd = (block.get("input") or {}).get("command", "") or ""
                            verb = classify_command_verb(cmd)
                            if verb:
                                parsed.transcript_tool_mix[verb] = parsed.transcript_tool_mix.get(verb, 0) + 1
                            kind = classify_command_kind(cmd)
                            if kind == "mutation":
                                parsed.transcript_mutations += 1
                                parsed.transcript_call_sequence.append("mutation")
                            elif kind == "query":
                                parsed.transcript_call_sequence.append("query")
                    if block.get("type") == "text":
                        text = block.get("text", "")
                        parsed.text_outputs.append(text)
                        text_blocks.append(text)
                if text_blocks and msg.get("error") and not error_message:
                    error_message = "\n".join(text_blocks)
        elif msg_type == "user":
            content = msg.get("message", {}).get("content", [])
            if isinstance(content, list):
                for block in content:
                    if not isinstance(block, dict) or block.get("type") != "tool_result":
                        continue
                    result_content = block.get("content", "")
                    if isinstance(result_content, str):
                        parsed.tool_outputs.append(result_content)
                        parsed.bytes_observation += len(result_content.encode())
                    elif isinstance(result_content, list):
                        for item in result_content:
                            if isinstance(item, dict) and item.get("type") == "text":
                                text = item.get("text", "")
                                parsed.tool_outputs.append(text)
                                parsed.bytes_observation += len(text.encode())

    combined = parsed.tool_outputs + parsed.text_outputs
    parsed.stdout = "\n".join(combined) if combined else raw_stdout
    parsed.runner_error = _summarize_runner_error(error_code, error_message)
    return parsed


def run_agent(
    task: BenchTask,
    mode: BenchMode,
    agent_cmd: str,
    md_binary: str,
    model: str | None = None,
    runner: str = "claude-cli",
    executor: str = "guarded",
    log_dir: str | None = None,
    max_turns: int = 30,
    oai_api_base: str | None = None,
    oai_api_key: str | None = None,
    oai_request_timeout: int = 60,
    oai_tool_timeout: int = 30,
    thinking_level: str | None = None,
) -> BenchResult:
    """Run an agent subprocess to complete a task."""
    workdir = tempfile.mkdtemp(prefix=f"mdtools_bench_{task.id}_{mode}_")
    workdir_path = Path(workdir)

    all_files = list(task.input_files)
    if task.support_files:
        all_files.extend(task.support_files)

    for inp in all_files:
        # Preserve subdirectory structure (e.g., t16_vault/frontend.md)
        inp_dir = os.path.dirname(inp)
        inp_parent = os.path.basename(inp_dir) if inp_dir else ""
        if inp_parent and inp_parent != "inputs":
            sub_dir = os.path.join(workdir, inp_parent)
            os.makedirs(sub_dir, exist_ok=True)
            shutil.copy2(inp, sub_dir)
        else:
            shutil.copy2(inp, workdir)

    # Copy md binary into workdir so it's accessible by relative path (the prompt
    # advertises "md is available at ./md"). CRITICAL (PR#10 Codex P1): in hybrid-no-md
    # the ./md copy MUST be the soft stub, NOT the real binary — otherwise an agent
    # that uses ./md (exactly as the prompt says) bypasses the PATH-level ablation, the
    # no-md baseline silently runs real md, and the md-lift/attribution gate measures
    # nothing. (Observed: no-md agents used `./md set-task`/`./md tasks`, md-probe=0.)
    md_dest = os.path.join(workdir, "md")
    if md_workdir_must_be_stub(mode):
        # Every non-real-md mode (fail-closed via command_policy.MD_REAL_MODES): the
        # two clean ablations (hybrid-no-md, native+md-no-md) AND the md-free baselines
        # (unix, native). The baseline must be md-free at the FILESYSTEM, not merely
        # un-advertised — else a stray ./md call silently runs real md and the
        # md-lift/attribution gate measures nothing. This single predicate replaces the
        # hand-maintained mode list that let the ./md-bypass family recur 4× (PR#10
        # hybrid-no-md, FRAC-194 native+md-no-md, then native). (FRAC-194 review #2.)
        with open(md_dest, "w") as f:
            f.write(_md_ablation_stub())
        os.chmod(md_dest, 0o755)
    elif md_binary != "md":
        shutil.copy2(md_binary, md_dest)
    else:
        shutil.copy2(shutil.which("md") or md_binary, md_dest)

    restricted_env = None
    child_env = os.environ.copy()
    guard_log_path: Path | None = None
    if executor == "guarded":
        restricted_env = create_restricted_shell_env(
            workdir_path,
            mode,
            Path(md_dest),
        )
        child_env = restricted_env.env
        guard_log_path = restricted_env.guard_log_path

    # Close the PATH-axis ./md-bypass (FRAC-194 #8): make bare `md` resolve to the
    # workdir copy so guard-blind runners (claude-cli) can't escape to the real md on
    # the system PATH. No-op under the guard, which overrides PATH to .bench-bin.
    _prepend_workdir_to_path(child_env, workdir)
    if md_workdir_must_be_stub(mode):
        # fail-closed: prove md is unreachable BEFORE the billed agent call, so a future
        # axis of the ./md-bypass family can never silently contaminate a clean ablation.
        # pi-json prepends its own bin dir to PATH at spawn (getShellEnv), which child_env
        # omits — probe it too so the proof matches the PATH pi actually runs with, else a
        # real md in ~/.pi/agent/bin would be invisible here but reachable to pi.
        extra_path_dirs = (str(_pi_agent_bin_dir()),) if runner == "pi-json" else ()
        _assert_no_md_reachable(child_env, workdir, extra_path_dirs=extra_path_dirs)

    prompt = build_prompt(task, mode, workdir)
    input_file = copied_input_path(task.input_files[0], workdir)

    bytes_prompt = len(prompt.encode())

    start = time.time()
    raw_stdout = ""
    stdout = ""
    tool_calls = 0
    num_turns = 0
    bytes_observation = 0
    tokens_in = 0
    tokens_out = 0
    mutations = 0
    policy_violations = 0
    requeried = False
    invalid_responses = 0
    unique_invalid_responses = 0
    runner_error = None
    resolved_model = model
    resolved_thinking_level = thinking_level if runner == "pi-json" else None
    all_tool_outputs: list[str] = []
    all_text_outputs: list[str] = []
    call_sequence: list[str] = []  # "mutation" or "query"
    tool_mix: dict[str, int] = {}  # per-verb tool-choice counts (free-choice/hybrid adoption signal)

    pi_audit_log_path: Path | None = None

    if runner == "oai-loop":
        if not oai_api_base:
            raise RuntimeError("oai-loop runner requires --oai-api-base or BENCH_OAI_API_BASE")
        if not oai_api_key:
            raise RuntimeError("oai-loop runner requires --oai-api-key or BENCH_OAI_API_KEY")
        if not model:
            raise RuntimeError("oai-loop runner requires a resolved model id")

        resolved_model = model
        bytes_output = 0
        trace = None
        try:
            trace = run_oai_loop(
                api_base=oai_api_base,
                api_key=oai_api_key,
                model=model,
                prompt=prompt,
                workdir=workdir_path,
                env=child_env,
                max_turns=max_turns,
                request_timeout_seconds=oai_request_timeout,
                tool_timeout_seconds=oai_tool_timeout,
            )
        except LoopError as exc:
            # The loop body raised mid-turn; preserve accumulated per-turn
            # counters (tool_calls, invalid_responses, turns, bytes_*) from
            # the partial trace so runner_error rows still carry diagnostic
            # signal instead of all-zero defaults.
            runner_error = f"oai_loop_error: {type(exc.cause).__name__}: {exc.cause}"
            trace = exc.partial
        except Exception as exc:  # noqa: BLE001 — surface any other loop failure as a recorded runner_error
            runner_error = f"oai_loop_error: {type(exc).__name__}: {exc}"
            raw_stdout = ""

        if trace is not None:
            raw_stdout = trace.raw_output
            bytes_output = trace.bytes_output
            tool_calls = trace.tool_calls
            num_turns = trace.turns
            bytes_observation = trace.bytes_observation
            invalid_responses = trace.invalid_responses
            unique_invalid_responses = trace.unique_invalid_responses
            tokens_in = trace.tokens_in
            tokens_out = trace.tokens_out
            all_tool_outputs = trace.tool_outputs
            all_text_outputs = trace.text_outputs
            stdout = "\n".join(all_tool_outputs + all_text_outputs)
    elif runner == "pi-json":
        pi_audit_log_path = workdir_path / ".pi-audit.jsonl"
        pi_session_dir = workdir_path / ".pi-sessions"
        audit_extension_path = default_audit_extension_path()
        cmd = build_pi_json_command(
            agent_cmd=agent_cmd,
            model=model,
            audit_extension_path=audit_extension_path,
            session_dir=pi_session_dir,
            tools=_pi_tools_for_mode(mode),
            thinking_level=thinking_level,
        )
        pi_env = child_env.copy()
        pi_env["PI_AUDIT_LOG"] = str(pi_audit_log_path)
        pi_env.setdefault("PI_SKIP_VERSION_CHECK", "1")
        if task.expected_artifact == "multi_file_contents_any":
            drift_task = drift_task_from_input_files(task.input_files)
            if drift_task:
                pi_env["BENCH_MULTIFILE_DRIFT_ENABLED"] = "1"
                pi_env["BENCH_MULTIFILE_DRIFT_TASK"] = drift_task
                pi_env["BENCH_MULTIFILE_DRIFT_SPECS"] = str((REPO_ROOT / DRIFT_SPECS_PATH).resolve())
        try:
            result = subprocess.run(
                [*cmd, prompt],
                capture_output=True,
                text=True,
                timeout=180,
                cwd=workdir,
                env=pi_env,
            )
            raw_stdout = result.stdout
            bytes_output = len(raw_stdout.encode())
            if result.returncode != 0:
                stderr = " ".join((result.stderr or "").split())
                runner_error = (
                    f"pi_json_error: exit {result.returncode}: {stderr}"
                    if stderr
                    else f"pi_json_error: exit {result.returncode}"
                )
        except subprocess.TimeoutExpired:
            raw_stdout = ""
            bytes_output = 0
            runner_error = "pi_json_error: timeout"

        parsed_output = parse_pi_json_output(raw_stdout)
        stdout = parsed_output.stdout
        tool_calls = parsed_output.tool_calls
        num_turns = parsed_output.turns
        bytes_observation = parsed_output.bytes_observation
        all_tool_outputs = parsed_output.tool_outputs
        all_text_outputs = parsed_output.text_outputs
        runner_error = runner_error or parsed_output.runner_error
        resolved_model = parsed_output.model or model
        resolved_thinking_level = parsed_output.thinking_level or thinking_level
    else:
        cmd = _build_agent_cmd(agent_cmd, mode, model, max_turns)
        try:
            result = subprocess.run(
                cmd,
                input=prompt,
                capture_output=True,
                text=True,
                timeout=180,
                cwd=workdir,
                env=child_env,
            )
            raw_stdout = result.stdout
            bytes_output = len(raw_stdout.encode())
        except subprocess.TimeoutExpired:
            raw_stdout = ""
            bytes_output = 0

        parsed_output = parse_agent_output(raw_stdout)
        stdout = parsed_output.stdout
        tool_calls = parsed_output.tool_calls
        num_turns = parsed_output.turns
        bytes_observation = parsed_output.bytes_observation
        tokens_in = parsed_output.tokens_in
        tokens_out = parsed_output.tokens_out
        all_tool_outputs = parsed_output.tool_outputs
        all_text_outputs = parsed_output.text_outputs
        runner_error = parsed_output.runner_error
        resolved_model = model or parsed_output.model
        # U4/U7 (FRAC-194): the Bash guard does not fire for claude-cli's Bash tool,
        # so the transcript parse is the adoption + mutation signal — fold it into
        # tool_mix and seed mutations. (The guard loop below still populates these for
        # oai-loop/pi-json from their guard logs; for claude-cli it's a no-op.)
        for _name, _n in parsed_output.transcript_tool_mix.items():
            tool_mix[_name] = tool_mix.get(_name, 0) + _n
        mutations += parsed_output.transcript_mutations
        # ordered trajectory so the query-after-mutation requery scan (below) works for
        # claude-cli, whose guard call_sequence is empty. (FRAC-194 #4.)
        call_sequence.extend(parsed_output.transcript_call_sequence)
    elapsed = time.time() - start

    guard_events = load_guard_events(guard_log_path) if guard_log_path is not None else []
    for event in guard_events:
        if event.decision != "allow":
            policy_violations += 1
            continue
        verb = event.verb
        if verb:
            tool_mix[verb] = tool_mix.get(verb, 0) + 1
        kind = event.kind
        if kind == "mutation":
            mutations += 1
            call_sequence.append("mutation")
        elif kind == "query":
            call_sequence.append("query")

    md_probe_log = workdir_path / ".md-probe.log"
    md_probe_count = len(md_probe_log.read_text().splitlines()) if md_probe_log.exists() else 0

    if pi_audit_log_path is not None:
        audit_counters = summarize_pi_audit_events(
            load_pi_audit_events(pi_audit_log_path),
            guard_events=guard_events,
        )
        if tool_calls == 0:
            tool_calls = audit_counters.tool_calls
        if bytes_observation == 0:
            bytes_observation = audit_counters.bytes_observation
        if audit_counters.model and (not resolved_model or resolved_model == model):
            resolved_model = audit_counters.model
        if audit_counters.thinking_level:
            resolved_thinking_level = audit_counters.thinking_level
        policy_violations = max(policy_violations, audit_counters.policy_violations)
        mutations = max(mutations, audit_counters.mutations)
        requeried = requeried or audit_counters.requeried
        # Fold pi's native edit/write/read counts into tool_mix — the guarded shell never
        # saw them (they bypass bash), so without this the native-arm adoption split would
        # show md verbs with no native-editor counterpart. md verbs come from the guard loop
        # above; the native-editor alternative comes from here.
        for _tool, _n in audit_counters.native_tool_mix.items():
            tool_mix[_tool] = tool_mix.get(_tool, 0) + _n

    requeried = requeried or _requeried_from_sequence(call_sequence)

    expected_content = (
        ""
        if task.expected_artifact == "multi_file_contents_any"
        else Path(task.expected_output).read_text()
    )

    if task.expected_artifact == "stdout_and_file":
        # Score both stdout text and file contents
        report_parts = []
        # Check stdout contains expected_stdout (agent may add explanation)
        ok_stdout = False
        if task.expected_stdout is not None:
            expected_text = task.expected_stdout.strip()
            # Last non-empty line must match exactly
            actual_lines = [l.strip() for l in stdout.strip().splitlines() if l.strip()]
            actual_last = actual_lines[-1] if actual_lines else ""
            ok_stdout = actual_last == expected_text
            report_parts.append(f"stdout: {'OK' if ok_stdout else 'MISMATCH'}")
            if not ok_stdout:
                report_parts.append(f"  expected last line: {expected_text!r}")
                report_parts.append(f"  actual last line:   {actual_last!r}")
        # Check file unchanged
        try:
            actual_file = Path(input_file).read_text()
        except FileNotFoundError:
            actual_file = ""
        ok_file_primary, ok_file_diagnostic, file_report = score_task(task, actual_file, expected_content, md_binary)
        report_parts.append(file_report)
        ok_primary = bool(ok_stdout and ok_file_primary)
        ok_diagnostic = bool(ok_stdout and ok_file_diagnostic)
        report = "\n".join(report_parts)
    elif task.expected_artifact == "file_contents":
        try:
            actual = Path(input_file).read_text()
        except FileNotFoundError:
            actual = ""
        ok_primary, ok_diagnostic, report = score_task(task, actual, expected_content, md_binary)
    elif task.expected_artifact == "multi_file_contents_any":
        ok_primary, ok_diagnostic, report = score_multifile_any(
            task,
            actual_root=workdir_path,
            expected_output=task.expected_output,
            md_binary=md_binary,
        )
        if pi_audit_log_path is None:
            proof_report = "multifile_drift_proof: INVALID no live pi audit log"
            ok_primary = False
            ok_diagnostic = False
        else:
            proof = summarize_drift_proof(load_pi_audit_events(pi_audit_log_path))
            proof_report = format_drift_proof(proof)
            if not proof.valid:
                ok_primary = False
                ok_diagnostic = False
        report = f"{report}\n{proof_report}"
    elif task.expected_artifact == "json_envelope":
        actual = select_json_envelope_actual(
            all_tool_outputs, all_text_outputs, stdout, expected_content
        )
        ok_primary, ok_diagnostic, report = score_task(task, actual, expected_content, md_binary)
    elif task.expected_artifact == "stdout_text":
        actual = extract_final_text(all_tool_outputs, all_text_outputs, stdout)
        ok_primary, ok_diagnostic, report = score_task(task, actual, expected_content, md_binary)
    else:
        actual = stdout
        ok_primary, ok_diagnostic, report = score_task(task, actual, expected_content, md_binary)

    if runner_error and (not ok_primary or not ok_diagnostic):
        report = f"runner_error: {runner_error}\n{report}" if report else f"runner_error: {runner_error}"

    correct, verdict, quarantine = _verdict_fields(
        task_id=task.id,
        mode=mode,
        primary=ok_primary,
        diagnostic=ok_diagnostic,
    )

    if log_dir:
        run_log_dir = Path(log_dir) / f"{task.id}_{mode}_{int(start)}"
        run_log_dir.mkdir(parents=True, exist_ok=True)
        (run_log_dir / "prompt.txt").write_text(prompt)
        (run_log_dir / "agent_output.txt").write_text(raw_stdout)
        if guard_log_path is not None and guard_log_path.exists():
            shutil.copy2(guard_log_path, run_log_dir / "guard.log")
        if pi_audit_log_path is not None and pi_audit_log_path.exists():
            shutil.copy2(pi_audit_log_path, run_log_dir / "pi-audit.jsonl")
        if task.expected_artifact == "multi_file_contents_any":
            copy_final_input_snapshot(task, workdir_path, run_log_dir)
        pi_session_root = workdir_path / ".pi-sessions"
        if pi_session_root.exists():
            shutil.copytree(pi_session_root, run_log_dir / "pi-sessions", dirs_exist_ok=True)

    shutil.rmtree(workdir, ignore_errors=True)

    return BenchResult(
        task_id=task.id,
        mode=mode,
        correct=correct,
        correct_neutral=bool(ok_primary),
        correct_diagnostic=ok_diagnostic,
        verdict=verdict,
        quarantine=quarantine,
        model=resolved_model,
        thinking_level=resolved_thinking_level,
        bytes_prompt=bytes_prompt,
        bytes_output=bytes_output,
        bytes_observation=bytes_observation,
        tool_calls=tool_calls,
        turns=num_turns,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        mutations=mutations,
        tool_mix=tool_mix,
        md_probe_count=md_probe_count,
        policy_violations=policy_violations,
        requeried=requeried,
        invalid_responses=invalid_responses,
        unique_invalid_responses=unique_invalid_responses,
        elapsed_seconds=round(elapsed, 2),
        diff_report=report,
        runner_error=runner_error,
    )


def _json_top_keys(parsed) -> set[str]:
    """Top-level key set of a parsed JSON value, used for shape matching.

    Returns a dict's keys, or the first element's keys for a non-empty
    list of dicts. Empty set otherwise (no observable key shape)."""
    if isinstance(parsed, dict):
        return set(parsed.keys())
    if isinstance(parsed, list) and parsed and isinstance(parsed[0], dict):
        return set(parsed[0].keys())
    return set()


def _expected_json_top_keys(expected_str: str) -> set[str] | None:
    """Top-level key set of the expected JSON, or None when no
    parseable shape is available (the caller then falls back to the
    pre-F4 'first non-empty JSON wins' rule)."""
    try:
        parsed = json.loads(expected_str)
    except (json.JSONDecodeError, TypeError):
        return None
    keys = _json_top_keys(parsed)
    return keys if keys else None


def select_json_envelope_actual(
    all_tool_outputs: list[str],
    all_text_outputs: list[str],
    stdout: str,
    expected_str: str,
) -> str:
    """Pick the best `actual` string for a json_envelope task.

    F4 closure (iter 30): when the expected JSON has a discoverable
    top-level shape, prefer tool outputs whose parsed shape's
    top-level keys overlap with the expected shape. This protects
    against intermediate tool calls (e.g. `./md tasks --json` emitting
    `{"schema_version":..., "results":[...]}`) being captured when the
    agent's correct projected answer is in assistant text. When no
    shape-matching tool output exists, fall through to text outputs
    first, and only then accept any non-empty parseable JSON tool
    output (preserving pre-F4 behavior as the final fallback).

    F8-1 closure (T8 iter 3): the F4 intersection check
    (`_json_top_keys(parsed) & expected_top_keys`) is satisfied by any
    pair of mdtools envelopes via the universal `schema_version` key,
    so it falsely accepts an intermediate `md tasks --json` envelope
    when the expected shape is `md outline --json`. The subset check
    (`expected_top_keys.issubset(_json_top_keys(parsed))`) requires
    every discriminating key from the expected shape to be present,
    which rejects schema_version-only overlap and surfaces the correct
    envelope (or, on no match, the fallback tool output)."""
    expected_top_keys = _expected_json_top_keys(expected_str)
    fallback_tool_actual = ""

    for tool_out in reversed(all_tool_outputs):
        try:
            parsed = json.loads(tool_out.strip())
        except (json.JSONDecodeError, TypeError):
            continue
        if not isinstance(parsed, (list, dict)) or len(parsed) == 0:
            continue
        if expected_top_keys is None:
            return tool_out.strip()
        if expected_top_keys.issubset(_json_top_keys(parsed)):
            return tool_out.strip()
        if not fallback_tool_actual:
            fallback_tool_actual = tool_out.strip()

    for text_out in reversed(all_text_outputs):
        candidate = extract_last_json(text_out)
        try:
            parsed = json.loads(candidate)
        except (json.JSONDecodeError, TypeError):
            continue
        if isinstance(parsed, (list, dict)) and len(parsed) > 0:
            return candidate

    if fallback_tool_actual:
        return fallback_tool_actual
    return extract_last_json(stdout)


def extract_last_json(text: str) -> str:
    """Extract the best JSON from agent output.
    Tries each valid JSON substring and returns the candidate whose
    source-span end position is highest.

    F8-2 closure (T8 iter 5): the legacy rule preferred the last array
    over the last object unconditionally, which selected an inner
    `entries`/`results`/`links` array over its own wrapping envelope
    when the envelope was embedded in agent prose. Highest-end-position
    subsumes both intended behaviors: when one candidate's span
    contains another (e.g. envelope wrapping a nested array), the
    container's end is greater; when candidates are non-overlapping
    siblings (independent JSON documents in the agent text), the later
    one has a greater end and is the agent's final answer.

    F8-3 closure (T8 iter 7): the depth scanner now skips characters
    between unescaped `"` boundaries so brace/bracket characters
    inside JSON string values do not falsely close a candidate. Pre-
    fix, a `}` inside a heading.text value caused the {/} pass to
    record a truncated candidate that failed json.loads, then reset
    start = -1, so the actual wrapping envelope was never enumerated.

    F8-4 closure (T8 iter 9): a global ` ``` ` fence-strip regex
    (`re.sub(r"```(?:json)?\s*\n?", "", text)`) used to run before
    the depth scanner. It was string-blind on backticks: a backtick
    triplet inside a JSON string value (e.g. an `entries[].heading.text`
    that names the language of a code-fence example) was silently
    stripped, mutating the candidate's string content while keeping
    the JSON syntactically parseable. Downstream `score_structural_json`
    then FAILed an agent answer that was byte-exact correct in the
    input text. The fix is to drop the preprocessor: the F8-3
    string-aware depth scanner already finds the JSON region inside
    surrounding ` ```json ` fences (the brace tracker enters at the
    inner `{`/`[` and ignores backticks entirely)."""
    try:
        parsed = json.loads(text)
        if isinstance(parsed, (list, dict)):
            return text
    except json.JSONDecodeError:
        pass

    candidates = []
    for opener, closer in [("[", "]"), ("{", "}")]:
        # Two-pass scan per opener/closer with an opener stack instead of
        # a depth counter. The stack tracks the *position* of every unmatched
        # opener; a closer pops the most recent opener and emits a candidate
        # for that span. A closer with no matching opener is ignored (stray
        # prose closer). Orphaned openers stay on the stack and don't poison
        # subsequent matches — `}{` followed by a real envelope `{...}` no
        # longer pins `start` at the stray opener position (PR #4 round 4).
        #
        # - Shielded pass: tracks quotes so braces/brackets inside JSON
        #   string values stay invisible (F8-3) AND balanced quoted braces
        #   in prose (`He said "}"`) don't leak into the brace count
        #   (PR #4 round 2). Newline aborts in_string so an unmatched
        #   prose quote can't latch forever (PR #4 round 1).
        # - Unshielded pass: ignores quotes entirely. Catches the same-line
        #   case where the JSON envelope follows an unmatched prose quote
        #   without any newline (PR #4 round 3). Invalid prose-shaped
        #   candidates are filtered by json.loads.
        #
        # Both passes' candidates are collected; the highest-end-position
        # rule (sort by end, return last) selects the genuine envelope.
        for shielded in (True, False):
            opener_stack: list[int] = []
            in_string = False
            escape = False
            for i, ch in enumerate(text):
                if shielded and in_string:
                    if escape:
                        escape = False
                    elif ch == "\\":
                        escape = True
                    elif ch == '"':
                        in_string = False
                    elif ch == "\n":
                        in_string = False
                    continue
                if shielded and ch == '"':
                    in_string = True
                    continue
                if ch == opener:
                    opener_stack.append(i)
                elif ch == closer:
                    if opener_stack:
                        start = opener_stack.pop()
                        end_exc = i + 1
                        candidate = text[start:end_exc]
                        try:
                            json.loads(candidate)
                            candidates.append((start, end_exc, candidate))
                        except json.JSONDecodeError:
                            pass
                    # else: stray prose closer with no matching opener; ignore

    if not candidates:
        return text

    candidates.sort(key=lambda c: c[1])
    return candidates[-1][2]


def extract_final_text(
    tool_outputs: list[str],
    text_outputs: list[str],
    combined_stdout: str,
) -> str:
    """Extract the final text artifact from agent output."""
    for text_out in reversed(text_outputs):
        if text_out.strip():
            return text_out.strip()
    for tool_out in reversed(tool_outputs):
        if tool_out.strip():
            return tool_out.strip()
    return combined_stdout.strip()


def selected_agent_modes(mode_args: list[BenchMode] | None) -> list[BenchMode]:
    return list(mode_args) if mode_args else ["unix", "mdtools", "hybrid"]


# ── CLI ───────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="mdtools benchmark harness")
    parser.add_argument("--run", action="store_true", help="Run agent track")
    parser.add_argument(
        "--headline",
        action="store_true",
        help="Validate the frozen bench v3 manifest before starting an agent run",
    )
    parser.add_argument("--mode", action="append", choices=[
        "unix", "mdtools", "hybrid", "hybrid-no-md",
        "native", "native+md", "native+md-no-md",  # native-rooted arm (claude-cli only)
    ], help="Mode to run. Repeat to run a preregistered multi-mode bundle.")
    parser.add_argument("--task", help="Run only this task ID")
    parser.add_argument("-N", type=int, default=1, help="Runs per task×mode (agent track)")
    parser.add_argument(
        "--runner",
        choices=["claude-cli", "oai-loop", "pi-json"],
        default="claude-cli",
        help="Runner backend for agent execution",
    )
    parser.add_argument("--agent", default="claude -p", help="Agent command")
    parser.add_argument("--model", default=None, help="Model override (e.g., openai-codex/gpt-5.3-codex-spark)")
    parser.add_argument(
        "--thinking",
        choices=["off", "minimal", "low", "medium", "high", "xhigh"],
        default=None,
        help="Thinking level for Pi-backed benchmark runs",
    )
    parser.add_argument("--md-binary", default="md", help="Path to md binary")
    parser.add_argument("--max-turns", type=int, default=30, help="Maximum agent turns per run")
    parser.add_argument(
        "--executor",
        choices=["guarded", "legacy"],
        default="guarded",
        help="Shell execution mode for agent runs",
    )
    parser.add_argument(
        "--log-dir",
        default=None,
        help="Optional directory to persist prompt/output/guard logs for each run",
    )
    parser.add_argument(
        "--results-dir",
        default=None,
        help="Optional directory to persist run.json, results.json, and task_ids.json",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Resume an existing --results-dir bundle by skipping completed mode×task×run cells",
    )
    parser.add_argument(
        "--tasks-path",
        default="bench/tasks/tasks.json",
        help="Path to the benchmark task corpus JSON file",
    )
    parser.add_argument(
        "--task-ids-path",
        default=None,
        help="Optional JSON file containing an ordered task-ID subset from --tasks-path",
    )
    parser.add_argument(
        "--oai-api-base",
        default=os.environ.get("BENCH_OAI_API_BASE") or os.environ.get("OPENAI_BASE_URL"),
        help="OpenAI-compatible API base URL for oai-loop runs",
    )
    parser.add_argument(
        "--oai-api-key",
        default=os.environ.get("BENCH_OAI_API_KEY") or os.environ.get("OPENAI_API_KEY"),
        help="OpenAI-compatible API key for oai-loop runs",
    )
    parser.add_argument(
        "--oai-request-timeout",
        type=int,
        default=180,
        help="HTTP timeout in seconds for each OAI completion request",
    )
    parser.add_argument(
        "--oai-tool-timeout",
        type=int,
        default=30,
        help="Timeout in seconds for each Bash command executed by the oai-loop runner",
    )
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    if args.headline and not args.run:
        parser.error("--headline is only valid with --run")
    if args.resume and not args.results_dir:
        parser.error("--resume requires --results-dir")
    if args.headline:
        try:
            validate_headline_run_request(
                manifest=load_manifest(MANIFEST_PATH),
                runs_per_task=args.N,
                tasks_path=args.tasks_path,
            )
        except ManifestConformanceError as exc:
            parser.error(str(exc))

    started_at = time.time()
    tasks = load_tasks(args.tasks_path)
    # L1 guard scope: the holdout fingerprints are bound to the canonical
    # corpus, not to whatever subset the user happens to be running. Always
    # check the canonical corpus so alternate --tasks-path experiments
    # (subsets, scratch corpora) can run without false-positive breach
    # errors, while loop tampering with bench/tasks/tasks.json still fires.
    holdout_breach = check_holdout_integrity()
    if holdout_breach is not None:
        parser.error(holdout_breach)
    holdout_version = read_holdout_version()
    if args.task_ids_path:
        try:
            tasks = select_tasks(tasks, load_task_ids(args.task_ids_path))
        except ValueError as exc:
            parser.error(str(exc))
    if args.task:
        tasks = [t for t in tasks if t.id == args.task]
        if not tasks:
            parser.error(f"unknown task ID: {args.task}")
    if not tasks:
        parser.error("no tasks selected")

    selected_task_ids = [task.id for task in tasks]

    if args.run and args.runner == "oai-loop":
        if not args.oai_api_base:
            parser.error("--runner oai-loop requires --oai-api-base or BENCH_OAI_API_BASE")
        if not args.oai_api_key:
            parser.error("--runner oai-loop requires --oai-api-key or BENCH_OAI_API_KEY")
        args.model = resolve_oai_model(args.oai_api_base, args.oai_api_key, args.model)

    if not args.run:
        print("=== DRY RUN: dual scorer validation ===\n")
        results = dry_run(tasks, args.md_binary)
        for r in results:
            primary_s = "DIVERGENT" if r.verdict == "divergent" else ("PASS" if r.correct else "FAIL")
            diag_s = "n/a" if r.correct_diagnostic is None else ("PASS" if r.correct_diagnostic else "FAIL")
            div = "" if r.verdict != "divergent" else " SCORER-DIVERGENCE"
            print(f"  {r.task_id}: primary={primary_s} diagnostic={diag_s}{div}")
            for line in r.diff_report.split("\n"):
                if line.strip():
                    print(f"    {line}")
        ok = all(r.correct and r.correct_neutral for r in results)
        if args.results_dir:
            metadata = build_run_metadata(
                run_kind="dry-run",
                tasks_path=args.tasks_path,
                task_ids_path=args.task_ids_path,
                selected_task_ids=selected_task_ids,
                modes=["mdtools"],
                md_binary=args.md_binary,
                runner=None,
                executor=None,
                model=None,
                runs_per_task=1,
                results=results,
                started_at=started_at,
                finished_at=time.time(),
                holdout_version=holdout_version,
            )
            write_run_artifacts(
                args.results_dir,
                metadata=metadata,
                results=results,
                selected_task_ids=selected_task_ids,
            )
        print(f"\n{'All tasks pass dual scorer.' if ok else 'SCORER ISSUES DETECTED.'}")
        sys.exit(0 if ok else 1)

    # Agent track
    modes = selected_agent_modes(args.mode)
    # U3 (FRAC-194): native* modes need claude-cli's native file tools — fail fast
    # if requested on a local runner that can't expose them.
    native_err = native_runner_error(modes, args.runner)
    if native_err:
        parser.error(native_err)
    all_results: list[BenchResult] = []
    effective_log_dir = args.log_dir
    if args.results_dir and not effective_log_dir:
        effective_log_dir = str(Path(args.results_dir) / "logs")
    completed_cells: set[tuple[str, str, int]] = set()
    if args.resume:
        try:
            all_results = load_resume_results(
                args.results_dir,
                selected_task_ids=selected_task_ids,
                modes=modes,
                runs_per_task=args.N,
                runner=args.runner,
                executor=args.executor,
                model=args.model,
                tasks_path=args.tasks_path,
                thinking_level=args.thinking if args.runner == "pi-json" else None,
            )
        except ValueError as exc:
            parser.error(str(exc))
        completed_cells = {
            (result.mode, result.task_id, result.run_index)
            for result in all_results
            if result.run_index is not None
        }
        print(f"Resuming {args.results_dir}: {len(completed_cells)} completed cells loaded.")

    for mode in modes:
        model_label = f", model={args.model}" if args.model else ""
        thinking_label = f", thinking={args.thinking}" if args.thinking and args.runner == "pi-json" else ""
        print(f"\n=== MODE: {mode} (N={args.N}{model_label}{thinking_label}) ===\n")
        for task in tasks:
            for run_i in range(args.N):
                label = f"{task.id} run {run_i+1}/{args.N}" if args.N > 1 else task.id
                if (mode, task.id, run_i) in completed_cells:
                    print(f"  {label}: already complete, skipping.")
                    continue
                print(f"  {label}: {task.description}...")
                result = run_agent(
                    task,
                    mode,
                    args.agent,
                    args.md_binary,
                    args.model,
                    runner=args.runner,
                    executor=args.executor,
                    log_dir=effective_log_dir,
                    max_turns=args.max_turns,
                    oai_api_base=args.oai_api_base,
                    oai_api_key=args.oai_api_key,
                    oai_request_timeout=args.oai_request_timeout,
                    oai_tool_timeout=args.oai_tool_timeout,
                    thinking_level=args.thinking,
                )
                result.run_index = run_i
                if result.quarantine is not None:
                    result.quarantine["run_index"] = run_i
                all_results.append(result)
                s = "DIVERGENT" if result.verdict == "divergent" else ("PASS" if result.correct else "FAIL")
                ns = "n/a" if result.correct_diagnostic is None else ("PASS" if result.correct_diagnostic else "FAIL")
                rq = "↻" if result.requeried else " "
                err = f" | err:{result.runner_error}" if result.runner_error else ""
                print(f"    primary={s} diagnostic={ns} | {result.elapsed_seconds}s | "
                      f"~{result.bytes_output}B out | obs:{result.bytes_observation}B | "
                      f"~{result.tool_calls} calls | {result.mutations} mut | "
                      f"deny:{result.policy_violations} {rq}{err}")
                if args.results_dir:
                    partial_metadata = build_run_metadata(
                        run_kind="agent-track",
                        tasks_path=args.tasks_path,
                        task_ids_path=args.task_ids_path,
                        selected_task_ids=selected_task_ids,
                        modes=modes,
                        md_binary=args.md_binary,
                        runner=args.runner,
                        executor=args.executor,
                        model=args.model,
                        runs_per_task=args.N,
                        thinking_level=args.thinking if args.runner == "pi-json" else None,
                        results=all_results,
                        started_at=started_at,
                        finished_at=time.time(),
                        holdout_version=holdout_version,
                    )
                    write_run_artifacts(
                        args.results_dir,
                        metadata=partial_metadata,
                        results=all_results,
                        selected_task_ids=selected_task_ids,
                    )

    # Summary
    print("\n=== SUMMARY ===\n")
    print(
        f"{'Task':<6} {'Mode':<10} {'Pass':<6} {'Time':>6} {'Calls':>5} "
        f"{'ObsKB':>6} {'Mut':>4} {'Deny':>5} {'RQ':>3}"
    )
    print("-" * 58)
    for r in all_results:
        obs_kb = r.bytes_observation / 1024
        rq = "↻" if r.requeried else ""
        print(f"{r.task_id:<6} {r.mode:<10} "
              f"{'✓' if r.correct else '✗':<6} "
              f"{r.elapsed_seconds:>5.0f}s "
              f"{r.tool_calls:>5} "
              f"{obs_kb:>5.1f}K "
              f"{r.mutations:>4} "
              f"{r.policy_violations:>5} "
              f"{rq:>3}")

    # Aggregate stats per mode
    for mode in modes:
        mode_results = [r for r in all_results if r.mode == mode]
        if mode_results:
            n = len(mode_results)
            pass_rate = sum(1 for r in mode_results if r.correct) / n
            avg_time = sum(r.elapsed_seconds for r in mode_results) / n
            avg_calls = sum(r.tool_calls for r in mode_results) / n
            avg_obs = sum(r.bytes_observation for r in mode_results) / n / 1024
            avg_mut = sum(r.mutations for r in mode_results) / n
            avg_deny = sum(r.policy_violations for r in mode_results) / n
            rq_rate = sum(1 for r in mode_results if r.requeried) / n
            print(f"\n  {mode}: {pass_rate:.0%} pass | {avg_time:.0f}s | {avg_calls:.1f} calls | "
                  f"{avg_obs:.0f}KB obs | {avg_mut:.1f} mut | {avg_deny:.1f} deny | "
                  f"{rq_rate:.0%} requery")

    if args.results_dir:
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path=args.tasks_path,
            task_ids_path=args.task_ids_path,
            selected_task_ids=selected_task_ids,
            modes=modes,
            md_binary=args.md_binary,
            runner=args.runner,
            executor=args.executor,
            model=args.model,
            runs_per_task=args.N,
            thinking_level=args.thinking if args.runner == "pi-json" else None,
            results=all_results,
            started_at=started_at,
            finished_at=time.time(),
            holdout_version=holdout_version,
        )
        write_run_artifacts(
            args.results_dir,
            metadata=metadata,
            results=all_results,
            selected_task_ids=selected_task_ids,
        )

    if args.json:
        print("\n" + json.dumps([asdict(r) for r in all_results], indent=2))


if __name__ == "__main__":
    main()
