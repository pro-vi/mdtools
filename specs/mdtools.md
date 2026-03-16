# mdtools Spec

## Product Thesis [id:sec-thesis]

`mdtools` is a markdown-aware CLI toolkit for LLM agents. Phase 1 optimizes for two properties at once: structural awareness over Markdown documents and token-efficient terminal ergonomics. The product bet is that a native CLI with typed `--json` envelopes can beat both raw Unix text tools, which are semantically blind, and verbose function-call APIs, which encourage high-token tool chatter on long documents.

## Scope [id:sec-scope]

Phase 1 defines the command surface, document model, mutation contracts, and edge-case behavior for a single-binary CLI named `md`. Phase 2 defines a benchmark harness that compares `mdtools`, raw Unix tools, and function-call tools under a shared evaluation contract.

## Files [id:sec-files]

| Path | Purpose |
|---|---|
| `specs/mdtools.md` | Product and interface spec for Phase 1 and benchmark contracts |
| `Cargo.toml` | Rust package manifest for the `md` binary |
| `src/main.rs` | CLI entrypoint and process exit mapping |
| `src/cli.rs` | `clap` argument definitions for all subcommands |
| `src/model.rs` | Shared document and result types exposed by command modules |
| `src/parser.rs` | Markdown parse boundary and source-span extraction surface |
| `src/commands/outline.rs` | `md outline` contract implementation |
| `src/commands/blocks.rs` | `md blocks` and `md block` contract implementation |
| `src/commands/section.rs` | `md section` and `md replace-section` contract implementation |
| `src/commands/replace.rs` | `md replace-block`, `md insert-block`, and `md delete-block` contract implementation |
| `src/commands/search.rs` | `md search` contract implementation |
| `src/commands/links.rs` | `md links` contract implementation |
| `src/commands/frontmatter.rs` | `md frontmatter` contract implementation |
| `src/commands/stats.rs` | `md stats` contract implementation |
| `src/output.rs` | Text and JSON output adapters |
| `src/errors.rs` | Error and diagnostic types |
| `tests/fixtures/` | Markdown fixtures and golden outputs |
| `tests/cli_read.rs` | Integration tests for read-only commands |
| `tests/cli_write.rs` | Integration tests for replacement commands |
| `tests/cli_search.rs` | Integration tests for `md search` filters and match envelopes |
| `bench/tasks/` | Benchmark task definitions and expected outputs |
| `bench/harness.py` | Phase 2 benchmark runner and scorer |

## Decisions [id:sec-decisions]

| ID | Decision | Chosen Contract | Rationale |
|---|---|---|---|
| [id:dec-language] | Phase 1 implementation language | Rust | Native single binary, low startup latency, predictable distribution for agent loops |
| [id:dec-parser] | Markdown parser boundary | `comrak` with source positions enabled and `MarkdownFeatureSet` extensions | Supports CommonMark plus the enumerated Phase 1 extensions while exposing source spans |
| [id:dec-binary-name] | Executable name | `md` | Minimal command length for repeated agent use |
| [id:dec-block-model] | Block identity in Phase 1 | Top-level document blocks only; frontmatter excluded from block indexing | Keeps commands stable and avoids hidden synthetic indices |
| [id:dec-section-selector] | Section addressing | Plaintext heading text plus optional `--occurrence`; reserved selector `:preamble` | Resolves duplicate heading ambiguity without inventing opaque IDs |
| [id:dec-mutation-contract] | Mutation behavior | Source-preserving replacement contract; renderer rewrites are out of scope | Trust depends on untouched bytes staying untouched |
| [id:dec-in-place] | In-place write flag | `--in-place` / `-i` on mutation commands writes the result back to the input file instead of stdout | Agent-first UX: avoids temp-file or `sponge` plumbing; restores benchmark parity with function-call mode |
| [id:dec-surface-alignment] | Phase 1 command surface | Include `insert-block`, `delete-block`, and `search` alongside the original read and replace commands | Brings the CLI surface closer to the benchmark tasks without introducing stronger primitives than the thesis allows |
| [id:dec-search-scope] | `search` semantics | Content search scoped to block bodies with optional block-kind filters | Makes task T4 expressible while keeping `search` weaker than direct structured mutation |
| [id:dec-line-endings] | Replacement line endings | Normalize inserted content to the document line-ending style when style is uniform; preserve payload bytes when style is mixed | Avoids unnecessary churn while handling legacy files |
| [id:dec-json-envelopes] | `--json` payload shape | Named result envelopes, not raw vectors or bare strings | Stable schemas are easier to validate and benchmark |
| [id:dec-frontmatter-output] | `frontmatter` stdout contract | Always emits JSON envelope in both text and `--json` modes | Frontmatter is metadata-first, and JSON is the useful representation |
| [id:dec-exit-codes] | Exit code surface | `0` success, `1` not found, `2` parse error, `3` invalid input, `4` conflict | Duplicate headings and invalid replace payloads need distinct handling |
| [id:dec-bench-fairness] | Benchmark normalization | Same model family, same system prompt template, same temperature, same max turns, mode-specific tools only | Benchmark claims are not defensible without fairness constraints |

## Common Contracts [id:sec-contracts-common]

```rust
pub const SCHEMA_VERSION: &str = "mdtools.v1"; // [id:contract-schema-version]

pub enum OutputFormat { // [id:contract-output-format]
    Text,
    Json,
}

pub enum MdExitCode { // [id:contract-exit-code]
    Success = 0,
    NotFound = 1,
    ParseError = 2,
    InvalidInput = 3,
    Conflict = 4,
}

pub enum HeadingMatchMode { // [id:contract-heading-match-mode]
    Exact,
    ExactIgnoreCase,
}

pub enum SearchMatchMode { // [id:contract-search-match-mode]
    Literal,
    LiteralIgnoreCase,
}

pub enum SectionSelectorKind { // [id:contract-section-selector-kind]
    HeadingText,
    Preamble,
}

pub enum LineEndingStyle { // [id:contract-line-ending-style]
    Lf,
    Crlf,
    Mixed,
}

pub struct SourceSpan { // [id:contract-source-span]
    pub line_start: u32,
    pub line_end: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}
// SourceSpan coordinate convention: [id:rule-source-span-coordinates]
// - line_start and line_end are 1-based and inclusive.
// - byte_start and byte_end are 0-based and half-open: [byte_start, byte_end).
// - A zero-width span (e.g. an insertion point) has byte_start == byte_end.
// - Block indices are 0-based throughout the CLI and JSON contracts.

pub struct SectionSelector { // [id:contract-section-selector]
    pub kind: SectionSelectorKind,
    pub heading_text: Option<String>,
    pub occurrence: Option<u32>,
    pub match_mode: HeadingMatchMode,
}

pub enum InsertLocation { // [id:contract-insert-location]
    Before(u32),
    After(u32),
    Start,
    End,
}

pub enum ErrorKind { // [id:contract-error-kind]
    Io,
    Parse,
    NotFound,
    InvalidInput,
    Conflict,
}

pub struct Diagnostic { // [id:contract-diagnostic]
    pub code: &'static str,
    pub message: String,
}

pub struct CommandError { // [id:contract-command-error]
    pub kind: ErrorKind,
    pub exit_code: MdExitCode,
    pub diagnostic: Diagnostic,
}
```

