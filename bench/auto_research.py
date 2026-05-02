#!/usr/bin/env python3
"""Auto-research orchestrator for the frontier loop.

Runs the full candidate pipeline in one command:
  1. Generator  — mdtools-blind LLM call → candidate JSON
  2. Realism    — separate LLM judge → yes/no verdict
  3. Measure    — harness.py --run in all 3 modes (N=1 seed)
  4. Adversary  — LLM proposes best unix strategy → gap_label
  5. Manifest   — assembles manifest.json + all artifacts
  6. Promote    — optionally runs N=3 promotion gate

Usage:
    python bench/auto_research.py \
        --md-binary target/release/md \
        --api-base http://localhost:10240/v1 \
        --slug my-new-family
"""
from __future__ import annotations

import argparse
import datetime
import json
import subprocess
import sys
import textwrap
import urllib.request
import urllib.error
from pathlib import Path
from typing import Any

BENCH_DIR = Path(__file__).parent
CANDIDATES_DIR = BENCH_DIR / "search" / "candidates"
HARNESS = BENCH_DIR / "harness.py"

DEFAULT_API_BASE = "http://localhost:10240/v1"
DEFAULT_API_KEY = "local"
DEFAULT_MODEL = None  # resolved from /models endpoint
DEFAULT_OAI_TIMEOUT = 180

# ── prompt templates ──────────────────────────────────────────────────────────

GENERATOR_SYSTEM = textwrap.dedent("""\
    You generate realistic Markdown document-maintenance tasks for an AI coding
    agent benchmark. Do NOT assume any special markdown CLI tool exists — agents
    only have standard unix tools (sed, awk, grep, head, tail, cat, mv, cp, wc).

    Return exactly one JSON object with these keys:
      slug            string — kebab-case identifier, max 60 chars
      description     string — precise task description (2-4 sentences)
      input_markdown  string — a realistic but concise Markdown document (200-600 words)
      expected_markdown string — input_markdown after the task is correctly applied
      scorer_policy   string — one of: "normalized_text: heading_tree + block_order + block_text"
                                       "normalized_text: heading_tree + block_order"
                                       "normalized_text: block_order + block_text"
      realism_rationale string — one sentence explaining why a real human would need this task

    The task must require STRUCTURAL awareness of the Markdown document. Good tasks:
    - Relocating a heading + its body between sections
    - Reordering sections within a parent heading
    - Moving a subsection to a different heading level context

    The input_markdown must include at least one decoy element (a heading that looks
    similar to the target, or a mention in a code fence / blockquote) to test precision.
    Do not wrap the JSON in markdown fences.
""")

GENERATOR_USER = textwrap.dedent("""\
    Generate a new structural Markdown maintenance task. Focus on section relocation
    or structural reordering in a realistic technical document (README, runbook,
    changelog, design doc, or task list). The document should be from a plausible
    software project context.
""")

REALISM_SYSTEM = textwrap.dedent("""\
    You are a realism judge for an AI agent benchmark. Your job is to decide whether
    a proposed task is realistic — would a real human actually need to perform this
    edit on a real document?

    Return exactly one JSON object with these keys:
      verdict                     "yes" or "no"
      confidence                  float 0.0-1.0
      rationale                   string — one sentence
      realistic_user              string — what kind of person would do this task
      concerns                    string — any realism concerns, or "" if none
      review_preceded_gap_measurement  boolean — always true (this is called before measurement)

    Do not wrap the JSON in markdown fences.
""")

