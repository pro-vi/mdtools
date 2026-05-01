# `md move-section` — Design Spec v1.1

Status: design approved, implementation pending.
Frozen artifact: this file is the agreed surface; deviations require a respec.

## Motivation

`bench/ledger.md` F10-1: 3 distinct frontier-loop candidates (T6 baseline +
iter 18 C-T10-26 + iter 27 C-T10-34) hit the same compound subsection-
relocation gap on Qwen3.5-27B-4bit. Bench scorer shows `heading_tree: OK`
in isolation but `block_order` / `block_text: MISMATCH` because the body
paragraph gets orphaned in the source location while the heading lands at
the destination. The agent cannot reliably compose extract-section +
delete-section + insert-section without losing body adjacency or
forgetting to shift heading levels.

Anti-folklore lock keeps `md move-block` (generic block movement)
forbidden. `md move-section` is admitted as section-aware — heading +
body + level cascade as a unit — meaningfully different from arbitrary
block movement. Lock-lift authority: F10-1 escalation, Pro 2-lane review
0.83 confidence both.

## CLI surface

```
md move-section <SOURCE_HEADING> [FILE]
                (--after <DEST_HEADING> | --before <DEST_HEADING> | --into <DEST_HEADING>)
                (--auto-level | --keep-level)
                [--ignore-case]
                [--source-occurrence N] [--dest-occurrence N]
                [-i / --in-place]
                [--json]
```

- Exactly one of `--after` / `--before` / `--into` required.
- Source heading text positional; reuses existing `SectionSelector` from
  `find_section()`.
- `FILE` optional; omit for stdin.
- `--source-occurrence N` and `--dest-occurrence N` (1-indexed) for
  disambiguating multiple headings with the same text.

### Destination semantics

