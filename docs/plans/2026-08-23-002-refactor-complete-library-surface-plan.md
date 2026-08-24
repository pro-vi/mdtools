---
title: Complete Reusable Library Surface
objective: Every pure single-document Markdown operation is reusable in-process while the md CLI preserves its existing process contract.
type: refactor
status: complete
date: 2026-08-23
origin: conversation
---

# Complete Reusable Library Surface

## Context

The first reusable-library tranche established one Cargo package with library
and binary targets, exact document revisions, checked slicing, one section
index, reusable task reads, and pure `set-task` candidates. Its CLI, Braid-shaped,
and reader-shaped probes passed.

The remaining single-document semantics still live in CLI command modules.
Finishing the foundation means extracting those semantics without moving paths,
multi-file policy, stdin/stdout, JSON v1, diagnostics, or persistence into the
library.

## Requirements

- **R1:** Every pure single-document query and mutation is callable through the
  library.
- **R2:** Existing CLI text, JSON, diagnostics, exit codes, error precedence,
  candidate bytes, and in-place bytes remain compatible.
- **R3:** The operation-facing document is immutable and internally consistent.
- **R4:** Direct library selectors cannot represent or resolve invalid states.
- **R5:** Target etags and whole-document revisions are distinct opaque types.
- **R6:** Core edit outcomes do not embed CLI v1 target or preservation DTOs.
- **R7:** Payload edits preserve guard-before-stdin failure ordering through
  prepared edit operations.
- **R8:** Every in-place CLI mutation verifies the whole-document revision
  immediately before atomic replacement.
- **R9:** `collect` remains CLI-owned multi-file orchestration over reusable
  per-document frontmatter projection.
- **R10:** Rendering, Construal state, agents, transpositions, and UI remain out
  of mdtools.

## Naming Ledger

| Role / meaning | Existing term | Chosen name | Owner | Status | Sibling disposition |
|---|---|---|---|---|---|
| Immutable operation-facing parsed source | mutable `ParsedDocument` | `Document` | library `document` | new | keep `ParsedDocument` as low-level compatibility until Braid migrates |
| Structurally valid section operand | invalid-state `SectionSelector` | `SectionTarget` | library `section` | new | existing `SectionSelector` remains CLI v1 wire projection |
| Resolved source section | CLI-shaped `SectionEntry` | `ResolvedSection` | library `section` | new | CLI maps to `SectionEntry` |
| Opaque exact-target fingerprint | `String` | `TargetEtag` | library `fingerprint` | new | remains distinct from `DocumentRevision` |
| Pure candidate result with domain target | `EditOutcome` with wire target | `EditOutcome<T>` | library `edit` | reshape | each edit family owns its target type |
| Core preservation evidence | wire `SourcePreservationInvariant` | `EditPreservation` | library `edit` | new | CLI derives v1 invariant |
| Prepared payload edit | command-local preflight | `Prepared*Edit` | owning domain module | new | only payload-bearing operations use it |
| Block query operand | command flags | `BlockQuery` / `SearchQuery` | library block/search modules | new | CLI refines flags |
| Typed table filter | raw `--where` string | `TablePredicate` | library table module | new | raw grammar stays CLI-only |
| Typed frontmatter path and operation | raw key plus booleans | `FrontmatterPath`, `FrontmatterEdit` | library frontmatter module | new | CLI scalar grammar stays CLI-only |

## Architecture Decision

**Approach:** Complete the existing operation-oriented library by Markdown
domain. Introduce an immutable `Document` wrapper as the only input accepted by
new operations. Retain the current low-level `ParsedDocument` for measured
Braid compatibility, but do not extend it with new operations.

Core edits return generic `EditOutcome<T>` values with domain targets and
candidate bytes. CLI adapters construct the existing v1 wire receipts and own
revision recheck plus atomic persistence.

**Rationale:** A big command-module move would preserve coupling. Domain-shaped
operations make invalid operands unrepresentable, keep wire compatibility
explicit, and allow Construal to consume only Markdown semantics.

**Rejected alternative:** Privatize `ParsedDocument` immediately. Braid still
reads its fields, and this repository cannot coordinate that separate migration
in the same branch. The immutable `Document` boundary protects all new
consumers without claiming the legacy type is safe for mutation.

