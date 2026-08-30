use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "mdtools.v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub line_start: u32,
    pub line_end: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum LineEndingStyle {
    Lf,
    Crlf,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum HeadingMatchMode {
    Exact,
    ExactIgnoreCase,
    Contains,
    ContainsIgnoreCase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchMatchMode {
    Literal,
    LiteralIgnoreCase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InsertMode {
    BeforeSibling,
    AfterSibling,
    IntoAsChild,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[cfg_attr(feature = "cli", clap(rename_all = "kebab-case"))]
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum LinkKind {
    Inline,
    Reference,
    Autolink,
}

impl std::fmt::Display for LinkKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FrontmatterFormat {
    Yaml,
    Toml,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ColumnAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "cli", clap(rename_all = "kebab-case"))]
pub enum TaskStatus {
    Pending,
    Done,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => formatter.write_str("pending"),
            Self::Done => formatter.write_str("done"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MutationDisposition {
    NoChange,
    Replaced,
    Inserted,
    Deleted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct DocumentStats {
    pub word_count: u32,
    pub heading_count: u32,
    pub block_count: u32,
    pub link_count: u32,
    pub section_count: u32,
    pub line_count: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct HeadingRef {
    pub level: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct SectionEntry {
    pub heading: Option<HeadingRef>,
    pub block_indices: Vec<u32>,
    pub span: SourceSpan,
}
