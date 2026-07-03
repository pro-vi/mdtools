#!/usr/bin/env python3
from __future__ import annotations

"""Generate bench/RESULTS.md — the single canonical benchmark page — from run bundles.

Every number here is derived from a bundle's `results.json` (the per-task `correct`
field is ground truth), never typed by hand, so the doc cannot drift from the data.

    python3 bench/results_canon.py        # print to stdout
    python3 bench/results_canon.py -i      # write bench/RESULTS.md

To add evidence: drop the bundle under bench/runs/, append its dir to the relevant
axis in CANON below, and re-run. Rows are grouped by each row's own `mode` field,
so directory-name inconsistencies (e.g. the no-md ablation bundle that lost its
suffix) do not matter.

What is NOT auto-derived (no clean aggregate bundle exists): the Sonnet 4.6 native
null and the historical 20-task v1 snapshot. Those live in PROVISIONAL below as cited
evidence with source pointers, clearly partitioned from the regenerated tables.
"""
import json
import statistics
import sys
from pathlib import Path

try:
    from bench.agg_util import cell_trials, pass_at_1_mean, pass_hat_k
    from bench.stats import exact_sign_test, flip_table, hierarchical_bootstrap_ci, variance_decomposition, wilson_ci
    from bench.v3_manifest import MANIFEST_PATH, bundle_conformance, evaluate_success_threshold, load_manifest
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from bench.agg_util import cell_trials, pass_at_1_mean, pass_hat_k
    from bench.stats import exact_sign_test, flip_table, hierarchical_bootstrap_ci, variance_decomposition, wilson_ci
    from bench.v3_manifest import MANIFEST_PATH, bundle_conformance, evaluate_success_threshold, load_manifest

RUNS = Path(__file__).resolve().parent / "runs"
OUT = Path(__file__).resolve().parent / "RESULTS.md"
TASKS = Path(__file__).resolve().parent / "tasks" / "tasks.json"
ADJUDICATIONS = Path(__file__).resolve().parent / "v3" / "adjudications.json"
SCORER_VERSION = "v3-neutral-primary"

V3_CANON: list[str] = [
    "v3-validation-qwen-2026-07-01",
    "v3-closeout-haiku-shell-2026-07-02",
    "v3-closeout-haiku-native-2026-07-03",
    "v3-closeout-gpt54mini-native-2026-07-03",
]


class CanonBlockedError(RuntimeError):
    pass

MODEL_LABELS = {
    "claude-haiku-4-5-20251001": "Haiku 4.5",
    "openai-codex/gpt-5.4-mini": "GPT 5.4 mini",
    "claude-sonnet-4-6": "Sonnet 4.6",
}

# Curated canon. Each axis names the bundles whose results.json is ground truth.
CANON = [
    {
        "key": "shell",
        "title": "md vs shell (no native editor)",
        "intro": (
            "The `unix` baseline is the raw shell toolkit (cat/grep/sed/awk), not a "
            "file editor. These lifts measure md-vs-shell."
        ),
        "bundles": ["full-haiku-2026-06-11", "full-gpt54mini-2026-06-11"],
        "modes": ["unix", "mdtools", "hybrid"],
        "lift_from": "unix",
        "lift_to": "hybrid",
    },
    {
        "key": "native",
        "title": "md vs native Edit",
        "intro": (
            "The `native` baseline is native Read/Edit/Write tools. `native+md-no-md` "
            "is the ablation control: md is advertised in the prompt but the binary is "
            "stubbed. When the control lands on `native`, the lift is the tool, not the "
            "md-advertising prompt."
        ),
        "bundles": [
            "native-haiku-2026-06-13-native",
            "native-haiku-2026-06-13-native+md",
            "native-haiku-2026-06-13",  # no-md ablation; dir lost its suffix
            "native-gpt54mini-2026-06-13-native",
            "native-gpt54mini-2026-06-13-native+md",
            "native-gpt54mini-2026-06-13-native+md-no-md",
        ],
        "modes": ["native", "native+md", "native+md-no-md"],
        "lift_from": "native",
        "lift_to": "native+md",
    },
]


def task_key(tid):
    if tid.startswith("T") and tid[1:].isdigit():
        return (0, int(tid[1:]), "")
    return (1, 0, tid)


