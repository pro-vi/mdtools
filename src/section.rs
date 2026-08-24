use std::num::NonZeroU32;

use crate::core_error::{CoreError, SectionMatch};
use crate::document::Document;
use crate::fingerprint::content_etag;
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
        let headings = document
            .blocks()
            .iter()
            .filter_map(|block| {
                let heading = block.heading.as_ref()?;
                let byte_end = find_section_byte_end(document, block.index, heading.level);
                let span = SourceSpan {
                    line_start: block.span.line_start,
                    line_end: section_line_end(document, byte_end),
                    byte_start: block.span.byte_start,
                    byte_end,
                };
                let block_indices = document
                    .blocks()
                    .iter()
                    .skip(block.index as usize)
                    .take_while(|candidate| {
                        candidate.index == block.index
                            || candidate
                                .heading
                                .as_ref()
                                .is_none_or(|next| next.level > heading.level)
                    })
                    .map(|candidate| candidate.index)
                    .collect();

                Some(SectionEntry {
                    kind: SectionKind::Heading,
                    heading: Some(HeadingRef {
                        level: heading.level,
                        text: heading.text.clone(),
                        block_index: block.index,
                        span: block.span,
                    }),
                    selector: SectionTarget::heading(
                        heading.text.clone(),
                        None,
                        HeadingMatchMode::Exact,
                    )
                    .expect("heading projection is a valid target")
                    .to_wire(),
                    depth: heading.level,
                    block_indices,
                    span,
                    etag: content_etag(document.slice(&span).as_bytes()),
                })
            })
            .collect();

        Self {
            preamble,
            headings,
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

fn build_preamble(document: &Document) -> SectionEntry {
    let block_indices = document
        .blocks()
        .iter()
        .take_while(|block| block.heading.is_none())
        .map(|block| block.index)
        .collect::<Vec<_>>();
    let span = match (block_indices.first(), block_indices.last()) {
        (Some(first), Some(last)) => SourceSpan {
            line_start: document.blocks()[*first as usize].span.line_start,
            line_end: document.blocks()[*last as usize].span.line_end,
            byte_start: document.blocks()[*first as usize].span.byte_start,
            byte_end: document.blocks()[*last as usize].span.byte_end,
        },
        _ => {
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
        etag: content_etag(document.slice(&span).as_bytes()),
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

fn section_line_end(document: &Document, byte_end: u32) -> u32 {
    if byte_end as usize >= document.source().len() {
        return document.line_count();
    }
    let line_at_end = document.byte_to_line(byte_end);
    if byte_end > 0 && document.source().as_bytes().get(byte_end as usize - 1) == Some(&b'\n') {
        line_at_end - 1
    } else {
        line_at_end
    }
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
