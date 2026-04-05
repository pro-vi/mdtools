# mdtools — Project Instructions

Structural markdown CLI for AI agents. Binary: `md`. Rust + comrak.

## Architecture

**Parser boundary:** All comrak interaction is in `src/parser.rs`. The rest of the codebase sees only model types (`src/model.rs`), never comrak types. This is the core invariant.

**Command pattern:**
- Read commands: parse → extract → emit JSON or TSV
- Mutation commands: parse → find target → read content via `output::read_content(args.from.as_deref())` → splice at byte offsets → emit `MutationResult`
- Multifile: `multifile::resolve_paths()` → `for_each_file()` with error aggregation

**Key files:**
- `src/cli.rs` — all command args (clap derive)
- `src/model.rs` — all types, schema version `mdtools.v1`
- `src/parser.rs` — comrak boundary, `ParsedDocument`, `BlockInfo`, `TaskItemInfo`
- `src/output.rs` — JSON/text output, `read_content()` for `--from` flag
- `src/commands/` — one module per command group

## Design rules

- **Markdown primitives only.** If it's in the GFM spec or comrak AST, it's our domain. Task IDs (`0.1`), phase headings, metadata patterns — consumer's job via `jq`.
- **No identity tracking in locs.** Loc is a structural dot-path (`9.0`, `14.4.0`). No versioning, no hashes. Identity-bearing refs are a cross-cutting concern for a future `--expect-etag` on all mutation commands.
- **Re-query pattern is the moat.** Agents query `md tasks --json`, mutate, re-query for fresh locs. Design new commands to support this cycle. Locs must be cheap to re-derive.
- **`--from` for all mutations.** `replace-section`, `replace-block`, `insert-block` accept `--from PATH` (or stdin). Agents write temp files instead of shell-escaping heredocs.
- **Hybrid > pure.** Agents perform best with both `md` and unix tools. Don't try to replace `sed` for simple edits.

## Comrak pin

`comrak = "=0.51.0"` — exact pin. `set-task` does 1-byte replacement at `symbol_byte_offset` from `NodeTaskItem.symbol_sourcepos`. Comrak changed sourcepos behavior across 0.48-0.51. When upgrading: re-run full test suite, verify CRLF/frontmatter/multibyte fixtures.

Parser options: `relaxed_tasklist_matching: false`, `tasklist_in_table: false` (strict GFM).

## Known limitations

- `search --ignore-case` spans break on Unicode case expansion (Turkish dotted I)
- `section --ignore-case` is ASCII-only (`eq_ignore_ascii_case`)
- T6 (complex multi-edit) fails in all modes — agent planning limitation, not tool gap

## Task loc format

`BLOCK.CHILD[.CHILD...]` — e.g., `9.0` (block 9, child 0), `14.4.0` (nested grandchild). For blockquote tasks, sibling lists get a list-counter prefix: `0.0.0` vs `0.1.0`.

## Build & test

```bash
cargo build --release
cargo test                           # 282 integration tests
cargo test --test cli_tasks          # task-specific tests
python bench/harness.py --md-binary target/release/md  # validate 20 benchmark scorers
```

## Benchmark

20 tasks across 3 modes (unix/mdtools/hybrid). Dual scoring: md binary + markdown-it-py.

```bash
# Run single task
python bench/harness.py --run --mode hybrid --md-binary target/release/md --task T10

# Run with different model
python bench/harness.py --run --mode hybrid --md-binary target/release/md --model claude-haiku-4-5-20251001

# Full matrix
for MODE in unix mdtools hybrid; do
  python bench/harness.py --run --mode $MODE --md-binary target/release/md --model claude-haiku-4-5-20251001
done

# Analyze
python bench/analyze.py /tmp/bench_*.txt
python bench/report.py /tmp/bench_*.txt --markdown
```

Headline: Haiku unix 50% → hybrid 87% (+37pp). Sonnet: +5pp correctness, 3-5x speed. Opus: efficiency only. Tool benefit inversely proportional to model capability.

## Task families

| Family | Tasks | mdtools advantage |
|--------|-------|-------------------|
| Extraction | T1,T5,T9,T11,T16,T19 | Strong — structural query vs multi-pass grep |
| Targeted mutation | T7,T10,T13,T20 | Moderate — loc addressing vs line numbers |
| Batch mutation | T12 | Strong — md set-task in a loop |
| Multi-step | T15,T18 | Strong — re-query pattern handles drift |
| Content delivery | T2,T3,T8,T17 | Moderate — --from avoids shell escaping |
| Safe-fail | T14 | Strong — md tasks detects ambiguity |
| Text manipulation | T4,T6 | Weak — unix wins simple sed/awk |

## Next steps (from improvement plan)

1. Ship to fract-ai oracle loop (original consumer)
2. Instrument real deployment (track tool choice, re-query rate)
3. T6 is the roadmap signal — transactional multi-edit gap, not a bug to fix
4. `md batch` is NOT on the roadmap (Pro review: prove planning vs execution gap first)
5. Table `--where` shipped. Next table features: row mutations, then `md collect` (vault-as-table)
