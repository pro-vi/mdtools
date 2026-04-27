# F8-1 — `select_json_envelope_actual` accepts schema_version-only overlap

**Status:** OPEN (filed 2026-04-26 at T8 iter 2)
**Surface:** scorer (`bench/harness.py:1481-1526`, `select_json_envelope_actual`)
**Class:** scorer false-negative
**Severity:** P1 — affects all `json_envelope` extraction tasks (T1, T5, T9, T11, T19, T22) when an agent runs >1 mdtools command and the last invocation is not the task's intended command.

## Failure trace

`probe.py` constructs the synthetic transcript:

  `all_tool_outputs = [outline_envelope, tasks_envelope]` (chronological)
  `expected = outline_envelope` (top-level keys: `schema_version, file, entries`)

The reverse iteration in `select_json_envelope_actual` sees the tasks
envelope first. Its top-level keys are `{schema_version, results}`.
Intersection with expected = `{schema_version}` — non-empty, so the F4
shape-match check (`_json_top_keys(parsed) & expected_top_keys`) accepts it.
The scorer returns the wrong envelope as `actual`. Downstream structural
comparison then marks the task FAIL — but the agent's correct answer is
in `all_tool_outputs[0]`, which the F4 selector skipped over.

## Why this matters

Every `mdtools.v1` envelope shares the top-level key `schema_version`.
F4's intersection check is therefore satisfied by *any* mdtools envelope
when the expected output is *any* mdtools envelope. The shape-match
provides no discrimination across mdtools commands; it only filters out
non-mdtools JSON.

## Hardening hypothesis (for the closure iteration, not this one)

Replace `_json_top_keys(parsed) & expected_top_keys` (intersection
non-empty) with `expected_top_keys.issubset(_json_top_keys(parsed))`
(every expected top-level key is present). This would:

- accept the outline envelope on T1 (subset matches)
- reject the tasks envelope on T1 (`file`, `entries` missing)
- still accept the tasks envelope on T22 if T22's expected shape is
  `{schema_version, results}` (no regression on tasks-shaped tasks)

Attribution probe for the closure: re-run this probe; expect exit 0.

## Out of scope (T8)

- Adding new selectors or new envelope versions. Anti-folklore lock.
- Changing the envelope shape itself. T7 substrate is frozen.
- Promoting to a unit test before the harness fix lands. The probe is
  the typed artifact; the unit test arrives with the fix to keep cheap-
  channel green.