## Parser Feature Contract [id:sec-parser-features]

```rust
pub struct MarkdownFeatureSet { // [id:contract-markdown-feature-set]
    pub table: bool,
    pub strikethrough: bool,
    pub autolink: bool,
    pub tasklist: bool,
    pub footnotes: bool,
    pub front_matter_delimiters: &'static [&'static str],
    pub header_ids: bool,
}
```

Phase 1 enables the following comrak extensions: [id:rule-parser-enabled-extensions]
- `table` — required by `BlockKind::Table`.
- `strikethrough` — inline formatting; does not affect block structure.
- `autolink` — required by `LinkKind::Autolink`.
- `tasklist` — inline formatting inside list items; does not affect block structure.
- `footnotes` — required by `BlockKind::FootnoteDefinition`.
- `front_matter_delimiters` — set to `&["---", "+++"]`; the parser attempts each delimiter in order. `---` detects YAML frontmatter, `+++` detects TOML frontmatter. Both are part of the comrak parse boundary and do not require a separate pre-parse layer. [id:rule-frontmatter-dual-delimiter]
- `header_ids` — disabled in Phase 1; slug generation is an output concern, not a parse concern.

Phase 1 does NOT enable: [id:rule-parser-disabled-extensions]
- `description_lists` — no corresponding `BlockKind`; description list syntax is parsed as plain paragraphs.
- `multiline_block_quotes` — no corresponding block kind; `>>>` syntax is parsed as a regular `BlockQuote`.
- `math_dollars`, `math_code` — no corresponding block or inline kind; `$` and `$$` are treated as literal text.
- `wikilinks_title_after_pipe`, `wikilinks_title_before_pipe` — `LinkKind::Wiki` is reserved in the read model but wiki link detection is deferred to Phase 2. Documents containing `[[...]]` syntax will not produce `Wiki` link entries in Phase 1.

Unsupported syntax fallback: any markdown syntax that requires a disabled extension is parsed as its CommonMark fallback (typically `Paragraph` or inline literal text). The CLI does not error on documents containing unsupported syntax. [id:rule-parser-unsupported-fallback]

## Read Model Contracts [id:sec-contracts-read]

```rust
pub enum BlockKind { // [id:contract-block-kind]
    Heading,
    Paragraph,
    List,
    BlockQuote,
    CodeFence,
    IndentedCode,
    ThematicBreak,
    Table,
    HtmlBlock,
    FootnoteDefinition,
}

// CLI token values for BlockKind (kebab-case). [id:contract-block-kind-cli-tokens]
// The `--kind` flag on `md search` and any future commands accepting BlockKind
// values MUST use exactly these kebab-case tokens:
//   heading, paragraph, list, block-quote, code-fence, indented-code,
//   thematic-break, table, html-block, footnote-definition.
// Matching is case-insensitive on input but canonical output always uses
// the lowercase kebab-case form above.
// JSON output uses PascalCase variant names (e.g. "CodeFence").

pub struct HeadingRef { // [id:contract-heading-ref]
    pub level: u8,
    pub text: String,
    pub block_index: u32,
    pub span: SourceSpan,
}

pub struct OutlineEntry { // [id:contract-outline-entry]
    pub heading: HeadingRef,
    pub section_span: SourceSpan,
}

pub struct OutlineResult { // [id:contract-outline-result]
    pub schema_version: &'static str,
    pub file: String,
    pub entries: Vec<OutlineEntry>,
}

pub struct BlockEntry { // [id:contract-block-entry]
    pub index: u32,
    pub kind: BlockKind,
    pub span: SourceSpan,
    pub preview: String,
}

pub struct BlocksResult { // [id:contract-blocks-result]
    pub schema_version: &'static str,
    pub file: String,
    pub blocks: Vec<BlockEntry>,
}

pub struct BlockReadResult { // [id:contract-block-read-result]
    pub schema_version: &'static str,
    pub file: String,
    pub block: BlockEntry,
    pub content: String,
}

pub struct RawFileReadResult { // [id:contract-raw-file-read-result]
    pub schema_version: &'static str,
    pub file: String,
    pub content: String,
}

pub enum SectionKind { // [id:contract-section-kind]
    Preamble,
    Heading,
}

pub struct SectionEntry { // [id:contract-section-entry]
    pub kind: SectionKind,
    pub heading: Option<HeadingRef>,
    pub selector: SectionSelector,
    pub depth: u8,
    pub block_indices: Vec<u32>,
    pub span: SourceSpan,
}

pub struct SectionReadResult { // [id:contract-section-read-result]
    pub schema_version: &'static str,
    pub file: String,
    pub section: SectionEntry,
    pub content: String,
}

pub enum LinkKind { // [id:contract-link-kind]
    Inline,
    Reference,
    Autolink,
    Wiki,
}

pub struct LinkEntry { // [id:contract-link-entry]
    pub kind: LinkKind,
    pub text: String,
    pub destination: Option<String>,
    pub title: Option<String>,
    pub source_block_index: u32,
    pub span: SourceSpan,
}

pub struct LinksResult { // [id:contract-links-result]
    pub schema_version: &'static str,
    pub file: String,
    pub links: Vec<LinkEntry>,
}

pub struct SearchMatch { // [id:contract-search-match]
    pub block_index: u32,
    pub block_kind: BlockKind,
    pub match_span: SourceSpan,
    pub preview: String,
}

pub struct SearchResult { // [id:contract-search-result]
    pub schema_version: &'static str,
    pub file: String,
    pub query: String,
    pub match_mode: SearchMatchMode,
    pub block_kinds: Vec<BlockKind>,
    pub matches: Vec<SearchMatch>,
}

pub enum FrontmatterFormat { // [id:contract-frontmatter-format]
    Yaml,
    Toml,
}

pub struct FrontmatterEnvelope { // [id:contract-frontmatter-envelope]
    pub format: FrontmatterFormat,
    pub span: SourceSpan,
    pub raw: String,
    pub data: serde_json::Value,
}

pub struct FrontmatterReadResult { // [id:contract-frontmatter-read-result]
    pub schema_version: &'static str,
    pub file: String,
    pub present: bool,
    pub frontmatter: Option<FrontmatterEnvelope>,
}

pub struct DocumentStats { // [id:contract-document-stats]
    pub word_count: u32,
    pub heading_count: u32,
    pub block_count: u32,
    pub link_count: u32,
    pub section_count: u32,
    pub line_count: u32,
}

pub struct StatsResult { // [id:contract-stats-result]
    pub schema_version: &'static str,
    pub file: String,
    pub stats: DocumentStats,
}
```

