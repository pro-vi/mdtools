# Semantic section fragments own normalization and placement boundaries

**Date:** 2026-08-30
**Status:** accepted 2026-08-30
**Deciders:** Provi, Codex

## Context

Semantic section operations need one portable representation, while literal
operations must preserve caller bytes. The implementation plan fixed that
split but left mixed line endings, final newlines, boundary conflicts, preamble
identity, and retained move behavior to implementation.

## Decision

- Accept any valid single-root semantic section syntax and normalize it to one
  relative ATX subtree. Setext soft breaks become spaces and hard breaks become
  inline `<br />` elements so the canonical heading remains one ATX line.
- Treat only spaces and tabs as blank-line content. Unicode whitespace remains
  Markdown content and cannot be discarded as owned boundary space.
- Render semantic fragments with the destination line-ending style. A mixed
  destination uses LF for new semantic bytes because it has no single style.
- Semantic changed replacements own their outer whitespace and end without a
  final newline at document end. The semantic `NoChange` path preserves the
  original bytes, including setext syntax and a final newline.
- Literal section and preamble operations emit the supplied non-empty bytes
  without rebasing, trimming, separator insertion, or line-ending conversion.
  Deletion remains a separate operation.
- Last-child insertion points conflict with any operation whose claimed span
  touches that point. Context-dependent failures, such as an unclosed fence
  absorbing an inserted heading, remain final structural-closure errors.
- `PreambleIdentity` carries only the document revision because its address is
  the constant preamble address.
- `src/fragment.rs` owns heading rebasing. `MoveSection` calls that authority
  and rejects setext releveling unless the caller keeps the original levels.

## Consequences

- Semantic callers do not count heading markers, boundary newlines, or source
  heading depth.
- Literal callers are responsible for any separators their exact bytes need.
- Some edit pairs that could be serialized byte-wise remain conservatively
  conflicting at a shared structural boundary.

## Revisit Triggers

- A mixed-line-ending consumer requires local-line-style placement rather than
  LF for new semantic bytes.
- A concrete batch workflow needs composition across a shared section boundary.
- The retained move surface survives U7 and needs semantic setext releveling.

## References

- `src/fragment.rs`
- `src/patch/planner.rs`
- `src/section_edit.rs`
- `tests/semantic_fragments.rs`
- `docs/plans/2026-08-27-001-refactor-unified-target-architecture-plan.md`