| Flag | Insert byte | Computed `new_top_level` |
|---|---|---|
| `--after X` | `X.section.byte_end` | `X.heading.level` (sibling, after X's children) |
| `--before X` | `X.section.byte_start` | `X.heading.level` (sibling, before X) |
| `--into X` | `X.section.byte_end` | `X.heading.level + 1` (last child of X) |

`level_delta = new_top_level − source.heading.level`. Apply uniformly to
the moved section's heading + every descendant heading, preserving
relative structure.

## Heading-level handling

### `--auto-level` (default)

| Scenario | Source top | Dest mode | Dest level | new_top | Delta | Children shift |
|---|---:|---|---:|---:|---:|---|
| Sibling, same level | 2 | `--after` | 2 | 2 | 0 | none |
| Sibling, promote | 3 | `--after` | 2 | 2 | −1 | all up by 1 |
| Sibling, demote | 2 | `--after` | 4 | 4 | +2 | all down by 2 |
| Child of shallow | 3 | `--into` | 1 | 2 | −1 | all up by 1 |
| Child of deep | 2 | `--into` | 4 | 5 | +3 | all down by 3 |
| Sibling-before, deep | 3 | `--before` | 2 | 2 | −1 | all up by 1 |

Worked example (sibling, promote):

```md
## Backend
### API                  ← source (level 3)
#### Auth                ← child (level 4)
## Frontend              ← --after target (level 2)
```

→ `--after Frontend`, delta = 2 − 3 = −1:

```md
## Backend
## Frontend
## API                   ← was h3, now h2
### Auth                 ← was h4, now h3
```

### `--keep-level`

`level_delta = 0` always; markers are byte-exact. Useful when source level
is intentional and destination is incidental.

### Clamp

Heading levels valid 1..=6. If any descendant would land out of range,
**error before mutation**:

```
cannot move-section: descendant 'Schema' would land at heading level 7
(max is 6); reduce destination depth or use --keep-level
```

## Setext handling

Setext headings (`Title\n=====` for h1, `Title\n-----` for h2) are
detectable but not byte-rewriteable:

- **Accept setext source iff `level_delta == 0`** — relocate bytes
  unchanged.
- **Reject otherwise** with: `setext heading 'X' (line N) cannot be
  re-leveled; convert to ATX (## X) first or use --keep-level`.

Detection: parse-side. Add `HeadingSourceKind { Atx, Setext }` to
`HeadingInfo` (parser boundary), expose downstream via `BlockInfo`.

## ATX indentation handling

CommonMark allows 0-3 leading spaces before `#`. Byte-rewrite scans from
the heading line start, skips up to 3 spaces, validates `#+ `, rewrites
the `#` count. Indentation preserved.

```md
   ## Indented Heading    ← 3 leading spaces, level 2
```

→ delta +1:

```md
   ### Indented Heading   ← indentation preserved, marker rewritten
```

Implementation requirement: extend `BlockInfo` with
`heading_marker_span: Option<SourceSpan>` (set by parser for ATX only;
None for setext). Auto-level rewrite uses this span instead of
`block.span.byte_start`. This is the parser-boundary discipline fix Pro
review flagged.

## Splice algorithm

Single sequence: **remove source → recompute insert byte → insert
relevelled bytes**.

```
src_span         = find_section(doc, source_selector).span
insert_byte_raw  = match dest_mode {
    AfterSibling | LastChild => find_section(doc, dest_selector).span.byte_end,
    BeforeSibling            => find_section(doc, dest_selector).span.byte_start,
}

// Validate ancestor/descendant
if dest fully contained in source: error("destination is inside source")
if source fully contained in dest AND mode is LastChild | AfterSibling:
    error("destination contains source; insert position is ambiguous")
    // BeforeSibling case is OK — insert before the parent boundary.

// Compute moved bytes
moved = doc.source[src_span.byte_start..src_span.byte_end].to_string()
if level_delta != 0 {
    moved = rewrite_atx_levels(moved, src_section, doc, level_delta)?
}

// Two splice sub-cases (handled uniformly via offset adjustment)
insert_byte = if insert_byte_raw > src_span.byte_end {
    insert_byte_raw - (src_span.byte_end - src_span.byte_start)
} else {
    insert_byte_raw
}

// Build output
output = doc.source[..min(src_span.byte_start, insert_byte)]
       + (intermediate slice depending on order)
       + moved
       + doc.source[max(src_span.byte_end, insert_byte_raw)..]
```

### Newline boundary handling

If the moved section did not end with `\n` (last section in a file with
no trailing newline) and the destination is followed by more content,
the splice must synthesize a `\n` between the moved bytes and the
following content. Detect via:

```
src_ends_with_newline = doc.source[src_span.byte_end - 1] == b'\n'
dest_followed_by_content = insert_byte_raw < doc.source.len()
if !src_ends_with_newline && dest_followed_by_content {
    moved.push('\n')
}
```

## MutationResult shape

`src/model.rs` additions:

```rust
// MutationCommandKind enum
MoveSection,

// MutationTargetRef enum
SectionMove(SectionMoveTargetRef),

#[derive(Serialize, Debug)]
pub struct SectionMoveTargetRef {
    pub source: SectionTargetRef,
    pub destination: SectionTargetRef,
    pub destination_mode: InsertMode,  // "after" | "before" | "into"
    pub level_shift_applied: i8,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum InsertMode { AfterSibling, BeforeSibling, IntoAsChild }
```

Standard envelope unchanged: `schema_version`, `file`, `command`,
`target`, `disposition`, `changed`, `line_endings`, `invariant`,
`content`.

## File touch list

| File | Action | Lines |
|---|---|---:|
| `src/cli.rs` | +1 enum variant + `MoveSectionArgs` struct | ~35 |
| `src/main.rs` | +1 dispatch arm | ~3 |
| `src/commands/mod.rs` | +1 `pub mod move_section;` | 1 |
| `src/commands/move_section.rs` | NEW | ~280 |
| `src/parser.rs` | Extend `HeadingInfo` with `marker_span` + `kind` | ~15 |
| `src/model.rs` | +1 enum variant + 1 struct + 1 enum (`InsertMode`) + extend `HeadingInfo` model | ~25 |
| `tests/cli_move_section.rs` | NEW | ~450 |
| `bench/command_policy.py` | +1 entry in `MUTATION_MD_COMMANDS` | 1 |
| `README.md` | +1 row in commands table | 2 |

Total: ~810 lines added, 1 file modified at parser boundary, single PR.

## Test plan

22 cases. Covers all core semantics + Pro-review-recommended edge cases.

### Core moves (5)
1. Sibling move same-level keep-level (`--after`, delta 0)
2. Sibling move auto-level promote (`--after`, delta −1)
3. Sibling move auto-level demote (`--after`, delta +2)
4. Child move auto-level (`--into`, delta varies)
5. Sibling-before move auto-level (`--before`, delta varies)

### Splice positions (4 — Pro 5.4-pro fixture)
6. Source before destination (forward splice)
7. Source after destination (backward splice)
8. Source inside destination (cycle: error)
9. Destination inside source (cycle: error for `--after`/`--into`; valid for `--before`)

### Boundary edge cases (5)
10. Last section in file
11. Last section in file with no trailing newline (Pro 5.5-pro golden)
12. Only top-level section in file
13. CRLF line endings preserved
14. Multibyte UTF-8 in heading text (`café`)

### Selectors (2)
15. Multiple same-text headings disambiguated via `--source-occurrence` / `--dest-occurrence`
16. Case-insensitive (`--ignore-case`)

### Heading representation (3)
17. Setext source with `--keep-level` (delta 0): allowed, byte-exact
18. Setext source with auto-level requiring shift: clean error
19. ATX with 0-3 space indentation: rewrite preserves indentation

### Errors (2)
20. Level clamp: descendant would exceed h6 → clean error
21. Conflicting destination flags: e.g. both `--after` and `--into` → clap error

### Output modes (1)
22. `--in-place` mutates file; `--json` emits `MutationResult` envelope; default prints to stdout. Single test exercises all three modes against same fixture.

### T6 fixture replay (smoke)
Plus a non-asserted run against `bench/inputs/t6_*.md` to prove the
command handles real-world fixture complexity without panic. Not a
correctness test — T6's actual task isn't a section move.

## Build sequence (single PR, ordered for incremental confidence)

1. **Parser metadata extension** — add `HeadingSourceKind` enum + `marker_span: Option<SourceSpan>` to `HeadingInfo`. Update `parser.rs` to populate. Update existing 282-test suite as needed (likely no callers depend on `HeadingInfo` exactly). `cargo test` green.
2. **Model + CLI scaffolding** — enum variants, args struct, dispatch arm, module file with `unimplemented!()`. `cargo build` clean.
3. **Source resolution + validation** (read-only) — implement source resolve, dest resolve, cycle validation, setext + clamp checks. Returns dummy `MutationResult`. Tests 8, 9, 18, 20, 21.
4. **Splice without level rewrite** (`--keep-level` path) — implement splice algorithm including newline boundary. Tests 1, 6, 7, 10, 11, 12, 13, 14, 17, 22.
5. **Auto-level rewrite** — implement `rewrite_atx_levels`. Tests 2, 3, 4, 5, 15, 16, 19.
6. **Final integration** — add `move-section` to `bench/command_policy.py`, README row, run full `cargo test` + `bench/harness.py --md-binary target/release/md` dry-run on all 24 corpus tasks. T6 fixture smoke. No regressions expected.

Estimated implementation: 4-6 hours straight-through.

## Out of scope (v2+ candidates)

- `--at-start-of <X>` (insert as first child, before existing children).
- `--target-level N` explicit level override.
- Loc-based selectors (`--source-loc 9.2`, `--dest-loc 14`).
- Setext source with auto-level (silent setext→ATX normalization).
- Multi-section batch move.
- Cross-file move (source in one file, dest in another).

Each requires its own evidence (e.g. another F-finding or explicit user
demand) before admission. The lock-lift granted by F10-1 covers the
single section-aware primitive above; not a generalized move-block IR.

## References

- F10-1 finding: `bench/ledger.md` (OPEN findings)
- T6 baseline failure: `bench/inputs/t6_*.md` + `bench/expected/t6_*.md`
- Iter 18 candidate: `bench/runs/t10-26-certificate-rotation-runbook-relocation-*-2026-04-29/`
- Iter 27 candidate: `bench/runs/t10-34-pager-rotation-review-relocation-*-2026-04-30/`
- Pro 2-lane review: `/tmp/move-section-design-review.md` + agentify keys `mdtools-move-section-gpt55-pro` and `mdtools-move-section-gpt54-pro`
- Anti-folklore lock context: `specs/frontier-loop.md` § Anti-folklore lock
