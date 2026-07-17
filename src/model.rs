use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "mdtools.v1";

// --- Common types ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line_start: u32,
    pub line_end: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LineEndingStyle {
    Lf,
    Crlf,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum HeadingMatchMode {
    Exact,
    ExactIgnoreCase,
    Contains,
    ContainsIgnoreCase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SearchMatchMode {
    Literal,
    LiteralIgnoreCase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SectionSelectorKind {
    HeadingText,
    Preamble,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SectionSelector {
    pub kind: SectionSelectorKind,
    pub heading_text: Option<String>,
    pub occurrence: Option<u32>,
    pub match_mode: HeadingMatchMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum InsertLocation {
    Before(u32),
    After(u32),
    Start,
    End,
}

// --- Block types ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum BlockKind {
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

impl std::fmt::Display for BlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Heading => "Heading",
            Self::Paragraph => "Paragraph",
            Self::List => "List",
            Self::BlockQuote => "BlockQuote",
            Self::CodeFence => "CodeFence",
            Self::IndentedCode => "IndentedCode",
            Self::ThematicBreak => "ThematicBreak",
            Self::Table => "Table",
            Self::HtmlBlock => "HtmlBlock",
            Self::FootnoteDefinition => "FootnoteDefinition",
        };
        write!(f, "{}", s)
    }
}

// --- Read model types ---

#[derive(Clone, Debug, Serialize)]
pub struct HeadingRef {
    pub level: u8,
    pub text: String,
    pub block_index: u32,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutlineEntry {
    pub heading: HeadingRef,
    pub section_span: SourceSpan,
    /// Content fingerprint of the full section span (heading through section
    /// end) — the value section mutations accept via --expect-etag.
    pub etag: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutlineResult {
    pub schema_version: String,
    pub file: String,
    pub entries: Vec<OutlineEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockEntry {
    pub index: u32,
    pub kind: BlockKind,
    pub span: SourceSpan,
    /// Content fingerprint of the block's bytes. Pass back via `--expect-etag`
    /// on a mutation to fail-closed if the index went stale between read/write.
    pub etag: String,
    pub preview: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlocksResult {
    pub schema_version: String,
    pub file: String,
    pub blocks: Vec<BlockEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockReadResult {
    pub schema_version: String,
    pub file: String,
    pub block: BlockEntry,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SectionKind {
    Preamble,
    Heading,
}

#[derive(Clone, Debug, Serialize)]
pub struct SectionEntry {
    pub kind: SectionKind,
    pub heading: Option<HeadingRef>,
    pub selector: SectionSelector,
    pub depth: u8,
    pub block_indices: Vec<u32>,
    pub span: SourceSpan,
    /// Content fingerprint of the section's exact source bytes. Pass back via
    /// `--expect-etag` on section mutations to fail-closed on stale reads.
    pub etag: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SectionReadResult {
    pub schema_version: String,
    pub file: String,
    pub section: SectionEntry,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LinkKind {
    Inline,
    Reference,
    Autolink,
}

impl std::fmt::Display for LinkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inline => write!(f, "Inline"),
            Self::Reference => write!(f, "Reference"),
            Self::Autolink => write!(f, "Autolink"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LinkEntry {
    pub kind: LinkKind,
    pub text: String,
    pub destination: Option<String>,
    pub title: Option<String>,
    pub source_block_index: u32,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinksResult {
    pub schema_version: String,
    pub file: String,
    pub links: Vec<LinkEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchMatch {
    pub block_index: u32,
    pub block_kind: BlockKind,
    pub match_span: SourceSpan,
    pub preview: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResult {
    pub schema_version: String,
    pub file: String,
    pub query: String,
    pub match_mode: SearchMatchMode,
    pub block_kinds: Vec<BlockKind>,
    pub matches: Vec<SearchMatch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FrontmatterFormat {
    Yaml,
    Toml,
}

#[derive(Clone, Debug, Serialize)]
pub struct FrontmatterEnvelope {
    pub format: FrontmatterFormat,
    pub span: SourceSpan,
    pub raw: String,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct FrontmatterReadResult {
    pub schema_version: String,
    pub file: String,
    pub present: bool,
    pub etag: String,
    pub frontmatter: Option<FrontmatterEnvelope>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FrontmatterFieldProjectionResult {
    pub schema_version: String,
    pub file: String,
    pub present: bool,
    pub etag: String,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CollectResult {
    pub schema_version: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentStats {
    pub word_count: u32,
    pub heading_count: u32,
    pub block_count: u32,
    pub link_count: u32,
    pub section_count: u32,
    pub line_count: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct StatsResult {
    pub schema_version: String,
    pub file: String,
    pub stats: DocumentStats,
}

// --- Table types ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ColumnAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Serialize)]
pub struct TableEntry {
    pub block_index: u32,
    pub span: SourceSpan,
    pub etag: String,
    pub headers: Vec<String>,
    pub row_count: u32,
    pub column_count: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TablesResult {
    pub schema_version: String,
    pub file: String,
    pub tables: Vec<TableEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TableReadResult {
    pub schema_version: String,
    pub file: String,
    pub block_index: u32,
    pub span: SourceSpan,
    pub etag: String,
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<Vec<String>>,
}

// --- Task types ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
#[clap(rename_all = "kebab-case")]
pub enum TaskStatus {
    Pending,
    Done,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Done => write!(f, "done"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskEntry {
    pub loc: String,
    pub block_index: u32,
    pub child_path: Vec<u32>,
    pub task_index: u32,
    pub status: TaskStatus,
    pub depth: u32,
    pub nearest_heading: Option<String>,
    pub nearest_heading_block_index: Option<u32>,
    pub span: SourceSpan,
    /// Content fingerprint of the task item's exact source bytes. Pass back via
    /// `--expect-etag` on `set-task` to fail-closed on stale reads.
    pub etag: String,
    pub summary_text: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskFileResult {
    pub file: String,
    pub tasks: Vec<TaskEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TasksResult {
    pub schema_version: String,
    pub results: Vec<TaskFileResult>,
    /// Per-file failures in multi-file mode. tasks keeps its single-object
    /// wire shape: structured failures ride here, never as NDJSON rows.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failures: Vec<TaskFileFailure>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskFileFailure {
    pub file: String,
    pub error: crate::errors::ErrorInfo,
}

// --- Mutation types ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MutationTargetKind {
    Block,
    Section,
    InsertLocation,
    FrontmatterField,
    TaskItem,
    TableRow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MutationCommandKind {
    ReplaceBlock,
    ReplaceSection,
    ReplaceTableRow,
    DeleteTableRow,
    InsertBlock,
    DeleteBlock,
    DeleteSection,
    SetFrontmatter,
    SetTask,
    MoveSection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MutationDisposition {
    NoChange,
    Replaced,
    Inserted,
    Deleted,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockTargetRef {
    pub kind: MutationTargetKind,
    pub block_index: u32,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Serialize)]
pub struct SectionTargetRef {
    pub kind: MutationTargetKind,
    pub selector: SectionSelector,
    pub section: SectionEntry,
}

#[derive(Clone, Debug, Serialize)]
pub struct InsertTargetRef {
    pub kind: MutationTargetKind,
    pub location: InsertLocation,
    pub anchor_span: Option<SourceSpan>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FrontmatterFieldTargetRef {
    pub kind: MutationTargetKind,
    pub key_path: String,
    pub format: FrontmatterFormat,
}

#[derive(Clone, Debug, Serialize)]
pub enum MutationTargetRef {
    Block(BlockTargetRef),
    Section(SectionTargetRef),
    Insert(InsertTargetRef),
    FrontmatterField(FrontmatterFieldTargetRef),
    TaskItem(TaskItemTargetRef),
    TableRow(TableRowTargetRef),
    SectionMove(SectionMoveTargetRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsertMode {
    AfterSibling,
    BeforeSibling,
    IntoAsChild,
}

#[derive(Clone, Debug, Serialize)]
pub struct SectionMoveTargetRef {
    pub kind: MutationTargetKind,
    pub source: SectionTargetRef,
    pub destination: SectionTargetRef,
    pub destination_mode: InsertMode,
    pub level_shift_applied: i8,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskItemTargetRef {
    pub kind: MutationTargetKind,
    pub loc: String,
    pub block_index: u32,
    pub child_path: Vec<u32>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Serialize)]
pub struct TableRowTargetRef {
    pub kind: MutationTargetKind,
    pub table_block_index: u32,
    pub row_index: u32,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourcePreservationInvariant {
    pub preserves_non_target_bytes: bool,
    pub target_span_before: Option<SourceSpan>,
    pub target_span_after: Option<SourceSpan>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MutationResult {
    pub schema_version: String,
    pub file: String,
    pub command: MutationCommandKind,
    pub target: MutationTargetRef,
    pub disposition: MutationDisposition,
    pub changed: bool,
    /// True when the mutation ran under an etag expectation
    /// (--expect-etag / --expect-source-etag / --expect-dest-etag), so guard
    /// adoption is observable to agents and usage instrumentation.
    pub guarded: bool,
    pub line_endings: LineEndingStyle,
    pub invariant: SourcePreservationInvariant,
    pub content: Option<String>,
}
