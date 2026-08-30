//! Closed headed-section fragment values and their validated semantic form.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core_error::CoreError;
use crate::document::Document;
use crate::edit::normalize_line_endings;
use crate::model::{LineEndingStyle, SourceSpan};
use crate::parser::HeadingSourceKind;
use crate::target::{TargetAddress, TargetKind};

/// A headed-section payload with explicit semantic or literal behavior.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum SectionFragment {
    /// One relative section subtree. Headings are rebased at placement time.
    Semantic { markdown: String },
    /// Exact caller bytes. No rebasing, trimming, or line-ending conversion.
    Literal { markdown: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PreparedMode {
    Semantic,
    Literal(String),
}

/// A validated one-root section subtree used only inside the patch planner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedSectionFragment {
    mode: PreparedMode,
    canonical: String,
    source_root_level: u8,
}

impl SectionFragment {
    pub(crate) fn prepare(&self) -> Result<PreparedSectionFragment, CoreError> {
        match self {
            Self::Semantic { markdown } => {
                let (canonical, source_root_level) = canonicalize(markdown)?;
                Ok(PreparedSectionFragment {
                    mode: PreparedMode::Semantic,
                    canonical,
                    source_root_level,
                })
            }
            Self::Literal { markdown } => {
                let (canonical, source_root_level) = canonicalize(markdown)?;
                Ok(PreparedSectionFragment {
                    mode: PreparedMode::Literal(markdown.clone()),
                    canonical,
                    source_root_level,
                })
            }
        }
    }

    pub(crate) fn from_placed_section(
        document: &Document,
        span: SourceSpan,
    ) -> Result<Self, CoreError> {
        let (markdown, _) = canonicalize(document.slice(&span)?)?;
        Ok(Self::Semantic { markdown })
    }
}

impl PreparedSectionFragment {
    pub(crate) fn canonical(&self) -> &str {
        &self.canonical
    }

    pub(crate) fn is_semantic(&self) -> bool {
        matches!(self.mode, PreparedMode::Semantic)
    }

    pub(crate) fn rendered_root_level(&self, parent_level: u8) -> Result<u8, CoreError> {
        match self.mode {
            PreparedMode::Semantic => checked_absolute_level(parent_level, 1),
            PreparedMode::Literal(_) => Ok(self.source_root_level),
        }
    }

