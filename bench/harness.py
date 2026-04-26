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
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, asdict, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal

try:
    from bench.command_policy import create_restricted_shell_env, load_guard_events
    from bench.oai_loop import LoopError, resolve_oai_model, run_oai_loop
    from bench.pi_audit_adapter import load_pi_audit_events, summarize_pi_audit_events
    from bench.pi_runner import build_pi_json_command, default_audit_extension_path, parse_pi_json_output
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from bench.command_policy import create_restricted_shell_env, load_guard_events
    from bench.oai_loop import LoopError, resolve_oai_model, run_oai_loop
    from bench.pi_audit_adapter import load_pi_audit_events, summarize_pi_audit_events
    from bench.pi_runner import build_pi_json_command, default_audit_extension_path, parse_pi_json_output

# ── Independent scorer (no md dependency) ─────────────────────

from markdown_it import MarkdownIt

_NEUTRAL_MD = MarkdownIt("commonmark", {"html": True}).enable(["table"])


def neutral_heading_tree(content: str) -> list[tuple[int, str]]:
    """Extract heading tree using markdown-it-py (independent of md binary)."""
    tokens = _NEUTRAL_MD.parse(content)
    tree = []
    for i, tok in enumerate(tokens):
        if tok.type == "heading_open":
            level = int(tok.tag[1:])
            # Next token is the inline content
            if i + 1 < len(tokens) and tokens[i + 1].type == "inline":
                text = tokens[i + 1].content
            else:
                text = ""
            tree.append((level, text))
    return tree


def neutral_block_texts(content: str) -> list[str]:
    """Extract normalized block-level text using markdown-it-py."""
    tokens = _NEUTRAL_MD.parse(content)
    texts = []
    i = 0
    while i < len(tokens):
        tok = tokens[i]
        # Collect top-level blocks: headings, paragraphs, code, tables, etc.
        if tok.type in ("heading_open", "paragraph_open", "bullet_list_open",
                        "ordered_list_open", "blockquote_open", "table_open",
                        "html_block", "code_block", "fence", "hr"):
            # Collect all content until the matching close
            if tok.type == "hr":
                texts.append("---")
                i += 1
                continue
            if tok.type in ("html_block", "code_block", "fence"):
                texts.append((tok.content or "").strip())
                i += 1
                continue
            # Find the close token
            close_type = tok.type.replace("_open", "_close")
            depth = 1
            parts = []
            i += 1
            while i < len(tokens) and depth > 0:
                t = tokens[i]
                if t.type == tok.type:
                    depth += 1
                elif t.type == close_type:
                    depth -= 1
                    if depth == 0:
                        break
                if t.type == "inline" and t.content:
                    parts.append(t.content)
                elif t.type in ("code_block", "fence") and t.content:
                    parts.append(t.content.strip())
                i += 1
            texts.append("\n".join(parts).strip())
        i += 1
    return [t for t in texts if t]  # Drop empties


# ── Types ─────────────────────────────────────────────────────

BenchMode = Literal["unix", "mdtools", "hybrid"]
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
    correct: bool
    correct_neutral: bool = True  # Independent scorer agreement
    model: str | None = None
    thinking_level: str | None = None
    bytes_prompt: int = 0
    bytes_output: int = 0
    bytes_observation: int = 0   # total tool-result content agent had to read
    tool_calls: int = 0
    turns: int = 0
    mutations: int = 0           # write tool calls (set-task, replace-*, insert-*, delete-*)
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
    tool_outputs: list[str] = field(default_factory=list)
    text_outputs: list[str] = field(default_factory=list)
    runner_error: str | None = None


# ── Tool inventories ──────────────────────────────────────────