## Mutation Contracts [id:sec-contracts-write]

```rust
pub enum MutationTargetKind { // [id:contract-mutation-target-kind]
    Block,
    Section,
    InsertLocation,
}

pub enum MutationCommandKind { // [id:contract-mutation-command-kind]
    ReplaceBlock,
    ReplaceSection,
    InsertBlock,
    DeleteBlock,
}

pub enum MutationDisposition { // [id:contract-mutation-disposition]
    NoChange,
    Replaced,
    Inserted,
    Deleted,
}

pub struct BlockTargetRef { // [id:contract-block-target-ref]
    pub kind: MutationTargetKind,
    pub block_index: u32,
    pub span: SourceSpan,
}

pub struct SectionTargetRef { // [id:contract-section-target-ref]
    pub kind: MutationTargetKind,
    pub selector: SectionSelector,
    pub section: SectionEntry,
}

pub struct InsertTargetRef { // [id:contract-insert-target-ref]
    pub kind: MutationTargetKind,
    pub location: InsertLocation,
    pub anchor_span: Option<SourceSpan>,
}

pub enum MutationTargetRef { // [id:contract-mutation-target-ref]
    Block(BlockTargetRef),
    Section(SectionTargetRef),
    Insert(InsertTargetRef),
}

pub struct SourcePreservationInvariant { // [id:contract-source-preservation]
    pub preserves_non_target_bytes: bool,
    pub target_span_before: Option<SourceSpan>,
    pub target_span_after: Option<SourceSpan>,
}
// Span nullability tied to MutationDisposition: [id:rule-span-nullability]
// - Inserted  => target_span_before = None,         target_span_after = Some(inserted span)
// - Deleted   => target_span_before = Some(deleted span), target_span_after = None
// - Replaced  => target_span_before = Some(old span),     target_span_after = Some(new span)
// - NoChange  => target_span_before = Some(span),         target_span_after = Some(span) (identical)

pub struct MutationResult { // [id:contract-mutation-result]
    pub schema_version: &'static str,
    pub file: String,
    pub command: MutationCommandKind,
    pub target: MutationTargetRef,
    pub disposition: MutationDisposition,
    pub changed: bool,
    pub line_endings: LineEndingStyle,
    pub invariant: SourcePreservationInvariant,
    pub content: Option<String>,
}

pub type ReplaceResult = MutationResult; // [id:contract-replace-result]
```

The mutation contract is defined by interface, not by algorithm. The required invariants are:

- `replace-block`, `replace-section`, `insert-block`, and `delete-block` emit the full updated document to stdout on success; when `--in-place` is set on a mutation command, the output is written back to the input file and stdout is silent in text mode or emits `MutationResult` in `--json` mode. [id:rule-replace-stdout]
- `MutationResult.invariant.preserves_non_target_bytes` is `true` for every successful mutation command. [id:rule-replace-preserve-bytes]
- `MutationResult.content` is `Some(updated_document)` when the successful mutation contract emits document bytes to stdout and `None` when the successful mutation contract writes the file in place; function-call mutation tools follow the in-place form and therefore return `content == None`. [id:rule-mutation-result-content]
- `--in-place` with `--json` returns `MutationResult` with `content == None`; `--in-place` without `--json` keeps stdout silent after the file write succeeds. [id:rule-in-place-content]
- Empty stdin is valid for `replace-block`, `replace-section`, and `insert-block`; empty stdin on replace commands yields `MutationDisposition::Deleted` when the selected target span becomes empty. [id:rule-replace-empty-stdin]
- Replacement content may change block count, heading count, section count, and link count. [id:rule-replace-structure-change]
- `delete-block` removes exactly the target block's byte span as defined by its `SourceSpan`. Inter-block whitespace outside the span is preserved verbatim; if deletion produces consecutive blank lines, those blank lines are kept as-is and not collapsed. [id:rule-delete-whitespace]
- When the source document line endings are uniformly `LF` or uniformly `CRLF`, inserted content is normalized to that style before output; when the source style is `Mixed`, inserted content is preserved as provided. [id:rule-replace-line-endings]

## CLI Contracts [id:sec-cli]