UNIX_ADVERSARY_SYSTEM = textwrap.dedent("""\
    You are a unix-strategy adversary for an AI agent benchmark. Given a task that
    an AI agent failed to solve using only unix tools (sed, awk, grep, etc.), you
    must:
    1. Propose the best possible unix strategy for solving it.
    2. Classify why the unix approach is hard.

    Gap labels (pick exactly one):
      AST-structural     — success requires tracking heading hierarchy and section
                           boundaries; sed/awk line-based heuristics can't reliably
                           locate the correct section end
      shell-quoting      — the unix approach would work but shell quoting/escaping
                           makes it unreliable in an agent context
      planning           — the agent failed to plan the steps correctly, not a
                           fundamental unix limitation
      prompting          — the agent could have used unix tools but the prompt/policy
                           confused it
      scorer-artifact    — the unix agent may have succeeded but the scorer rejected
                           it incorrectly
      current-mdtools-command-shape-match — the task maps directly to an existing
                           mdtools command; unix is harder only because of the
                           available tools, not structural complexity

    Return exactly one JSON object with these keys:
      best_unix_strategy    string — the specific sed/awk plan that would solve this
      gap_label             string — one of the labels above
      rationale             string — 2-3 sentences explaining why
      accepted_as_ast_structural  boolean — true only if gap_label == "AST-structural"

    Do not wrap the JSON in markdown fences.
""")


# ── OAI helpers ───────────────────────────────────────────────────────────────

def _normalize_base(api_base: str) -> str:
    base = api_base.rstrip("/")
    if not base.endswith("/v1"):
        base = f"{base}/v1"
    return base


