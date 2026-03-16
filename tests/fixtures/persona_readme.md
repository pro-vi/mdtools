# mdtools

[![Build Status](https://img.shields.io/github/actions/workflow/status/user/mdtools/ci.yml)](https://github.com/user/mdtools/actions)
[![Coverage](https://img.shields.io/codecov/c/github/user/mdtools)](https://codecov.io/gh/user/mdtools)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> A markdown-aware CLI toolkit for LLM agents.

## Installation

```bash
cargo install mdtools
```

## Quick Start

1. Get the outline:
   ```bash
   md outline doc.md
   ```
2. Read a section:
   ```bash
   md section "Methods" doc.md
   ```
3. Replace content:
   ```bash
   echo "New text" | md replace-block 3 doc.md -i
   ```

## Features

- [x] Structural parsing
- [x] Source-preserving mutations
- [ ] Benchmark harness
- [ ] Plugin system

### Nested Feature List

- Block operations
  - Read blocks
  - Replace blocks
  - Insert blocks
    - Before/after
    - At start/end
  - Delete blocks
- Section operations
  - Read sections
  - Replace sections

## API

| Command | Input | Output |
|---------|-------|--------|
| `outline` | file | heading tree |
| `blocks` | file | block list |
| `search` | query, file | matches |

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [the docs](../docs/API.md) for more.
