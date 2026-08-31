//! Exact source bytes and byte-derived metadata for one parsed document.

use crate::core_error::CoreError;
use crate::model::{LineEndingStyle, SourceSpan};
use crate::revision::DocumentRevision;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParsePolicy {
    Lenient,
    StrictRead,
    Mutation,
}

/// The sole source-bearing state retained for a parsed document.
pub(crate) struct DocumentSource {
    text: String,
    lines: LineIndex,
    revision: DocumentRevision,
    policy: ParsePolicy,
    line_endings: LineEndingStyle,
}

impl DocumentSource {
    pub(crate) fn new(text: String, policy: ParsePolicy) -> Result<Self, CoreError> {
        // LineIndex stores one initial line start plus at most one start per byte.
        // Keeping the source one byte below u32::MAX makes both byte offsets and
        // the worst-case line count representable by SourceSpan's u32 fields.
        validate_source_len(text.len())?;

        let lines = LineIndex::new(&text);
        let revision = DocumentRevision::for_source(&text);
        let line_endings = detect_line_endings(&text);
        Ok(Self {
            text,
            lines,
            revision,
            policy,
            line_endings,
        })
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn len(&self) -> usize {
        self.text.len()
    }

    pub(crate) fn is_char_boundary(&self, offset: usize) -> bool {
        self.text.is_char_boundary(offset)
    }

    pub(crate) fn lines(&self) -> &LineIndex {
        &self.lines
    }

    pub(crate) fn line_count(&self) -> u32 {
        self.lines.line_count()
    }

    pub(crate) fn byte_to_line(&self, byte_offset: u32) -> u32 {
        self.lines.byte_to_line(byte_offset as usize) as u32
    }

    pub(crate) fn line_to_byte(&self, line: u32) -> Option<u32> {
        self.lines.line_start_byte(line).map(|byte| byte as u32)
    }

    pub(crate) fn span_for_byte_range(&self, byte_start: u32, byte_end: u32) -> SourceSpan {
        let line_start = self.byte_to_line(byte_start);
        let line_end = if byte_end > byte_start {
            self.byte_to_line(byte_end - 1)
        } else {
            line_start
        };
        SourceSpan {
            line_start,
            line_end,
            byte_start,
            byte_end,
        }
    }

    pub(crate) fn slice_unchecked(&self, span: &SourceSpan) -> &str {
        &self.text[span.byte_start as usize..span.byte_end as usize]
    }

    pub(crate) fn try_slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        let start = span.byte_start as usize;
        let end = span.byte_end as usize;
        let reason = if start > end {
            Some("start is after end")
        } else if end > self.text.len() {
            Some("end is outside the source")
        } else if !self.is_char_boundary(start) || !self.is_char_boundary(end) {
            Some("offset is not a UTF-8 character boundary")
        } else {
            None
        };

        if let Some(reason) = reason {
            return Err(CoreError::InvalidSpan {
                span: *span,
                source_len: self.text.len(),
                reason,
            });
        }

        Ok(&self.text[start..end])
    }

    pub(crate) fn revision(&self) -> &DocumentRevision {
        &self.revision
    }

    pub(crate) fn policy(&self) -> ParsePolicy {
        self.policy
    }

    pub(crate) fn line_ending_style(&self) -> LineEndingStyle {
        self.line_endings
    }
}

/// Byte offsets of the start of each 1-based source line.
pub(crate) struct LineIndex {
    starts: Vec<usize>,
    content_ends: Vec<usize>,
    source_len: usize,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }
        let content_ends = starts
            .iter()
            .enumerate()
            .map(|(index, start)| {
                let mut end = starts.get(index + 1).copied().unwrap_or(source.len());
                if end > *start && source.as_bytes()[end - 1] == b'\n' {
                    end -= 1;
                    if end > *start && source.as_bytes()[end - 1] == b'\r' {
                        end -= 1;
                    }
                }
                end
            })
            .collect();
        Self {
            starts,
            content_ends,
            source_len: source.len(),
        }
    }

    pub(crate) fn to_byte(&self, line: usize, column: usize) -> Option<usize> {
        if line == 0 || column == 0 {
            return None;
        }
        let index = line - 1;
        let start = *self.starts.get(index)?;
        let content_end = self.content_ends[index];
        start
            .checked_add(column - 1)
            .filter(|offset| *offset <= content_end)
    }

    pub(crate) fn to_byte_end(&self, line: usize, column: usize) -> Option<usize> {
        if line == 0 || column == 0 {
            return None;
        }
        let index = line - 1;
        let start = *self.starts.get(index)?;
        let physical_end = self
            .starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.source_len);
        start
            .checked_add(column)
            .filter(|offset| *offset <= physical_end)
    }

    pub(crate) fn source_len(&self) -> usize {
        self.source_len
    }

    pub(crate) fn line_count(&self) -> u32 {
        self.starts.len() as u32
    }

    pub(crate) fn line_start_byte(&self, line: u32) -> Option<usize> {
        if line == 0 {
            return None;
        }
        self.starts.get((line - 1) as usize).copied()
    }

    pub(crate) fn byte_to_line(&self, byte_offset: usize) -> usize {
        match self.starts.binary_search(&byte_offset) {
            Ok(index) => index + 1,
            Err(index) => index,
        }
    }
}

fn detect_line_endings(source: &str) -> LineEndingStyle {
    let has_crlf = source.contains("\r\n");
    let has_bare_lf = source.bytes().enumerate().any(|(index, byte)| {
        byte == b'\n' && (index == 0 || source.as_bytes()[index - 1] != b'\r')
    });
    match (has_crlf, has_bare_lf) {
        (true, false) => LineEndingStyle::Crlf,
        (false, true) | (false, false) => LineEndingStyle::Lf,
        (true, true) => LineEndingStyle::Mixed,
    }
}

fn validate_source_len(source_len: usize) -> Result<(), CoreError> {
    const MAX_SOURCE_BYTES: usize = u32::MAX as usize - 1;
    if source_len > MAX_SOURCE_BYTES {
        Err(CoreError::ParseFailed(format!(
            "document is {source_len} bytes; maximum supported size is {MAX_SOURCE_BYTES} bytes"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_source_preserves_exact_bytes_and_derived_metadata() {
        for source in ["", "one\n", "one\r\ntwo", "one\r\ntwo\n世界"] {
            let indexed = DocumentSource::new(source.into(), ParsePolicy::Mutation).unwrap();
            assert_eq!(indexed.text(), source);
            assert_eq!(indexed.revision(), &DocumentRevision::for_source(source));
            assert_eq!(indexed.policy(), ParsePolicy::Mutation);
            assert_eq!(indexed.line_to_byte(1), Some(0));
        }
    }

    #[test]
    fn line_index_rejects_invalid_or_out_of_range_coordinates() {
        let indexed = DocumentSource::new("é\nend".into(), ParsePolicy::Lenient).unwrap();
        assert_eq!(indexed.lines().to_byte(0, 1), None);
        assert_eq!(indexed.lines().to_byte(1, 0), None);
        assert_eq!(indexed.lines().to_byte(3, 1), None);
        assert_eq!(indexed.lines().to_byte(2, 4), Some(indexed.len()));
        assert_eq!(indexed.lines().to_byte(2, 5), None);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn oversized_sources_fail_before_offsets_can_wrap() {
        assert!(validate_source_len(u32::MAX as usize - 1).is_ok());
        assert!(matches!(
            validate_source_len(u32::MAX as usize),
            Err(CoreError::ParseFailed(message)) if message.contains("maximum supported size")
        ));
    }
}
