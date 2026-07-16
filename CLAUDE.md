# mdtools — Project Instructions

Structural markdown CLI for AI agents. Binary: `md`. Rust + comrak.

> ⚠️ **PUBLIC REPOSITORY** (`github.com/pro-vi/mdtools`). Never commit: secrets / API
> keys, **other projects' internals** (tickets, architecture, IP — e.g. fract-ai),
> agent/session transcripts or dumps (`agent_output.txt`, `*.pi-audit.jsonl`,
> `guard.log`), or personal usage telemetry. The private research/benchmarking loop
> lives in gitignored **`.loop/`**; machine-specific workflow goes in gitignored
> **`CLAUDE.local.md`**. When unsure if something is public-safe, keep it out — a leak
> in git history needs a force-push rewrite to undo.

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
- **Loc carries no identity; etag fingerprints content.** Loc is a structural dot-path (`9.0`, `14.4.0`) — no versioning in the loc itself. For drift-safety, `md blocks`/`md block`, `md section --json`, `md table --json`, and `md tasks --json` expose a target `etag` (FNV-1a exact-byte content fingerprint), and `replace-block`/`delete-block`/`insert-block --before|--after`, `replace-section`/`delete-section`, `replace-table-row`/`delete-table-row`, and `set-task` accept `--expect-etag <hash>` to fail-closed (exit 4, `EtagMismatch`) when the target's current fingerprint differs. This guards the read→mutate path against target-content drift, so the safe pattern is still read, mutate, then re-query before the next mutation.
- **Re-query pattern is the moat.** Agents query `md tasks --json`, mutate, re-query for fresh locs. Design new commands to support this cycle. Locs must be cheap to re-derive.
- **Payload-bearing vs payload-free mutations.** `replace-section`, `replace-block`, `replace-table-row`, and `insert-block` accept `--from PATH` (or stdin). Agents write temp files instead of shell-escaping heredocs. `delete-table-row` is intentionally payload-free: selector only, no stdin or `--from`. `replace-block` and `replace-table-row` strip one trailing line-ending from the content (matching the newline-excluded target-span convention) so the trailing `\n` that `cat`/editors/`echo` universally append doesn't inject a spurious blank line; the strip is skipped for blocks whose span includes a trailing newline (indented code), while table-row replacement preserves the row's existing line ending by keeping it outside the replaced span.
- **Hybrid > pure.** Agents perform best with both `md` and unix tools. Don't try to replace `sed` for simple edits.

## Comrak pin

`comrak = "=0.51.0"` — exact pin. `set-task` does 1-byte replacement at `symbol_byte_offset` from `NodeTaskItem.symbol_sourcepos`. Comrak changed sourcepos behavior across 0.48-0.51. When upgrading: re-run full test suite, verify CRLF/frontmatter/multibyte fixtures.

Parser options: `relaxed_tasklist_matching: false`, `tasklist_in_table: false` (strict GFM).

## Known limitations

- `search --ignore-case` spans break on Unicode case expansion (Turkish dotted I)
- `section --ignore-case` is ASCII-only (`eq_ignore_ascii_case`)
- T6 (complex multi-edit) fails in all modes — agent planning limitation, not tool gap
- `--expect-etag` is **content-addressed, not identity-addressed**: it verifies the
  selected block, section, table, or task item still has the expected content fingerprint from
  the earlier read when a later command invocation mutates it. In a doc with
  **duplicate-content blocks**, an intervening edit can shift indices so the old index
  lands on a *different* same-content block whose fingerprint also matches — the guard
  passes against the wrong target. Mitigation today: **re-query immediately before
  mutating** (the moat) to shrink the window. A proper fix (binding the expectation to
  positional identity / span, or failing closed on hash ambiguity) is a design decision
  that trades against "loc carries no identity" — deferred as follow-up.
- **Bench ablation integrity (maintain this).** The no-md / `native*` baselines must keep
  `md` unreachable by *every* form — the `./md` workdir copy, bare `md` on PATH, and the
  `BASH_ENV` the claude-cli shell never sources. This recurred as a bypass **5× across axes**
  (2026-06) before being closed by one `MD_REAL_MODES` predicate (new modes default to
  md-excluded) **plus** a fail-closed `_assert_no_md_reachable` preflight that proves
  unreachability before any run (`bench/harness.py`). **Never add a no-md/ablation mode without
  routing it through that predicate + preflight** — a new axis silently contaminates the
  attribution data, and contaminated data outlives the code fix (a re-measure is part of the
  fix, not optional). **This generalizes from modes to RUNNERS:** enabling a new native-capable
  runner (2026-06-13: `pi-json`) requires verifying the preflight probes *that runner's actual
  PATH* — pi prepends `~/.pi/agent/bin` (`getShellEnv`), invisible to the harness `child_env`, so
  `_assert_no_md_reachable` gained `extra_path_dirs` to probe it. A new runner with its own
  PATH-prepend needs the same, or a real `md` there is unreachable to the proof yet reachable to
  the agent. (Decision record: `docs/decisions/2026-06-13-pi-json-native-arm.md`.)

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

**Do not cite pre-v3 benchmark headlines.** The old Haiku/Sonnet/Opus numbers are
retracted. Current v3 result: `md` shows large directional weak-model lifts with clean
no-md ablations, but the preregistered broad headline failed certification. The paired
lift estimates are +28.3pp (Haiku shell), +27.5pp (Haiku native), and +30.8pp
(GPT-5.4-mini native); their 95% CI lower bounds are +10.0pp, +9.2pp, and +10.8pp,
below the frozen +15pp floor. Treat the broad result as directional/exploratory.

Supported claim: `md` helps weak/tool-poor agents read and target Markdown structure
more reliably, mainly reducing wrong-target, format, and quoting failures. Not
supported: a certified broad benchmark headline, a frontier-native-tool edge, a
>10k-line document edge, or a proven agent advantage from `--expect-etag`. Sources:
`bench/RESULTS.md`, `bench/V3.md`, and
`docs/decisions/2026-07-04-md-positioning-after-probes.md`.

Default corpus: 28 tasks in `bench/tasks/tasks.json`; v3 headline aggregation uses the
24 core tasks and reports adversarially mined tasks separately. Dual scoring:
v3-neutral primary plus diagnostic scorer.

```bash
# Run single task
python bench/harness.py --run --runner oai-loop --mode hybrid \
  --md-binary target/release/md --oai-api-base http://localhost:10240/v1 \
  --oai-api-key $OMLX_API_KEY --task T10

# Full matrix — one invocation PER mode. `--mode` is single-valued (action="store"):
# repeated `--mode A --mode B` flags silently keep only the LAST. Loop to run a matrix.
for MODE in unix mdtools hybrid; do
  python bench/harness.py --run --runner oai-loop --mode $MODE \
    --md-binary target/release/md --oai-api-base http://localhost:10240/v1 \
    --oai-api-key $OMLX_API_KEY --model Qwen3.5-27B-4bit
done

# Analyze
python bench/analyze.py /tmp/bench_*.txt
python bench/report.py /tmp/bench_*.txt --markdown
```

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
5. Table row mutations shipped (`replace-table-row`, `delete-table-row`). `md collect` now covers the narrow vault-as-table read path; keep follow-on work out of mutation/query-engine territory.
