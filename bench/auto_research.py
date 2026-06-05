#!/usr/bin/env python3
"""Auto-research orchestrator for the frontier loop.

Runs the full candidate pipeline in one command:
  1. Generator       — mdtools-blind LLM call → candidate JSON
  2. Realism         — separate LLM judge → yes/no verdict
  3. Measure         — harness.py --run in all 3 modes (N=1 seed)
  4. Unix-adversary  — LLM proposes best unix strategy → gap_label
  5. Native-adversary — PAID claude-cli `native`-mode run (native Edit, no md);
                        if native Edit solves it → rejected-no-md-edge (ramp stage 1,
                        the load-bearing md-edge gate; opt-in via --native-adversary)
  6. Manifest        — assembles manifest.json + all artifacts
  7. Promote         — optionally runs N=3 promotion gate

The native-adversary is the deployment-true md-edge gate: md's claimed edge is
structural entropy that native Read/Edit/Write should struggle with. A candidate
is only a real md-edge candidate once native Edit has been shown to STRUGGLE on it
(fails it). Tasks native Edit solves correctly carry no md correctness lift →
rejected-no-md-edge. Without the gate, an otherwise-promotable candidate lands in
`pending-native-adversary` (promotion-pending the native gate), never the
trustworthy `pending-cross-seed`.

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
GENERATOR_FALLBACK_MODELS = ("Hermes-4-70B-4bit", "magnum-v4-123b-4bit")

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


def _call_json(
    api_base: str,
    api_key: str,
    model: str,
    system: str,
    user: str,
    timeout: int,
    fallback_models: tuple[str, ...] = (),
    step_name: str = "step",
) -> tuple[Any, str]:
    models = [model, *fallback_models]
    last_error: Exception | None = None
    for idx, candidate in enumerate(models):
        try:
            raw = _call_oai(
                api_base=api_base,
                api_key=api_key,
                model=candidate,
                system=system,
                user=user,
                timeout=timeout,
            )
            result = _extract_json(raw)
            if idx > 0:
                print(f"    {step_name}: fell back to model {candidate}", flush=True)
            return result, candidate
        except Exception as exc:  # broad on purpose: parser + transport
            last_error = exc
            continue
    raise ValueError(f"{step_name}: failed to obtain JSON (models {models}); last_error={last_error}")


def _extract_json(text: str) -> Any:
    """Extract the first JSON object or array from a string.

    Handles two common LLM failure modes:
    - Markdown fences wrapping the JSON (skips to first { or [)
    - Braces inside string values confusing simple depth-counting

    Uses a proper string-aware scanner so awk/sed snippets with { } inside
    JSON string values don't corrupt the depth counter.
    """
    def _strip_markdown_fence(raw: str) -> str:
        import re as _re
        for match in _re.finditer(r"```json\s*(.*?)```", raw, flags=_re.IGNORECASE | _re.S):
            candidate = match.group(1).strip()
            if candidate:
                return candidate
        return raw

    def _json_from(i_start: int, raw: str) -> Any:
        i = i_start
        depth = 0
        n = len(raw)
        while i < n:
            ch = raw[i]
            if ch == '"':
                # Skip over the entire JSON string value
                i += 1
                while i < n:
                    c = raw[i]
                    if c == "\\":
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
                    chunk = raw[i_start : i + 1]
                    try:
                        return json.loads(chunk)
                    except json.JSONDecodeError:
                        import re as _re
                        cleaned = _re.sub(
                            r"(?<!\\)([\x00-\x1f])",
                            lambda m: repr(m.group(0))[1:-1],
                            chunk,
                        )
                        return json.loads(cleaned)
            i += 1
        raise ValueError(f"Unterminated JSON in response: {raw[i_start:i_start+300]}")

    text = _strip_markdown_fence(text)
    for i, ch in enumerate(text):
        if ch not in "{[":
            continue
        try:
            return _json_from(i, text)
        except ValueError:
            # Keep scanning for another potential JSON object later in the response
            continue
        except json.JSONDecodeError:
            # Keep scanning for recoverable JSON later in the response
            continue

    raise ValueError(f"No JSON found in response:\n{text[:300]}")


# ── pipeline steps ────────────────────────────────────────────────────────────

def step_generate(api_base: str, api_key: str, model: str, timeout: int) -> dict[str, Any]:
    print("[1/6] Generating candidate...", flush=True)
    result, used_model = _call_json(
        api_base,
        api_key,
        model,
        GENERATOR_SYSTEM,
        GENERATOR_USER,
        timeout,
        fallback_models=GENERATOR_FALLBACK_MODELS,
        step_name="generator",
    )
    required = {"slug", "description", "input_markdown", "expected_markdown",
                "scorer_policy", "realism_rationale"}
    missing = required - set(result.keys())
    if missing:
        raise ValueError(f"Generator output missing keys: {missing}")
    print(f"    model: {used_model}", flush=True)
    print(f"    slug: {result['slug']}", flush=True)
    result["_generator_model"] = used_model
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
    result, used_model = _call_json(
        api_base,
        api_key,
        model,
        REALISM_SYSTEM,
        user,
        timeout,
        fallback_models=GENERATOR_FALLBACK_MODELS,
        step_name="realism",
    )
    result.setdefault("review_preceded_gap_measurement", True)
    result["reviewer_model"] = used_model
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
            "-N", str(runs_per_task),
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
            task_runs = [r for r in results_list if r.get("task_id") == task_ids[0]]
            outcomes[mode] = {
                "pass": any(r.get("correct", False) for r in task_runs),
                "neutral_pass": any(r.get("correct_neutral", False) for r in task_runs),
                "elapsed_seconds": max((r.get("elapsed_seconds", 0) for r in task_runs), default=0),
                "tool_calls": max((r.get("tool_calls", 0) for r in task_runs), default=0),
                "turns": max((r.get("turns", 0) for r in task_runs), default=0),
                "mutations": max((r.get("mutations", 0) for r in task_runs), default=0),
                "invalid_responses": max((r.get("invalid_responses", 0) for r in task_runs), default=0),
            } if task_runs else {"pass": False, "neutral_pass": False, "error": "no results for task"}
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
    result, used_model = _call_json(
        api_base,
        api_key,
        model,
        UNIX_ADVERSARY_SYSTEM,
        user,
        timeout,
        fallback_models=GENERATOR_FALLBACK_MODELS,
        step_name="unix-adversary",
    )
    result["reviewer_model"] = used_model
    result.setdefault("accepted_as_ast_structural",
                      result.get("gap_label") == "AST-structural")
    print(f"    gap_label: {result.get('gap_label')}", flush=True)
    return result


def _native_adversary_verdict(
    results_list: list[dict[str, Any]],
    task_id: str,
) -> dict[str, Any]:
    """Compute the native-adversary verdict from a `native`-mode results list.

    The load-bearing md-edge gate (ramp stage 1): run `native` mode (native
    Read/Edit/Write, NO md) on the candidate. If native Edit *correctly* solves
    it, md has no correctness lift to offer → recommend rejected-no-md-edge.
    Only tasks native Edit STRUGGLES on (fails them) are real md-edge candidates.

    Correctness-primary, threshold-free: the gate keys on whether native Edit
    *passes* (majority of N), not on a magic cost cutoff. Cost medians are
    recorded so the stage-2 native-vs-native+md comparison can still surface a
    cost-edge family (native passes but thrashes) — but stage 1 never auto-promotes
    on cost (that would need the native+md comparison it does not run here).
    """
    task_runs = [r for r in results_list if r.get("task_id") == task_id]
    if not task_runs:
        return {
            "verdict": "no-native-results",
            "native_pass": False,
            "native_pass_rate": 0.0,
            "runs": 0,
            "recommend_reject_no_md_edge": False,
            "note": "native harness produced no results for the task (gate inconclusive)",
        }

    def _med(values: list[int]) -> int:
        s = sorted(values)
        return s[len(s) // 2] if s else 0

    n = len(task_runs)
    passes = sum(1 for r in task_runs if r.get("correct"))
    pass_rate = passes / n
    native_pass = pass_rate >= 0.5  # majority of N
    median_tokens = _med([
        int(r.get("tokens_in", 0) or 0) + int(r.get("tokens_out", 0) or 0)
        for r in task_runs
    ])
    return {
        "verdict": "native-solves" if native_pass else "native-struggles-correctness",
        "native_pass": native_pass,
        "native_pass_rate": pass_rate,
        "runs": n,
        "median_tokens": median_tokens,
        "median_turns": _med([int(r.get("turns", 0) or 0) for r in task_runs]),
        "median_tool_calls": _med([int(r.get("tool_calls", 0) or 0) for r in task_runs]),
        "median_mutations": _med([int(r.get("mutations", 0) or 0) for r in task_runs]),
        # correctness-primary gate: native solving correctly ⇒ no md correctness lift
        "recommend_reject_no_md_edge": native_pass,
        "note": (
            "native Edit solves this correctly → no md correctness lift "
            "(a cost-edge is still possible; confirm in stage-2 native-vs-native+md)"
            if native_pass else
            "native Edit FAILS this → real md-edge candidate (potential correctness lift)"
        ),
    }


def step_native_adversary(
    slug: str,
    candidate_dir: Path,
    md_binary: str,
    runs: int,
    today: str,
) -> dict[str, Any]:
    """Ramp stage 1 — the native-adversary (PAID claude-cli `native`-mode run).

    Runs `native` mode (native Read/Edit/Write, no md = the deployment-true
    baseline) on the candidate and emits the md-edge gate verdict. This spends
    real money (claude-cli); the *caller* (the loop orchestrator) owns the
    budget / SPEND-ledger bookkeeping — this function just runs the cell.
    """
    print("[5/7] Native-adversary (native Edit, no md — PAID) ...", flush=True)
    task_ids = json.loads((candidate_dir / "task_ids.json").read_text())
    task_id = task_ids[0]
    bundle_name = f"auto-research-{slug}-native-adversary-{today}"
    bundle_dir = BENCH_DIR / "runs" / bundle_name
    cmd = [
        sys.executable, str(HARNESS),
        "--run",
        "--runner", "claude-cli",
        "--mode", "native",
        "--md-binary", md_binary,
        "--tasks-path", str(candidate_dir / "tasks.json"),
        "--task-ids-path", str(candidate_dir / "task_ids.json"),
        "--results-dir", str(bundle_dir),
        "-N", str(runs),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        print(f"    WARN: native-adversary harness exited {proc.returncode}", flush=True)
        print(proc.stderr[-500:], flush=True)
    results_file = bundle_dir / "results.json"
    results_list = json.loads(results_file.read_text()) if results_file.exists() else []
    verdict = _native_adversary_verdict(results_list, task_id)
    verdict["bundle"] = f"bench/runs/{bundle_name}/"
    verdict["runner"] = "claude-cli"
    verdict["mode"] = "native"
    verdict["runs_requested"] = runs
    print(
        f"    verdict: {verdict['verdict']} "
        f"(native_pass={verdict['native_pass']}, pass_rate={verdict['native_pass_rate']:.2f})",
        flush=True,
    )
    return verdict


def _resolve_status(
    *,
    realism_ok: bool,
    hybrid_pass: bool,
    unix_pass: bool,
    ast_structural: bool,
    gap_label: str | None,
    gap_pp: float,
    native_adversary: dict[str, Any] | None,
) -> str:
    """Pure candidate-status decision.

    The unix-pipeline gates (realism / hybrid-pass / unix-fail / AST-structural /
    gap>0) come first, unchanged. The native-adversary is then the load-bearing
    md-edge gate: a candidate may only reach the trustworthy `pending-cross-seed`
    once native Edit has been shown to STRUGGLE on it. Back-compat strengthening:
    when the native-adversary did not run, an otherwise-promotable candidate lands
    in `pending-native-adversary` (promotion-pending the native gate) — strictly
    more honest than the old unconditional `pending-cross-seed`, never weaker.
    """
    if not realism_ok:
        return "rejected-planning"  # closest bucket for realism fail
    if not hybrid_pass:
        return "rejected-hybrid-fail-no-gap"
    if unix_pass:
        return "rejected-both-pass-no-gap"
    if not ast_structural:
        return f"rejected-{(gap_label or 'unknown').replace('/', '-')}"
    if gap_pp <= 0:
        return "rejected-both-fail-no-gap"
    # Passed the unix-pipeline gates — now the deployment-true native-root gate:
    na = native_adversary or {}
    verdict = na.get("verdict", "not-run")
    if verdict in (None, "not-run", "no-native-results"):
        return "pending-native-adversary"   # native gate not applied / inconclusive
    if na.get("recommend_reject_no_md_edge"):
        return "rejected-no-md-edge"         # native Edit solves it → no md edge
    return "pending-cross-seed"              # native struggles → real md-edge candidate


def step_assemble_manifest(
    slug: str,
    candidate_dir: Path,
    generated: dict[str, Any],
    realism: dict[str, Any],
    measurement: dict[str, Any],
    adversary: dict[str, Any],
    model: str,
    today: str,
    native_adversary: dict[str, Any] | None = None,
) -> dict[str, Any]:
    print("[6/7] Assembling manifest...", flush=True)
    results = measurement.get("results", {})
    gap = measurement.get("gap", {})
    hybrid_pass = results.get("hybrid", {}).get("pass", False)
    unix_pass = results.get("unix", {}).get("pass", False)
    gap_pp = gap.get("hybrid_minus_unix_pp", 0.0)
    ast_structural = adversary.get("accepted_as_ast_structural", False)
    realism_ok = realism.get("verdict") == "yes"

    status = _resolve_status(
        realism_ok=realism_ok,
        hybrid_pass=hybrid_pass,
        unix_pass=unix_pass,
        ast_structural=ast_structural,
        gap_label=adversary.get("gap_label"),
        gap_pp=gap_pp,
        native_adversary=native_adversary,
    )

    task_ids = measurement.get("task_ids", [])

    manifest: dict[str, Any] = {
        "family_slug": slug,
        "family_name": generated.get("description", slug)[:80],
        "status": status,
        "created_at": today,
        "source": {
            "generator_model": generated.get("_generator_model", model),
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
            "runs_per_task": measurement.get("runs_per_task", 1),
            "holdout_version": 1,
            "request_timeout_seconds": measurement.get("request_timeout_seconds"),
            "bundles": measurement.get("bundles", []),
            "outcome": {
                "mdtools": "pass" if results.get("mdtools", {}).get("pass") else "fail",
                "hybrid": "pass" if hybrid_pass else "fail",
                "unix": "pass" if unix_pass else "fail",
                "hybrid_minus_unix_pp": gap_pp,
                "mdtools_minus_unix_pp": gap.get("mdtools_minus_unix_pp", 0.0),
            },
        },
        "unix_adversary_review": {
            "artifact": "unix-adversary-review.json",
            "reviewer_model": model,
            "gap_label": adversary.get("gap_label"),
            "accepted_as_ast_structural": ast_structural,
        },
        "native_adversary_review": {
            "artifact": "native-adversary-review.json" if native_adversary else None,
            "ran": native_adversary is not None,
            "verdict": (native_adversary or {}).get("verdict", "not-run"),
            "native_pass": (native_adversary or {}).get("native_pass"),
            "native_pass_rate": (native_adversary or {}).get("native_pass_rate"),
            "recommend_reject_no_md_edge": (native_adversary or {}).get(
                "recommend_reject_no_md_edge", False),
            "bundle": (native_adversary or {}).get("bundle"),
        },
        "promotion_notes": (
            f"Auto-generated. Status: {status}. Gap: {gap_pp:+.1f}pp. "
            f"Realism: {realism.get('verdict')}. Unix adversary: {adversary.get('gap_label')}. "
            f"Native adversary: {(native_adversary or {}).get('verdict', 'not-run')}."
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
    parser.add_argument("--native-adversary", action="store_true",
                        help="Run the native-adversary (ramp stage 1): a PAID claude-cli "
                             "`native`-mode run that rejects candidates native Edit solves "
                             "at parity (no md edge). Off by default — it spends real money.")
    parser.add_argument("--native-adversary-runs", type=int, default=3,
                        help="N for the native-adversary native cell (default 3, the N>=3 gate)")
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
        print("[3/7] Skipping measurement (--skip-measure)", flush=True)
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

    # Step 5: Native-adversary (the load-bearing md-edge gate) — PAID, opt-in.
    native_adversary: dict[str, Any] | None = None
    if args.native_adversary and not args.skip_measure:
        native_adversary = step_native_adversary(
            slug, candidate_dir, args.md_binary, args.native_adversary_runs, today
        )
        (candidate_dir / "native-adversary-review.json").write_text(
            json.dumps(native_adversary, indent=2))
    else:
        reason = "--skip-measure" if args.skip_measure else "--native-adversary off"
        print(f"[5/7] Native-adversary: SKIPPED ({reason}) — an otherwise-promotable "
              "candidate stays pending-native-adversary (native gate not applied)", flush=True)

    # Step 6: Manifest
    manifest = step_assemble_manifest(
        slug, candidate_dir, generated, realism, measurement, adversary, model, today,
        native_adversary=native_adversary,
    )
    (candidate_dir / "manifest.json").write_text(json.dumps(manifest, indent=2))

    print(f"\n[7/7] Done.", flush=True)
    print(f"  candidate: {candidate_dir}", flush=True)
    print(f"  status:    {manifest['status']}", flush=True)
    gap_display = measurement.get("gap", {}).get("hybrid_minus_unix_pp", 0.0)
    print(f"  gap:       {gap_display:+.1f}pp (hybrid − unix)", flush=True)
    if native_adversary is not None:
        print(f"  native:    {native_adversary['verdict']} "
              f"(pass_rate={native_adversary['native_pass_rate']:.2f})", flush=True)
    print(f"  manifest:  {candidate_dir}/manifest.json", flush=True)

    if manifest["status"] == "pending-native-adversary":
        print("\nNext step: apply the native-adversary gate (PAID claude-cli native run):", flush=True)
        print(f"  python bench/auto_research.py --native-adversary --slug {slug} ...  # (re-runs the gate)")
        print(f"  or directly: python bench/harness.py --run --runner claude-cli --mode native -N 3 \\")
        print(f"    --md-binary {args.md_binary} --tasks-path {candidate_dir}/tasks.json \\")
        print(f"    --task-ids-path {candidate_dir}/task_ids.json")
    elif manifest["status"] == "pending-cross-seed":
        print("\nNative gate CLEARED (native Edit struggles). Next: cross-seed promotion gate (N=3):", flush=True)
        print(f"  python bench/harness.py --run --mode hybrid -N 3 \\")
        print(f"    --md-binary {args.md_binary} --tasks-path {candidate_dir}/tasks.json \\")
        print(f"    --task-ids-path {candidate_dir}/task_ids.json")


if __name__ == "__main__":
    main()
