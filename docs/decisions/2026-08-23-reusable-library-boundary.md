# Reusable library boundary beneath the `md` CLI

**Date:** 2026-08-23
**Status:** accepted and implemented on `codex/reusable-library-foundation`
**Deciders:** Provi, Codex; informed by a sanitized ChatGPT Pro architecture review

## Context

The `mdtools` Cargo package already shipped a library target beside the `md`
binary, but only parser internals and CLI-shaped models/errors were reusable.
Command semantics such as section indexing, task discovery, and guarded task
edits remained binary-private. Braid proved direct parsing worked, then exposed
the incomplete boundary by reimplementing traversal. Its closed BRA-17 attempt
copied the whole repository into Braid and created a second source authority.

The future Markdown reader needs direct in-process structural operations, while
the public `md` process protocol must remain compatible and mdtools must stay
limited to Markdown primitives.

## Decision

Keep one repository and one Cargo package with its existing library and binary
targets.

- The library is the sole source-in/source-out Markdown semantic authority.
- The `md` binary is the compatibility, filesystem, and persistence adapter.
- Same-process consumers call library operations directly.
- External agents continue to use the versioned CLI protocol.
- Viewer, agent, transposition, approval, and UI concepts remain outside
  mdtools.
- Rendering remains outside the initial reusable surface until an executable
  viewer supplies a concrete security and source-mapping contract.

Every pure edit outcome carries the whole-document revision from which it was
derived. Target etags resolve one current target; the document revision guards
promotion of the complete candidate. Persistence owners verify that revision
immediately before replacement.

## Rationale

A second Cargo package would add a package name, release ordering, and another
SemVer surface without solving a demonstrated problem that the existing
lib/bin boundary cannot solve. The evidence supports a first-class library API,
not a separate lifecycle.

The first vertical extraction includes both reads and one mutation. This tests
the difficult candidate/persistence boundary before bulk-moving easier query
commands.

## Consequences

Positive:

- CLI and library semantics share one release and one implementation.
- Rust consumers avoid subprocess and JSON overhead.
- CLI-only dependencies can be disabled for library consumers.
- Consumer code cannot acquire filesystem authority through core edits.

Negative:

- The public library API now carries compatibility cost alongside the CLI.
- Core and CLI wire types require explicit conversion instead of convenient
  shared serialization.
- Remaining commands still need consumer-led extraction after this foundation.

## Revisit Triggers

- An active consumer needs a release cadence or dependency closure that one
  package cannot provide.
- A CLI/core differential test finds candidate or protocol drift.
- A viewer rendering slice establishes a safe fragment contract worth sharing.
- A future edit cannot express target and whole-document authority separately.

## Evidence

- [Core consumer topology protocol](../../probes/core_consumer_topology/PROTOCOL.md)
- [Core consumer topology results](../../probes/core_consumer_topology/RESULTS.md)
- [Positioning decision that gated library extraction on a real consumer](../plans/2026-06-03-md-frontier-positioning-decision.md)
- ChatGPT Pro review run `9b984022-0a46-48b0-a102-55de9592bd0f`