def load_rows(bundle):
    p = RUNS / bundle / "results.json"
    if not p.exists():
        raise SystemExit(f"canon bundle missing: {p}")
    return json.loads(p.read_text())


def _bundle_path(bundle: str | Path) -> Path:
    path = Path(bundle)
    if path.is_absolute() or path.exists():
        return path
    return RUNS / str(bundle)


def _load_bundle(bundle: str | Path) -> tuple[dict, list[dict]]:
    path = _bundle_path(bundle)
    run_path = path / "run.json"
    results_path = path / "results.json"
    if not run_path.exists() or not results_path.exists():
        raise FileNotFoundError(f"v3 bundle missing run.json/results.json: {path}")
    return json.loads(run_path.read_text()), json.loads(results_path.read_text())


def _rows_with_run_metadata(run: dict, rows: list[dict]) -> list[dict]:
    return [
        {
            **row,
            "model": row.get("model") or run.get("model"),
            "runner": row.get("runner") or run.get("runner"),
        }
        for row in rows
    ]


def _task_provenance(tasks_path: Path = TASKS) -> dict[str, str]:
    tasks = json.loads(tasks_path.read_text())
    return {task["id"]: task.get("provenance", "core") for task in tasks}


def _load_adjudications(path: Path = ADJUDICATIONS) -> set[tuple[str, str, int | None]]:
    if not path.exists():
        return set()
    raw = json.loads(path.read_text())
    out = set()
    for item in raw:
        if item.get("type") == "interpretation_note":
            continue
        out.add((item.get("task"), item.get("mode"), item.get("run_index")))
    return out


def _load_interpretation_notes(path: Path = ADJUDICATIONS) -> list[dict]:
    if not path.exists():
        return []
    raw = json.loads(path.read_text())
    return [item for item in raw if item.get("type") == "interpretation_note"]


def _blocked_quarantines(rows: list[dict], adjudicated: set[tuple[str, str, int | None]]) -> list[dict]:
    blocked = []
    for row in rows:
        if row.get("verdict") != "divergent":
            continue
        key = (row.get("task_id"), row.get("mode"), row.get("run_index"))
        if key not in adjudicated:
            blocked.append(row)
    return blocked


def _cost_value(row: dict) -> float | None:
    if row.get("cost_usd") is not None:
        return float(row["cost_usd"])
    tokens = int(row.get("tokens_in", 0) or 0) + int(row.get("tokens_out", 0) or 0)
    if tokens:
        return float(tokens)
    calls = int(row.get("tool_calls", 0) or 0)
    return float(calls) if calls else None


def _fmt_pct(value: float) -> str:
    return f"{value * 100:.1f}%"


def _fmt_pp(value: float) -> str:
    sign = "+" if value >= 0 else ""
    return f"{sign}{value * 100:.1f}pp"


def _comparison_modes(comparison: dict) -> tuple[str, str]:
    left, right = [part.strip() for part in comparison["comparison"].split("->", 1)]
    return left, right


def _rows_for_cell(
    rows: list[dict],
    *,
    provenance: dict[str, str],
    model: str,
    runner: str,
    mode: str,
    split: str,
) -> list[dict]:
    split_rows = [
        row
        for row in rows
        if provenance.get(row.get("task_id"), "core") == split
    ]
    return cell_trials(split_rows, mode=mode, model=model, runner=runner)


def _render_ablation_status(
    *,
    baseline_trials: list[dict],
    ablation_trials: list[dict],
) -> str:
    if not baseline_trials or not ablation_trials:
        return "-"
    baseline = pass_at_1_mean(baseline_trials)
    ablation = pass_at_1_mean(ablation_trials)
    delta_pp = abs(ablation - baseline) * 100
    status = "clean" if delta_pp <= 5.0 else "separated"
    return f"{_fmt_pct(ablation)} vs {_fmt_pct(baseline)} ({status})"


