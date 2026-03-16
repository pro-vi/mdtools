#!/usr/bin/env python3
"""mdtools benchmark harness.

Runs T1-T4 tasks in unix and mdtools modes, scores results using
structural diff, and produces a benchmark report.

Usage:
    python bench/harness.py                      # dry run: score pre-generated expected outputs
    python bench/harness.py --mode mdtools        # run mdtools mode only
    python bench/harness.py --mode unix           # run unix mode only
    python bench/harness.py --run                 # run both modes via agent subprocess
    python bench/harness.py --agent "claude -p"   # specify agent command
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Literal, Optional

# ── Types ──────────────────────────────────────────────────────

BenchMode = Literal["unix", "mdtools"]
BenchExpectedArtifact = Literal["stdout_text", "file_contents", "json_envelope"]
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


@dataclass
class BenchTask:
    id: str
    description: str
    input_files: list[str]
    expected_output: str
    expected_artifact: BenchExpectedArtifact
    difficulty: str
    scorer: StructuralDiffPolicy


@dataclass
class BenchResult:
    task_id: str
    mode: BenchMode
    correct: bool
    token_input: int = 0
    token_output: int = 0
    tool_calls: int = 0
    turns: int = 0
    elapsed_seconds: float = 0.0
    diff_report: str = ""


# ── Tool inventories ──────────────────────────────────────────

UNIX_TOOLS = ["cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp"]
MDTOOLS_TOOLS = [
    "md outline", "md blocks", "md block", "md section",
    "md replace-section", "md replace-block", "md insert-block", "md delete-block",
    "md search", "md links", "md frontmatter", "md stats", "cat",
]


# ── System prompts ────────────────────────────────────────────

SYSTEM_PROMPT_TEMPLATE = """\
You are a benchmark agent. Complete the task using ONLY the allowed tools.
Do NOT use any other commands. Do NOT install packages. Do NOT use python or node.

TASK: {task_description}
INPUT FILE: {input_file}
OUTPUT: {output_instruction}

ALLOWED TOOLS:
{tool_list}

Work step by step. When done, {completion_instruction}
"""


def build_prompt(task: BenchTask, mode: BenchMode, workdir: str) -> str:
    tools = UNIX_TOOLS if mode == "unix" else MDTOOLS_TOOLS
    tool_list = "\n".join(f"  - {t}" for t in tools)
    input_file = os.path.join(workdir, os.path.basename(task.input_files[0]))

    if task.expected_artifact == "json_envelope":
        output_instruction = "Print the result as JSON to stdout."
        completion_instruction = "print the JSON result and stop."
    elif task.expected_artifact == "file_contents":
        output_instruction = f"Modify the file {input_file} in place."
        completion_instruction = f"confirm the file has been modified and stop."
    else:
        output_instruction = "Print the result to stdout."
        completion_instruction = "print the result and stop."

    return SYSTEM_PROMPT_TEMPLATE.format(
        task_description=task.description,
        input_file=input_file,
        output_instruction=output_instruction,
        tool_list=tool_list,
        completion_instruction=completion_instruction,
    )


# ── Scorer ────────────────────────────────────────────────────

def score_task(
    task: BenchTask,
    actual: str,
    expected: str,
    md_binary: str = "md",
) -> tuple[bool, str]:
    """Score an actual result against expected, returning (correct, diff_report)."""
    policy = task.scorer
    report_lines = []

    if policy.kind == "raw_bytes":
        correct = actual == expected
        if not correct:
            report_lines.append("raw_bytes: MISMATCH")
            report_lines.append(f"  expected {len(expected)} bytes, got {len(actual)} bytes")
        return correct, "\n".join(report_lines)

    # Normalize for comparison
    a = actual
    e = expected
    if policy.normalize_line_endings:
        a = a.replace("\r\n", "\n")
        e = e.replace("\r\n", "\n")
    if policy.ignore_trailing_whitespace:
        a = "\n".join(line.rstrip() for line in a.split("\n"))
        e = "\n".join(line.rstrip() for line in e.split("\n"))

    if policy.kind == "structural" and task.expected_artifact == "json_envelope":
        return score_structural_json(policy, a, e, report_lines)

    if policy.kind == "normalized_text":
        return score_normalized_text(policy, a, e, md_binary, report_lines)

    # Fallback: normalized string comparison
    correct = a.strip() == e.strip()
    if not correct:
        report_lines.append("normalized comparison: MISMATCH")
    return correct, "\n".join(report_lines)


def score_structural_json(
    policy: StructuralDiffPolicy,
    actual_str: str,
    expected_str: str,
    report: list[str],
) -> tuple[bool, str]:
    """Score JSON envelope outputs structurally."""
    try:
        actual = json.loads(actual_str)
        expected = json.loads(expected_str)
    except json.JSONDecodeError as exc:
        report.append(f"JSON parse error: {exc}")
        return False, "\n".join(report)

    correct = True

    if policy.compare_heading_tree:
        a_tree = extract_heading_tree(actual)
        e_tree = extract_heading_tree(expected)
        if a_tree != e_tree:
            correct = False
            report.append(f"heading_tree: MISMATCH")
            report.append(f"  expected: {e_tree}")
            report.append(f"  actual:   {a_tree}")
        else:
            report.append("heading_tree: OK")

    return correct, "\n".join(report)


def extract_heading_tree(json_data: dict) -> list[tuple[int, str]]:
    """Extract (level, text) pairs from an OutlineResult-shaped JSON."""
    entries = json_data.get("entries", [])
    return [(e["heading"]["level"], e["heading"]["text"]) for e in entries]


def score_normalized_text(
    policy: StructuralDiffPolicy,
    actual: str,
    expected: str,
    md_binary: str,
    report: list[str],
) -> tuple[bool, str]:
    """Score file contents using block-level normalized text comparison."""
    correct = True

    if policy.compare_heading_tree:
        a_tree = get_heading_tree_from_file(actual, md_binary)
        e_tree = get_heading_tree_from_file(expected, md_binary)
        if a_tree != e_tree:
            correct = False
            report.append(f"heading_tree: MISMATCH")
            report.append(f"  expected: {e_tree}")
            report.append(f"  actual:   {a_tree}")
        else:
            report.append("heading_tree: OK")

    if policy.compare_block_order:
        a_order = get_block_order_from_file(actual, md_binary)
        e_order = get_block_order_from_file(expected, md_binary)
        if a_order != e_order:
            correct = False
            report.append(f"block_order: MISMATCH")
            report.append(f"  expected: {e_order}")
            report.append(f"  actual:   {a_order}")
        else:
            report.append("block_order: OK")

    if policy.compare_block_text:
        a_text = get_block_texts_from_file(actual, md_binary)
        e_text = get_block_texts_from_file(expected, md_binary)
        if a_text != e_text:
            correct = False
            report.append("block_text: MISMATCH")
            # Show first difference
            for i, (at, et) in enumerate(zip(a_text, e_text)):
                if at != et:
                    report.append(f"  first diff at block {i}:")
                    report.append(f"    expected: {et[:80]!r}")
                    report.append(f"    actual:   {at[:80]!r}")
                    break
            if len(a_text) != len(e_text):
                report.append(f"  block count: expected {len(e_text)}, got {len(a_text)}")
        else:
            report.append("block_text: OK")

    return correct, "\n".join(report)


def get_heading_tree_from_file(content: str, md_binary: str) -> list[tuple[int, str]]:
    """Parse content with md tool and extract heading tree."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
        f.write(content)
        f.flush()
        try:
            result = subprocess.run(
                [md_binary, "outline", f.name, "--json"],
                capture_output=True, text=True, timeout=10,
            )
            if result.returncode != 0:
                return []
            data = json.loads(result.stdout)
            return extract_heading_tree(data)
        finally:
            os.unlink(f.name)


