# F8-3 — `extract_last_json` depth scanner is brace-in-string blind

**Status:** CLOSED T8 iter 7. The depth scanner in
`extract_last_json` (`bench/harness.py`) now skips characters between
unescaped `"` boundaries in both `{`/`}` and `[`/`]` passes, so brace
or bracket characters inside JSON string values no longer falsely
close the candidate. Pinned by
`bench/test_harness_json.py::test_extract_last_json_honors_string_boundaries_in_depth_scan`
+ `::test_extract_last_json_handles_escaped_quotes_in_string_value`.
Attribution probe rerun: `probe.py` exit 0 on both stages (filed
state was exit 1).

**Surface:** `bench/harness.py` `extract_last_json` (lines ~1539–1587)
and the text-output branch of `select_json_envelope_actual` (lines
~1525–1532) which propagates the misextracted candidate unchanged.

**Severity:** P1 false-NEGATIVE on json_envelope tasks where the
agent's wrapping envelope embeds a string value containing `}`, `]`,
`{`, or `[`.

## Hypothesis

`extract_last_json` runs two independent passes — one for `[…]`,
one for `{…}` — counting depth via a single character walk. The walk
does not honor JSON string boundaries, so a `}` (or `]`/`{`/`[`)
inside a string value is counted as a real closer. When the wrapping
envelope's heading.text contains a `}`, the `{`/`}` pass closes
prematurely, the truncated candidate fails `json.loads`, and `start`
resets to `-1`. The scanner never re-anchors on the actual outer
envelope, so no `{…}` candidate survives. The only valid candidate is
the inner `entries` array recorded by the `[`/`]` pass (which is
brace-blind in a benign way).

The text-output branch of `select_json_envelope_actual` then accepts
the array unchecked at line 1531 (`isinstance(parsed, (list, dict))`
with non-empty length is the only gate), propagating the wrong shape
to `score_structural_json` which early-outs FAIL on
"expected top-level JSON object."

## Symmetry to F8-1 / F8-2

F8-1 (CLOSED iter 3) hardened the tool-output branch's shape match
from intersection to subset. F8-2 (CLOSED iter 5) replaced
extract_last_json's preference rule (array vs object) with a
highest-end-position comparator. Both prior closures assumed the
wrapping envelope is always recorded as a candidate; F8-3 falsifies
that assumption — when string values carry brace characters, the
wrapping envelope is not recorded at all, so neither prior closure
fires. F8-3 is fresh.

## Reproduction

```bash
python3 bench/probes/F8-3-brace-in-string-value/probe.py
```

Pre-fix exit = 1 (live failing trace). Both stages fail:
- Stage A: direct `extract_last_json` returns the inner `entries` array
- Stage B: `select_json_envelope_actual` text-output branch
  propagates the inner array unchecked

Post-fix exit = 0 (inert).

## Realism note

The current 8 json_envelope bench tasks (T1, T5, T9, T11, T16, T19,
T21, T22) have heading text without brace characters, so this trace
does not currently surface in the benchmark. The bug is structural:
any future task instance whose source markdown carries headings or
code-formatted text with `{…}` syntax (extremely common in technical
documents — Configuration `{key: value}`, format strings `{name}`,
shell expansions `${VAR}`, code samples) will hit it. The probe is
filed pre-emptively to lock the json_envelope path against a known
realistic shape, in the same tradition as F8-2's pre-emptive closure.

## Closure plan

Replace the dual-character-walk depth scanner with a string-aware
scanner that skips characters between unescaped `"` boundaries when
counting brace/bracket depth. Pin with a unit test in
`bench/test_harness_json.py` mirroring the probe's stage-A shape.
Attribution probe: rerun `probe.py`; expect exit 0 on both stages.

## Files

- `probe.py` — standalone reproduction (exit 1 = live, exit 0 = inert)
- `verdict.txt` — recorded probe verdicts (filed and, post-closure, closure)
