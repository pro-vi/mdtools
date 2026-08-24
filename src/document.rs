//! Immutable operation-facing Markdown document.
//!
//! [`ParsedDocument`](crate::parser::ParsedDocument) remains available as the
//! low-level parser projection for compatibility. Core operations accept this
//! wrapper so callers cannot mutate source bytes independently of their spans
//! and revision.

use crate::core_error::CoreError;
use crate::model::{LineEndingStyle, SourceSpan};
use crate::parser::{BlockInfo, FrontmatterInfo, FrontmatterState, ParsedDocument};
use crate::revision::DocumentRevision;

pub struct Document {
    parsed: ParsedDocument,
}

impl Document {
    pub fn parse(source: impl Into<String>) -> Result<Self, CoreError> {
        ParsedDocument::parse(source.into()).map(Self::from_parsed)
    }

    pub fn parse_for_frontmatter(source: impl Into<String>) -> Result<Self, CoreError> {
        ParsedDocument::parse_for_frontmatter(source.into()).map(Self::from_parsed)
    }

    pub fn parse_for_frontmatter_mutation(source: impl Into<String>) -> Result<Self, CoreError> {
        ParsedDocument::parse_for_frontmatter_mutation(source.into()).map(Self::from_parsed)
    }

    fn from_parsed(parsed: ParsedDocument) -> Self {
        Self { parsed }
    }

    pub fn source(&self) -> &str {
        &self.parsed.source
    }

    pub fn blocks(&self) -> &[BlockInfo] {
        &self.parsed.blocks
    }

    pub fn frontmatter(&self) -> Option<&FrontmatterInfo> {
        self.parsed.frontmatter.as_ref()
    }

    pub fn line_count(&self) -> u32 {
        self.parsed.line_count()
    }

    pub fn byte_to_line(&self, byte_offset: u32) -> u32 {
        self.parsed.byte_to_line(byte_offset)
    }

    pub fn span_for_byte_range(&self, byte_start: u32, byte_end: u32) -> SourceSpan {
        self.parsed.span_for_byte_range(byte_start, byte_end)
    }

    pub fn slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        self.parsed.try_slice(span)
    }

    pub(crate) fn slice_unchecked(&self, span: &SourceSpan) -> &str {
        self.parsed.slice(span)
    }

    pub fn try_slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        self.slice(span)
    }

    pub fn revision(&self) -> &DocumentRevision {
        self.parsed.revision()
    }

    pub fn frontmatter_state(&self) -> FrontmatterState<'_> {
        self.parsed.frontmatter_state()
    }

    pub fn line_ending_style(&self) -> LineEndingStyle {
        self.parsed.line_ending_style()
    }

    /// Read-only access to the low-level parser projection for adapters that
    /// still need parser-specific metadata during migration.
    pub fn projection(&self) -> &ParsedDocument {
        &self.parsed
    }
}