```rust
#[derive(clap::Parser)] // [id:cli-root]
#[command(name = "md", about = "Markdown-aware CLI for agent operations")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)] // [id:cli-command]
pub enum Command {
    Outline(OutlineArgs),
    Section(SectionArgs),
    Blocks(BlocksArgs),
    Block(BlockArgs),
    ReplaceSection(ReplaceSectionArgs),
    ReplaceBlock(ReplaceBlockArgs),
    InsertBlock(InsertBlockArgs),
    DeleteBlock(DeleteBlockArgs),
    Search(SearchArgs),
    Links(LinksArgs),
    Frontmatter(FrontmatterArgs),
    Stats(StatsArgs),
}

pub struct OutlineArgs { // [id:cli-outline]
    pub file: std::path::PathBuf,
}

pub struct SectionArgs { // [id:cli-section]
    #[arg(value_name = "SELECTOR")]
    pub selector: String,
    pub file: std::path::PathBuf,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "occurrence")]
    pub occurrence: Option<u32>,
}

pub struct BlocksArgs { // [id:cli-blocks]
    pub file: std::path::PathBuf,
}

pub struct BlockArgs { // [id:cli-block]
    pub index: u32,
    pub file: std::path::PathBuf,
}

pub struct ReplaceSectionArgs { // [id:cli-replace-section]
    #[arg(value_name = "SELECTOR")]
    pub selector: String,
    pub file: std::path::PathBuf,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "occurrence")]
    pub occurrence: Option<u32>,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

pub struct ReplaceBlockArgs { // [id:cli-replace-block]
    pub index: u32,
    pub file: std::path::PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

pub struct InsertBlockArgs { // [id:cli-insert-block]
    #[arg(long = "before", value_name = "INDEX")]
    pub before: Option<u32>,
    #[arg(long = "after", value_name = "INDEX")]
    pub after: Option<u32>,
    #[arg(long = "at-start")]
    pub at_start: bool,
    #[arg(long = "at-end")]
    pub at_end: bool,
    pub file: std::path::PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}
// Exactly one of --before, --after, --at-start, --at-end must be provided;
// the selected flag maps to InsertLocation as defined in rule-cli-insert-location-map.
// Supplying zero or more than one is InvalidInput (exit 3). [id:rule-cli-insert-location-exclusivity]

pub struct DeleteBlockArgs { // [id:cli-delete-block]
    pub index: u32,
    pub file: std::path::PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

pub struct SearchArgs { // [id:cli-search]
    pub query: String,
    pub file: std::path::PathBuf,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "kind")]
    pub kinds: Vec<BlockKind>,
}

pub struct LinksArgs { // [id:cli-links]
    pub file: std::path::PathBuf,
}

pub struct FrontmatterArgs { // [id:cli-frontmatter]
    pub file: std::path::PathBuf,
}

pub struct StatsArgs { // [id:cli-stats]
    pub file: std::path::PathBuf,
}
```

CLI I/O rules:

- All commands require a positional file path; Phase 1 does not define whole-document stdin input. [id:rule-cli-file-input]
- `replace-block`, `replace-section`, and `insert-block` additionally read replacement content from stdin. [id:rule-cli-stdin-replace]
- Successful command output is written to stdout only. [id:rule-cli-stdout]
- Errors are written to stderr as exactly one human-readable line derived from `Diagnostic.message`. [id:rule-cli-stderr]
- `frontmatter` emits `FrontmatterReadResult` JSON in both default and `--json` modes. [id:rule-cli-frontmatter-json]
- `md replace-block`, `md replace-section`, `md insert-block`, and `md delete-block` accept `--in-place` / `-i` to write the successful mutation result back to the input path. [id:rule-cli-in-place]
- `md section <SELECTOR> <FILE> --ignore-case --occurrence <N>` maps directly to `SectionSelector`; `SELECTOR=:preamble` maps to `SectionSelectorKind::Preamble` and ignores `--occurrence`. [id:rule-cli-section-selector-map]
- `md replace-section <SELECTOR> <FILE> --ignore-case --occurrence <N>` uses the same selector mapping as `md section`. [id:rule-cli-replace-section-selector-map]
- `md insert-block --before <INDEX> <FILE>`, `md insert-block --after <INDEX> <FILE>`, `md insert-block --at-start <FILE>`, and `md insert-block --at-end <FILE>` map to `InsertLocation::Before`, `InsertLocation::After`, `InsertLocation::Start`, and `InsertLocation::End` respectively. [id:rule-cli-insert-location-map]
- `md search <QUERY> <FILE> --kind <BLOCK_KIND>... --ignore-case` performs content search over the selected top-level block kinds; omitting `--kind` searches every Phase 1 block kind. `<BLOCK_KIND>` values use the kebab-case CLI tokens defined under `contract-block-kind-cli-tokens` (e.g. `--kind code-fence --kind paragraph`). [id:rule-cli-search-map]

## Text and JSON Output Contracts [id:sec-output]

The default stdout contract is optimized for shell composition. `--json` switches to the named result envelope shown in the contracts above.

| Command | Text stdout contract | JSON stdout contract |
|---|---|---|
| `md outline` | Canonical text grammar (see below) | `OutlineResult` |
| `md section` | Raw section bytes | `SectionReadResult` |
| `md blocks` | Canonical text grammar (see below) | `BlocksResult` |
| `md block` | Raw block bytes | `BlockReadResult` |
| `md replace-section` | Full updated document bytes | `MutationResult` |
| `md replace-block` | Full updated document bytes | `MutationResult` |
| `md insert-block` | Full updated document bytes | `MutationResult` |
| `md delete-block` | Full updated document bytes | `MutationResult` |
| `md search` | Canonical text grammar (see below) | `SearchResult` |
| `md links` | Canonical text grammar (see below) | `LinksResult` |
| `md frontmatter` | `FrontmatterReadResult` JSON | `FrontmatterReadResult` |
| `md stats` | Canonical text grammar (see below) | `StatsResult` |

Canonical text output grammars: [id:rule-text-output-grammars]

These grammars are normative. All fields are separated by a single tab character. Lines are terminated by the platform line ending (`\n` on unix). Conforming implementations must produce output matching these patterns exactly.

Canonical text escaping rule: [id:rule-text-field-escaping]
User-derived text fields (`heading_text`, `preview`, `destination`, `query` echo) must be sanitized before emission in text mode. Tab characters (`\t`) are replaced with a single space. Newline (`\n`) and carriage return (`\r`) characters are replaced with a single space. No other escaping is applied. This ensures tab-separated field boundaries are unambiguous and each record occupies exactly one line.

- `md outline` text format: one line per heading. [id:text-grammar-outline]
  `{depth_marker} {heading_text}\t{line_start}-{line_end}\tblock:{block_index}`
  where `depth_marker` is `#` repeated `level` times (e.g. `## Introduction\t5-12\tblock:1`).

