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
    TargetEtagMismatch {
        target: String,
        expected: String,
        actual: String,
    },
    TargetEtagAmbiguous {
        target_kind: &'static str,
        expected: String,
        count: usize,
    },
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
            Self::TargetEtagMismatch {
                target,
                expected,
                actual,
            } => write!(
                f,
                "{target} etag mismatch: expected {expected:?}, found {actual:?}"
            ),
            Self::TargetEtagAmbiguous {
                target_kind,
                expected,
                count,
            } => write!(
                f,
                "{target_kind} etag {expected:?} is ambiguous: {count} same-content {target_kind}s share this fingerprint"
            ),
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
