use std::num::NonZeroU32;

use crate::core_error::{CoreError, SectionMatch};
use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::model::{
    HeadingMatchMode, HeadingRef, OutlineEntry, SectionEntry, SectionKind, SectionSelector,
    SectionSelectorKind, SourceSpan,
};
use crate::revision::DocumentRevision;

/// A section selector whose invalid states cannot be constructed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectionTarget {
    Preamble,
    Heading {
        text: String,
        occurrence: Option<NonZeroU32>,
        match_mode: HeadingMatchMode,
    },
}

impl SectionTarget {
    pub fn preamble() -> Self {
        Self::Preamble
    }

    pub fn heading(
        text: impl Into<String>,
        occurrence: Option<u32>,
        match_mode: HeadingMatchMode,
    ) -> Result<Self, CoreError> {
        let text = text.into();
        if text.is_empty()
            && matches!(
                match_mode,
                HeadingMatchMode::Contains | HeadingMatchMode::ContainsIgnoreCase
            )
        {
            return Err(CoreError::InvalidSelector(
                "empty selector cannot be used with contains matching".into(),
            ));
        }
        let occurrence = occurrence
            .map(|value| {
                NonZeroU32::new(value).ok_or_else(|| {
                    CoreError::InvalidSelector(
                        "occurrence is 1-based; 0 is not a valid occurrence".into(),
                    )
                })
            })
            .transpose()?;
        Ok(Self::Heading {
            text,
            occurrence,
            match_mode,
        })
    }