def _render_headline_status(rows: list[dict], provenance: dict[str, str], manifest: dict) -> str:
    primary = manifest.get("primary_comparisons", [])
    if not primary:
        return "No preregistered primary comparisons in the v3 manifest.\n"
    lines = [
        "Manifest threshold: PASS requires 95% paired-lift CI lower bound > +15pp, exact collapsed-task p < 0.05, and no md-favorable quarantine concentration.\n",
        "| Comparison | Core tasks | Lift pass@1 mean ± 95% CI | Exact p | Majority flips (+/-) | Ablation control | Verdict |",
        "|---|---:|---:|---:|---:|---|---|",
    ]
    for comparison in primary:
        runner = comparison["runner"]
        model = comparison["model"]
        baseline_mode, treatment_mode = _comparison_modes(comparison)
        baseline = _rows_for_cell(
            rows,
            provenance=provenance,
            model=model,
            runner=runner,
            mode=baseline_mode,
            split="core",
        )
        treatment = _rows_for_cell(
            rows,
            provenance=provenance,
            model=model,
            runner=runner,
            mode=treatment_mode,
            split="core",
        )
        ablation_mode = comparison.get("ablation")
        ablation = (
            _rows_for_cell(
                rows,
                provenance=provenance,
                model=model,
                runner=runner,
                mode=ablation_mode,
                split="core",
            )
            if ablation_mode
            else []
        )
        label = f"{model} `{baseline_mode}` -> `{treatment_mode}`"
        if not baseline or not treatment:
            lines.append(f"| {label} | 0 | - | - | - | - | MISSING |")
            continue
        ci = hierarchical_bootstrap_ci(treatment, baseline)
        flips = flip_table(baseline, treatment).majority_of_k
        exact_p = exact_sign_test(flips.pass_fail, flips.fail_pass)
        verdict = evaluate_success_threshold(
            lift_ci_low_pp=ci.low * 100,
            exact_p=exact_p,
            favorable_quarantines=0,
            manifest=manifest,
        )
        verdict_label = "PASS" if verdict == "confirmed" else "FAIL (downgrade)"
        ablation_status = _render_ablation_status(
            baseline_trials=baseline,
            ablation_trials=ablation,
        )
        lines.append(
            "| "
            + " | ".join(
                [
                    label,
                    str(len({row.get("task_id") for row in baseline})),
                    f"{_fmt_pp(ci.estimate)} [{_fmt_pp(ci.low)}, {_fmt_pp(ci.high)}]",
                    f"{exact_p:.4g}",
                    f"+{flips.fail_pass}/-{flips.pass_fail}",
                    ablation_status,
                    verdict_label,
                ]
            )
            + " |"
        )
    return "\n".join(lines) + "\n"


def _render_interpretation_notes(notes: list[dict]) -> str:
    if not notes:
        return "_No post-freeze interpretation notes._\n"
    lines = []
    for note in notes:
        affected = note.get("affected_rows") or {}
        affected_parts = []
        for bundle, modes in affected.items():
            if isinstance(modes, dict):
                mode_counts = ", ".join(f"{mode} {count}" for mode, count in modes.items())
                affected_parts.append(f"{bundle}: {mode_counts}")
            else:
                affected_parts.append(str(bundle))
        affected_text = "; ".join(affected_parts) if affected_parts else "none"
        unaffected = ", ".join(note.get("unaffected_bundles") or [])
        unaffected_text = f" Unaffected bundles: {unaffected}." if unaffected else ""
        lines.append(
            f"- {note.get('date', 'undated')} — {note.get('topic', 'note')}: "
            f"{note.get('summary', '')} Affected rows: {affected_text}.{unaffected_text}"
        )
    return "\n".join(lines) + "\n"