**Trade-offs:** Two parsed-document types coexist temporarily; explicit
core-to-wire mapping adds code; the public library surface grows before
publication.

**Approval criteria:** Every single-document command delegates semantics to the
library, the CLI baseline remains byte-compatible, the minimal consumer has no
CLI dependency, and no new operation accepts a path or writes a file.

## Program Obligations

- **O1:** `Document` cannot expose mutable access to source, blocks,
  frontmatter, line index, or revision.
- **O2:** `SectionTarget` uses `NonZeroU32` or equivalent construction so zero
  occurrence is impossible after refinement.
- **O3:** `TargetEtag` cannot be supplied where `DocumentRevision` is required.
- **O4:** `EditOutcome<T>` contains one semantic target discriminator; CLI v1
  redundant fields are derived at the adapter.
- **O5:** Every payload-bearing CLI command runs core preparation before reading
  stdin or `--from`.
- **O6:** Every changed in-place edit uses one shared revision-checking
  persistence adapter.
- **O7:** `BlockKind` search and stats policy is compiler-exhaustive.

## High-Level Technical Design

```text
source bytes -> Document (immutable)
                 ├─ block / search / links / stats
                 ├─ section / tasks
                 ├─ frontmatter projection
                 ├─ table index/query
                 └─ prepared or immediate edits
                       -> EditOutcome<DomainTarget>
                            base revision
                            candidate bytes
                            disposition
                            preservation evidence

md CLI
  refine flags -> read source -> call core -> map v1 DTO
  -> if changed + in-place: reread -> verify revision -> atomic replace
```

## Implementation Units

### U1. Repair public semantic boundaries

- **Goal:** Make direct library use safe before adding the remaining surface.
- **Requirements:** R3–R6.
- **Dependencies:** None.
- **Files:** Create `src/document.rs`; modify `src/section.rs`, `src/task.rs`,
  `src/edit.rs`, `src/fingerprint.rs`, `src/core_error.rs`, `src/model.rs`,
  `src/lib.rs`, and current CLI adapters/tests.
- **Approach:** Add immutable `Document`; add `SectionTarget` and
  `ResolvedSection`; introduce `TargetEtag`; make `EditOutcome<T>` core-only;
  preserve current v1 DTOs through explicit mapping.
- **Test scenarios:** occurrence zero and impossible preamble combinations are
  rejected; snapshot revision always matches source; target/document tokens do
  not interchange; task core and CLI candidates remain identical.
- **Verification:** Existing foundation operations accept `Document`, and no
  core outcome contains `MutationTargetRef` or
  `SourcePreservationInvariant`.
- **Proven through:** compile-time API tests, direct library tests, and existing
  CLI/probe parity.
- **Runtime evidence:** unverified until library and CLI tests run.
- **Checkpoint:** `pause — inspect the final public boundary before broad extraction`.
- **Pause warrant:** All later units publish operands against this shape; later
  units change if it is wrong.

### U2. Document inspection reads

- **Goal:** Extract blocks, one-block reads, search, links, and statistics.
- **Requirements:** R1, R2, R7.
- **Dependencies:** U1.
- **Files:** Create `src/block.rs`, `src/search.rs`, `src/link.rs`,
  `src/stats.rs`; modify matching command modules and tests.
- **Approach:** Core owns block resolution, previews, search spans and Unicode
  provenance, link projection, word/section counts, and exhaustive block-kind
  policies. CLI owns paths, multi-file prefixes, and display envelopes.
- **Test scenarios:** all current fixtures plus Unicode lowercase expansion,
  code-block filters, UTF-8 spans, footnotes, HTML blocks, CRLF, empty source,
  and future-block-kind exhaustiveness.
- **Verification:** No semantic helper remains in the five command modules.
- **Proven through:** new direct-library suites and unchanged CLI suites.
- **Runtime evidence:** existing CLI behavior is proven; direct composition is
  unverified until this unit.
- **Checkpoint:** `auto — direct/core CLI differential read suite`.

### U3. Frontmatter reads, collection projection, and edits

- **Goal:** Extract per-document frontmatter semantics while keeping collect
  orchestration in the CLI.
- **Requirements:** R1, R2, R7, R9.
- **Dependencies:** U1.
- **Files:** Create `src/frontmatter.rs`; modify `commands/frontmatter.rs`,
  `commands/collect.rs`, `commands/set.rs`, errors/models/tests.