- `md blocks` text format: one line per block. [id:text-grammar-blocks]
  `{index}\t{kind}\t{line_start}-{line_end}\t{preview}`
  where `kind` is the PascalCase `BlockKind` variant name and `preview` is truncated to 80 characters with `...` appended if truncated (e.g. `0\tHeading\t1-1\t# Introduction`).

- `md search` text format: one line per match. [id:text-grammar-search]
  `{block_index}\t{kind}\t{line_start}-{line_end}\t{preview}`
  where `kind` and `preview` follow the same conventions as `md blocks`.

- `md links` text format: one line per link. [id:text-grammar-links]
  `{kind}\t{destination}\tblock:{source_block_index}\t{line_start}-{line_end}`
  where `kind` is the PascalCase `LinkKind` variant name and `destination` is the raw URL or empty string if absent.

- `md stats` text format: one line per stat. [id:text-grammar-stats]
  `{key}={value}` where keys are: `words`, `headings`, `blocks`, `links`, `sections`, `lines`.

Example JSON payloads:

```json
{
  "schema_version": "mdtools.v1",
  "file": "doc.md",
  "entries": [
    {
      "heading": {
        "level": 1,
        "text": "Intro",
        "block_index": 0,
        "span": { "line_start": 1, "line_end": 1, "byte_start": 0, "byte_end": 7 }
      },
      "section_span": { "line_start": 1, "line_end": 8, "byte_start": 0, "byte_end": 128 }
    }
  ]
}
``` [id:json-example-outline]

```json
{
  "schema_version": "mdtools.v1",
  "file": "doc.md",
  "blocks": [
    {
      "index": 0,
      "kind": "Heading",
      "span": { "line_start": 1, "line_end": 1, "byte_start": 0, "byte_end": 7 },
      "preview": "Intro"
    }
  ]
}
``` [id:json-example-blocks]

```json
{
  "schema_version": "mdtools.v1",
  "file": "doc.md",
  "query": "method",
  "match_mode": "Literal",
  "block_kinds": ["Paragraph", "List", "BlockQuote"],
  "matches": [
    {
      "block_index": 4,
      "block_kind": "Paragraph",
      "match_span": { "line_start": 15, "line_end": 15, "byte_start": 260, "byte_end": 266 },
      "preview": "This method keeps the body text intact."
    }
  ]
}
``` [id:json-example-search]

```json
{
  "schema_version": "mdtools.v1",
  "file": "doc.md",
  "command": "ReplaceBlock",
  "target": {
    "Block": {
      "kind": "Block",
      "block_index": 4,
      "span": { "line_start": 15, "line_end": 15, "byte_start": 252, "byte_end": 291 }
    }
  },
  "disposition": "Replaced",
  "changed": true,
  "line_endings": "Lf",
  "invariant": {
    "preserves_non_target_bytes": true,
    "target_span_before": { "line_start": 15, "line_end": 15, "byte_start": 252, "byte_end": 291 },
    "target_span_after": { "line_start": 15, "line_end": 15, "byte_start": 252, "byte_end": 293 }
  },
  "content": "# Intro\n\nUpdated paragraph.\n"
}
``` [id:json-example-mutation]

```json
{
  "schema_version": "mdtools.v1",
  "file": "doc.md",
  "links": [
    {
      "kind": "Inline",
      "text": "repo",
      "destination": "https://example.com",
      "title": null,
      "source_block_index": 3,
      "span": { "line_start": 12, "line_end": 12, "byte_start": 210, "byte_end": 238 }
    }
  ]
}
``` [id:json-example-links]

## Document Semantics [id:sec-semantics]

Document model rules:

- A Phase 1 block is a top-level Markdown block node at the document root after frontmatter extraction; nested blocks inside lists, block quotes, tables, or footnotes do not receive independent top-level block indices. [id:sem-block-root]
- Frontmatter is metadata, not a top-level block, and never shifts `BlockEntry.index` values. [id:sem-frontmatter-excluded]
- `SourceSpan.line_start` and `SourceSpan.line_end` are 1-based inclusive line coordinates; `SourceSpan.byte_start` is a 0-based inclusive byte offset and `SourceSpan.byte_end` is a 0-based exclusive byte offset. [id:sem-source-span-coordinates]
- `BlockEntry.index`, `HeadingRef.block_index`, and `LinkEntry.source_block_index` are zero-based in document order; `SectionSelector.occurrence` is one-based in match order. [id:sem-index-coordinates]
- The preamble is the contiguous root-level content after frontmatter and before the first top-level heading. It is addressable as `SectionSelector { kind: Preamble, ... }` and via the reserved CLI selector `:preamble`. [id:sem-preamble]
- For the preamble section, `SectionEntry.kind == SectionKind::Preamble`, `SectionEntry.heading == None`, `SectionEntry.depth == 0`, and `SectionEntry.block_indices` lists every root-level Phase 1 block before the first top-level heading. [id:sem-preamble-section-entry]
- For a headed section, `SectionEntry.kind == SectionKind::Heading`, `SectionEntry.depth == heading.level` (e.g. an `## H2` section has `depth == 2`), and `SectionEntry.block_indices` always includes the heading block itself as its first element followed by all subsequent root-level blocks that belong to the section. [id:sem-headed-section-entry]
- A section begins at a top-level heading block and ends immediately before the next top-level heading whose level is less than or equal to the current heading level. [id:sem-section-boundary]
- Heading text matching uses the plaintext rendering of heading content. ATX and setext headings are equivalent under this matching contract. [id:sem-heading-plaintext]
- Headings that appear inside block quotes, lists, tables, footnotes, or code blocks do not create top-level sections. [id:sem-nested-heading-exclusion]
- Duplicate heading text is a conflict unless `--occurrence` is supplied; `--occurrence 1` selects the first matching heading in document order. [id:sem-duplicate-headings]
- The selector string `:preamble` is reserved for addressing the preamble section. A heading whose plaintext is literally `:preamble` cannot be selected by text match in Phase 1; this collision is documented as an accepted limitation. [id:sem-preamble-reserved]
- `search` matches content spans inside block bodies, not structural node names; heading text participates only when `BlockKind::Heading` is included in the filter set. [id:sem-search-content]
- `SearchMatch.match_span` identifies the exact matched content occurrence inside the document source, not the containing block span. [id:sem-search-match-span]
- `preview` is a single-line human-readable summary and is not a stable identifier. [id:sem-preview-nonidentity]
- `DocumentStats` counting rules: `word_count` counts whitespace-delimited tokens in the plaintext rendering of all block bodies (headings, paragraphs, list items, and table cells contribute; code fence contents, HTML block contents, and frontmatter do not). `heading_count` counts top-level headings only (matching `OutlineResult.entries.len()`). `block_count` counts top-level Phase 1 blocks (matching `BlocksResult.entries.len()`). `link_count` counts all links (matching `LinksResult.entries.len()`). `section_count` counts all sections including the preamble when it is non-empty; an empty preamble (zero blocks before the first heading) does not contribute to `section_count`. `line_count` counts newline characters in the raw source plus one (i.e. the last line counts even without a trailing newline). [id:sem-stats-counting-rules]

