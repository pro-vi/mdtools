#!/usr/bin/env python3
"""Generate bench/RESULTS.md — the single canonical benchmark page — from run bundles.

Every number here is derived from a bundle's `results.json` (the per-task `correct`
field is ground truth), never typed by hand, so the doc cannot drift from the data.

    python3 bench/results_canon.py        # print to stdout
    python3 bench/results_canon.py -i      # write bench/RESULTS.md

To add evidence: drop the bundle under bench/runs/, append its dir to the relevant
axis in CANON below, and re-run. Rows are grouped by each row's own `mode` field,
so directory-name inconsistencies (e.g. the no-md ablation bundle that lost its
suffix) do not matter.

What is NOT auto-derived (no clean aggregate bundle exists): the Qwen local-runner
row, the Sonnet 4.6 native null, and the historical 20-task v1 snapshot. Those live
in PROVISIONAL below as cited evidence with source pointers, clearly partitioned from
the regenerated tables.
"""
import json
import sys
from pathlib import Path

RUNS = Path(__file__).resolve().parent / "runs"
OUT = Path(__file__).resolve().parent / "RESULTS.md"

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


def render(date_stamp):
    collected = [(axis, *collect(axis)) for axis in CANON]
    uf = {axis["key"]: universal_fails(agg, axis["modes"]) for axis, agg, _d, _r in collected}

    def fmt(key):
        ids = uf.get(key) or []
        return ", ".join(ids) if ids else "none"

    parts = [
        "# mdtools benchmark — canonical results\n",
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
