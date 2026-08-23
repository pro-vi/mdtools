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