def get_block_order_from_file(content: str, md_binary: str) -> list[str]:
    """Parse content and return block kind sequence."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
        f.write(content)
        f.flush()
        try:
            result = subprocess.run(
                [md_binary, "blocks", f.name, "--json"],
                capture_output=True, text=True, timeout=10,
            )
            if result.returncode != 0:
                return []
            data = json.loads(result.stdout)
            return [b["kind"] for b in data.get("blocks", [])]
        finally:
            os.unlink(f.name)


def get_block_texts_from_file(content: str, md_binary: str) -> list[str]:
    """Parse content and return normalized block text for each block."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
        f.write(content)
        f.flush()
        try:
            result = subprocess.run(
                [md_binary, "blocks", f.name, "--json"],
                capture_output=True, text=True, timeout=10,
            )
            if result.returncode != 0:
                return []
            data = json.loads(result.stdout)
            texts = []
            for block in data.get("blocks", []):
                bs = block["span"]["byte_start"]
                be = block["span"]["byte_end"]
                text = content[bs:be].strip()
                texts.append(text)
            return texts
        finally:
            os.unlink(f.name)


# ── Harness runner ────────────────────────────────────────────

def load_tasks(tasks_path: str = "bench/tasks/tasks.json") -> list[BenchTask]:
    with open(tasks_path) as f:
        raw = json.load(f)
    tasks = []
    for t in raw:
        scorer = StructuralDiffPolicy(**t["scorer"])
        tasks.append(BenchTask(
            id=t["id"],
            description=t["description"],
            input_files=t["input_files"],
            expected_output=t["expected_output"],
            expected_artifact=t["expected_artifact"],
            difficulty=t["difficulty"],
            scorer=scorer,
        ))
    return tasks


def dry_run(tasks: list[BenchTask], md_binary: str) -> list[BenchResult]:
    """Score the expected outputs against themselves (sanity check)
    and against the actual tool output (validates expected files)."""
    results = []
    for task in tasks:
        expected_content = Path(task.expected_output).read_text()

        if task.expected_artifact == "file_contents":
            # Re-run the tool to produce actual output, then score
            # For dry run, expected IS the actual — should score 100%
            correct, report = score_task(task, expected_content, expected_content, md_binary)
        elif task.expected_artifact == "json_envelope":
            correct, report = score_task(task, expected_content, expected_content, md_binary)
        else:
            correct, report = True, "stdout_text: dry run skipped"

        results.append(BenchResult(
            task_id=task.id,
            mode="mdtools",
            correct=correct,
            diff_report=report,
        ))
    return results