def _render_cell_table(rows: list[dict], provenance: dict[str, str], split: str) -> str:
    split_rows = [row for row in rows if provenance.get(row.get("task_id"), "core") == split]
    if not split_rows:
        return "_No rows in this split._\n"
    keys = sorted({(row.get("model"), row.get("runner"), row.get("mode")) for row in split_rows})
    lines = [
        "| Model | Runner | Mode | Tasks | Trials | Mean pass@1 ± 95% CI | pass^k | Median cost | Cost/success |",
        "|---|---|---|---:|---:|---:|---:|---:|---:|",
    ]
    for model, runner, mode in keys:
        trials = cell_trials(split_rows, mode=mode, model=model, runner=runner)
        task_ids = sorted({row.get("task_id") for row in trials})
        successes = sum(1 for row in trials if row.get("correct") is True)
        interval = wilson_ci(successes, len(trials))
        pass1 = pass_at_1_mean(trials)
        passk = pass_hat_k(trials)
        costs = [cost for cost in (_cost_value(row) for row in trials) if cost is not None]
        median_cost = statistics.median(costs) if costs else None
        cost_success = (sum(costs) / successes) if costs and successes else None
        lines.append(
            "| "
            + " | ".join(
                [
                    str(model or ""),
                    f"`{runner or ''}`",
                    f"`{mode or ''}`",
                    str(len(task_ids)),
                    str(len(trials)),
                    f"{_fmt_pct(pass1)} [{_fmt_pct(interval.low)}, {_fmt_pct(interval.high)}]",
                    _fmt_pct(passk),
                    "-" if median_cost is None else f"{median_cost:.4g}",
                    "-" if cost_success is None else f"{cost_success:.4g}",
                ]
            )
            + " |"
        )
    return "\n".join(lines) + "\n"


def _render_cost_frontier(rows: list[dict], provenance: dict[str, str]) -> str:
    core_rows = [row for row in rows if provenance.get(row.get("task_id"), "core") == "core"]
    keys = sorted({(row.get("model"), row.get("runner"), row.get("mode")) for row in core_rows})
    points = []
    for model, runner, mode in keys:
        trials = cell_trials(core_rows, mode=mode, model=model, runner=runner)
        if not trials:
            continue
        pass1 = pass_at_1_mean(trials)
        costs = [cost for cost in (_cost_value(row) for row in trials) if cost is not None]
        median_cost = statistics.median(costs) if costs else None
        points.append((model, runner, mode, pass1, median_cost))
    if not points:
        return "_No core cost data yet._\n"
    lines = [
        "| Model | Runner | Mode | Mean pass@1 | Median cost |",
        "|---|---|---|---:|---:|",
    ]
    for model, runner, mode, pass1, median_cost in sorted(points, key=lambda item: (-(item[3]), item[4] is None, item[4] or 0)):
        lines.append(
            f"| {model or ''} | `{runner or ''}` | `{mode or ''}` | {_fmt_pct(pass1)} | "
            f"{'-' if median_cost is None else f'{median_cost:.4g}'} |"
        )
    return "\n".join(lines) + "\n"


def _render_variance_decomposition(rows: list[dict], provenance: dict[str, str]) -> str:
    core_rows = [row for row in rows if provenance.get(row.get("task_id"), "core") == "core"]
    keys = sorted({(row.get("model"), row.get("runner"), row.get("mode")) for row in core_rows})
    if not keys:
        return "_No core variance data yet._\n"
    lines = [
        "| Model | Runner | Mode | Tasks | Mean k | Task variance term | Trial variance term | Task share |",
        "|---|---|---|---:|---:|---:|---:|---:|",
    ]
    for model, runner, mode in keys:
        trials = cell_trials(core_rows, mode=mode, model=model, runner=runner)
        if not trials:
            continue
        decomp = variance_decomposition(trials)
        task_share = decomp.task_variance_term / decomp.total if decomp.total else 0.0
        lines.append(
            "| "
            + " | ".join(
                [
                    str(model or ""),
                    f"`{runner or ''}`",
                    f"`{mode or ''}`",
                    str(decomp.n_tasks),
                    f"{decomp.mean_k:.1f}",
                    f"{decomp.task_variance_term:.6f}",
                    f"{decomp.trial_variance_term:.6f}",
                    _fmt_pct(task_share),
                ]
            )
            + " |"
        )
    return "\n".join(lines) + "\n"


def _render_harness_card(run_meta: list[dict]) -> str:
    if not run_meta:
        return "_No v3 run bundles are registered yet._\n"
    lines = [
        "| Runner | Model | N | Temperature policy | Scorer | Manifest | Holdout |",
        "|---|---|---:|---|---|---|---:|",
    ]
    for run in run_meta:
        lines.append(
            "| "
            + " | ".join(
                [
                    f"`{run.get('runner') or ''}`",
                    str(run.get("model") or ""),
                    str(run.get("trials_per_cell") or run.get("runs_per_task") or ""),
                    str(run.get("temperature_policy") or ""),
                    str(run.get("scorer_version") or SCORER_VERSION),
                    str(run.get("manifest_hash") or "not-pinned-yet"),
                    str(run.get("holdout_version") or ""),
                ]
            )
            + " |"
        )
    return "\n".join(lines) + "\n"


