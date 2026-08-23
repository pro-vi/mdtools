use crate::model::SourceSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    ParseFailed(String),
    FrontmatterParseFailed(String),
    InvalidTableRow(String),
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
            | Self::InvalidTableRow(message) => write!(f, "{message}"),
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
