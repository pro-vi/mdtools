# Bench Ledger

Concise human-memory surface for the frontier loop. Weaker evidence than typed
artifacts under `bench/runs/`. OPEN findings gate claim-expansion; they are
cleared only after a typed artifact confirms the finding, not by prose alone.

## OPEN findings

### F3 — T22 structural scorer rejects list-shape JSON with mode-neutral task description
- **Status:** OPEN (re-opened 2026-04-24 after external review retraction)
- **Axis:** oracle trustworthiness
- **Anchor:** `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/` pre-fix bundles both show `T22: json_envelope: MISMATCH expected top-level JSON object (actual=list, expected=dict)`. The original T22 description ("List all standard inline markdown links as JSON. Ignore wiki links and image syntax; correctness is the ordered list of link kinds and destinations.") is deliberately tool-neutral because the same description is injected into all three benchmark modes including `unix` (where `md` is forbidden by `UNIX_DOCS` at `bench/harness.py:252`). Agents reasonably emit a top-level JSON list; the structural scorer rejects because `score_structural_json` at `bench/harness.py:450` demands a top-level object.
- **Closure criterion:** a mode-neutral fix at the scorer or system-prompt layer. Options:
  - Scorer: when `compare_link_destinations` is the only check requested, accept a top-level JSON list of `{kind, destination}` objects and treat it as equivalent to `{"links": [...]}`.
  - System prompt: for `expected_artifact == "json_envelope"` tasks, amend `output_instruction` in `build_prompt` to describe the expected envelope shape tool-agnostically (e.g., "Print the result as a JSON object with top-level fields described by the task") rather than just "Print the result as JSON to stdout."
  - Not acceptable: editing the task description in `bench/tasks/tasks.json`. T22 is in `bench/holdout/task_ids.json`; post-hoc tuning of a holdout task destroys its holdout validity for any rerun.
- **Prior mis-fix (retracted):** an earlier iteration rewrote T22's description to include `using \`md links --json\``, which (a) mode-contaminated by telling unix-mode agents to use a forbidden tool, and (b) tuned a holdout task after seeing its failure. Description has been reverted. The four bundles produced against the tuned task have been moved to `bench/retracted_2026-04-24/` (outside `bench/runs/`) so `bench/analyze.py` / `bench/report.py` / glob-based tooling do not treat them as valid evidence. See that directory's `README.md`.

### L1 — Loop lacked holdout-immutability guard
- **Status:** OPEN (loop-level learning)
- **Axis:** oracle trustworthiness (meta)
- **Anchor:** this session's iteration sequence made a change to `bench/tasks/tasks.json` where the edited task ID was in `bench/holdout/task_ids.json`, then reran holdout and published the new pass rates as confirmation. The loop's iteration protocol did not catch this. External review (2026-04-24) surfaced it.
- **Closure criterion:** either a procedural rule ("before editing any task in `bench/tasks/tasks.json`, check whether its ID is in `bench/holdout/task_ids.json` — if yes, the edit invalidates the holdout until the task is rotated out of holdout or the split is regenerated") documented in the frontier-loop spec or in `bench/ledger.md`, or a mechanical guard (pre-commit hook / harness assertion that flags holdout-task description diffs) that prevents the same mistake.

## FIXED_PENDING_CONFIRMATION

### F2 — Legacy N=3 snapshot overlaps the current holdout set
- **Status:** FIXED_PENDING_CONFIRMATION (2026-04-24)
- **Axis:** specification coherence
- **Typed artifact:** `bench/RESULTS.md` now opens with a "Legacy N=3 Haiku snapshot" header and a split-disclosure note stating the snapshot predates the search/holdout split, naming T4/T14/T20 as now-holdout rows and T22–T24 as post-snapshot tasks. Readers encounter the caveat before the per-task table.
- **Closure criterion:** satisfied. Promoted once the next review pass does not re-raise.

### F1 — Search-split pilots lack holdout confirmation (partial)
- **Status:** FIXED_PENDING_CONFIRMATION (2026-04-24) for ability-to-run-holdout; durability claim retracted
- **Axis:** oracle trustworthiness / specification coherence
- **Typed artifacts:** `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/` (pre-fix, 50% each). Four additional bundles produced during the loop have been moved to `bench/retracted_2026-04-24/` with an invalidation README — see that directory.
- **Substantive outcome:** holdout can now be run, durable bundles exist, and the first real holdout run exposed scorer-surface defects the search split hid. The narrower claim "holdout-confirmed at 100%" has been retracted — current valid holdout result is 50% for the best-in-class search cell, with the F3 scorer defect preventing any honest reconfirmation until F3 is fixed mode-neutrally.

### F3-a — `raw_bytes` scorer now honors EOF whitespace
- **Status:** FIXED_PENDING_CONFIRMATION (2026-04-24)
- **Axis:** oracle trustworthiness
- **Typed artifact:** `bench/harness.py:339-341` — `.rstrip()` added on the whole normalized string after per-line rstrip, so `ignore_trailing_whitespace: true` covers end-of-file whitespace consistent with the option name. Mode-neutral change; external review accepted. Harness dry-run confirms all 24 tasks still pass dual scorer.
- **Closure criterion:** satisfied. Promoted once the next review pass does not re-raise.

## CLOSED

_(none yet)_
