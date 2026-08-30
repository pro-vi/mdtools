use crate::model::SourceSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionMatch {
    pub block_index: u32,
    pub occurrence: u32,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    ParseFailed(String),
    FrontmatterParseFailed(String),
    InvalidTableRow(String),
    InvalidSelector(String),
    InvalidTargetEtag(String),
    InvalidDocumentRevision(String),
    InvalidTargetAddress {
        reason: String,
    },
    InvalidPatch(String),
    HeadingDepthOverflow {
        parent_level: u8,
        relative_level: u8,
    },
    TargetAuthorityMismatch {
        target: String,
        expected: String,
        actual: String,
    },
    PatchInvariant(String),
    TargetNotFound {
        target: String,
    },
    AmbiguousTargetQuery {
        count: usize,
    },
    AmbiguousTargetAddress {
        target: String,
        count: usize,
    },
    TargetKindMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    InvalidKeyPath {
        path: String,
        reason: &'static str,
    },
    FrontmatterFieldConflict {
        path: String,
        prefix: String,
    },
    BlockIndexOutOfRange {
        index: u32,
        block_count: u32,
    },
    ByteOffsetOutOfRange {
        byte_offset: u32,
        source_len: u32,
    },
    LineOutOfRange {
        line: u32,
        line_count: u32,
    },
    NoTables,
    NotTable {
        block_index: u32,
    },
    ColumnNotFound {
        column: String,
        headers: Vec<String>,
    },
    TableRowOutOfRange {
        table_block_index: u32,
        row_index: u32,
        row_count: u32,
        insertion: bool,
    },
    HeadingNotFound {
        heading: String,
    },
    DuplicateHeading {
        heading: String,
        matches: Vec<SectionMatch>,
    },
    OccurrenceOutOfRange {
        heading: String,
        requested: u32,
        matches: Vec<SectionMatch>,
    },
    InvalidTaskLoc {
        loc: String,
    },
    TaskBlockOutOfRange {
        loc: String,
        block_index: u32,
        block_count: u32,
    },
    TaskNotFound {
        loc: String,
    },
    NotTaskList {
        block_index: u32,
    },
    DocumentRevisionMismatch {
        expected: String,
        actual: String,
    },
    DocumentIndexMismatch,
    InvalidSpan {
        span: SourceSpan,
        source_len: usize,
        reason: &'static str,
    },
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseFailed(message)
            | Self::FrontmatterParseFailed(message)
            | Self::InvalidTableRow(message)
            | Self::InvalidSelector(message) => write!(f, "{message}"),
            Self::InvalidTargetEtag(value) => {
                write!(f, "invalid target etag {value:?} (expected 64 hexadecimal characters)")
            }
            Self::InvalidDocumentRevision(value) => write!(
                f,
                "invalid document revision {value:?} (expected 64 hexadecimal characters)"
            ),
            Self::InvalidTargetAddress { reason } => {
                write!(f, "invalid target address: {reason}")
            }
            Self::InvalidPatch(reason) => write!(f, "invalid patch: {reason}"),
            Self::HeadingDepthOverflow {
                parent_level,
                relative_level,
            } => write!(
                f,
                "heading depth overflow: parent level {parent_level} plus relative level {relative_level} exceeds level 6"
            ),
            Self::TargetAuthorityMismatch {
                target,
                expected,
                actual,
            } => write!(
                f,
                "target authority mismatch for {target}: expected {expected}, found {actual}"
            ),
            Self::PatchInvariant(reason) => write!(f, "patch invariant failed: {reason}"),
            Self::TargetNotFound { target } => write!(f, "target not found: {target}"),
            Self::AmbiguousTargetQuery { count } => {
                write!(f, "target query matched {count} targets; use an exact address")
            }
            Self::AmbiguousTargetAddress { target, count } => write!(
                f,
                "exact target address {target} resolved to {count} targets"
            ),
            Self::TargetKindMismatch { expected, actual } => {
                write!(f, "target kind mismatch: expected {expected}, found {actual}")
            }
            Self::InvalidKeyPath { path, reason } => {
                write!(f, "invalid key path {path:?}: {reason}")
            }
            Self::FrontmatterFieldConflict { path, prefix } => write!(
                f,
                "cannot set {path:?}: {prefix:?} is not an object"
            ),
            Self::BlockIndexOutOfRange { index, block_count } => write!(
                f,
                "block index {index} out of range (document has {block_count} blocks)"
            ),
            Self::ByteOffsetOutOfRange {
                byte_offset,
                source_len,
            } => write!(
                f,
                "byte offset {byte_offset} out of range (document has {source_len} bytes)"
            ),
            Self::LineOutOfRange { line, line_count } => write!(
                f,
                "line {line} out of range (document has {line_count} lines)"
            ),
            Self::NoTables => write!(f, "no tables found in document"),
            Self::NotTable { block_index } => write!(f, "block {block_index} is not a table"),
            Self::ColumnNotFound { column, headers } => write!(
                f,
                "column {column:?} not found (available: {})",
                headers.join(", ")
            ),
            Self::TableRowOutOfRange {
                table_block_index,
                row_index,
                row_count,
                insertion,
            } => write!(
                f,
                "table row {} {} in block {} (table has {} rows)",
                row_index,
                if *insertion { "insertion out of range" } else { "not found" },
                table_block_index,
                row_count
            ),
            Self::HeadingNotFound { heading } => write!(f, "heading not found: {heading}"),
            Self::DuplicateHeading { heading, matches } => {
                write!(f, "heading {heading:?} matches {} sections", matches.len())
            }
            Self::OccurrenceOutOfRange {
                heading,
                requested,
                matches,
            } => write!(
                f,
                "heading not found: {heading} (occurrence {requested} of {})",
                matches.len()
            ),
            Self::InvalidTaskLoc { loc } => {
                write!(f, "invalid task loc: {loc:?} (expected N.N[.N...] format)")
            }
            Self::TaskBlockOutOfRange {
                loc,
                block_index,
                block_count,
            } => write!(
                f,
                "task item not found: {loc} (block index {block_index} out of range; document has {block_count} blocks)"
            ),
            Self::TaskNotFound { loc } => write!(f, "task item not found: {loc}"),
            Self::NotTaskList { block_index } => {
                write!(f, "block {block_index} has no task items")
            }
            Self::DocumentRevisionMismatch { expected, actual } => write!(
                f,
                "document revision mismatch: expected {expected:?}, found {actual:?}"
            ),
            Self::DocumentIndexMismatch => {
                write!(f, "resolved target belongs to a different document index")
            }
            Self::InvalidSpan {
                span,
                source_len,
                reason,
            } => write!(
                f,
                "invalid source span {}..{} for {} bytes: {}",
                span.byte_start, span.byte_end, source_len, reason
            ),
        }
    }
}

impl std::error::Error for CoreError {}
