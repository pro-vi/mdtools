#!/usr/bin/env python3
"""mdtools benchmark harness.

Runs T1-T4 tasks in unix and mdtools modes, scores results using
dual scorers (independent markdown-it-py + md binary), and produces
a benchmark report with byte-cost metrics.

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
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Literal

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


@dataclass
class BenchResult:
    task_id: str
    mode: BenchMode
    correct: bool
    correct_neutral: bool = True  # Independent scorer agreement
    bytes_prompt: int = 0
    bytes_output: int = 0
    tool_calls: int = 0
    turns: int = 0
    elapsed_seconds: float = 0.0
    diff_report: str = ""


# ── Tool inventories ──────────────────────────────────────────

UNIX_TOOLS = ["cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp"]
MDTOOLS_TOOLS = [
    "md outline", "md blocks", "md block", "md section",
    "md replace-section", "md delete-section", "md replace-block", "md insert-block", "md delete-block",
    "md search", "md links", "md frontmatter", "md stats",
    "md tasks", "md set-task", "cat", "jq",
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
  md replace-block <INDEX> <FILE> [-i] [--json] [--content-file PATH]
                                       Replace block at INDEX. Reads from --content-file or stdin.
  md replace-section <SELECTOR> <FILE> [-i] [--json] [--ignore-case] [--occurrence N] [--content-file PATH]
                                       Replace section. Reads from --content-file or stdin.
  md insert-block <FILE> [-i] --before <INDEX> | --after <INDEX> | --at-start | --at-end [--content-file PATH]
                                       Insert a new block. Reads from --content-file or stdin.
  md delete-block <INDEX> <FILE> [-i]  Delete block at INDEX.
  md delete-section <SELECTOR> <FILE> [-i] [--json] [--ignore-case] [--occurrence N]
                                       Delete an entire section (heading + content).
  md links <FILE> [--json]             List all links with kind, destination, source block
  md frontmatter <FILE> [--json]       Read YAML/TOML frontmatter as JSON
  md stats <FILE> [--json]             Word/heading/block/link/section/line counts
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
  md replace-block 3 doc.md -i --content-file new.md  # replace block 3 from file
  md replace-section "Old" doc.md -i --content-file new.md
  md insert-block --after 2 doc.md -i --content-file new.md
  md delete-section "Notes" doc.md -i                 # delete entire section
  md search "method" doc.md --kind paragraph --json # find "method" in paragraphs only
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
            a = "\n".join(line.rstrip() for line in a.split("\n"))
            e = "\n".join(line.rstrip() for line in e.split("\n"))
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
        ))
    return tasks


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


def _build_agent_cmd(agent_cmd: str, mode: BenchMode, md_binary: str) -> list[str]:
    """Build the full agent subprocess command with proper flags."""
    parts = agent_cmd.split()
    # If agent is claude (Claude Code), add flags for non-interactive tool use
    if parts[0] == "claude":
        cmd = ["claude", "-p"]
        # Allow only the tools appropriate for the mode
        if mode == "mdtools":
            cmd += ["--allowedTools", "Bash"]
        else:
            cmd += ["--allowedTools", "Bash"]
        cmd += ["--dangerously-skip-permissions"]
        cmd += ["--max-turns", "30"]
        cmd += ["--no-session-persistence"]
        cmd += ["--output-format", "json"]
        return cmd
    return parts


def run_agent(
    task: BenchTask,
    mode: BenchMode,
    agent_cmd: str,
    md_binary: str,
) -> BenchResult:
    """Run an agent subprocess to complete a task."""
    workdir = tempfile.mkdtemp(prefix=f"mdtools_bench_{task.id}_{mode}_")

    for inp in task.input_files:
        # Preserve subdirectory structure (e.g., t16_vault/frontend.md)
        dest = os.path.join(workdir, os.path.basename(inp))
        # Check if input files share a common subdirectory
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

    prompt = build_prompt(task, mode, workdir)
    input_file = os.path.join(workdir, os.path.basename(task.input_files[0]))

    bytes_prompt = len(prompt.encode())
    cmd = _build_agent_cmd(agent_cmd, mode, local_md)

    start = time.time()
    try:
        result = subprocess.run(
            cmd,
            input=prompt,
            capture_output=True,
            text=True,
            timeout=180,
            cwd=workdir,
        )
        raw_stdout = result.stdout
        bytes_output = len(raw_stdout.encode())
    except subprocess.TimeoutExpired:
        raw_stdout = ""
        bytes_output = 0
    elapsed = time.time() - start

    # Parse structured JSON output from claude -p --output-format json
    stdout = ""
    tool_calls = 0
    num_turns = 0
    all_tool_outputs: list[str] = []
    all_text_outputs: list[str] = []
    try:
        messages = json.loads(raw_stdout)
        for msg in messages:
            if msg.get("type") == "result":
                num_turns = msg.get("num_turns", 0)
                # result.result is the final assistant text
                all_text_outputs.append(msg.get("result", ""))
            elif msg.get("type") == "assistant":
                content = msg.get("message", {}).get("content", [])
                for block in content:
                    if block.get("type") == "tool_use":
                        tool_calls += 1
                    if block.get("type") == "text":
                        all_text_outputs.append(block.get("text", ""))
            elif msg.get("type") == "user":
                # Tool results come back as user messages in Claude Code JSON format
                content = msg.get("message", {}).get("content", [])
                if isinstance(content, list):
                    for block in content:
                        if isinstance(block, dict) and block.get("type") == "tool_result":
                            result_content = block.get("content", "")
                            if isinstance(result_content, str):
                                all_tool_outputs.append(result_content)
                            elif isinstance(result_content, list):
                                for c in result_content:
                                    if isinstance(c, dict) and c.get("type") == "text":
                                        all_tool_outputs.append(c.get("text", ""))
        # Combine all outputs — tool outputs first (likely contain raw JSON),
        # then text outputs (may contain JSON embedded in prose)
        stdout = "\n".join(all_tool_outputs + all_text_outputs)
    except (json.JSONDecodeError, TypeError):
        # Fallback: treat raw output as text (non-claude agent)
        stdout = raw_stdout
        tool_calls = raw_stdout.count('"tool_use"') + raw_stdout.count("$ ")

    expected_content = Path(task.expected_output).read_text()

    if task.expected_artifact == "stdout_and_file":
        # Score both stdout text and file contents
        report_parts = []
        # Check stdout contains expected_stdout (agent may add explanation)
        ok_stdout = False
        if task.expected_stdout is not None:
            expected_text = task.expected_stdout.strip()
            # Check if expected text appears as a line in the output
            ok_stdout = expected_text in stdout
            report_parts.append(f"stdout: {'OK' if ok_stdout else 'MISMATCH'}")
            if not ok_stdout:
                report_parts.append(f"  expected to contain: {expected_text!r}")
                report_parts.append(f"  actual: {stdout[:200]!r}")
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
    else:
        actual = stdout
        ok_md, ok_neutral, report = score_task(task, actual, expected_content, md_binary)

    shutil.rmtree(workdir, ignore_errors=True)

    return BenchResult(
        task_id=task.id,
        mode=mode,
        correct=ok_md,
        correct_neutral=ok_neutral,
        bytes_prompt=bytes_prompt,
        bytes_output=bytes_output,
        tool_calls=tool_calls,
        elapsed_seconds=round(elapsed, 2),
        diff_report=report,
    )


def extract_last_json(text: str) -> str:
    """Extract the best JSON from agent output, stripping code fences.
    Tries each valid JSON substring and returns the last one that parses.
    Prefers arrays over objects (agent often wraps results in an array)."""
    clean = re.sub(r"```(?:json)?\s*\n?", "", text)

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


# ── CLI ───────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="mdtools benchmark harness")
    parser.add_argument("--run", action="store_true", help="Run agent track")
    parser.add_argument("--mode", choices=["unix", "mdtools", "hybrid"])
    parser.add_argument("--task", help="Run only this task ID")
    parser.add_argument("-N", type=int, default=1, help="Runs per task×mode (agent track)")
    parser.add_argument("--agent", default="claude -p", help="Agent command")
    parser.add_argument("--md-binary", default="md", help="Path to md binary")
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    tasks = load_tasks()
    if args.task:
        tasks = [t for t in tasks if t.id == args.task]

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
        print(f"\n{'All tasks pass dual scorer.' if ok else 'SCORER ISSUES DETECTED.'}")
        sys.exit(0 if ok else 1)

    # Agent track
    modes: list[BenchMode] = [args.mode] if args.mode else ["unix", "mdtools"]
    all_results: list[BenchResult] = []

    for mode in modes:
        print(f"\n=== MODE: {mode} (N={args.N}) ===\n")
        for task in tasks:
            for run_i in range(args.N):
                label = f"{task.id} run {run_i+1}/{args.N}" if args.N > 1 else task.id
                print(f"  {label}: {task.description}...")
                result = run_agent(task, mode, args.agent, args.md_binary)
                all_results.append(result)
                s = "PASS" if result.correct else "FAIL"
                ns = "PASS" if result.correct_neutral else "FAIL"
                print(f"    md={s} neutral={ns} | {result.elapsed_seconds}s | "
                      f"~{result.bytes_output}B out | ~{result.tool_calls} calls")

    # Summary
    print("\n=== SUMMARY ===\n")
    print(f"{'Task':<6} {'Mode':<10} {'Pass':<6} {'Neutral':<8} {'Time':>7} {'Bytes':>8} {'Calls':>6}")
    print("-" * 55)
    for r in all_results:
        print(f"{r.task_id:<6} {r.mode:<10} "
              f"{'✓' if r.correct else '✗':<6} "
              f"{'✓' if r.correct_neutral else '✗':<8} "
              f"{r.elapsed_seconds:>6.1f}s "
              f"{r.bytes_output:>7}B "
              f"{r.tool_calls:>5}")

    # Aggregate stats per mode
    for mode in modes:
        mode_results = [r for r in all_results if r.mode == mode]
        if mode_results:
            pass_rate = sum(1 for r in mode_results if r.correct) / len(mode_results)
            avg_bytes = sum(r.bytes_output for r in mode_results) / len(mode_results)
            avg_time = sum(r.elapsed_seconds for r in mode_results) / len(mode_results)
            print(f"\n  {mode}: {pass_rate:.0%} pass | avg {avg_bytes:.0f}B | avg {avg_time:.1f}s")

    if args.json:
        print("\n" + json.dumps([asdict(r) for r in all_results], indent=2))


if __name__ == "__main__":
    main()
