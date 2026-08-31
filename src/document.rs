//! Immutable operation-facing Markdown document.
//!
//! Parser-specific state remains private behind the indexed target API.

use crate::core_error::CoreError;
use crate::index::DocumentIndex;
use crate::model::{FrontmatterFormat, LineEndingStyle, SourceSpan};
use crate::parser::{BlockFact, FrontmatterFact, ParsedFacts};
use crate::read::TargetRead;
use crate::revision::DocumentRevision;
use crate::source::{DocumentSource, ParsePolicy};
use crate::target::{QueryResult, ResolvedTarget, TargetAddress, TargetQuery, TargetSnapshot};
use sha2::{Digest, Sha256};

pub(crate) struct FrontmatterState<'a> {
    pub(crate) span: Option<SourceSpan>,
    pub(crate) raw: Option<&'a str>,
    pub(crate) format: Option<FrontmatterFormat>,
    pub(crate) etag: String,
}

pub struct Document {
    index: DocumentIndex,
}

impl Document {
    pub fn parse(source: impl Into<String>) -> Result<Self, CoreError> {
        Self::parse_with_policy(source.into(), ParsePolicy::Lenient, true)
    }

    pub fn parse_for_frontmatter(source: impl Into<String>) -> Result<Self, CoreError> {
        Self::parse_with_policy(source.into(), ParsePolicy::StrictRead, true)
    }

    pub fn parse_for_frontmatter_mutation(source: impl Into<String>) -> Result<Self, CoreError> {
        Self::parse_with_policy(source.into(), ParsePolicy::Mutation, true)
    }

    pub(crate) fn parse_fragment(source: impl Into<String>) -> Result<Self, CoreError> {
        Self::parse_with_policy(source.into(), ParsePolicy::Lenient, false)
    }

    fn parse_with_policy(
        source: String,
        policy: ParsePolicy,
        frontmatter_enabled: bool,
    ) -> Result<Self, CoreError> {
        let source = DocumentSource::new(source, policy)?;
        let facts = if frontmatter_enabled {
            ParsedFacts::parse(&source)?
        } else {
            ParsedFacts::parse_without_frontmatter(&source)?
        };
        Ok(Self {
            index: DocumentIndex::build(source, facts)?,
        })
    }

    pub fn source(&self) -> &str {
        self.index.source().text()
    }

    pub(crate) fn blocks(&self) -> &[BlockFact] {
        &self.index.legacy_facts().blocks
    }

    pub(crate) fn frontmatter(&self) -> Option<&FrontmatterFact> {
        self.index.legacy_facts().frontmatter.as_ref()
    }

    pub fn has_frontmatter(&self) -> bool {
        self.frontmatter().is_some()
    }

    pub fn line_count(&self) -> u32 {
        self.index.source().line_count()
    }

    pub fn byte_to_line(&self, byte_offset: u32) -> u32 {
        self.index.source().byte_to_line(byte_offset)
    }

    /// Byte offset of the first byte of a 1-based line, or `None` when the
    /// line is 0 or beyond [`line_count`](Self::line_count).
    pub fn line_to_byte(&self, line: u32) -> Option<u32> {
        self.index.source().line_to_byte(line)
    }

    pub fn span_for_byte_range(&self, byte_start: u32, byte_end: u32) -> SourceSpan {
        self.index
            .source()
            .span_for_byte_range(byte_start, byte_end)
    }

    pub fn slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        self.index.source().try_slice(span)
    }

    pub(crate) fn slice_unchecked(&self, span: &SourceSpan) -> &str {
        self.index.source().slice_unchecked(span)
    }

    pub fn try_slice(&self, span: &SourceSpan) -> Result<&str, CoreError> {
        self.slice(span)
    }

    pub fn revision(&self) -> &DocumentRevision {
        self.index.source().revision()
    }

    pub(crate) fn frontmatter_state(&self) -> FrontmatterState<'_> {
        match self.frontmatter() {
            Some(frontmatter) => {
                let raw = self.slice_unchecked(&frontmatter.span);
                FrontmatterState {
                    span: Some(frontmatter.span),
                    raw: Some(raw),
                    format: Some(frontmatter.format),
                    etag: frontmatter_state_etag(Some(raw)),
                }
            }
            None => FrontmatterState {
                span: None,
                raw: None,
                format: None,
                etag: frontmatter_state_etag(None),
            },
        }
    }

    pub fn line_ending_style(&self) -> LineEndingStyle {
        self.index.source().line_ending_style()
    }

    pub fn index(&self) -> &DocumentIndex {
        &self.index
    }

    pub fn map(&self) -> Result<Vec<TargetSnapshot>, CoreError> {
        crate::target::map(self)
    }

    pub fn query(&self, query: &TargetQuery) -> Result<Vec<QueryResult>, CoreError> {
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
        match self.index.source().policy() {
            ParsePolicy::Lenient => Self::parse(source),
            ParsePolicy::StrictRead => Self::parse_for_frontmatter(source),
            ParsePolicy::Mutation => Self::parse_for_frontmatter_mutation(source),
        }
    }
}

fn frontmatter_state_etag(raw: Option<&str>) -> String {
    const ABSENT_DOMAIN: &[u8] = b"mdtools.frontmatter.absent";
    const PRESENT_DOMAIN: &[u8] = b"mdtools.frontmatter.present\0";

    let mut hash = Sha256::new();
    let bytes = raw.map(str::as_bytes);
    hash.update(if bytes.is_some() {
        PRESENT_DOMAIN
    } else {
        ABSENT_DOMAIN
    });
    if let Some(bytes) = bytes {
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())
}