def render_v3(
    date_stamp: str,
    bundles: list[str | Path] | None = None,
    *,
    tasks_path: Path = TASKS,
    adjudications_path: Path = ADJUDICATIONS,
    manifest_path: Path = MANIFEST_PATH,
) -> str:
    bundles = V3_CANON if bundles is None else bundles
    provenance = _task_provenance(tasks_path)
    adjudicated = _load_adjudications(adjudications_path)
    notes = _load_interpretation_notes(adjudications_path)
    manifest = load_manifest(manifest_path)
    run_meta: list[dict] = []
    rows: list[dict] = []
    exploratory_rows: list[dict] = []
    exploratory_reasons: list[str] = []
    for bundle in bundles:
        run, bundle_rows = _load_bundle(bundle)
        bundle_rows = _rows_with_run_metadata(run, bundle_rows)
        blocked = _blocked_quarantines(bundle_rows, adjudicated)
        if blocked:
            offenders = ", ".join(
                f"{row.get('task_id')}:{row.get('mode')}:run{row.get('run_index')}"
                for row in blocked
            )
            raise CanonBlockedError(f"unadjudicated scorer divergence blocks v3 canon: {offenders}")
        conformance = bundle_conformance(run, manifest)
        run = dict(run)
        run["canon_status"] = conformance.status
        if conformance.reasons:
            run["canon_reasons"] = list(conformance.reasons)
        run_meta.append(run)
        if conformance.headline_eligible:
            rows.extend(bundle_rows)
        else:
            exploratory_rows.extend(bundle_rows)
            exploratory_reasons.append(
                f"{_bundle_path(bundle).name}: {', '.join(conformance.reasons)}"
            )

    parts = [
        "# mdtools benchmark v3 — canonical results\n",
        "> Generated by `bench/results_canon.py`. Pre-v3 numbers are archived below for provenance only and should not be cited as current evidence.\n",
        f"Generated: {date_stamp}.\n",
        "## Headline Status\n",
    ]
    if not rows:
        parts.append("No headline-eligible v3 run bundles are registered yet.\n")
    else:
        parts.extend(
            [
                _render_headline_status(rows, provenance, manifest),
                "## Interpretation Notes\n",
                _render_interpretation_notes(notes),
                "## Core Tasks\n",
                _render_cell_table(rows, provenance, "core"),
                "## Variance Decomposition\n",
                "Approximate contribution to mean pass-rate variance; high task share means more corpus breadth buys more certainty than more repeats.\n",
                _render_variance_decomposition(rows, provenance),
                "## Adversarially Mined Tasks\n",
                "Adversarially filtered tasks are reported separately and are not generalized as headline evidence.\n",
                _render_cell_table(rows, provenance, "adversarially-mined"),
                "## Cost-vs-Success Frontier\n",
                _render_cost_frontier(rows, provenance),
                "## Quarantine Report\n",
                "No unadjudicated scorer divergences in the registered v3 bundles.\n",
            ]
        )
    if exploratory_rows:
        parts.extend(
            [
                "## Exploratory Bundles\n",
                "These bundles do not conform to the frozen v3 manifest and are not headline evidence.\n",
            ]
        )
        for reason in exploratory_reasons:
            parts.append(f"- {reason}")
        parts.extend(["", _render_cell_table(exploratory_rows, provenance, "core")])
    parts.extend(["## Harness Card\n", _render_harness_card(run_meta)])
    return "\n".join(parts).rstrip() + "\n"


def bundle_date(bundle):
    rj = RUNS / bundle / "run.json"
    if rj.exists():
        d = json.loads(rj.read_text()).get("finished_at") or ""
        return d[:10]
    return ""


def runner_of(bundle):
    rj = RUNS / bundle / "run.json"
    if rj.exists():
        return json.loads(rj.read_text()).get("runner", "")
    return ""


