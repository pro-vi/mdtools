# mdtools ↔ fract-ai Bridge Contract

## Purpose

Decide whether mdtools should build new product surfaces (`md apply`, `md move-block`, generalized `md set-state`) to support a future fract-ai integration, or whether the right move is "ship what we have, instrument heavily, wait." This document does the analysis without writing any new CLI surface.

This artifact replaces the earlier "shadow demand test" framing. fract-ai has no live mdtools episodes to sample (mdtools is explicitly deferred per `fract-ai/.loop/FRAC-147/PROGRESS.md:863-869`). What we *can* do is map the published ChangeSet IR against current mdtools and classify the gaps.

## Source documents

- `fract-ai/.loop/FRAC-147/PROGRESS.md` (Seed status, 2026-04-26) — defines `Address`, `AgentOp`, `ChangeSet`, `Proposal` types, anchors, semantic journal, apply layer, 14 invariants, failure matrix, 14-phase spine. Names mdtools as a future agent surface deferred to pi-agent-core or Space Kernel V2.
- `CLAUDE.md` (mdtools repo) — current command surface and design rules.
- This repo's `src/` and `bench/RESULTS.md` — current op vocabulary and benchmark coverage.

## The contract matrix

Each row classifies a ChangeSet concern against current mdtools. **My analysis** is the technical read; **Owner verdict** is left blank for the fract-ai-side maintainer to mark `would call` / `would not` / `unknown` / `bridge-owned` / `Lexical-owned`.

| ChangeSet concern | Current mdtools candidate | My analysis | Owner verdict |
|---|---|---|---|
| `replace_text(target, start, end, text)` — projection-offset substring replace | none (no sub-block text op) | **C + E.** Missing op AND fundamentally Lexical-shaped: projection-offsets, `splitText`, NodeKey preservation. Markdown has no analog; comrak parses block-shaped, not character-shaped. | |
| `replace_block(target, format='markdown', body)` | `md replace-block <loc> --from PATH` | **A.** Direct map. Body format matches. Address translation needed (path → loc) but the op exists. | |
| `replace_block(target, format='html', body)` | none (mdtools is markdown-only) | **C + F.** Out of mdtools' domain. Bridge would need a separate HTML applier. | |
| `insert_after(after, format='markdown', body)` | `md insert-block <loc> --from PATH` | **A.** Direct map. Same address-translation caveat as `replace_block`. | |
| `insert_after(after, format='html', body)` | none | **C + F.** Same as HTML `replace_block`. | |
| `move_block(target, after)` | none; cannot synthesize cleanly (no `md delete-block`) | **C + E.** Missing op AND identity is unrecoverable in the Markdown round-trip — moving a block re-creates it; any external reference to the original is orphaned. Bridge must own identity preservation. | |
| `set_state(target, key='taskStatus', value)` | `md set-task <loc> -i\|c\|...` | **A.** Direct map for the task-state special case. | |
| `set_state(target, key=<arbitrary>, value)` | none | **F.** Markdown has no general state mechanism. Lexical state (collapsed/expanded, custom workflow, etc.) does not materialize into Markdown syntax for arbitrary keys. Generalizing `set-task` to `set-state` would invent a Markdown convention without a consumer. | |
| `Address.path` resolution (projection path + textHash + sectionTrailHash fingerprint) | `loc` (BLOCK.CHILD dot-path) + `nearest_heading` | **B → D.** Conceptually similar but not identical. mdtools' `loc` is comrak's child-index path; FRAC-147's `path` walks the *addressable-block projection*. Bridge needs a translation layer; mdtools could help by exposing a fingerprint primitive (textHash on demand) but doesn't today. | |
| `Address.anchor` resolution (uid in `anchorsState` NodeState) | none | **H.** Bridge-owned. Markdown has no native anchor; encoding via HTML comments or inline attributes is a sidecar concern that belongs in the bridge layer, not in mdtools. | |
| `Address.selector` resolution (fuzzy quote / sectionTrail / ordinalHint) | `--where` (tables only); `section --ignore-case` (ASCII) | **H.** Bridge-owned. Fuzzy recovery is the bridge's job; mdtools should not implement weighted-confidence-margin matching as a generic primitive. | |
| Lexical NodeKey preservation across materialize → edit → read-back | none | **E (impossible).** Markdown round-trip through comrak loses non-canonical formatting and has no NodeKey. The bridge must preserve NodeKeys via an external mapping table or sidecar metadata. mdtools cannot help here at all. | |
| Atomicity (all-or-nothing multi-op `ChangeSet`) | none (`md apply` does not exist) | **H (probable).** Bridge can achieve atomicity via temp-file wrapper: snapshot the file, run N mdtools commands, validate, commit-or-revert. `md apply` is only justified if the bridge wants mdtools itself as the transaction boundary, which FRAC-147 does not establish. | |
| Idempotency / `baseRevision` precondition checking | `compute_task_fingerprint` (internal) | **B → C.** mdtools has fingerprint primitives internally (used by L1 holdout guard) but does not expose them as a CLI surface. Bridge could compute its own canonical-tree hash (`RevisionId`) directly from the parsed Markdown without mdtools help, but exposing `md fingerprint <loc>` would be cheap and useful. | |

## Representative ChangeSet test cases

Each case is a realistic ChangeSet shape from FRAC-147's IR. **Blocker class** is the dominant reason current mdtools cannot handle it cleanly:

- A — existing mdtools handles it
- B — existing mdtools handles it with awkward composition
- C — missing operation
- D — address resolution
- E — Lexical identity / NodeKey preservation
- F — Markdown cannot represent the state
- G — scorer / diff / read-back
- H — bridge would bypass mdtools entirely

