#!/usr/bin/env python3
"""Billed-$ re-score of the native-arm sweep (FRAC-194 U7 validity check).

The harness cost metric is RAW tokens (input+cache_creation+cache_read+output, all
1.0x). That over-weights cached re-reads: native+md is ~88% cache_read, which raw
counts at 1.0x but Sonnet bills at 0.1x. Re-scoring from the transcripts in
input-equivalents (input 1, cache_create 1.25, cache_read 0.1, output 5) shrinks the
native+md penalty from raw +73% to billed +28% on the Targeted cell (sign survives,
magnitude was inflated ~2.5x). Run: python3 billed_rescore.py
"""
import glob, json, os, statistics

W = dict(inp=1.0, cc=1.25, cr=0.1, out=5.0)  # Sonnet 4.6 input-equivalents
CELLS = {"Targeted": {"T7", "T10", "T13", "T20"}, "Batch": {"T12"}}
HERE = os.path.dirname(os.path.abspath(__file__))

rows = []
for f in glob.glob(os.path.join(HERE, "*/logs/*/agent_output.txt")):
    mode = os.path.basename(f.split("/logs/")[0]).split("__")[0]
    task = os.path.basename(os.path.dirname(f)).split("_")[0]
    try:
        data = json.load(open(f))
    except Exception:
        continue
    u = next((m.get("usage") or {} for m in data
              if isinstance(m, dict) and m.get("type") == "result"), {})
    inp, cc = int(u.get("input_tokens") or 0), int(u.get("cache_creation_input_tokens") or 0)
    cr, out = int(u.get("cache_read_input_tokens") or 0), int(u.get("output_tokens") or 0)
    rows.append((mode, task, inp + cc + cr + out,
                 inp * W["inp"] + cc * W["cc"] + cr * W["cr"] + out * W["out"]))


def cell(mode, tasks, idx):
    per = [statistics.median([r[idx] for r in rows if r[0] == mode and r[1] == t])
           for t in tasks if any(r[0] == mode and r[1] == t for r in rows)]
    return statistics.median(per) if per else None


for name, tasks in CELLS.items():
    br, bb = cell("native", tasks, 2), cell("native", tasks, 3)
    print(f"\n[{name}]  (vs native: raw / billed-$)")
    for mode in ("native", "native+md", "native+md-no-md"):
        r, b = cell(mode, tasks, 2), cell(mode, tasks, 3)
        if r is None:
            continue
        dr = f"{(r/br-1)*100:+.0f}%" if br else "-"
        db = f"{(b/bb-1)*100:+.0f}%" if bb else "-"
        print(f"  {mode:16s} raw={r:8.0f} ({dr:>5s})   billed={b:8.0f} ({db:>5s})")
