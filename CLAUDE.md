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
- **Loc carries no identity; etag does.** Loc is a structural dot-path (`9.0`, `14.4.0`) — no versioning in the loc itself. For drift-safety, `md blocks`/`md block` JSON carry a per-block `etag` (FNV-1a content fingerprint of the block's bytes), and `replace-block`/`delete-block`/`insert-block --before|--after` accept `--expect-etag <hash>` to fail-closed (exit 4, `EtagMismatch`) when the target block's current fingerprint differs — making the read→mutate→re-query loop safe. Section/task mutations don't carry etag yet (the remaining slice of the "etag on all mutation commands" cross-cutting concern).
- **Re-query pattern is the moat.** Agents query `md tasks --json`, mutate, re-query for fresh locs. Design new commands to support this cycle. Locs must be cheap to re-derive.
- **`--from` for all mutations.** `replace-section`, `replace-block`, `insert-block` accept `--from PATH` (or stdin). Agents write temp files instead of shell-escaping heredocs. `replace-block` strips one trailing line-ending from the content (matching the newline-excluded block-span convention) so the trailing `\n` that `cat`/editors/`echo` universally append doesn't inject a spurious blank line; the strip is skipped for blocks whose span includes a trailing newline (indented code).
- **Hybrid > pure.** Agents perform best with both `md` and unix tools. Don't try to replace `sed` for simple edits.

## Comrak pin

`comrak = "=0.51.0"` — exact pin. `set-task` does 1-byte replacement at `symbol_byte_offset` from `NodeTaskItem.symbol_sourcepos`. Comrak changed sourcepos behavior across 0.48-0.51. When upgrading: re-run full test suite, verify CRLF/frontmatter/multibyte fixtures.

Parser options: `relaxed_tasklist_matching: false`, `tasklist_in_table: false` (strict GFM).

## Known limitations

- `search --ignore-case` spans break on Unicode case expansion (Turkish dotted I)
- `section --ignore-case` is ASCII-only (`eq_ignore_ascii_case`)
- T6 (complex multi-edit) fails in all modes — agent planning limitation, not tool gap
- `--expect-etag` is **content-addressed, not identity-addressed**: it verifies the
  block at the given index still has the expected content fingerprint. In a doc with
  **duplicate-content blocks**, an intervening edit can shift indices so the old index
  lands on a *different* same-content block whose fingerprint also matches — the guard
  passes against the wrong target. Mitigation today: **re-query immediately before
  mutating** (the moat) to shrink the window. A proper fix (binding the expectation to
  positional identity / span, or failing closed on hash ambiguity) is a design decision
  that trades against "loc carries no identity" — deferred as follow-up.

## Task loc format

`BLOCK.CHILD[.CHILD...]` — e.g., `9.0` (block 9, child 0), `14.4.0` (nested grandchild). For blockquote tasks, sibling lists get a list-counter prefix: `0.0.0` vs `0.1.0`.

## Build & test

```bash
cargo build --release
cargo test                           # 337 integration tests
cargo test --test cli_tasks          # task-specific tests
python bench/harness.py --md-binary target/release/md  # validate 20 benchmark scorers
```

## Benchmark

20 tasks across 3 modes (unix/mdtools/hybrid). Dual scoring: md binary + markdown-it-py.

```bash
# Run single task
python bench/harness.py --run --runner oai-loop --mode hybrid \
  --md-binary target/release/md --oai-api-base http://localhost:10240/v1 \
  --oai-api-key $OMLX_API_KEY --task T10

# Full matrix (oai-loop, primary model)
for MODE in unix mdtools hybrid; do
  python bench/harness.py --run --runner oai-loop --mode $MODE \
    --md-binary target/release/md --oai-api-base http://localhost:10240/v1 \
    --oai-api-key $OMLX_API_KEY --model Qwen3.5-27B-4bit
done

# Analyze
python bench/analyze.py /tmp/bench_*.txt
python bench/report.py /tmp/bench_*.txt --markdown
```

Headline: Haiku unix 50% → hybrid 87% (+37pp). Sonnet: +5pp correctness, 3-5x speed. Opus: efficiency only. Tool benefit inversely proportional to model capability.

## Auto-research (candidate pipeline)

`bench/auto_research.py` runs the full candidate pipeline in one command:
generator (mdtools-blind) → realism review → harness measurement (3 modes) →
unix-adversary review → manifest assembly.

```bash
# Generate + measure a new candidate (uses primary model by default)
python bench/auto_research.py \
  --md-binary target/release/md \
  --api-base http://localhost:10240/v1 \
  --api-key $OMLX_API_KEY

# Dry-run (skip harness measurement — just generator + reviews)
python bench/auto_research.py \
  --md-binary target/release/md \
  --api-base http://localhost:10240/v1 \
  --api-key $OMLX_API_KEY \
  --skip-measure

# Use a specific generator model (e.g. gemma for speed, Qwen for quality)
python bench/auto_research.py \
  --md-binary target/release/md \
  --api-base http://localhost:10240/v1 \
  --api-key $OMLX_API_KEY \
  --model gemma-4-e4b-it-8bit
```

Outputs land in `bench/search/candidates/<slug>/`. Status values:
- `pending-cross-seed` — gap exists, AST-structural, ready for N=3 promotion gate
- `rejected-hybrid-fail-no-gap` — both modes failed; generator made task too hard
- `rejected-both-pass-no-gap` — no gap; unix solved it too
- `rejected-planning` — realism review said no
- `rejected-<gap-label>` — gap exists but unix adversary labeled it non-structural

OAI endpoint: `http://localhost:10240/v1`, API key in `~/.omlx/settings.json`.

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

1. Ship to the original consumer (oracle-loop integration)
2. Instrument real deployment (track tool choice, re-query rate)
3. T6 is the roadmap signal — transactional multi-edit gap, not a bug to fix
4. `md batch` is NOT on the roadmap (Pro review: prove planning vs execution gap first)
5. Table `--where` shipped. Next table features: row mutations, then `md collect` (vault-as-table)
