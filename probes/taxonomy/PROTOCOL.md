# Failure Taxonomy Protocol

Date locked: 2026-07-03

## Scope

Label every failed trial in these v3 close-out bundles:

- `bench/runs/v3-closeout-haiku-shell-2026-07-02`
- `bench/runs/v3-closeout-haiku-native-2026-07-03`
- `bench/runs/v3-closeout-gpt54mini-native-2026-07-03`

Only the baseline and treated cells are in scope:

- Haiku shell: `unix`, `hybrid`
- Haiku native: `native`, `native+md`
- GPT 5.4 mini native: `native`, `native+md`

The no-md ablation controls remain part of the v3 headline evidence, but are not labeled
for this mechanism table because U2 asks which baseline failure classes the treated md
arms remove.

## Closed Class Set

Each failed trial receives exactly one primary class:

- `wrong-target` - edited or extracted from the wrong section, heading, list item, or task.
- `structure-corruption` - broke Markdown fences, headings, nesting, list shape, or table structure.
- `quoting-escaping` - shell quoting or escaping changed the intended literal content.
- `duplicate-collision` - failed because the target text or heading was non-unique.
- `format-noncompliance` - content was plausibly right, but output schema, keys, or formatting violated the task contract.
- `incomplete-multistep` - an expected later operation was skipped, drift was not handled, or the task stopped after a partial edit.
- `gave-up` - explicit refusal, unsupported/ambiguous claim, or a non-attempt.
- `infra` - runner error, timeout, sandbox/tooling failure, or scorer-inapplicable infrastructure fault.
- `other` - a real task failure not covered by the closed set.

Classes may not be added, removed, renamed, or merged after this protocol is committed.

## Primary-Class Rule

When a trajectory contains more than one failure shape, label the first failure that made
success unrecoverable. Later symptoms are recorded only in notes.

Examples:

- Wrong section first, then malformed output: `wrong-target`.
- Correct target first, shell quoting mangles the insertion, then scorer fails: `quoting-escaping`.
- Agent does step 1 correctly but never performs step 2: `incomplete-multistep`.
- Runner error prevents meaningful judgment: `infra`.

## Labeling Method

1. Extract all failed in-scope trials from `results.json` plus their corresponding logs.
2. Assign one primary class from the closed set, using the result diagnostics and the
   trajectory log together.
3. Double-label a deterministic 20% sample: every fifth failed trial after sorting by
   bundle, mode, task id, and run index.
4. Report agreement as exact primary-class agreement. If agreement is below 0.80, publish
   the table with the disagreement rate flagged.
5. Human adjudication decides any sampled disagreement; the adjudicated class is the
   published class.

## Output

Write `bench/v3/failure_taxonomy.json` with:

- protocol metadata and bundle list
- one label per failed in-scope trial
- double-label sample and agreement summary
- per-cell class counts

Render the counts in `bench/RESULTS.md` under a "Mechanism Evidence (Exploratory)"
header. The table is exploratory and must not feed the v3 headline gate.
