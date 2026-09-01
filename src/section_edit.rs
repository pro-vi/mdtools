use crate::core_error::CoreError;
use crate::document::Document;
use crate::edit::SourceEdit;
use crate::index::{DocumentIndex, IndexNode};
use crate::model::{BlockKind, InsertMode, LineEndingStyle, SourceSpan};
use crate::parser::HeadingSourceKind;
use crate::section::{SectionPlanEntry, SectionPlanTarget};

pub(crate) struct PlannedSectionMove {
    pub(crate) edits: Vec<SourceEdit>,
    pub(crate) result_edit: usize,
    pub(crate) result_range: std::ops::Range<usize>,
}

pub(crate) fn plan_section_move(
    document: &Document,
    source: SectionPlanTarget,
    destination: SectionPlanTarget,
    destination_mode: InsertMode,
    keep_level: bool,
) -> Result<PlannedSectionMove, CoreError> {
    source.ensure_document(document)?;
    destination.ensure_document(document)?;
    let source = source.into_entry();
    let destination = destination.into_entry();
    let source_span = source.span;
    let destination_span = destination.span;
    let insert_byte = match destination_mode {
        InsertMode::AfterSibling | InsertMode::IntoAsChild => destination_span.byte_end,
        InsertMode::BeforeSibling => destination_span.byte_start,
    };
    let destination_level = destination.heading_level.ok_or_else(|| {
        CoreError::InvalidSelector("destination must be a heading section, not the preamble".into())
    })?;
    let source_level = source.heading_level.ok_or_else(|| {
        CoreError::InvalidSelector("source must be a heading section, not the preamble".into())
    })?;
    let destination_inside_source = destination_span.byte_start >= source_span.byte_start
        && destination_span.byte_end <= source_span.byte_end;
    let source_inside_destination = source_span.byte_start >= destination_span.byte_start
        && source_span.byte_end <= destination_span.byte_end;
    if destination_inside_source {
        return Err(CoreError::InvalidSelector(
            "cannot move section: destination is inside source".into(),
        ));
    }
    if source_inside_destination
        && matches!(
            destination_mode,
            InsertMode::AfterSibling | InsertMode::IntoAsChild
        )
    {
        return Err(CoreError::InvalidSelector(
            "cannot move section: destination contains source; insert position is ambiguous".into(),
        ));
    }
    let new_level = match destination_mode {
        InsertMode::AfterSibling | InsertMode::BeforeSibling => destination_level,
        InsertMode::IntoAsChild => destination_level + 1,
    };
    let delta = if keep_level {
        0
    } else {
        new_level as i32 - source_level as i32
    };
    if delta != 0 {
        validate_relevel(document.index(), &source, delta)?;
    }
    let mut moved = document.slice_unchecked(&source_span).to_string();
    if delta != 0 {
        moved = crate::fragment::rebase_section_headings(&moved, new_level)?;
    }
    let separator = if document.line_ending_style() == LineEndingStyle::Crlf {
        "\r\n"
    } else {
        "\n"
    };
    let source_start = source_span.byte_start;
    let source_end = source_span.byte_end;
    let content_follows = if insert_byte <= source_start {
        insert_byte < source_start || (source_end as usize) < document.source().len()
    } else {
        (insert_byte as usize) < document.source().len()
    };
    if content_follows && !moved.ends_with('\n') {
        moved.push_str(separator);
    }
    if content_follows
        && following_setext_heading(
            document.index(),
            document.source(),
            insert_byte,
            source_start,
            source_end,
        )
        .is_some()
    {
        let trailing = count_trailing_line_breaks(moved.as_bytes(), moved.len(), 0).0;
        let last_kind = source
            .block_nodes
            .last()
            .and_then(|node| document.index().source_block_kind(*node))
            .unwrap_or(BlockKind::Paragraph);
        moved.push_str(
            &separator.repeat(setext_boundary_breaks(last_kind).saturating_sub(trailing)),
        );
    }
    let starts_setext = source.block_nodes.first().is_some_and(|node| {
        document.index().heading_source_kind(*node) == Some(HeadingSourceKind::Setext)
    });
    let (walk_start, lower_bound) = if insert_byte <= source_start {
        (insert_byte as usize, 0)
    } else if insert_byte == source_end {
        (source_start as usize, 0)
    } else {
        (insert_byte as usize, source_end as usize)
    };
    let (preceding_breaks, has_preceding) =
        count_trailing_line_breaks(document.source().as_bytes(), walk_start, lower_bound);
    let moved_breaks = count_leading_line_breaks(moved.as_bytes());
    let leading_count = if starts_setext {
        preceding_block_kind(document, source_span, insert_byte)
            .map(setext_boundary_breaks)
            .unwrap_or(0)
            .saturating_sub(preceding_breaks + moved_breaks)
    } else if has_preceding {
        1usize.saturating_sub(preceding_breaks + moved_breaks)
    } else {
        0
    };
    let leading = separator.repeat(leading_count);
    let mut insertion = String::with_capacity(leading.len() + moved.len());
    insertion.push_str(&leading);
    let result_start = insertion.len();
    insertion.push_str(&moved);
    let result_end = insertion.len();
    Ok(PlannedSectionMove {
        edits: vec![
            SourceEdit {
                start: source_start as usize,
                end: source_end as usize,
                replacement: String::new(),
            },
            SourceEdit {
                start: insert_byte as usize,
                end: insert_byte as usize,
                replacement: insertion,
            },
        ],
        result_edit: 1,
        result_range: result_start..result_end,
    })
}