| # | Case | Blocker |
|---|---|:---:|
| 1 | Replace text inside a single-TextNode paragraph (`replace_text(start=0, end=5, text="Hello")`) | **C+E** |
| 2 | Replace text spanning bold/plain/italic (`replace_text` across format runs — INV-10 case) | **C+E** |
| 3 | Replace an entire paragraph block, body in markdown | **A** (direct map to `md replace-block`) |
| 4 | Replace an entire paragraph block, body in HTML | **C+F** |
| 5 | Insert a new heading + paragraph after a section's last block, body in markdown | **A** (`md insert-block`) |
| 6 | Move a block from section X to section Y | **C+E** (synthesis impossible) |
| 7 | Toggle a task checkbox (`- [ ]` → `- [x]`) via `set_state(key='taskStatus')` | **A** (`md set-task`) |
| 8 | Set a non-task state on a paragraph (`set_state(key='collapsed', value=true)`) | **F** (no Markdown encoding) |
| 9 | Edit a nested list item via `replace_text` projection-offset | **C+E** (same shape as case 2) |
| 10 | Edit a table cell — `replace_text` or whole-block swap | **B+D** (mdtools tables exist; address translation question) |
| 11 | Edit a paragraph under one of two duplicate headings (disambiguation) | **D** (mdtools' `nearest_heading` is loc-based; FRAC-147's `sectionTrailHash` is hash-based) |
| 12 | Multi-op atomic ChangeSet: replace section AND update abstract paragraph | **H** (bridge-owned via temp-file wrapper; no `md apply` needed) |
| 13 | Reject ambiguous selector resolution (low-confidence fuzzy match) | **H** (bridge-owned recovery layer) |

## Aggregate read

| Blocker class | Count |
|---|---:|
| A — existing mdtools handles | 3 (cases 3, 5, 7) |
| B — awkward composition | 0 (case 10 is B+D, dominantly D) |
| C — missing operation | 5 (cases 1, 2, 4, 6, 9) |
| D — address resolution | 2 (cases 10, 11) |
| E — Lexical identity preservation | 5 (cases 1, 2, 6, 9; structural) |
| F — Markdown cannot represent | 2 (cases 4, 8) |
| G — scorer / diff / read-back | 0 in cases (matrix row noted) |
| H — bridge would bypass mdtools | 2 (cases 12, 13) |

**The dominant blockers are E (Lexical identity, 5 cases) and C (missing op, 5 cases) — but every C case is also an E case.** That is, the missing operations (`replace_text`, HTML body, `move_block`) are all things that, even if mdtools added them, would still fail the bridge invariant because Markdown cannot preserve Lexical NodeKeys.

## Decision rules (Pro's framework, applied)

Per Pro's review (`mdtools-T7-loop-design` thread, 2026-04-26): build a primitive only if its blocker class accounts for ≥50% of cases, AND the failure is not dominantly E or H.

| Candidate primitive | Justified? | Reason |
|---|:---:|---|
| `md apply` (atomic multi-op) | **NO** | Case 12 is the only atomicity case; bridge can use temp-file wrapper. |
| `md move-block` | **NO** | Case 6 is C, but co-classified E. Building it ships a Markdown move that fails the bridge identity invariant. |
| Generalized `md set-state` | **NO** | Case 8 is F (Markdown cannot represent the state). Generalizing would invent a Markdown convention without a consumer. |
| `md fingerprint <loc>` (expose textHash on demand) | **MAYBE** | Cheap, no-regret, addresses one D case partially. Not load-bearing — bridge could compute its own. |
| `md replace-text` (sub-block character-range edit) | **NO** | Even if shipped, fails E for cases 1, 2, 9. The Lexical `splitText` machinery cannot live in mdtools. |
| HTML body support on existing ops | **NO** | Out of mdtools' markdown-only domain. |

## Outstanding unknowns to escalate to fract-ai owner

These cannot be answered from PROGRESS.md alone. They require the fract-ai bridge owner's verdict:

1. **Does the future bridge plan to call mdtools at all, or implement its own Markdown applier in TypeScript?** PROGRESS.md says "ChangeSet IR is surface-agnostic — adding bash/mdtools later requires zero changes." This is consistent with both "we'll plug mdtools in later" and "we're keeping options open."
2. **Where does NodeKey preservation live?** If the bridge owns this via metadata/sidecar, mdtools' Markdown round-trip is fine. If the bridge expects mdtools to preserve identity, the answer is "impossible."
3. **Is atomicity a CLI concern or a wrapper concern?** If the bridge already has temp-file snapshots, `md apply` is unnecessary. If the bridge wants mdtools as the transaction boundary, build it.
4. **What states are actually materialized into Markdown syntax?** If the answer is "task checkboxes only," `set-task` is sufficient and `set-state` is invented generality.

A one-page version of the matrix above, sent to the fract-ai owner with these four questions, would resolve the Outstanding column and finalize the decision.

## What this artifact is for

This is the falsification spike Pro recommended. Its job is to prevent mdtools from anchoring a 50+ iteration product loop on a speculative integration shape. The matrix shows that **no new mdtools product surface is justified by FRAC-147 alone.** The cheapest no-regret moves are:

- `md fingerprint <loc>` (small ratchet, exposes existing internal primitive)
- Telemetry on existing commands (Pro's no-regret list)
- Documentation of what mdtools can already map to (this document is part of that)

Anything larger waits for one of:
- The fract-ai bridge owner verdict on the four questions above
- A second real consumer surfacing different demand
- The materialized-file bridge actually entering scope

## Halt

This artifact is a one-shot deliverable. There is no follow-on loop until either (a) owner verdicts arrive and unblock a specific build, or (b) a second consumer creates demand. The next product loop should not anchor on this document; it should anchor on **research / hill-climb on existing mdtools surfaces** (T8, drafted separately) which is independent of bridge-contract uncertainty.