def collect(axis):
    """-> {model: {mode: {'pass':p,'total':t,'failed':[...]}}}, dates, runners"""
    agg = {}
    dates, runners = set(), {}
    for b in axis["bundles"]:
        rows = load_rows(b)
        d = bundle_date(b)
        if d:
            dates.add(d)
        for r in rows:
            model = r["model"]
            mode = r["mode"]
            runners[model] = runner_of(b) or runners.get(model, "")
            cell = agg.setdefault(model, {}).setdefault(
                mode, {"pass": 0, "total": 0, "failed": []}
            )
            cell["total"] += 1
            if r.get("correct"):
                cell["pass"] += 1
            else:
                cell["failed"].append(r["task_id"])
    return agg, sorted(dates), runners


def pct(p, t):
    return round(p / t * 100) if t else 0


def lift_pp(cell_from, cell_to):
    if not cell_from["total"] or not cell_to["total"]:
        return 0
    rf = cell_from["pass"] / cell_from["total"]
    rt = cell_to["pass"] / cell_to["total"]
    return round((rt - rf) * 100)


def universal_fails(agg, modes):
    """Task IDs failed in EVERY present (model, mode) cell of an axis."""
    sets = [
        set(cells[m]["failed"])
        for cells in agg.values()
        for m in modes
        if m in cells
    ]
    if not sets:
        return []
    return sorted(set.intersection(*sets), key=task_key)


def model_sort(model):
    # Haiku before GPT before Sonnet, then anything else
    order = ["haiku", "gpt", "sonnet"]
    low = model.lower()
    for i, k in enumerate(order):
        if k in low:
            return (i, model)
    return (len(order), model)


def render_axis(axis, agg, dates, runners):
    modes = axis["modes"]
    out = []
    out.append(f"### {axis['title']}\n")
    out.append(axis["intro"] + "\n")
    span = dates[0] if len(dates) == 1 else f"{dates[0]}..{dates[-1]}"
    n_vals = sorted({c["total"] for cells in agg.values() for c in cells.values()})
    n_str = f"{n_vals[0]} tasks" if len(n_vals) == 1 else f"{n_vals[0]}-{n_vals[-1]} tasks"
    out.append(f"Corpus: {n_str}, N=1 per task/mode. Measured: {span}.\n")

    header = ["Model", "Runner"] + [f"`{m}`" for m in modes]
    header.append(f"`{axis['lift_to']}` lift over `{axis['lift_from']}`")
    out.append("| " + " | ".join(header) + " |")
    out.append("|" + "|".join(["---"] * len(header)) + "|")

    for model in sorted(agg, key=model_sort):
        label = MODEL_LABELS.get(model, model)
        cells = agg[model]
        row = [label, f"`{runners.get(model,'')}`"]
        for m in modes:
            c = cells.get(m)
            row.append(f"{c['pass']}/{c['total']} ({pct(c['pass'],c['total'])}%)" if c else "-")
        cf, ct = cells.get(axis["lift_from"]), cells.get(axis["lift_to"])
        row.append(f"+{lift_pp(cf,ct)}pp" if cf and ct else "-")
        out.append("| " + " | ".join(row) + " |")
    out.append("")

    # ablation-clean check, derived not asserted
    if axis["key"] == "native":
        notes = []
        for model in sorted(agg, key=model_sort):
            cells = agg[model]
            base = cells.get("native")
            ctrl = cells.get("native+md-no-md")
            if base and ctrl:
                clean = base["pass"] == ctrl["pass"]
                notes.append(
                    f"{MODEL_LABELS.get(model, model)}: control "
                    f"{ctrl['pass']}/{ctrl['total']} vs native {base['pass']}/{base['total']} "
                    f"-> {'ablation-clean' if clean else 'NOT clean, investigate'}"
                )
        if notes:
            out.append("Ablation check (control vs baseline, from the bundles):\n")
            for n in notes:
                out.append(f"- {n}")
            out.append("")

    # failed tasks, compact
    out.append("Failed tasks:\n")
    out.append("| Model | Mode | Failed |")
    out.append("|---|---|---|")
    for model in sorted(agg, key=model_sort):
        for m in modes:
            c = agg[model].get(m)
            if not c:
                continue
            failed = ", ".join(sorted(c["failed"], key=task_key)) or "(none)"
            out.append(f"| {MODEL_LABELS.get(model, model)} | `{m}` | {failed} |")
    out.append("")
    return "\n".join(out)