UNIX_TOOLS = ["cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp"]
MDTOOLS_TOOLS = [
    "md outline", "md blocks", "md block", "md section",
    "md replace-section", "md delete-section", "md replace-block", "md insert-block", "md delete-block",
    "md search", "md links", "md frontmatter", "md stats",
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
  md insert-block <FILE> [-i] --before <INDEX> | --after <INDEX> | --at-start | --at-end [--from PATH]
                                       Insert a new block. Reads from --from or stdin.
  md delete-block <INDEX> <FILE> [-i]  Delete block at INDEX.
  md delete-section <SELECTOR> <FILE> [-i] [--json] [--ignore-case] [--occurrence N]
                                       Delete an entire section (heading + content).
  md links <FILE> [--json]             List all links with kind, destination, source block
  md frontmatter <FILE> [--json]       Read YAML/TOML frontmatter as JSON
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
  md delete-section "Notes" doc.md -i                 # delete entire section
  md search "method" doc.md --kind paragraph --json # find "method" in paragraphs only
  md table report.md --select Feature,Status         # project table columns as TSV
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

HYBRID_DOCS = MDTOOLS_DOCS.rstrip() + """

STANDARD POSIX TOOLS (also available):
  cat, grep, sed, awk, head, tail, wc, tee, mv, cp
  Shell: pipes (|), redirection (>, >>), temp files (mktemp)

Choose the best tool for each step. Use md commands for structural
markdown operations and unix tools for simple text manipulation.
Do NOT use: python, perl, ruby, node, or any other scripting language.
"""


def build_prompt(task: BenchTask, mode: BenchMode, workdir: str) -> str:
    if mode == "mdtools":
        tool_docs = MDTOOLS_DOCS
    elif mode == "hybrid":
        tool_docs = HYBRID_DOCS
    else:
        tool_docs = UNIX_DOCS

    # Determine input reference for prompt
    if len(task.input_files) > 1:
        inp_dir = os.path.basename(os.path.dirname(task.input_files[0]))
        if inp_dir and inp_dir != "inputs":
            input_file = os.path.join(workdir, inp_dir) + "/"
        else:
            input_file = " ".join(
                os.path.join(workdir, os.path.basename(f)) for f in task.input_files
            )
    else:
        input_file = os.path.join(workdir, os.path.basename(task.input_files[0]))

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


# ── Dual scorer ───────────────────────────────────────────────

def score_task(
    task: BenchTask,
    actual: str,
    expected: str,
    md_binary: str = "md",
) -> tuple[bool, bool, str]:
    """Score using both md-based and neutral scorers.
    Returns (correct_md, correct_neutral, diff_report)."""
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
        ok = a == e
        if not ok:
            report.append(f"raw_bytes: MISMATCH ({len(e)}b expected, {len(a)}b actual)")
        return ok, ok, "\n".join(report)

    # Normalize
    a, e = actual, expected
    if policy.normalize_line_endings:
        a = a.replace("\r\n", "\n")
        e = e.replace("\r\n", "\n")
    if policy.ignore_trailing_whitespace:
        a = "\n".join(line.rstrip() for line in a.split("\n"))
        e = "\n".join(line.rstrip() for line in e.split("\n"))

    if policy.kind == "structural" and policy.json_canonical:
        ok_md, ok_neutral = score_json_canonical(policy, a, e, report)
        return ok_md, ok_neutral, "\n".join(report)

    if policy.kind == "structural" and task.expected_artifact == "json_envelope":
        ok_md, ok_neutral = score_structural_json(policy, a, e, report)
        return ok_md, ok_neutral, "\n".join(report)

    if policy.kind == "normalized_text":
        ok_md = score_normalized_text_md(policy, a, e, md_binary, report)
        ok_neutral = score_normalized_text_neutral(policy, a, e, report)
        if ok_md != ok_neutral:
            report.append(f"  ⚠ SCORER DIVERGENCE: md={ok_md} neutral={ok_neutral}")
        return ok_md, ok_neutral, "\n".join(report)

    ok = a.strip() == e.strip()
    return ok, ok, "\n".join(report)


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
    """Score JSON output with canonical comparison. Returns (ok, ok)."""
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
    """Score JSON envelope. Returns (md_ok, neutral_ok)."""
    try:
        actual_json = json.loads(actual_str)
        expected_json = json.loads(expected_str)
    except json.JSONDecodeError as exc:
        report.append(f"JSON parse error: {exc}")
        return False, False

    if not isinstance(actual_json, dict) or not isinstance(expected_json, dict):
        report.append(
            "json_envelope: MISMATCH expected top-level JSON object "
            f"(actual={type(actual_json).__name__}, expected={type(expected_json).__name__})"
        )
        return False, False

    ok_md = True
    ok_neutral = True

    if policy.compare_heading_tree:
        # md-based: from JSON structure
        a_tree = [(e["heading"]["level"], e["heading"]["text"]) for e in actual_json.get("entries", [])]
        e_tree = [(e["heading"]["level"], e["heading"]["text"]) for e in expected_json.get("entries", [])]
        if a_tree != e_tree:
            ok_md = False
            report.append(f"heading_tree [md]: MISMATCH")
        else:
            report.append("heading_tree [md]: OK")

        # Neutral check: can't easily re-parse JSON task expected from markdown
        # For JSON envelope tasks, both scorers agree on the JSON structure
        ok_neutral = ok_md
        report.append(f"heading_tree [neutral]: {'OK' if ok_neutral else 'MISMATCH'}")

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
            ok_md = False
            ok_neutral = False
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
            ok_md = False
            ok_neutral = False
            report.append(f"link_destinations: MISMATCH {expected_links} vs {actual_links}")
        else:
            report.append("link_destinations: OK")

    return ok_md, ok_neutral


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
    return [content[b["span"]["byte_start"]:b["span"]["byte_end"]].strip()
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
        "thinking_level": resolved_thinking_level,
        "runs_per_task": runs_per_task,
        "aggregates": {
            "overall": aggregate_results(results),
            "by_mode": by_mode,
        },
    }


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


def dry_run(tasks: list[BenchTask], md_binary: str) -> list[BenchResult]:
    """Validate dual scorer: expected vs itself should pass both paths."""
    results = []
    for task in tasks:
        expected = Path(task.expected_output).read_text()

        if task.expected_artifact == "stdout_and_file":
            # For safe-fail tasks: expected_output is the unchanged input file,
            # expected_stdout is the text that should appear on stdout.
            # Dry-run: verify file identity passes and stdout matches itself.
            ok_file, _, file_report = score_task(task, expected, expected, md_binary)
            ok_stdout = task.expected_stdout is not None
            report = file_report
            if ok_stdout:
                report += "\nstdout_check: OK (dry-run)"
            ok_md = ok_file and ok_stdout
            ok_neutral = ok_md
        else:
            ok_md, ok_neutral, report = score_task(task, expected, expected, md_binary)

        results.append(BenchResult(
            task_id=task.id, mode="mdtools",
            correct=ok_md, correct_neutral=ok_neutral,
            diff_report=report,
        ))
    return results


def _build_agent_cmd(
    agent_cmd: str,
    mode: BenchMode,
    md_binary: str,
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
        cmd += ["--bare"]
        cmd += ["--tools", "Bash"]
        cmd += ["--allowedTools", "Bash"]
        cmd += ["--dangerously-skip-permissions"]
        cmd += ["--max-turns", str(max_turns)]
        cmd += ["--no-session-persistence"]
        cmd += ["--output-format", "json"]
        return cmd
    return parts


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

    # Copy md binary into workdir so it's accessible by relative path
    if md_binary != "md":
        md_dest = os.path.join(workdir, "md")
        shutil.copy2(md_binary, md_dest)
        local_md = "./md"
    else:
        local_md = "md"
        md_dest = os.path.join(workdir, "md")
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

    prompt = build_prompt(task, mode, workdir)
    input_file = os.path.join(workdir, os.path.basename(task.input_files[0]))

    bytes_prompt = len(prompt.encode())

    start = time.time()
    raw_stdout = ""
    stdout = ""
    tool_calls = 0
    num_turns = 0
    bytes_observation = 0
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
            tools=("bash",),
            thinking_level=thinking_level,
        )
        pi_env = child_env.copy()
        pi_env["PI_AUDIT_LOG"] = str(pi_audit_log_path)
        pi_env.setdefault("PI_SKIP_VERSION_CHECK", "1")
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
        cmd = _build_agent_cmd(agent_cmd, mode, local_md, model, max_turns)
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
        all_tool_outputs = parsed_output.tool_outputs
        all_text_outputs = parsed_output.text_outputs
        runner_error = parsed_output.runner_error
        resolved_model = model or parsed_output.model
    elapsed = time.time() - start

    guard_events = load_guard_events(guard_log_path) if guard_log_path is not None else []
    for event in guard_events:
        if event.decision != "allow":
            policy_violations += 1
            continue
        kind = event.kind
        if kind == "mutation":
            mutations += 1
            call_sequence.append("mutation")
        elif kind == "query":
            call_sequence.append("query")

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

    saw_mutation = False
    for kind in call_sequence:
        if kind == "mutation":
            saw_mutation = True
        elif kind == "query" and saw_mutation:
            requeried = True
            break

    expected_content = Path(task.expected_output).read_text()

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
        ok_file_md, ok_file_n, file_report = score_task(task, actual_file, expected_content, md_binary)
        report_parts.append(file_report)
        ok_md = ok_stdout and ok_file_md
        ok_neutral = ok_stdout and ok_file_n
        report = "\n".join(report_parts)
    elif task.expected_artifact == "file_contents":
        try:
            actual = Path(input_file).read_text()
        except FileNotFoundError:
            actual = ""
        ok_md, ok_neutral, report = score_task(task, actual, expected_content, md_binary)
    elif task.expected_artifact == "json_envelope":
        # Search tool outputs individually (last non-trivial JSON wins),
        # then fall back to text outputs, then combined stdout.
        actual = ""
        for tool_out in reversed(all_tool_outputs):
            try:
                parsed = json.loads(tool_out.strip())
                if (isinstance(parsed, (list, dict)) and len(parsed) > 0):
                    actual = tool_out.strip()
                    break
            except (json.JSONDecodeError, TypeError):
                continue
        if not actual:
            # Try extracting from text outputs (agent may embed JSON in prose)
            for text_out in reversed(all_text_outputs):
                candidate = extract_last_json(text_out)
                try:
                    parsed = json.loads(candidate)
                    if (isinstance(parsed, (list, dict)) and len(parsed) > 0):
                        actual = candidate
                        break
                except (json.JSONDecodeError, TypeError):
                    continue
        if not actual:
            actual = extract_last_json(stdout)
        ok_md, ok_neutral, report = score_task(task, actual, expected_content, md_binary)
    elif task.expected_artifact == "stdout_text":
        actual = extract_final_text(all_tool_outputs, all_text_outputs, stdout)
        ok_md, ok_neutral, report = score_task(task, actual, expected_content, md_binary)
    else:
        actual = stdout
        ok_md, ok_neutral, report = score_task(task, actual, expected_content, md_binary)

    if runner_error and (not ok_md or not ok_neutral):
        report = f"runner_error: {runner_error}\n{report}" if report else f"runner_error: {runner_error}"

    if log_dir:
        run_log_dir = Path(log_dir) / f"{task.id}_{mode}_{int(start)}"
        run_log_dir.mkdir(parents=True, exist_ok=True)
        (run_log_dir / "prompt.txt").write_text(prompt)
        (run_log_dir / "agent_output.txt").write_text(raw_stdout)
        if guard_log_path is not None and guard_log_path.exists():
            shutil.copy2(guard_log_path, run_log_dir / "guard.log")
        if pi_audit_log_path is not None and pi_audit_log_path.exists():
            shutil.copy2(pi_audit_log_path, run_log_dir / "pi-audit.jsonl")
        pi_session_root = workdir_path / ".pi-sessions"
        if pi_session_root.exists():
            shutil.copytree(pi_session_root, run_log_dir / "pi-sessions", dirs_exist_ok=True)

    shutil.rmtree(workdir, ignore_errors=True)

    return BenchResult(
        task_id=task.id,
        mode=mode,
        correct=ok_md,
        correct_neutral=ok_neutral,
        model=resolved_model,
        thinking_level=resolved_thinking_level,
        bytes_prompt=bytes_prompt,
        bytes_output=bytes_output,
        bytes_observation=bytes_observation,
        tool_calls=tool_calls,
        turns=num_turns,
        mutations=mutations,
        policy_violations=policy_violations,
        requeried=requeried,
        invalid_responses=invalid_responses,
        unique_invalid_responses=unique_invalid_responses,
        elapsed_seconds=round(elapsed, 2),
        diff_report=report,
        runner_error=runner_error,
    )


def extract_last_json(text: str) -> str:
    """Extract the best JSON from agent output, stripping code fences.
    Tries each valid JSON substring and returns the last one that parses.
    Prefers arrays over objects (agent often wraps results in an array)."""
    clean = re.sub(r"```(?:json)?\s*\n?", "", text)

    try:
        parsed = json.loads(clean)
        if isinstance(parsed, (list, dict)):
            return clean
    except json.JSONDecodeError:
        pass

    candidates = []
    for opener, closer in [("[", "]"), ("{", "}")]:
        depth = 0
        start = -1
        for i, ch in enumerate(clean):
            if ch == opener:
                if depth == 0:
                    start = i
                depth += 1
            elif ch == closer:
                depth -= 1
                if depth == 0 and start >= 0:
                    candidate = clean[start:i + 1]
                    try:
                        json.loads(candidate)
                        candidates.append((start, candidate, opener))
                    except json.JSONDecodeError:
                        pass
                    start = -1

    if not candidates:
        return text

    # Prefer: last array > last object
    # "Last" because the agent typically produces the final answer after intermediate results
    arrays = [c for _, c, o in candidates if o == "["]
    objects = [c for _, c, o in candidates if o == "{"]

    if arrays:
        return arrays[-1]
    if objects:
        return objects[-1]
    return text


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


# ── CLI ───────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="mdtools benchmark harness")
    parser.add_argument("--run", action="store_true", help="Run agent track")
    parser.add_argument("--mode", choices=["unix", "mdtools", "hybrid"])
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
        default=60,
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

    started_at = time.time()
    tasks = load_tasks(args.tasks_path)
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
            md_s = "PASS" if r.correct else "FAIL"
            ne_s = "PASS" if r.correct_neutral else "FAIL"
            div = "" if r.correct == r.correct_neutral else " ⚠ DIVERGENCE"
            print(f"  {r.task_id}: md={md_s} neutral={ne_s}{div}")
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
    modes: list[BenchMode] = [args.mode] if args.mode else ["unix", "mdtools", "hybrid"]
    all_results: list[BenchResult] = []
    effective_log_dir = args.log_dir
    if args.results_dir and not effective_log_dir:
        effective_log_dir = str(Path(args.results_dir) / "logs")

    for mode in modes:
        model_label = f", model={args.model}" if args.model else ""
        thinking_label = f", thinking={args.thinking}" if args.thinking and args.runner == "pi-json" else ""
        print(f"\n=== MODE: {mode} (N={args.N}{model_label}{thinking_label}) ===\n")
        for task in tasks:
            for run_i in range(args.N):
                label = f"{task.id} run {run_i+1}/{args.N}" if args.N > 1 else task.id
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
                all_results.append(result)
                s = "PASS" if result.correct else "FAIL"
                ns = "PASS" if result.correct_neutral else "FAIL"
                rq = "↻" if result.requeried else " "
                err = f" | err:{result.runner_error}" if result.runner_error else ""
                print(f"    md={s} neutral={ns} | {result.elapsed_seconds}s | "
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