## Error Model [id:sec-errors]

```rust
pub enum DiagnosticCode { // [id:contract-diagnostic-code]
    IoOpenFailed,
    ParseFailed,
    FrontmatterParseFailed,
    HeadingNotFound,
    BlockIndexOutOfRange,
    DuplicateHeadingMatch,
    InvalidSelector,
    InvalidUtf8OnStdin,
}
```

Error mapping rules:

- `HeadingNotFound` and `BlockIndexOutOfRange` map to `MdExitCode::NotFound`. [id:err-map-not-found]
- `ParseFailed` maps to `MdExitCode::ParseError`. [id:err-map-parse]
- `FrontmatterParseFailed` maps to `MdExitCode::ParseError`. [id:err-map-frontmatter-parse]
- `InvalidSelector` and `InvalidUtf8OnStdin` map to `MdExitCode::InvalidInput`. [id:err-map-invalid]
- `DuplicateHeadingMatch` maps to `MdExitCode::Conflict`. [id:err-map-conflict]
- Stderr uses `Diagnostic.message` only; machine-readable codes belong to JSON wrappers and tests, not stderr formatting. [id:err-stderr-format]

## Edge Cases [id:sec-edge-cases]

- Empty document: `outline`, `blocks`, and `links` return empty result sets; `stats` returns zero counts for `word_count`, `heading_count`, `block_count`, `link_count`, and `section_count`, but `line_count` is `1` (zero newlines plus one, per `[id:sem-stats-counting-rules]`); `section :preamble` returns an empty section; any heading selector returns `NotFound`. [id:edge-empty-document]
- Frontmatter-only document: `frontmatter.present == true`, `block_count == 0`, and `section :preamble` returns empty content after the frontmatter span. [id:edge-frontmatter-only]
- Fenced code blocks containing heading syntax or links: inner text never creates headings, sections, or links. [id:edge-fenced-code]
- Setext headings: represented as `BlockKind::Heading` and matched exactly like ATX headings. [id:edge-setext-heading]
- CRLF documents: source spans are byte-accurate over CRLF bytes; replacement output follows the replacement line-ending contract defined above. [id:edge-crlf]
- HTML blocks: represented as `BlockKind::HtmlBlock`; embedded headings are not section delimiters in Phase 1. [id:edge-html-block]
- Indented code blocks: represented as `BlockKind::IndentedCode`; internal markdown syntax is ignored structurally. [id:edge-indented-code]
- `search` over `CodeFence` or `IndentedCode` is valid and returns content matches inside those blocks when those kinds are explicitly requested. [id:edge-search-code-blocks]
- Malformed frontmatter: when a document begins with `---` or `+++` delimiters but the enclosed content is not valid YAML or TOML, the `md frontmatter` command exits with `FrontmatterParseFailed` and `MdExitCode::ParseError`. All other commands (outline, blocks, section, search, links, stats, and mutation commands) fall back to treating the malformed frontmatter block as plain markdown content — the delimiters and body become one or more top-level blocks (typically `ThematicBreak` and `Paragraph`), frontmatter is reported as not present, and the command proceeds without error. [id:edge-malformed-frontmatter]
- Unclosed frontmatter: when a document begins with `---` or `+++` but no matching closing delimiter is found, the entire document is treated as plain markdown content; frontmatter is reported as not present. This applies to all commands including `md frontmatter`, which returns `FrontmatterReadResult { present: false, frontmatter: None }` rather than erroring. [id:edge-unclosed-frontmatter]
- `insert-block --at-start` on a document with frontmatter inserts after the frontmatter span and before the first Phase 1 block. [id:edge-insert-after-frontmatter]
- Last-block replacement or deletion: valid for both block and section mutations; the resulting document trailing newline is determined by the replacement payload after line-ending normalization. [id:edge-last-block]

## Phase 2 Benchmark Contracts [id:sec-benchmark]

```python
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

BenchMode = Literal["unix", "mdtools", "function_calls"]  # [id:bench-mode]

BenchExpectedArtifact = Literal["stdout_text", "file_contents", "json_envelope"]  # [id:bench-expected-artifact]
# stdout_text:    correctness is scored against the agent's final stdout capture.
# file_contents:  correctness is scored against the on-disk file state after the agent finishes.
# json_envelope:  correctness is scored against a JSON result parsed from the agent's final stdout.

ScorerKind = Literal["structural", "normalized_text", "raw_bytes"]  # [id:bench-scorer-kind]
# structural:       compare using StructuralDiffPolicy fields only (heading tree, block order, etc.).
# normalized_text:  compare block-level textual content after normalizing line endings and trimming trailing whitespace per block. Catches body-text mutations that structural comparison would miss.
# raw_bytes:        exact byte equality between expected and actual artifact. No normalization.

@dataclass
class StructuralDiffPolicy:  # [id:bench-structural-diff-policy]
    kind: ScorerKind
    normalize_line_endings: bool
    ignore_trailing_whitespace: bool
    compare_frontmatter_json: bool
    compare_heading_tree: bool
    compare_block_order: bool
    compare_link_destinations: bool
    compare_block_text: bool

@dataclass
class BenchTask:  # [id:bench-task]
    id: str
    description: str
    input_files: list[Path]
    expected_output: Path
    expected_artifact: BenchExpectedArtifact
    difficulty: Literal["basic", "intermediate", "advanced"]
    scorer: StructuralDiffPolicy

@dataclass
class ToolInventory:  # [id:bench-tool-inventory]
    mode: BenchMode
    tools: list[str]

@dataclass
class FunctionToolSpec:  # [id:bench-function-tool-spec]
    name: str
    input_schema: dict
    result_ref: str

@dataclass
class BenchRunConfig:  # [id:bench-run-config]
    model_id: str
    system_prompt_template: str
    temperature: float
    max_turns: int
    inventories: list[ToolInventory]
    function_tools: list[FunctionToolSpec]

@dataclass
class BenchResult:  # [id:bench-result]
    task_id: str
    mode: BenchMode
    correct: bool
    token_input: int
    token_output: int
    tool_calls: int
    turns: int
    elapsed_seconds: float
    conversation_log: Path
    structural_diff_report: Path
```

