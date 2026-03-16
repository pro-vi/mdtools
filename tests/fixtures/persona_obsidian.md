---
tags:
  - source/book
  - topic/cli
aliases:
  - CLI Tools
  - Command Line
created: 2024-01-15T10:00:00
---

# CLI Design Principles

The key insight is that agents need structural awareness. ^key-insight

## Wikilinks and References

See [[Other Note]] for background. Also check [[Design Doc|the design document]].

This connects to [[Architecture#Heading Three]] for the details.

## Callouts

> [!WARNING] Breaking Change
> The old API is deprecated. Use the new method instead.
> This affects all downstream consumers.

> [!NOTE]
> Regular callouts are just blockquotes with special syntax.

## Block References

This paragraph has a block reference at the end. ^ref-block-1

Another paragraph with an embed reference: ![[attachment.png]]

## Task Tracking

- [x] Implement parser boundary
- [x] Add source span projection
- [ ] Write integration tests
  - [x] Read commands
  - [ ] Write commands
  - [ ] Search commands
- [ ] Benchmark harness