def _resolve_model(api_base: str, api_key: str, model: str | None) -> str:
    if model:
        return model
    base = _normalize_base(api_base)
    req = urllib.request.Request(
        f"{base}/models",
        headers={"Authorization": f"Bearer {api_key}"},
        method="GET",
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        data = json.load(resp)
    models = data.get("data", [])
    if not models:
        raise RuntimeError("No models available at OAI endpoint")
    return models[0]["id"]


def _call_oai(
    api_base: str,
    api_key: str,
    model: str,
    system: str,
    user: str,
    timeout: int = DEFAULT_OAI_TIMEOUT,
) -> str:
    base = _normalize_base(api_base)
    payload = json.dumps({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.7,
        "max_tokens": 2048,
    }).encode()
    req = urllib.request.Request(
        f"{base}/chat/completions",
        data=payload,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.load(resp)
    except urllib.error.HTTPError as exc:
        body = exc.read().decode(errors="replace")
        raise RuntimeError(f"OAI HTTP {exc.code}: {body[:200]}") from exc
    return data["choices"][0]["message"]["content"]


def _extract_json(text: str) -> Any:
    """Extract the first JSON object or array from a string.

    Handles two common LLM failure modes:
    - Markdown fences wrapping the JSON (skips to first { or [)
    - Braces inside string values confusing simple depth-counting

    Uses a proper string-aware scanner so awk/sed snippets with { } inside
    JSON string values don't corrupt the depth counter.
    """
    # Find first structural character
    start = -1
    for i, ch in enumerate(text):
        if ch in "{[":
            start = i
            break
    if start == -1:
        raise ValueError(f"No JSON found in response:\n{text[:300]}")

    i = start
    depth = 0
    n = len(text)
    while i < n:
        ch = text[i]
        if ch == '"':
            # Skip over the entire JSON string value
            i += 1
            while i < n:
                c = text[i]
                if c == '\\':
                    i += 2  # skip escaped character
                    continue
                if c == '"':
                    i += 1
                    break
                i += 1
            continue
        if ch in "{[":
            depth += 1
        elif ch in "}]":
            depth -= 1
            if depth == 0:
                chunk = text[start : i + 1]
                try:
                    return json.loads(chunk)
                except json.JSONDecodeError:
                    # Try replacing bare control characters outside of string
                    # parsing (e.g. raw \n in a value gemma forgot to escape)
                    import re as _re
                    cleaned = _re.sub(
                        r'(?<!\\)([\x00-\x1f])',
                        lambda m: repr(m.group(0))[1:-1],
                        chunk,
                    )
                    return json.loads(cleaned)
        i += 1
    raise ValueError(f"Unterminated JSON in response:\n{text[:300]}")


# ── pipeline steps ────────────────────────────────────────────────────────────

def step_generate(api_base: str, api_key: str, model: str, timeout: int) -> dict[str, Any]:
    print("[1/6] Generating candidate...", flush=True)
    raw = _call_oai(api_base, api_key, model, GENERATOR_SYSTEM, GENERATOR_USER, timeout)
    result = _extract_json(raw)
    required = {"slug", "description", "input_markdown", "expected_markdown",
                "scorer_policy", "realism_rationale"}
    missing = required - set(result.keys())
    if missing:
        raise ValueError(f"Generator output missing keys: {missing}")
    print(f"    slug: {result['slug']}", flush=True)
    return result


def step_realism(
    api_base: str,
    api_key: str,
    model: str,
    generated: dict[str, Any],
    timeout: int,
) -> dict[str, Any]:
    print("[2/6] Realism review...", flush=True)
    user = textwrap.dedent(f"""\
        Task description: {generated['description']}

        Input document (excerpt):
        {generated['input_markdown'][:800]}

        Expected output (excerpt):
        {generated['expected_markdown'][:800]}

        Realism rationale provided by generator: {generated['realism_rationale']}

        Is this a task a real human would need to perform?
    """)
    raw = _call_oai(api_base, api_key, model, REALISM_SYSTEM, user, timeout)
    result = _extract_json(raw)
    result.setdefault("review_preceded_gap_measurement", True)
    verdict = result.get("verdict", "no")
    confidence = result.get("confidence", 0.0)
    print(f"    verdict: {verdict} (confidence {confidence})", flush=True)
    return result


def step_measure(
    slug: str,
    candidate_dir: Path,
    md_binary: str,
    api_base: str,
    api_key: str | None,
    model: str,
    oai_timeout: int,
    runs_per_task: int,
) -> dict[str, Any]:
    print("[3/6] Running harness measurement (3 modes)...", flush=True)
    task_ids_path = candidate_dir / "task_ids.json"
    task_ids = json.loads(task_ids_path.read_text())

    today = datetime.date.today().isoformat()
    bundles: dict[str, str] = {}
    outcomes: dict[str, Any] = {}

    for mode in ("mdtools", "hybrid", "unix"):
        bundle_name = f"auto-research-{slug}-{mode}-{model}-{today}"
        bundle_dir = BENCH_DIR / "runs" / bundle_name
        print(f"    mode={mode} → {bundle_name}", flush=True)

        cmd = [
            sys.executable, str(HARNESS),
            "--run",
            "--runner", "oai-loop",
            "--mode", mode,
            "--md-binary", md_binary,
            "--oai-api-base", api_base,
            "--oai-api-key", api_key or "local",
            "--model", model,
            "--oai-request-timeout", str(oai_timeout),
            "--tasks-path", str(candidate_dir / "tasks.json"),
            "--task-ids-path", str(task_ids_path),
            "--results-dir", str(bundle_dir),
        ]

        proc = subprocess.run(cmd, capture_output=True, text=True)
        if proc.returncode != 0:
            print(f"    WARN: harness exited {proc.returncode}", flush=True)
            print(proc.stderr[-500:], flush=True)

        bundles[mode] = f"bench/runs/{bundle_name}/"
        # parse results.json if present (list of BenchResult dicts)
        results_file = bundle_dir / "results.json"
        if results_file.exists():
            results_list = json.loads(results_file.read_text())
            task_result = next(
                (r for r in results_list if r.get("task_id") == task_ids[0]),
                {}
            )
            outcomes[mode] = {
                "pass": task_result.get("correct", False),
                "neutral_pass": task_result.get("correct_neutral", False),
                "elapsed_seconds": task_result.get("elapsed_seconds", 0),
                "tool_calls": task_result.get("tool_calls", 0),
                "turns": task_result.get("turns", 0),
                "mutations": task_result.get("mutations", 0),
                "invalid_responses": task_result.get("invalid_responses", 0),
            }
        else:
            outcomes[mode] = {"pass": False, "neutral_pass": False, "error": "no results.json"}

    hybrid_pass = outcomes.get("hybrid", {}).get("pass", False)
    unix_pass = outcomes.get("unix", {}).get("pass", False)
    gap_pp = (100.0 if hybrid_pass else 0.0) - (100.0 if unix_pass else 0.0)

    return {
        "measured_at": today,
        "model": model,
        "runner": "oai-loop",
        "executor": "guarded",
        "runs_per_task": runs_per_task,
        "holdout_version": 1,
        "request_timeout_seconds": oai_timeout,
        "task_ids": task_ids,
        "bundles": bundles,
        "results": outcomes,
        "gap": {
            "hybrid_minus_unix_pp": gap_pp,
            "mdtools_minus_unix_pp": (100.0 if outcomes.get("mdtools", {}).get("pass") else 0.0)
                                     - (100.0 if unix_pass else 0.0),
        },
    }


def step_unix_adversary(
    api_base: str,
    api_key: str,
    model: str,
    generated: dict[str, Any],
    measurement: dict[str, Any],
    timeout: int,
) -> dict[str, Any]:
    print("[4/6] Unix-adversary review...", flush=True)
    unix_outcome = measurement["results"].get("unix", {})
    user = textwrap.dedent(f"""\
        Task: {generated['description']}

        Input document:
        {generated['input_markdown'][:800]}

        Expected output:
        {generated['expected_markdown'][:600]}

        Unix run outcome: pass={unix_outcome.get('pass', False)},
        turns={unix_outcome.get('turns', 0)}, mutations={unix_outcome.get('mutations', 0)},
        invalid_responses={unix_outcome.get('invalid_responses', 0)}

        Hybrid pass: {measurement['results'].get('hybrid', {}).get('pass', False)}

        Classify why unix failed and propose the best unix strategy.
    """)
    raw = _call_oai(api_base, api_key, model, UNIX_ADVERSARY_SYSTEM, user, timeout)
    result = _extract_json(raw)
    result.setdefault("accepted_as_ast_structural",
                      result.get("gap_label") == "AST-structural")
    print(f"    gap_label: {result.get('gap_label')}", flush=True)
    return result


def step_assemble_manifest(
    slug: str,
    candidate_dir: Path,
    generated: dict[str, Any],
    realism: dict[str, Any],
    measurement: dict[str, Any],
    adversary: dict[str, Any],
    model: str,
    today: str,
) -> dict[str, Any]:
    print("[5/6] Assembling manifest...", flush=True)
    hybrid_pass = measurement["results"].get("hybrid", {}).get("pass", False)
    unix_pass = measurement["results"].get("unix", {}).get("pass", False)
    gap_pp = measurement["gap"]["hybrid_minus_unix_pp"]
    ast_structural = adversary.get("accepted_as_ast_structural", False)
    realism_ok = realism.get("verdict") == "yes"

    if not realism_ok:
        status = "rejected-planning"  # closest bucket for realism fail
    elif not hybrid_pass:
        status = "rejected-hybrid-fail-no-gap"
    elif unix_pass:
        status = "rejected-both-pass-no-gap"
    elif not ast_structural:
        status = f"rejected-{adversary.get('gap_label', 'unknown').replace('/', '-')}"
    elif gap_pp <= 0:
        status = "rejected-both-fail-no-gap"
    else:
        status = "pending-cross-seed"  # ready for promotion gate

    task_ids = measurement["task_ids"]

    manifest: dict[str, Any] = {
        "family_slug": slug,
        "family_name": generated.get("description", slug)[:80],
        "status": status,
        "created_at": today,
        "source": {
            "generator_model": model,
            "generator_prompt": "generator-prompt.md",
            "generator_output": "generator-output.json",
        },
        "realism_review": {
            "reviewer_model": model,
            "artifact": "realism-review.json",
            "verdict": realism.get("verdict"),
            "confidence": realism.get("confidence"),
            "review_preceded_gap_measurement": True,
        },
        "task_definition": {
            "tasks_path": "tasks.json",
            "task_ids_path": "task_ids.json",
            "task_ids": task_ids,
            "input_docs": ["input.md"],
            "expected_outputs": ["expected.md"],
            "scorer_policies": [generated.get("scorer_policy", "normalized_text: heading_tree + block_order + block_text")],
        },
        "measurements": {
            "artifact": "measurement-summary.json",
            "model": model,
            "runner": "oai-loop",
            "executor": "guarded",
            "runs_per_task": 1,
            "holdout_version": 1,
            "request_timeout_seconds": measurement["request_timeout_seconds"],
            "bundles": measurement["bundles"],
            "outcome": {
                "mdtools": "pass" if measurement["results"].get("mdtools", {}).get("pass") else "fail",
                "hybrid": "pass" if hybrid_pass else "fail",
                "unix": "pass" if unix_pass else "fail",
                "hybrid_minus_unix_pp": gap_pp,
                "mdtools_minus_unix_pp": measurement["gap"]["mdtools_minus_unix_pp"],
            },
        },
        "unix_adversary_review": {
            "artifact": "unix-adversary-review.json",
            "reviewer_model": model,
            "gap_label": adversary.get("gap_label"),
            "accepted_as_ast_structural": ast_structural,
        },
        "promotion_notes": (
            f"Auto-generated. Status: {status}. Gap: {gap_pp:+.1f}pp. "
            f"Realism: {realism.get('verdict')}. Unix adversary: {adversary.get('gap_label')}."
        ),
    }
    return manifest


def _write_task_json(candidate_dir: Path, slug: str, generated: dict[str, Any]) -> list[str]:
    """Write tasks.json and task_ids.json; return task_ids list."""
    # Derive a task ID from the existing candidate count
    existing = [d for d in CANDIDATES_DIR.iterdir() if d.is_dir()]
    task_num = len(existing) + 10  # offset to avoid collision with corpus tasks
    task_id = f"C-AR-{task_num:03d}"

    scorer_policy = generated.get("scorer_policy",
                                  "normalized_text: heading_tree + block_order + block_text")
    # parse the scorer_policy string into scorer fields
    compare_heading_tree = "heading_tree" in scorer_policy
    compare_block_order = "block_order" in scorer_policy
    compare_block_text = "block_text" in scorer_policy

    task = {
        "id": task_id,
        "description": generated["description"],
        "input_files": [f"bench/search/candidates/{slug}/input.md"],
        "expected_output": f"bench/search/candidates/{slug}/expected.md",
        "expected_artifact": "file_contents",
        "difficulty": "intermediate",
        "scorer": {
            "kind": "normalized_text",
            "normalize_line_endings": True,
            "ignore_trailing_whitespace": True,
            "compare_frontmatter_json": False,
            "compare_heading_tree": compare_heading_tree,
            "compare_block_order": compare_block_order,
            "compare_link_destinations": False,
            "compare_block_text": compare_block_text,
        },
    }
    (candidate_dir / "tasks.json").write_text(json.dumps([task], indent=2))
    (candidate_dir / "task_ids.json").write_text(json.dumps([task_id], indent=2))
    return [task_id]


# ── main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--md-binary", required=True, help="Path to md binary")
    parser.add_argument("--api-base", default=DEFAULT_API_BASE)
    parser.add_argument("--api-key", default=DEFAULT_API_KEY)
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--oai-timeout", type=int, default=DEFAULT_OAI_TIMEOUT)
    parser.add_argument("--slug", default=None,
                        help="Override slug (default: use generator output)")
    parser.add_argument("--skip-measure", action="store_true",
                        help="Skip harness measurement (dry-run through pipeline)")
    parser.add_argument("--runs-per-task", type=int, default=1,
                        help="Number of harness runs per task per mode")
    args = parser.parse_args()

    api_base = args.api_base
    api_key = args.api_key
    oai_timeout = args.oai_timeout
    today = datetime.date.today().isoformat()

    model = _resolve_model(api_base, api_key, args.model)
    print(f"Using model: {model}", flush=True)

    # Step 1: Generate
    generated = step_generate(api_base, api_key, model, oai_timeout)
    slug = args.slug or generated["slug"]

    candidate_dir = CANDIDATES_DIR / slug
    if candidate_dir.exists():
        print(f"ERROR: candidate dir already exists: {candidate_dir}", file=sys.stderr)
        sys.exit(1)
    candidate_dir.mkdir(parents=True)

    # Write generator artifacts
    (candidate_dir / "generator-prompt.md").write_text(
        "Auto-research pipeline generator prompt (see auto_research.py GENERATOR_SYSTEM + GENERATOR_USER)"
    )
    (candidate_dir / "generator-output.json").write_text(json.dumps(generated, indent=2))
    (candidate_dir / "input.md").write_text(generated["input_markdown"])
    (candidate_dir / "expected.md").write_text(generated["expected_markdown"])
    _write_task_json(candidate_dir, slug, generated)

    # Step 2: Realism
    realism = step_realism(api_base, api_key, model, generated, oai_timeout)
    (candidate_dir / "realism-review.json").write_text(json.dumps(realism, indent=2))

    if realism.get("verdict") != "yes":
        print(f"REJECT: realism verdict={realism.get('verdict')}", flush=True)
        manifest = step_assemble_manifest(
            slug, candidate_dir, generated, realism, {}, {}, model, today
        )
        (candidate_dir / "manifest.json").write_text(json.dumps(manifest, indent=2))
        print(f"Done — {candidate_dir}/manifest.json (status: {manifest['status']})")
        return

    # Step 3: Measure
    if args.skip_measure:
        print("[3/6] Skipping measurement (--skip-measure)", flush=True)
        measurement: dict[str, Any] = {
            "measured_at": today, "model": model, "runner": "oai-loop",
            "executor": "guarded", "runs_per_task": 0, "holdout_version": 1,
            "request_timeout_seconds": oai_timeout,
            "task_ids": json.loads((candidate_dir / "task_ids.json").read_text()),
            "bundles": {}, "results": {}, "gap": {"hybrid_minus_unix_pp": 0, "mdtools_minus_unix_pp": 0},
        }
    else:
        measurement = step_measure(
            slug, candidate_dir, args.md_binary, api_base, api_key, model,
            oai_timeout, args.runs_per_task,
        )
    (candidate_dir / "measurement-summary.json").write_text(json.dumps(measurement, indent=2))

    # Step 4: Unix-adversary
    adversary = step_unix_adversary(api_base, api_key, model, generated, measurement, oai_timeout)
    (candidate_dir / "unix-adversary-review.json").write_text(json.dumps(adversary, indent=2))

    # Step 5: Manifest
    manifest = step_assemble_manifest(
        slug, candidate_dir, generated, realism, measurement, adversary, model, today
    )
    (candidate_dir / "manifest.json").write_text(json.dumps(manifest, indent=2))

    print(f"\n[6/6] Done.", flush=True)
    print(f"  candidate: {candidate_dir}", flush=True)
    print(f"  status:    {manifest['status']}", flush=True)
    gap = measurement["gap"]["hybrid_minus_unix_pp"]
    print(f"  gap:       {gap:+.1f}pp (hybrid − unix)", flush=True)
    print(f"  manifest:  {candidate_dir}/manifest.json", flush=True)

    if manifest["status"] == "pending-cross-seed":
        print("\nNext step: run cross-seed promotion gate (N=3):", flush=True)
        print(f"  python bench/harness.py --run --mode hybrid --runs-per-task 3 \\")
        print(f"    --md-binary {args.md_binary} --tasks {candidate_dir}/tasks.json \\")
        print(f"    --task-ids {candidate_dir}/task_ids.json")


if __name__ == "__main__":
    main()
