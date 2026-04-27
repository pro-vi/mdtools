# F8-2 — `extract_last_json` prefers a nested array over the wrapping object

**Status:** OPEN (filed T8 iter 4, 2026-04-26).
**Surface:** `bench/harness.py` — `extract_last_json` (line 1539) and the
text-output branch of `select_json_envelope_actual` (lines 1525-1532).
**Severity:** P1 (false-NEGATIVE on the unix-mode path through the
text-output fallback for json_envelope tasks).

## Failing trace

When the agent's text output contains the correct envelope as a wrapping
dict whose `entries` (or `results`/`links`) field is a non-empty list of
dicts, embedded in prose, `extract_last_json` returns the *inner array*
instead of the wrapping dict. The text-output branch of
`select_json_envelope_actual` does not shape-check the returned candidate,
so the array propagates to `score_structural_json`, which fails the task
with "expected top-level JSON object" — even though the agent's correct
envelope was present in the text.

This is the text-output mirror of F8-1 (which addressed the tool-output
shape-match path via subset check). The hardening landscape is symmetric:
F8-1 added shape-aware preference on the tool-output side; F8-2 needs
shape-aware preference (or a "prefer outermost" rule) on the text-output
side.

## Reproduction

```
python3 bench/probes/F8-2-extract-prefers-nested-array/probe.py
```

Exit 0 = inert (hardening landed). Exit 1 = live (current state).
Exit 2 = unexpected output.

The probe runs two stages: a direct call to `extract_last_json` (Stage A)
and a call through `select_json_envelope_actual` with an empty tool
transcript (Stage B). Both are expected to return the wrapping envelope
once F8-2 closes; both currently return the nested array.

## Closure plan (next iteration)

1. Harden `extract_last_json` so a wrapping object is preferred over a
   nested array when the array's source span is contained within an
   object's source span. (Or: drop the unconditional "prefer arrays"
   rule and instead prefer the latest top-level candidate, treating
   array and object as equal-rank.)
2. Add a unit test in `bench/test_harness_json.py` pinning the new
   precedence using the same `WRAPPING_ENVELOPE` shape.
3. Rerun this probe; expect exit 0.
4. Verify F8-1 regression coverage still passes (the F8-1 test pins the
   tool-output-side selector behavior; the text-output extraction is a
   separate code path).

## Why this matters

For unix-mode and hybrid-mode runs where the agent uses jq/cat to build
the JSON answer in their final text response (rather than capturing it
from a tool output), the text-output branch is the only code path that
recovers the answer. A scorer false-NEGATIVE on this path under-credits
unix-mode performance and contaminates the cross-mode comparison the
benchmark headline depends on. F8-1 already removed the analogous
false-NEGATIVE on the tool-output path; F8-2 closes the symmetric gap.