Mode inventory contracts:

- `unix` mode inventory is exactly: `cat`, `grep`, `sed`, `awk`, `head`, `tail`, `wc`, `tee`, `mv`, `cp`. The agent shell environment supports standard POSIX redirection operators (`>`, `>>`, `<`, `|`) and temp-file creation via `mktemp`. This enables `file_contents` tasks: agents may use `sed` or shell redirection to write modified files in place. [id:bench-unix-inventory]
- `mdtools` mode inventory is exactly the Phase 1 CLI commands in this spec. [id:bench-mdtools-inventory]
- `mdtools` mode inventory additionally includes `cat` for raw file reading parity with `unix` mode. [id:bench-mdtools-cat]
- `function_calls` mode inventory must be semantically aligned to the Phase 1 CLI surface and may not expose stronger document primitives than `mdtools`. [id:bench-function-parity]
- `function_calls` mode inventory additionally includes `read_file` for raw file reading parity with `unix` mode `cat` and `mdtools` mode `cat`. [id:bench-function-read-file]
- Function-call `replace_section`, `replace_block`, `insert_block`, and `delete_block` tools write the file atomically and return `MutationResult`; this matches the CLI `--in-place` behavior, not the stdout-emit behavior. [id:bench-function-replace-write]

Function-call tool schemas:

```json
{
  "name": "read_file",
  "inputSchema": {
    "type": "object",
    "properties": { "file": { "type": "string" } },
    "required": ["file"]
  },
  "resultRef": "RawFileReadResult"
}
``` [id:bench-tool-read-file]

```json
{
  "name": "get_outline",
  "inputSchema": {
    "type": "object",
    "properties": { "file": { "type": "string" } },
    "required": ["file"]
  },
  "resultRef": "OutlineResult"
}
``` [id:bench-tool-get-outline]

```json
{
  "name": "list_blocks",
  "inputSchema": {
    "type": "object",
    "properties": { "file": { "type": "string" } },
    "required": ["file"]
  },
  "resultRef": "BlocksResult"
}
``` [id:bench-tool-list-blocks]

```json
{
  "name": "get_block",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "index": { "type": "integer", "minimum": 0 }
    },
    "required": ["file", "index"]
  },
  "resultRef": "BlockReadResult"
}
``` [id:bench-tool-get-block]

```json
{
  "name": "get_section",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "selector": { "type": "string" },
      "ignore_case": { "type": "boolean" },
      "occurrence": { "type": "integer", "minimum": 1 }
    },
    "required": ["file", "selector"]
  },
  "resultRef": "SectionReadResult"
}
``` [id:bench-tool-get-section]

```json
{
  "name": "replace_section",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "selector": { "type": "string" },
      "ignore_case": { "type": "boolean" },
      "occurrence": { "type": "integer", "minimum": 1 },
      "content": { "type": "string" }
    },
    "required": ["file", "selector", "content"]
  },
  "resultRef": "MutationResult"
}
``` [id:bench-tool-replace-section]

```json
{
  "name": "replace_block",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "index": { "type": "integer", "minimum": 0 },
      "content": { "type": "string" }
    },
    "required": ["file", "index", "content"]
  },
  "resultRef": "MutationResult"
}
``` [id:bench-tool-replace-block]

```json
{
  "name": "insert_block",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "location": {
        "oneOf": [
          {
            "type": "object",
            "properties": {
              "kind": { "const": "Before" },
              "index": { "type": "integer", "minimum": 0 }
            },
            "required": ["kind", "index"]
          },
          {
            "type": "object",
            "properties": {
              "kind": { "const": "After" },
              "index": { "type": "integer", "minimum": 0 }
            },
            "required": ["kind", "index"]
          },
          {
            "type": "object",
            "properties": {
              "kind": { "const": "Start" }
            },
            "required": ["kind"]
          },
          {
            "type": "object",
            "properties": {
              "kind": { "const": "End" }
            },
            "required": ["kind"]
          }
        ]
      },
      "content": { "type": "string" }
    },
    "required": ["file", "location", "content"]
  },
  "resultRef": "MutationResult"
}
``` [id:bench-tool-insert-block]

```json
{
  "name": "delete_block",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "index": { "type": "integer", "minimum": 0 }
    },
    "required": ["file", "index"]
  },
  "resultRef": "MutationResult"
}
``` [id:bench-tool-delete-block]

```json
{
  "name": "search_content",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" },
      "query": { "type": "string" },
      "ignore_case": { "type": "boolean" },
      "kinds": {
        "type": "array",
        "items": {
          "type": "string",
          "enum": [
            "Heading",
            "Paragraph",
            "List",
            "BlockQuote",
            "CodeFence",
            "IndentedCode",
            "ThematicBreak",
            "Table",
            "HtmlBlock",
            "FootnoteDefinition"
          ]
        }
      }
    },
    "required": ["file", "query"]
  },
  "resultRef": "SearchResult"
}
``` [id:bench-tool-search-content]

```json
{
  "name": "list_links",
  "inputSchema": {
    "type": "object",
    "properties": { "file": { "type": "string" } },
    "required": ["file"]
  },
  "resultRef": "LinksResult"
}
``` [id:bench-tool-list-links]

```json
{
  "name": "get_frontmatter",
  "inputSchema": {
    "type": "object",
    "properties": { "file": { "type": "string" } },
    "required": ["file"]
  },
  "resultRef": "FrontmatterReadResult"
}
``` [id:bench-tool-get-frontmatter]

