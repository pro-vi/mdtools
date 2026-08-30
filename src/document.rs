//! Immutable operation-facing Markdown document.
//!
//! [`ParsedDocument`] remains available as the
//! low-level parser projection for compatibility. Core operations accept this
//! wrapper so callers cannot mutate source bytes independently of their spans
//! and revision.

use crate::core_error::CoreError;
use crate::index::DocumentIndex;
use crate::model::{LineEndingStyle, SourceSpan};
use crate::parser::{BlockInfo, FrontmatterInfo, FrontmatterState, ParsePolicy, ParsedDocument};
use crate::read::TargetRead;
use crate::revision::DocumentRevision;
use crate::target::{ResolvedTarget, TargetAddress, TargetQuery, TargetSnapshot};

pub struct Document {
    index: DocumentIndex,
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

    pub(crate) fn parse_fragment(source: impl Into<String>) -> Result<Self, CoreError> {
        ParsedDocument::parse_without_frontmatter(source.into()).map(Self::from_parsed)
    }

    fn from_parsed(parsed: ParsedDocument) -> Self {
        Self {
            index: DocumentIndex::build(parsed),
        }
    }

    pub fn source(&self) -> &str {
        &self.index.projection().source
    }

    pub fn blocks(&self) -> &[BlockInfo] {
        &self.index.projection().blocks
    }

    pub fn frontmatter(&self) -> Option<&FrontmatterInfo> {
        self.index.projection().frontmatter.as_ref()
    }

    pub fn line_count(&self) -> u32 {
        self.index.projection().line_count()
    }

    pub fn byte_to_line(&self, byte_offset: u32) -> u32 {
        self.index.projection().byte_to_line(byte_offset)
    }

    /// Byte offset of the first byte of a 1-based line, or `None` when the
    /// line is 0 or beyond [`line_count`](Self::line_count).
    pub fn line_to_byte(&self, line: u32) -> Option<u32> {
        self.index.projection().line_to_byte(line)
    }

    pub fn span_for_byte_range(&self, byte_start: u32, byte_end: u32) -> SourceSpan {
        self.index
            .projection()
            .span_for_byte_range(byte_start, byte_end)
    }

    pub fn slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        self.index.projection().try_slice(span)
    }

    pub(crate) fn slice_unchecked(&self, span: &SourceSpan) -> &str {
        self.index.projection().slice(span)
    }

    pub fn try_slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        self.slice(span)
    }

    pub fn revision(&self) -> &DocumentRevision {
        self.index.projection().revision()
    }

    pub fn frontmatter_state(&self) -> FrontmatterState<'_> {
        self.index.projection().frontmatter_state()
    }

    pub fn line_ending_style(&self) -> LineEndingStyle {
        self.index.projection().line_ending_style()
    }

    pub fn index(&self) -> &DocumentIndex {
        &self.index
    }

    pub fn map(&self) -> Result<Vec<TargetSnapshot>, CoreError> {
        crate::target::map(self)
    }

    pub fn query(&self, query: &TargetQuery) -> Result<Vec<TargetSnapshot>, CoreError> {
        crate::target::query(self, query)
    }

    pub fn query_one(&self, query: &TargetQuery) -> Result<ResolvedTarget, CoreError> {
        crate::target::query_one(self, query)
    }

    pub fn resolve(&self, address: &TargetAddress) -> Result<ResolvedTarget, CoreError> {
        crate::target::resolve(self, address)
    }

    pub fn locate_targets(&self, byte_offset: u32) -> Result<Vec<TargetSnapshot>, CoreError> {
        crate::target::locate(self, byte_offset)
    }

    pub fn read_target(&self, target: &ResolvedTarget) -> Result<TargetRead, CoreError> {
        crate::read::read(self, target)
    }

    pub(crate) fn reparse(&self, source: impl Into<String>) -> Result<Self, CoreError> {
        let source = source.into();
        match self.index.projection().policy() {
            ParsePolicy::Lenient => Self::parse(source),
            ParsePolicy::StrictRead => Self::parse_for_frontmatter(source),
            ParsePolicy::Mutation => Self::parse_for_frontmatter_mutation(source),
        }
    }

    /// Read-only access to the low-level parser projection for adapters that
    /// still need parser-specific metadata during migration.
    pub fn projection(&self) -> &ParsedDocument {
        self.index.projection()
    }
}