    pub fn to_wire(&self) -> SectionSelector {
        match self {
            Self::Preamble => SectionSelector {
                kind: SectionSelectorKind::Preamble,
                heading_text: None,
                occurrence: None,
                match_mode: HeadingMatchMode::Exact,
            },
            Self::Heading {
                text,
                occurrence,
                match_mode,
            } => SectionSelector {
                kind: SectionSelectorKind::HeadingText,
                heading_text: Some(text.clone()),
                occurrence: occurrence.map(NonZeroU32::get),
                match_mode: *match_mode,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct SectionIndex {
    preamble: SectionEntry,
    headings: Vec<SectionEntry>,
    /// Span of each block, indexed by block index, so a block can be located by
    /// source position rather than by its place in the parser's block order.
    block_spans: Vec<SourceSpan>,
    /// The bytes the preamble owns. Wider than `preamble.span`, which stops at
    /// its last block: this runs to the first heading, so the blank lines
    /// before that heading belong to the preamble the way a heading section's
    /// blank lines belong to it.
    preamble_region: std::ops::Range<u32>,
    revision: DocumentRevision,
}

/// A section resolved from one immutable document snapshot.
///
/// The entry and its source spans cannot be changed or constructed directly,
/// and edits verify that the originating document revision still matches.
#[derive(Clone, Debug)]
pub struct ResolvedSection {
    entry: SectionEntry,
    revision: DocumentRevision,
}

impl ResolvedSection {
    pub fn entry(&self) -> &SectionEntry {
        &self.entry
    }

    pub(crate) fn ensure_document(&self, document: &Document) -> Result<(), CoreError> {
        if self.revision == *document.revision() {
            Ok(())
        } else {
            Err(CoreError::DocumentRevisionMismatch {
                expected: self.revision.to_string(),
                actual: document.revision().to_string(),
            })
        }
    }

    pub(crate) fn into_entry(self) -> SectionEntry {
        self.entry
    }
}

impl std::ops::Deref for ResolvedSection {
    type Target = SectionEntry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl SectionIndex {
    pub fn new(document: &Document) -> Self {
        let preamble = build_preamble(document);
        let headings: Vec<SectionEntry> = document
            .blocks()
            .iter()
            .filter_map(|block| {
                let heading = block.heading.as_ref()?;
                let byte_end = find_section_byte_end(document, block.index, heading.level);
                let span = document.index().section_span(block.span, byte_end);
                let block_indices = document.index().section_block_indices(Some(block.index));

                Some(SectionEntry {
                    kind: SectionKind::Heading,
                    heading: Some(HeadingRef {
                        level: heading.level,
                        text: heading.text.clone(),
                        block_index: block.index,
                        span: block.span,
                    }),
                    // The occurrence is load-bearing, not decoration: a
                    // consumer that reads an entry straight out of the index
                    // (rather than through `resolve`, which overwrites the
                    // selector with the caller's own) can only re-address a
                    // duplicated heading if its ordinal travels with it.
                    selector: SectionTarget::heading(
                        heading.text.clone(),
                        Some(occurrence_of(document, &heading.text, block.index)),
                        HeadingMatchMode::Exact,
                    )
                    .expect("heading projection is a valid target")
                    .to_wire(),
                    depth: heading.level,
                    block_indices,
                    span,
                    etag: TargetEtag::for_bytes(document.slice_unchecked(&span).as_bytes())
                        .into_string(),
                })
            })
            .collect();

        let block_spans = document.blocks().iter().map(|block| block.span).collect();
        let preamble_region = document
            .frontmatter()
            .map(|frontmatter| frontmatter.span.byte_end)
            .unwrap_or(0)
            ..headings
                .first()
                .map(|section: &SectionEntry| section.span.byte_start)
                .unwrap_or(document.source().len() as u32);

        Self {
            preamble,
            headings,
            block_spans,
            preamble_region,
            revision: document.revision().clone(),
        }
    }

    pub fn outline(&self) -> Vec<OutlineEntry> {
        self.headings
            .iter()
            .map(|section| OutlineEntry {
                heading: section.heading.clone().expect("heading section"),
                section_span: section.span,
                etag: section.etag.clone(),
            })
            .collect()
    }

    pub fn all_etags(&self) -> Vec<String> {
        std::iter::once(self.preamble.etag.clone())
            .chain(self.headings.iter().map(|section| section.etag.clone()))
            .collect()
    }

    /// The innermost section owning a block.
    ///
    /// Defined by the block's source position, not by its place in the parser's
    /// block order: comrak emits footnote definitions after the blocks that
    /// reference them, so a definition sitting under one heading in the source
    /// is adjacent to a different heading in block order. Returns a plain
    /// [`SectionEntry`], not a [`ResolvedSection`]: this answers a read, it is
    /// not an edit handle.
    pub fn section_for_block(&self, block_index: u32) -> Option<SectionEntry> {
        let span = self.block_spans.get(block_index as usize)?;
        self.section_for_byte(span.byte_start)
    }

    /// The innermost section containing a byte offset.
    ///
    /// Heading sections nest, so the deepest containing one is the innermost.
    /// A blank line between blocks still sits inside the section around it,
    /// in the preamble as well as under a heading. Frontmatter bytes precede
    /// every section and resolve to `None`.
    pub fn section_for_byte(&self, byte_offset: u32) -> Option<SectionEntry> {
        self.headings
            .iter()
            .filter(|section| {
                section.span.byte_start <= byte_offset && byte_offset < section.span.byte_end
            })
            .max_by_key(|section| section.depth)
            .or_else(|| {
                self.preamble_region
                    .contains(&byte_offset)
                    .then_some(&self.preamble)
            })
            .cloned()
    }

    pub fn resolve(&self, target: &SectionTarget) -> Result<ResolvedSection, CoreError> {
        let entry = match target {
            SectionTarget::Preamble => self.preamble.clone(),
            SectionTarget::Heading { .. } => self.resolve_heading(target)?,
        };
        Ok(ResolvedSection {
            entry,
            revision: self.revision.clone(),
        })
    }

    fn resolve_heading(&self, target: &SectionTarget) -> Result<SectionEntry, CoreError> {
        let SectionTarget::Heading {
            text: heading_text,
            occurrence,
            match_mode,
        } = target
        else {
            unreachable!("resolve_heading called with preamble")
        };
        let matches = self
            .headings
            .iter()
            .filter(|section| {
                let actual = &section.heading.as_ref().expect("heading section").text;
                heading_matches(actual, heading_text, *match_mode)
            })
            .collect::<Vec<_>>();
        let match_refs = matches
            .iter()
            .enumerate()
            .map(|(index, section)| SectionMatch {
                block_index: section
                    .heading
                    .as_ref()
                    .expect("heading section")
                    .block_index,
                occurrence: (index + 1) as u32,
                line: section.span.line_start,
            })
            .collect::<Vec<_>>();

        if matches.is_empty() {
            return Err(CoreError::HeadingNotFound {
                heading: heading_text.to_string(),
            });
        }
        if matches.len() > 1 && occurrence.is_none() {
            return Err(CoreError::DuplicateHeading {
                heading: heading_text.to_string(),
                matches: match_refs,
            });
        }

        let selected = match occurrence {
            Some(occurrence) => matches
                .get((occurrence.get() - 1) as usize)
                .ok_or_else(|| CoreError::OccurrenceOutOfRange {
                    heading: heading_text.to_string(),
                    requested: occurrence.get(),
                    matches: match_refs,
                })?,
            None => matches[0],
        };
        let mut result = (*selected).clone();
        result.selector = target.to_wire();
        Ok(result)
    }
}

pub(crate) fn resolve_address(
    document: &Document,
    address: &crate::target::SectionAddress,
) -> Result<ResolvedSection, CoreError> {
    let target_address = match address {
        crate::target::SectionAddress::Preamble => crate::target::TargetAddress::Preamble,
        crate::target::SectionAddress::Heading { path } => {
            crate::target::TargetAddress::Section { path: path.clone() }
        }
    };
    let resolved = document.resolve(&target_address)?;
    let entry = match address {
        crate::target::SectionAddress::Preamble => SectionIndex::new(document).preamble,
        crate::target::SectionAddress::Heading { .. } => SectionIndex::new(document)
            .section_for_byte(resolved.snapshot().selection_span.unwrap().byte_start)
            .ok_or_else(|| CoreError::TargetNotFound {
                target: target_address.to_string(),
            })?,
    };
    Ok(ResolvedSection {
        entry,
        revision: document.revision().clone(),
    })
}

/// 1-based position of a heading block among the exact-text matches for its
/// own text, matching how `resolve_heading` counts occurrences.
fn occurrence_of(document: &Document, text: &str, block_index: u32) -> u32 {
    document
        .blocks()
        .iter()
        .filter_map(|block| {
            block
                .heading
                .as_ref()
                .filter(|heading| heading.text == text)
                .map(|_| block.index)
        })
        .position(|index| index == block_index)
        .map(|index| index as u32 + 1)
        .unwrap_or(1)
}

fn build_preamble(document: &Document) -> SectionEntry {
    let block_indices = document.index().section_block_indices(None);
    // Bounds by source position, not by position in the block vec: comrak
    // emits footnote definitions after the blocks referencing them, so
    // first/last in vec order can yield byte_start > byte_end and panic the
    // slice below.
    let bounds = block_indices
        .iter()
        .map(|index| document.blocks()[*index as usize].span)
        .reduce(|left, right| SourceSpan {
            line_start: left.line_start.min(right.line_start),
            line_end: left.line_end.max(right.line_end),
            byte_start: left.byte_start.min(right.byte_start),
            byte_end: left.byte_end.max(right.byte_end),
        });
    let span = match bounds {
        Some(span) => span,
        None => {
            let byte_start = document
                .frontmatter()
                .map(|frontmatter| frontmatter.span.byte_end)
                .unwrap_or(0);
            let line = document
                .blocks()
                .first()
                .map(|block| block.span.line_start)
                .unwrap_or(1);
            SourceSpan {
                line_start: line,
                line_end: line,
                byte_start,
                byte_end: byte_start,
            }
        }
    };

    SectionEntry {
        kind: SectionKind::Preamble,
        heading: None,
        selector: SectionSelector {
            kind: SectionSelectorKind::Preamble,
            heading_text: None,
            occurrence: None,
            match_mode: HeadingMatchMode::Exact,
        },
        depth: 0,
        block_indices,
        span,
        etag: TargetEtag::for_bytes(document.slice_unchecked(&span).as_bytes()).into_string(),
    }
}

fn find_section_byte_end(document: &Document, heading_index: u32, level: u8) -> u32 {
    document
        .blocks()
        .iter()
        .skip((heading_index + 1) as usize)
        .find_map(|block| {
            block
                .heading
                .as_ref()
                .filter(|heading| heading.level <= level)
                .map(|_| block.span.byte_start)
        })
        .unwrap_or(document.source().len() as u32)
}

fn heading_matches(actual: &str, expected: &str, mode: HeadingMatchMode) -> bool {
    match mode {
        HeadingMatchMode::Exact => actual == expected,
        HeadingMatchMode::ExactIgnoreCase => actual.to_lowercase() == expected.to_lowercase(),
        HeadingMatchMode::Contains => actual.contains(expected),
        HeadingMatchMode::ContainsIgnoreCase => {
            actual.to_lowercase().contains(&expected.to_lowercase())
        }
    }
}