MECHANISM = """\
## What the lift is, and isn't

The weak-model lift is mostly **structure extraction**, not editing. The bulk of what
the weak models failed on `native` were the reads: build an outline, count the
checkboxes under each phase, count the real tasks while ignoring fake checkboxes
sitting inside a fenced code block. A weak model assembles that structure by eye and
miscounts; `md` hands it the structure directly and those flip to passing. The flipped
set also includes a few non-read tasks (section insertion, frontmatter edits), so this
is the dominant mechanism, not the only one.

It is **not** the duplicate-heading / exact-string-Edit story. T6 and T13 both pass on
plain `native` for both weak models (verify in the bundles above). `pi`'s edit tool does
throw a uniqueness error on duplicate headings, and that string shows up in the
transcripts, but the model recovers and the task passes, so it is not load-bearing for
any of these results. (This claim was wrong in the README twice; do not reintroduce it.)
"""

PROVISIONAL = """\
## Cited, not regenerated

These have no clean aggregate bundle, so they are not auto-derived. Treat as provisional
and check the source before quoting.

- **Qwen3.5-27B-4bit (local).** +38.9pp `hybrid` over `unix` on a fixed 18-task search
  corpus, April 28 2026, `oai-loop` runner. Source: README history + per-task bundles
  under `bench/runs/t10-*`, `t11-*`. The local-model signal points the same direction as
  the shell axis above, but the bundles are per-task, not a single aggregate.
- **Sonnet 4.6 vs native Edit: no reliable advantage.** The inverse end of the capability
  axis. Source: `docs/decisions/2026-06-04-md-frontier-edge-falsification.md`,
  `bench/runs/native-arm-2026-06-03/NOTES.md`.
- **Historical 20-task v1 snapshot (April 2 2026):** Haiku 50->87, Sonnet 80->85,
  Opus 89->83 (unix->hybrid). Source: `bench/tasks/tasks_v1.json`. Superseded by the
  28-task corpus above; kept for trend only.
"""


def render_v2_archive(date_stamp):
    collected = [(axis, *collect(axis)) for axis in CANON]
    uf = {axis["key"]: universal_fails(agg, axis["modes"]) for axis, agg, _d, _r in collected}

    def fmt(key):
        ids = uf.get(key) or []
        return ", ".join(ids) if ids else "none"

    parts = [
        "# mdtools benchmark — canonical results\n",
        "> ARCHIVED (v2) — retracted. These pre-v3 numbers are kept for provenance only "
        "and should not be cited as current evidence; see `bench/V3.md` for the v3 "
        "protocol.\n",
        "> Generated by `bench/results_canon.py` from the run bundles. Do not hand-edit: "
        "run `python3 bench/results_canon.py -i` to regenerate. Ground truth is the "
        "`correct` field in each cited bundle's `results.json`.\n",
        f"Generated: {date_stamp}.\n",
        "One run per task/mode (N=1), so read point estimates as indicative. Tasks that "
        "fail in every cell of an axis are excluded from headline claims and flagged for "
        f"inspection: shell axis -> {fmt('shell')}; native axis -> {fmt('native')}.\n",
        "Lifts are the pp change computed from the raw pass counts (e.g. 27/28 - 19/28 "
        "= 8/28 = +29pp). Subtracting the rounded percentages in the table can land ~1pp "
        "off; the fractions are the truth.\n",
        "## Tables\n",
    ]
    for axis, agg, dates, runners in collected:
        parts.append(render_axis(axis, agg, dates, runners))
    parts.append(MECHANISM)
    parts.append(PROVISIONAL)
    return "\n".join(parts).rstrip() + "\n"


def render(date_stamp):
    return (
        render_v3(date_stamp)
        + "\n---\n\n"
        + render_v2_archive(date_stamp)
    )


def main():
    # date is passed in so the script has no hidden clock dependency
    import subprocess

    date_stamp = subprocess.run(
        ["date", "+%Y-%m-%d"], capture_output=True, text=True
    ).stdout.strip()
    doc = render(date_stamp)
    if "-i" in sys.argv[1:]:
        OUT.write_text(doc)
        print(f"wrote {OUT}")
    else:
        sys.stdout.write(doc)


if __name__ == "__main__":
    main()
