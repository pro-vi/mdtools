use crate::model::{LineEndingStyle, MutationDisposition, SourceSpan};
use crate::revision::DocumentRevision;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceEdit {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) replacement: String,
}

#[derive(Clone, Debug)]
pub struct EditOutcome<T> {
    pub base_revision: DocumentRevision,
    pub target: T,
    pub disposition: MutationDisposition,
    pub guarded: bool,
    pub line_endings: LineEndingStyle,
    pub preservation: EditPreservation,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditPreservation {
    pub preserves_non_target_bytes: bool,
    pub target_span_before: Option<SourceSpan>,
    pub target_span_after: Option<SourceSpan>,
}

impl<T> EditOutcome<T> {
    pub fn changed(&self) -> bool {
        self.disposition != MutationDisposition::NoChange
    }
}

pub(crate) fn normalize_line_endings(content: &str, style: LineEndingStyle) -> String {
    match style {
        LineEndingStyle::Lf => content.replace("\r\n", "\n"),
        LineEndingStyle::Crlf => content.replace("\r\n", "\n").replace('\n', "\r\n"),
        LineEndingStyle::Mixed => content.to_string(),
    }
}

pub(crate) fn strip_one_trailing_newline(mut content: String) -> String {
    if content.ends_with("\r\n") {
        content.truncate(content.len() - 2);
    } else if content.ends_with('\n') {
        content.truncate(content.len() - 1);
    }
    content
}

pub(crate) fn replacement_span_after(span: SourceSpan, replacement: &str) -> SourceSpan {
    let newlines = replacement.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let trailing = u32::from(replacement.as_bytes().last() == Some(&b'\n'));
    SourceSpan {
        line_start: span.line_start,
        line_end: span.line_start + newlines.saturating_sub(trailing),
        byte_start: span.byte_start,
        byte_end: span.byte_start + replacement.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ending_normalization_preserves_each_style_contract() {
        assert_eq!(
            normalize_line_endings("one\r\ntwo\n", LineEndingStyle::Lf),
            "one\ntwo\n"
        );
        assert_eq!(
            normalize_line_endings("one\r\ntwo\n", LineEndingStyle::Crlf),
            "one\r\ntwo\r\n"
        );
        assert_eq!(
            normalize_line_endings("one\r\ntwo\n", LineEndingStyle::Mixed),
            "one\r\ntwo\n"
        );
    }

    #[test]
    fn trailing_newline_removal_strips_exactly_one_line_ending() {
        assert_eq!(strip_one_trailing_newline("row\r\n".into()), "row");
        assert_eq!(strip_one_trailing_newline("row\n\n".into()), "row\n");
        assert_eq!(strip_one_trailing_newline("row".into()), "row");
    }

    #[test]
    fn replacement_span_tracks_payload_bytes_and_owned_lines() {
        let before = SourceSpan {
            line_start: 4,
            line_end: 8,
            byte_start: 20,
            byte_end: 80,
        };
        assert_eq!(
            replacement_span_after(before, "first\nsecond\n"),
            SourceSpan {
                line_start: 4,
                line_end: 5,
                byte_start: 20,
                byte_end: 33,
            }
        );
        assert_eq!(
            replacement_span_after(before, ""),
            SourceSpan {
                line_start: 4,
                line_end: 4,
                byte_start: 20,
                byte_end: 20,
            }
        );
    }
}