```json
{
  "name": "get_stats",
  "inputSchema": {
    "type": "object",
    "properties": { "file": { "type": "string" } },
    "required": ["file"]
  },
  "resultRef": "StatsResult"
}
``` [id:bench-tool-get-stats]

Benchmark validation rules:

- `BenchRunConfig.model_id`, `system_prompt_template`, `temperature`, and `max_turns` must be identical across all modes in a benchmark run. [id:bench-fairness-config]
- Correctness is evaluated by the task-specific `StructuralDiffPolicy`. The `kind` field selects the primary comparison mode: `structural` checks only the enabled structural fields, `normalized_text` additionally compares block-level textual content (catching body-text mutations), and `raw_bytes` requires exact byte equality. Tasks that involve body-text edits (e.g. T4) must use `normalized_text` or `raw_bytes` to prevent false passes from structurally-similar but textually-wrong output. [id:bench-correctness-structural]
- The structural diff report is a first-class output artifact for every benchmark result. [id:bench-diff-report]
- Benchmark tasks that require metadata awareness must specify whether frontmatter and link destinations participate in correctness. [id:bench-task-scorer]
- Every `BenchTask` must declare `expected_artifact` to specify whether the harness scores `stdout_text` (agent's final captured output), `file_contents` (on-disk file state after agent finishes), or `json_envelope` (structured JSON parsed from stdout). The harness compares `expected_output` against exactly the declared artifact. The task's declared `expected_artifact` governs unconditionally — there is no implicit default by task category. [id:bench-task-artifact-rule]

Benchmark task set validation:

- `T1` extracts an outline from a mixed-heading document and is expressible via `outline` alone; `expected_artifact = "json_envelope"`. All three modes produce an `OutlineResult`-shaped JSON object (CLI via `--json`, function-call natively, unix via agent-constructed JSON). The scorer uses `kind = "structural"` with `compare_heading_tree = true` so evaluation measures structural access parity rather than text-format reconstruction. This ensures function-call mode is not penalized for lacking a raw-text rendering path. [id:bench-task-t1]
- `T2` inserts a new block after the third top-level block and is expressible via `insert-block --after 2`; `expected_artifact = "file_contents"`. Scorer uses `kind = "normalized_text"` with `compare_block_order = true` and `compare_block_text = true`. [id:bench-task-t2]
- `T3` replaces a duplicate heading-selected section and requires `--occurrence` to avoid a conflict; `expected_artifact = "file_contents"`. Scorer uses `kind = "normalized_text"` with `compare_heading_tree = true` and `compare_block_text = true`. [id:bench-task-t3]
- `T4` replaces `"method"` with `"approach"` in body text but not in headings or code blocks and requires only the `search`, `block`, and `replace-block` primitives already defined for Phase 1; `expected_artifact = "file_contents"`. Scorer uses `kind = "normalized_text"` with `compare_block_text = true` to catch textually-wrong but structurally-similar output. [id:bench-task-t4]

## Test Strategy [id:sec-tests]

Phase 1 uses fixture-driven integration tests. Each CLI command receives at least one fixture Markdown file, one command invocation, and one expected stdout or expected JSON payload.

Example fixture pair:

- Fixture input: `tests/fixtures/duplicate_headings.md` with two `## Methods` headings. [id:test-fixture-duplicate-headings]
- Expected behavior: `md section "Methods" tests/fixtures/duplicate_headings.md` exits with `Conflict`; adding `--occurrence 2` returns the second section. [id:test-expected-duplicate-headings]

## Ordered Steps [id:sec-steps]

1. Define the final Phase 1 interface contracts in [specs/mdtools.md](/Users/provi/Development/_projs/mdtools/specs/mdtools.md). [id:step-1]
Verify: the artifact contains the Files table, Decisions table, stable anchors, Rust contracts, mutation and search envelopes, benchmark dataclasses, and JSON examples. [id:verify-1]

2. Add the Rust crate surface in `Cargo.toml`, `src/main.rs`, `src/cli.rs`, `src/model.rs`, and `src/errors.rs`. [id:step-2]
Verify: the crate builds a binary named `md`, and each subcommand, selector flag, and insert-location flag parses according to the contracts in [specs/mdtools.md](/Users/provi/Development/_projs/mdtools/specs/mdtools.md). [id:verify-2]

3. Implement the parse boundary and semantic projection in `src/parser.rs` and `src/model.rs`. [id:step-3]
Verify: source spans, block kinds, frontmatter extraction, and section selectors can be produced for every Phase 1 fixture. [id:verify-3]

4. Implement read-only commands in `src/commands/outline.rs`, `src/commands/blocks.rs`, `src/commands/section.rs`, `src/commands/search.rs`, `src/commands/links.rs`, `src/commands/frontmatter.rs`, and `src/commands/stats.rs`. [id:step-4]
Verify: text mode and `--json` mode both match the output contracts for all read commands, including block-kind filtered content search. [id:verify-4]

5. Implement `replace-block`, `replace-section`, `insert-block`, and `delete-block` in `src/commands/replace.rs` and `src/commands/section.rs`. [id:step-5]
Verify: every mutation command returns `MutationResult`, preserves non-target bytes, and satisfies the edge cases in [specs/mdtools.md](/Users/provi/Development/_projs/mdtools/specs/mdtools.md). [id:verify-5]

6. Add fixture-driven integration coverage in `tests/cli_read.rs`, `tests/cli_write.rs`, `tests/cli_search.rs`, and `tests/fixtures/`. [id:step-6]
Verify: fixtures cover duplicate headings, preamble selection, setext headings, fenced code, CRLF, frontmatter-only documents, content search filters, insert-after-frontmatter, and last-block deletion. [id:verify-6]

7. Define benchmark tasks and run configuration in `bench/tasks/` and `bench/harness.py`. [id:step-7]
Verify: each task declares a `StructuralDiffPolicy`, each mode inventory matches the contracts in [specs/mdtools.md](/Users/provi/Development/_projs/mdtools/specs/mdtools.md), and tasks `T1` through `T4` remain expressible without stronger-than-specified primitives. [id:verify-7]

8. Validate the benchmark before Phase 2 claims are published. [id:step-8]
Verify: one dry run produces per-mode conversation logs, structural diff reports, token counts, and elapsed time under a shared `BenchRunConfig`. [id:verify-8]
