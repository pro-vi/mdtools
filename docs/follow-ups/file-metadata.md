# Preserve ACL and hard-link topology during file commits

**Status:** deferred, non-blocking

## Problem

The verified file adapter preserves mode bits and, on macOS, extended
attributes. Rename-based replacement does not preserve POSIX ACL entries and
replaces one hard-link name with a new inode.

## Affected invariant

A commit should preserve filesystem metadata and link topology that callers
reasonably treat as part of the document.

## Why this is separate

ACL cloning needs platform-specific ACL APIs and failure semantics. Hard-link
preservation conflicts with atomic pathname replacement and needs an explicit
product choice between atomicity and in-place inode mutation.

## Acceptance tests

- A deny ACL remains byte-identical after commit.
- Both names of a hard-linked document continue to expose the committed bytes,
  or the public contract explicitly rejects multi-link targets before staging.

## Current containment

macOS Finder tags and Spotlight metadata are preserved as extended attributes.
Mode bits are preserved. The limitation is documented in the README.