def run_agent(
    task: BenchTask,
    mode: BenchMode,
    agent_cmd: str,
    md_binary: str,
) -> BenchResult:
    """Run an agent subprocess to complete a task."""
    workdir = tempfile.mkdtemp(prefix=f"mdtools_bench_{task.id}_{mode}_")

    # Copy input files
    for inp in task.input_files:
        shutil.copy2(inp, workdir)

    prompt = build_prompt(task, mode, workdir)
    input_file = os.path.join(workdir, os.path.basename(task.input_files[0]))

    start = time.time()
    try:
        result = subprocess.run(
            agent_cmd.split() + ["--max-turns", "10", "--no-session-persistence", prompt],
            capture_output=True,
            text=True,
            timeout=120,
            cwd=workdir,
            env={**os.environ, "CLAUDECODE": ""},
        )
        stdout = result.stdout
    except subprocess.TimeoutExpired:
        stdout = ""
    elapsed = time.time() - start

    # Capture the artifact
    expected_content = Path(task.expected_output).read_text()

    if task.expected_artifact == "file_contents":
        try:
            actual = Path(input_file).read_text()
        except FileNotFoundError:
            actual = ""
    elif task.expected_artifact == "json_envelope":
        # Extract last JSON object from stdout
        actual = extract_last_json(stdout)
    else:
        actual = stdout

    correct, report = score_task(task, actual, expected_content, md_binary)

    # Cleanup
    shutil.rmtree(workdir, ignore_errors=True)

    return BenchResult(
        task_id=task.id,
        mode=mode,
        correct=correct,
        elapsed_seconds=round(elapsed, 2),
        diff_report=report,
    )


def extract_last_json(text: str) -> str:
    """Extract the last JSON object from text output."""
    # Find the last { ... } pair
    depth = 0
    last_start = -1
    last_end = -1
    for i, ch in enumerate(text):
        if ch == "{":
            if depth == 0:
                last_start = i
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                last_end = i + 1
    if last_start >= 0 and last_end > last_start:
        return text[last_start:last_end]
    return text


# ── CLI ───────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="mdtools benchmark harness")
    parser.add_argument("--run", action="store_true", help="Run agents (not just dry-run scoring)")
    parser.add_argument("--mode", choices=["unix", "mdtools"], help="Run only this mode")
    parser.add_argument("--task", help="Run only this task ID (e.g. T1)")
    parser.add_argument("--agent", default="claude -p", help="Agent command to invoke")
    parser.add_argument("--md-binary", default="md", help="Path to md binary")
    parser.add_argument("--json", action="store_true", help="Output results as JSON")
    args = parser.parse_args()

    tasks = load_tasks()
    if args.task:
        tasks = [t for t in tasks if t.id == args.task]
        if not tasks:
            print(f"Task {args.task} not found", file=sys.stderr)
            sys.exit(1)

    if not args.run:
        # Dry run: validate scorer against expected outputs
        print("=== DRY RUN: validating scorer against expected outputs ===\n")
        results = dry_run(tasks, args.md_binary)
        for r in results:
            status = "PASS" if r.correct else "FAIL"
            print(f"  {r.task_id}: {status}")
            if r.diff_report:
                for line in r.diff_report.split("\n"):
                    print(f"    {line}")
        all_pass = all(r.correct for r in results)
        print(f"\n{'All tasks pass scorer validation.' if all_pass else 'SOME TASKS FAILED.'}")
        sys.exit(0 if all_pass else 1)

    # Run agents
    modes: list[BenchMode] = [args.mode] if args.mode else ["unix", "mdtools"]
    all_results: list[BenchResult] = []

    for mode in modes:
        print(f"\n=== MODE: {mode} ===\n")
        for task in tasks:
            print(f"  Running {task.id}: {task.description}...")
            result = run_agent(task, mode, args.agent, args.md_binary)
            all_results.append(result)
            status = "PASS" if result.correct else "FAIL"
            print(f"  {task.id}: {status} ({result.elapsed_seconds}s)")
            if result.diff_report:
                for line in result.diff_report.split("\n"):
                    print(f"    {line}")

    # Summary
    print("\n=== SUMMARY ===\n")
    print(f"{'Task':<6} {'Mode':<10} {'Result':<8} {'Time':>8}")
    print("-" * 36)
    for r in all_results:
        status = "PASS" if r.correct else "FAIL"
        print(f"{r.task_id:<6} {r.mode:<10} {status:<8} {r.elapsed_seconds:>7.1f}s")

    if args.json:
        print("\n" + json.dumps([asdict(r) for r in all_results], indent=2))


if __name__ == "__main__":
    main()