fn preceding_block_kind(
    document: &Document,
    source: SourceSpan,
    insert_byte: u32,
) -> Option<BlockKind> {
    document
        .index()
        .source_blocks()
        .filter_map(|entry| {
            let span = entry.node.span();
            let inside_source =
                span.byte_start >= source.byte_start && span.byte_end <= source.byte_end;
            (!inside_source && span.byte_start < insert_byte)
                .then(|| {
                    document
                        .index()
                        .source_block_kind(entry.id)
                        .map(|kind| (span, kind))
                })
                .flatten()
        })
        .max_by_key(|(span, _)| span.byte_start)
        .map(|(_, kind)| kind)
}

fn setext_boundary_breaks(kind: BlockKind) -> usize {
    match kind {
        BlockKind::Heading | BlockKind::CodeFence | BlockKind::ThematicBreak => 1,
        _ => 2,
    }
}

fn validate_relevel(
    index: &DocumentIndex,
    source: &SectionPlanEntry,
    delta: i32,
) -> Result<(), CoreError> {
    for node in &source.block_nodes {
        if let IndexNode::Heading {
            span, level, text, ..
        } = &index.entry(*node).node
        {
            if index.heading_source_kind(*node) == Some(HeadingSourceKind::Setext) {
                return Err(CoreError::InvalidSelector(format!(
                    "setext heading {:?} (line {}) cannot be re-leveled; convert to ATX (## {}) first or use --keep-level",
                    text, span.line_start, text
                )));
            }
            let new_level = *level as i32 + delta;
            if !(1..=6).contains(&new_level) {
                return Err(CoreError::InvalidSelector(format!(
                    "cannot move section: descendant {:?} would land at heading level {} (max is 6)",
                    text, new_level
                )));
            }
        }
    }
    Ok(())
}

fn following_setext_heading(
    index: &DocumentIndex,
    source: &str,
    insert: u32,
    source_start: u32,
    source_end: u32,
) -> Option<(String, u8)> {
    let following = if insert == source_start {
        source_end
    } else {
        insert
    };
    if following as usize >= source.len() {
        return None;
    }
    index.source_blocks().find_map(|entry| {
        let IndexNode::Heading {
            span, text, level, ..
        } = &entry.node
        else {
            return None;
        };
        (index.heading_source_kind(entry.id) == Some(HeadingSourceKind::Setext)
            && line_start(source.as_bytes(), span.byte_start as usize) == following as usize)
            .then(|| (text.clone(), *level))
    })
}

#[allow(clippy::too_many_arguments)]
fn count_trailing_line_breaks(bytes: &[u8], start: usize, lower: usize) -> (usize, bool) {
    let mut count = 0;
    let mut position = start;
    while position > lower && bytes[position - 1] == b'\n' {
        count += 1;
        position -= 1;
        if position > lower && bytes[position - 1] == b'\r' {
            position -= 1;
        }
    }
    (count, position > 0)
}

fn count_leading_line_breaks(bytes: &[u8]) -> usize {
    let mut count = 0;
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'\n' {
            count += 1;
            position += 1;
        } else if position + 1 < bytes.len()
            && bytes[position] == b'\r'
            && bytes[position + 1] == b'\n'
        {
            count += 1;
            position += 2;
        } else {
            break;
        }
    }
    count
}

fn line_start(bytes: &[u8], position: usize) -> usize {
    if position >= bytes.len() {
        return bytes.len();
    }
    let mut start = position;
    while start > 0 && bytes[start - 1] != b'\n' {
        start -= 1;
    }
    start
}