- **Approach:** Core owns absent/present state, strict read/mutation parsing,
  typed field paths, field projection, set/delete, YAML/TOML conversion, and
  candidate generation. CLI owns raw scalar grammar, paths, sorting, partial
  failures, TSV, and persistence.
- **Test scenarios:** absent versus present-empty etags, malformed/unclosed
  input, YAML/TOML objects, scalar conflicts, nested set/delete, body-byte
  preservation, guard-before-value parsing, and collect parity.
- **Verification:** `collect` calls core projection but remains multi-file CLI
  code; `set` emits a pure candidate before persistence.
- **Proven through:** direct frontmatter tests and existing read/set/collect
  suites.
- **Runtime evidence:** unverified until this unit.
- **Checkpoint:** `auto — frontmatter and collect parity suites`.

### U4. Table reads and row edits

- **Goal:** Extract table discovery, typed queries, and row mutations.
- **Requirements:** R1, R2, R7, R8.
- **Dependencies:** U1.
- **Files:** Create `src/table.rs`; modify `commands/table.rs`, parser helpers,
  errors/models/tests.
- **Approach:** `TableIndex` owns table resolution; typed column selectors and
  predicates replace raw filter strings after CLI refinement. Prepared row
  edits preserve guard-before-payload ordering.
- **Test scenarios:** multiple-table ambiguity, column names and indices,
  operators, escaped pipes, row bounds, LF/CRLF/EOF deletion ownership,
  invalid payload precedence, stale/ambiguous guards, and candidate parity.
- **Verification:** `commands/table.rs` contains only grammar, I/O, DTO mapping,
  and persistence.
- **Proven through:** direct table tests plus the existing table and contract
  suites.
- **Runtime evidence:** unverified until this unit.
- **Checkpoint:** `auto — table read/edit differential suite`.

### U5. Block edits and relocation

- **Goal:** Extract replace, insert, delete, and move-block semantics.
- **Requirements:** R1, R2, R7, R8.
- **Dependencies:** U1, U2.
- **Files:** Extend `src/block.rs` and `src/edit.rs`; reduce
  `commands/replace.rs` and `commands/move_block.rs`; add direct tests.
- **Approach:** Prepared block payload edits own guards and target resolution;
  core owns line endings, newline trimming, insertion separators, spans,
  permutation, positional gaps, and structural-closure validation.
- **Test scenarios:** empty payload rules, no-change inode behavior, mixed
  endings, indented code, Setext adjacency, duplicate etags, source-before-dest
  guards, gap preservation, invalid permutation, and exact candidates.
- **Verification:** Block command modules contain no candidate-building logic.
- **Proven through:** direct block-edit tests and existing write/move suites.
- **Runtime evidence:** unverified until this unit.
- **Checkpoint:** `auto — block mutation differential suite`.

### U6. Section edits and relocation

- **Goal:** Extract replace, delete, and move-section semantics last.
- **Requirements:** R1, R2, R7, R8.
- **Dependencies:** U1, U5.
- **Files:** Extend `src/section.rs`; create `src/section_move.rs` if separation
  clarifies the 700-line relocation algorithm; reduce section command modules;
  add direct tests.
- **Approach:** Core owns boundary newline floors, containment, typed placement,
  auto/keep leveling, ATX rewriting, Setext rejection, minimal separators,
  reparsing validation, and preservation evidence. CLI retains source-file versus
  stdin selection and v1 diagnostics.
- **Test scenarios:** the complete existing 2,272-line move suite plus direct
  core/CLI candidates, source/destination error roles, preamble rejection,
  containment, levels 1–6, Setext, LF/CRLF/mixed, and stale revision.
- **Verification:** Section command modules contain no candidate-building or
  relocation logic.
- **Proven through:** direct section-edit tests and unchanged section suites.
- **Runtime evidence:** unverified until this unit.
- **Checkpoint:** `pause — inspect the highest-risk relocation extraction`.
- **Pause warrant:** This is the largest semantic move and determines whether
  the full surface is ready to publish.

### U7. Completion parity and consumer proof

- **Goal:** Prove the library foundation is complete and narrow.
- **Requirements:** R1–R10.
- **Dependencies:** U2–U6.
- **Files:** Extend the core-consumer probe, `README.crate.md`, decision/status
  docs, and library examples; modify no consumer repository.