    pub(crate) fn render(
        &self,
        parent_level: u8,
        line_endings: LineEndingStyle,
    ) -> Result<String, CoreError> {
        match &self.mode {
            PreparedMode::Literal(markdown) => Ok(markdown.clone()),
            PreparedMode::Semantic => {
                let document = Document::parse_fragment(self.canonical.clone())?;
                let mut rendered = self.canonical.clone();
                let mut edits = document
                    .index()
                    .source_block_indices()
                    .into_iter()
                    .filter_map(|index| {
                        let block = &document.blocks()[index as usize];
                        block.heading.as_ref().map(|heading| {
                            Ok((
                                heading.marker_span.byte_start as usize,
                                heading.marker_span.byte_end as usize,
                                checked_absolute_level(parent_level, heading.level)?,
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>, CoreError>>()?;
                edits.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
                for (start, end, level) in edits {
                    rendered.replace_range(start..end, &"#".repeat(level as usize));
                }
                Ok(normalize_line_endings(&rendered, line_endings))
            }
        }
    }
}

fn checked_absolute_level(parent_level: u8, relative_level: u8) -> Result<u8, CoreError> {
    let level = parent_level.saturating_add(relative_level);
    if level > 6 {
        Err(CoreError::HeadingDepthOverflow {
            parent_level,
            relative_level,
        })
    } else {
        Ok(level)
    }
}

fn canonicalize(source: &str) -> Result<(String, u8), CoreError> {
    if source.is_empty() {
        return Err(invalid_fragment("section fragment cannot be empty"));
    }
    let document = Document::parse_fragment(source.to_string())?;
    let sections = document
        .map()?
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Section)
        .collect::<Vec<_>>();
    let roots = sections
        .iter()
        .filter(|snapshot| {
            matches!(&snapshot.address, TargetAddress::Section { path } if path.len() == 1)
        })
        .collect::<Vec<_>>();
    let [root] = roots.as_slice() else {
        return Err(invalid_fragment(
            "semantic section fragment must contain exactly one root section",
        ));
    };
    let root_span = root.selection_span.ok_or_else(|| {
        CoreError::PatchInvariant("fragment root section has no selection span".into())
    })?;
    if !source[..root_span.byte_start as usize]
        .chars()
        .all(is_markdown_boundary_whitespace)
    {
        return Err(invalid_fragment(
            "section fragment cannot contain non-whitespace before its root heading",
        ));
    }
    let semantic_end = last_content_line_end(
        source,
        root_span.byte_start as usize,
        root_span.byte_end as usize,
    );
    let mut headings = document
        .index()
        .source_block_indices()
        .into_iter()
        .filter_map(|index| {
            let block = &document.blocks()[index as usize];
            block
                .heading
                .as_ref()
                .filter(|_| {
                    block.span.byte_start >= root_span.byte_start
                        && block.span.byte_end <= root_span.byte_end
                })
                .map(|heading| (block.span, heading))
        })
        .collect::<Vec<_>>();
    headings.sort_by_key(|(span, _)| span.byte_start);
    let Some((_, root_heading)) = headings.first() else {
        return Err(invalid_fragment("section fragment has no root heading"));
    };
    let root_level = root_heading.level;
    let root_start = root_span.byte_start as usize;
    let mut canonical = source[root_start..semantic_end].to_string();
    let mut edits = Vec::with_capacity(headings.len());
    for (span, heading) in headings {
        let relative_level = heading
            .level
            .checked_sub(root_level)
            .and_then(|level| level.checked_add(1))
            .ok_or_else(|| invalid_fragment("fragment heading escapes its root section"))?;
        let start = span.byte_start as usize - root_start;
        match heading.kind {
            HeadingSourceKind::Atx => edits.push((
                start,
                heading.marker_span.byte_end as usize - root_start,
                "#".repeat(relative_level as usize),
            )),
            HeadingSourceKind::Setext => {
                let content = setext_heading_content(
                    source,
                    span.byte_start as usize,
                    heading.marker_span.byte_start as usize,
                );
                edits.push((
                    start,
                    span.byte_end as usize - root_start,
                    format!("{} {content}", "#".repeat(relative_level as usize)),
                ));
            }
        }
    }
    edits.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
    for (start, end, replacement) in edits {
        canonical.replace_range(start..end, &replacement);
    }
    Ok((canonical.replace("\r\n", "\n"), root_level))
}

fn setext_heading_content(source: &str, start: usize, marker_start: usize) -> String {
    source[start..marker_start]
        .lines()
        .map(|line| line.trim_matches([' ', '\t', '\r']))
        .collect::<Vec<_>>()
        .join(" ")
}

fn last_content_line_end(source: &str, start: usize, end: usize) -> usize {
    let bytes = source.as_bytes();
    let mut line_start = start;
    let mut last_content_end = start;
    let mut position = start;
    while position <= end {
        let at_end = position == end;
        let at_newline = !at_end && bytes[position] == b'\n';
        if at_end || at_newline {
            let mut content_end = position;
            if content_end > line_start && bytes[content_end - 1] == b'\r' {
                content_end -= 1;
            }
            if source[line_start..content_end]
                .chars()
                .any(|character| !is_markdown_blank_line_content(character))
            {
                last_content_end = content_end;
            }
            line_start = position.saturating_add(1);
        }
        if at_end {
            break;
        }
        position += 1;
    }
    last_content_end
}

fn is_markdown_boundary_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\r' | '\n')
}

fn is_markdown_blank_line_content(character: char) -> bool {
    matches!(character, ' ' | '\t')
}

fn invalid_fragment(reason: impl Into<String>) -> CoreError {
    CoreError::InvalidPatch(reason.into())
}