- **Approach:** Differentially compare every read and mutation family against
  baseline; build a clean minimal consumer that composes every public operation;
  grep command modules for surviving semantic helpers and direct writes.
- **Test scenarios:** full Rust suite, no-default library build, package closure,
  every CLI family, every candidate/final byte path, operational-failure
  distinction, and Braid-/Construal-shaped consumers.
- **Verification:** Every pure single-document operation has one library
  implementation; CLI owns only declared adapter responsibilities.
- **Proven through:** extended preregistered probe and package consumer runs.
- **Runtime evidence:** unverified until the final probe.
- **Checkpoint:** `pause — review completion verdict before Construal work`.
- **Pause warrant:** The evidence decides whether mdtools is finished or needs
  another foundation repair.

## Scope Boundaries

- No renderer.
- No Construal code or repository changes.
- No Braid source changes.
- No CLI v1 additions or removals.
- No `collect` path/multi-file extraction.
- No generic plugin, visitor, transaction, or batch framework.
- No parser upgrade.
- No performance rewrite.

## System-Wide Impact

- **Interaction graph:** every command becomes refine → parse `Document` → core
  operation → v1 mapping → optional guarded persistence.
- **Error propagation:** core errors carry typed domain/role context; CLI maps
  exhaustively to existing diagnostics.
- **State lifecycle:** pure candidates have no durable side effect; one adapter
  owns revision recheck and atomic replacement.
- **API parity:** semantic domain values remain distinct from CLI envelope and
  aggregate types.
- **Unchanged invariants:** exact source bytes, re-query pattern, target-etag
  ambiguity, parser boundary, and CLI v1 remain unchanged.

## Disconfirming Evidence

- Any CLI/core candidate mismatch stops the affected unit.
- Any operation that requires a path, stdout, exit code, or viewer concept in
  core falsifies the boundary.
- Any payload command that reads stdin before guard resolution falsifies parity.
- Any in-place command that bypasses the shared revision guard blocks completion.
- Any clean consumer dependency on Clap or Walkdir narrows the result.
- Any surviving semantic candidate builder in a command module means the
  foundation is incomplete.

## Requirement Cross-Check

| Requirement | Owning unit | Match? |
|---|---|---:|
| Complete single-document surface | U2–U6 | Yes |
| Preserve CLI | every unit + U7 | Yes |
| Safe direct selectors/snapshots | U1 | Yes |
| Pure candidate authority | U1, U3–U6 | Yes |
| Guard-before-payload | U3–U6 prepared edits | Yes |
| Shared revision persistence | U1 and every mutation adapter | Yes |
| Keep collect orchestration in CLI | U3 | Yes |
| Keep Construal outside | scope boundary | Yes |

## Build Execution Contract

- **Closed decisions:** One Cargo package; immutable `Document` for new
  operations; legacy `ParsedDocument` retained only for Braid compatibility;
  domain modules; generic core outcomes; prepared payload edits; no renderer.
- **Builder autonomy:** Exact internal module subdivision, helper visibility,
  and test fixture reuse are reversible choices.
- **Verify at contact:** Preserve each command's error precedence and newline
  rules from tests; compare actual code before moving; use current source when
  plan locators drift.
- **Stop conditions:** CLI wire change, second semantic implementation after
  cutover, filesystem authority in core, or a required Braid/Construal source
  modification.
- **Authority boundaries:** Do not push, publish, modify consumer repos, or
  create Construal code. Local temp consumers are allowed.
- **Expected gate map:** U1 foundation tests; U2 read suites; U3 frontmatter/
  collect; U4 table; U5 block; U6 section; U7 full suite/probe/package.
- **Pause warrants:** U1 public boundary, U6 relocation extraction, U7 final
  completion verdict.

## Risks & Dependencies

| Risk | Mitigation |
|---|---|
| Big-bang behavior drift | Domain-sized commits and differential parity per unit |
| Payload error precedence changes | Prepared edit APIs |
| Core/wire representations converge accidentally | Generic outcomes plus explicit mapping tests |
| Mutable legacy parser leaks into new APIs | New operations accept only `Document` |
| Braid compatibility constrains design | Retain low-level type; migrate separately |
| Move-section extraction hides a subtle boundary bug | Extract last and run full existing suite |
| Public API grows without evidence | U7 consumer proof and no renderer |
