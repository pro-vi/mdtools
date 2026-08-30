use std::collections::HashSet;

use crate::block_edit::GuardRole;
use crate::core_error::{CoreError, EtagTarget};
use crate::document::Document;
use crate::edit::{
    normalize_line_endings, replacement_span_after, EditOutcome, EditPreservation, SourceEdit,
};
use crate::fingerprint::{TargetEtag, TargetEtagGuard};
use crate::model::{
    BlockKind, InsertMode, LineEndingStyle, MutationDisposition, SectionEntry, SourceSpan,
};
use crate::parser::{HeadingSourceKind, ParsedDocument};
use crate::section::{ResolvedSection, SectionIndex};

#[derive(Clone, Debug)]
pub enum SectionEditTarget {
    Section(SectionEntry),
    Move {
        source: SectionEntry,
        destination: SectionEntry,
        destination_mode: InsertMode,
        level_shift_applied: i8,
    },
}

pub struct PreparedSectionReplace<'a> {
    document: &'a Document,
    section: SectionEntry,
    guarded: bool,
}

pub(crate) struct PlannedSectionMove {
    pub(crate) edits: Vec<SourceEdit>,
    pub(crate) result_edit: usize,
    pub(crate) result_range: std::ops::Range<usize>,
}

pub(crate) fn plan_section_move(
    document: &Document,
    source: ResolvedSection,
    destination: ResolvedSection,
    destination_mode: InsertMode,
    keep_level: bool,
) -> Result<PlannedSectionMove, CoreError> {
    source.ensure_document(document)?;
    destination.ensure_document(document)?;
    let source = source.into_entry();
    let destination = destination.into_entry();
    let parsed = document.projection();
    let source_span = source.span;
    let destination_span = destination.span;
    let insert_byte = match destination_mode {
        InsertMode::AfterSibling | InsertMode::IntoAsChild => destination_span.byte_end,
        InsertMode::BeforeSibling => destination_span.byte_start,
    };
    let destination_level = destination
        .heading
        .as_ref()
        .map(|heading| heading.level)
        .ok_or_else(|| {
            CoreError::InvalidSelector(
                "destination must be a heading section, not the preamble".into(),
            )
        })?;
    let source_level = source
        .heading
        .as_ref()
        .map(|heading| heading.level)
        .ok_or_else(|| {
            CoreError::InvalidSelector("source must be a heading section, not the preamble".into())
        })?;
    let destination_inside_source = destination_span.byte_start >= source_span.byte_start
        && destination_span.byte_end <= source_span.byte_end;
    let source_inside_destination = source_span.byte_start >= destination_span.byte_start
        && source_span.byte_end <= destination_span.byte_end;
    if destination_inside_source {
        return Err(CoreError::InvalidSelector(
            "cannot move-section: destination is inside source".into(),
        ));
    }
    if source_inside_destination
        && matches!(
            destination_mode,
            InsertMode::AfterSibling | InsertMode::IntoAsChild
        )
    {
        return Err(CoreError::InvalidSelector(
            "cannot move-section: destination contains source; insert position is ambiguous".into(),
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
        validate_relevel(parsed, &source, delta)?;
    }
    let mut moved = document.slice_unchecked(&source_span).to_string();
    if delta != 0 {
        moved = rewrite_atx_levels(moved, &source, parsed, source_span.byte_start, delta as i8)?;
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
        && following_setext_heading(parsed, insert_byte, source_start, source_end).is_some()
    {
        let trailing = count_trailing_line_breaks(moved.as_bytes(), moved.len(), 0).0;
        let last_kind = source
            .block_indices
            .last()
            .map(|index| document.blocks()[*index as usize].kind)
            .unwrap_or(BlockKind::Paragraph);
        moved.push_str(
            &separator.repeat(setext_boundary_breaks(last_kind).saturating_sub(trailing)),
        );
    }
    let starts_setext = parsed
        .blocks
        .get(source.block_indices[0] as usize)
        .and_then(|block| block.heading.as_ref())
        .is_some_and(|heading| heading.kind == HeadingSourceKind::Setext);
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
        .source_block_indices()
        .into_iter()
        .filter_map(|index| {
            let block = &document.blocks()[index as usize];
            let inside_source = block.span.byte_start >= source.byte_start
                && block.span.byte_end <= source.byte_end;
            (!inside_source && block.span.byte_start < insert_byte).then_some(block)
        })
        .max_by_key(|block| block.span.byte_start)
        .map(|block| block.kind)
}

fn setext_boundary_breaks(kind: BlockKind) -> usize {
    match kind {
        BlockKind::Heading | BlockKind::CodeFence | BlockKind::ThematicBreak => 1,
        _ => 2,
    }
}

pub fn prepare_replace<'a>(
    document: &'a Document,
    section: ResolvedSection,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<PreparedSectionReplace<'a>, CoreError> {
    section.ensure_document(document)?;
    let section = section.into_entry();
    verify_guard(document, &section, expect_etag, None)?;
    Ok(PreparedSectionReplace {
        document,
        section,
        guarded: expect_etag.is_some(),
    })
}

impl PreparedSectionReplace<'_> {
    pub fn apply(self, replacement: impl Into<String>) -> EditOutcome<SectionEditTarget> {
        let span = self.section.span;
        let replacement =
            normalize_line_endings(&replacement.into(), self.document.line_ending_style());
        let replacement = preserve_following_boundary(
            self.document.slice_unchecked(&span),
            &replacement,
            (span.byte_end as usize) < self.document.source().len(),
        );
        let disposition = if replacement == self.document.slice_unchecked(&span) {
            MutationDisposition::NoChange
        } else if replacement.is_empty() {
            MutationDisposition::Deleted
        } else {
            MutationDisposition::Replaced
        };
        let content = format!(
            "{}{}{}",
            &self.document.source()[..span.byte_start as usize],
            replacement,
            &self.document.source()[span.byte_end as usize..]
        );
        let after = match disposition {
            MutationDisposition::Deleted => None,
            MutationDisposition::NoChange => Some(span),
            MutationDisposition::Replaced => Some(replacement_span_after(span, &replacement)),
            MutationDisposition::Inserted => unreachable!(),
        };
        outcome(
            self.document,
            SectionEditTarget::Section(self.section),
            disposition,
            self.guarded,
            EditPreservation {
                preserves_non_target_bytes: true,
                target_span_before: Some(span),
                target_span_after: after,
            },
            content,
        )
    }
}

pub fn delete(
    document: &Document,
    section: ResolvedSection,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<EditOutcome<SectionEditTarget>, CoreError> {
    section.ensure_document(document)?;
    let section = section.into_entry();
    verify_guard(document, &section, expect_etag, None)?;
    let span = section.span;
    let content = format!(
        "{}{}",
        &document.source()[..span.byte_start as usize],
        &document.source()[span.byte_end as usize..]
    );
    Ok(outcome(
        document,
        SectionEditTarget::Section(section),
        MutationDisposition::Deleted,
        expect_etag.is_some(),
        EditPreservation {
            preserves_non_target_bytes: true,
            target_span_before: Some(span),
            target_span_after: None,
        },
        content,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn move_section(
    document: &Document,
    source: ResolvedSection,
    destination: ResolvedSection,
    destination_mode: InsertMode,
    keep_level: bool,
    expect_source_etag: Option<&TargetEtagGuard>,
    expect_destination_etag: Option<&TargetEtagGuard>,
) -> Result<EditOutcome<SectionEditTarget>, CoreError> {
    source.ensure_document(document)?;
    destination.ensure_document(document)?;
    let source = source.into_entry();
    let destination = destination.into_entry();
    verify_guard(
        document,
        &source,
        expect_source_etag,
        Some(GuardRole::Source),
    )?;
    verify_guard(
        document,
        &destination,
        expect_destination_etag,
        Some(GuardRole::Destination),
    )?;
    let parsed = document.projection();
    let source_span = source.span;
    let destination_span = destination.span;
    let insert_byte = match destination_mode {
        InsertMode::AfterSibling | InsertMode::IntoAsChild => destination_span.byte_end,
        InsertMode::BeforeSibling => destination_span.byte_start,
    };
    let destination_level = destination
        .heading
        .as_ref()
        .map(|heading| heading.level)
        .ok_or_else(|| {
            CoreError::InvalidSelector(
                "destination must be a heading section, not the preamble".into(),
            )
        })?;
    let source_level = source
        .heading
        .as_ref()
        .map(|heading| heading.level)
        .ok_or_else(|| {
            CoreError::InvalidSelector("source must be a heading section, not the preamble".into())
        })?;
    let new_level = match destination_mode {
        InsertMode::AfterSibling | InsertMode::BeforeSibling => destination_level,
        InsertMode::IntoAsChild => destination_level + 1,
    };
    let destination_inside_source = destination_span.byte_start >= source_span.byte_start
        && destination_span.byte_end <= source_span.byte_end;
    let source_inside_destination = source_span.byte_start >= destination_span.byte_start
        && source_span.byte_end <= destination_span.byte_end;
    if destination_inside_source {
        return Err(CoreError::InvalidSelector(
            "cannot move-section: destination is inside source".into(),
        ));
    }
    if source_inside_destination
        && matches!(
            destination_mode,
            InsertMode::AfterSibling | InsertMode::IntoAsChild
        )
    {
        return Err(CoreError::InvalidSelector(
            "cannot move-section: destination contains source; insert position is ambiguous".into(),
        ));
    }
    let delta = if keep_level {
        0
    } else {
        new_level as i32 - source_level as i32
    };
    if delta != 0 {
        validate_relevel(parsed, &source, delta)?;
    }
    let delta = delta as i8;
    let separator = if document.line_ending_style() == LineEndingStyle::Crlf {
        "\r\n"
    } else {
        "\n"
    };
    let mut moved = document.source()
        [source_span.byte_start as usize..source_span.byte_end as usize]
        .to_string();
    if delta != 0 {
        moved = rewrite_atx_levels(moved, &source, parsed, source_span.byte_start, delta)?;
    }
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
    if content_follows {
        if let Some((text, level)) =
            following_setext_heading(parsed, insert_byte, source_start, source_end)
        {
            let extra = choose_trailing_separators(
                document,
                document.source(),
                &moved,
                separator,
                insert_byte,
                source_start,
                source_end,
                &text,
                level,
            );
            moved.push_str(&separator.repeat(extra));
        }
    }
    let starts_setext = parsed
        .blocks
        .get(source.block_indices[0] as usize)
        .and_then(|block| block.heading.as_ref())
        .is_some_and(|heading| heading.kind == HeadingSourceKind::Setext);
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
    let source_heading = source
        .heading
        .as_ref()
        .map(|heading| heading.text.clone())
        .unwrap_or_default();
    let leading_count = if starts_setext {
        choose_setext_leading_separators(
            document,
            document.source(),
            &moved,
            separator,
            insert_byte,
            source_start,
            source_end,
            &source_heading,
            source_level,
        )
    } else if has_preceding {
        1usize.saturating_sub(preceding_breaks + moved_breaks)
    } else {
        0
    };
    let leading = separator.repeat(leading_count);
    let (content, moved_start) = splice(
        document.source(),
        &moved,
        &leading,
        insert_byte,
        source_start,
        source_end,
    );
    let expected_level = (source_level as i32 + delta as i32) as u8;
    if !moved_section_reparses_at(
        document,
        &content,
        moved_start as usize,
        moved.len(),
        &source_heading,
        expected_level,
    ) {
        return Err(CoreError::InvalidSelector(
            "cannot move-section: moved section would absorb or lose adjacent headings; use --auto-level or choose a destination that preserves the section boundary".into(),
        ));
    }
    verify_unmoved_structure(document, &content, &source, moved_start, moved.len())?;
    let disposition = if content == document.source() {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let after = if disposition == MutationDisposition::NoChange {
        source_span
    } else {
        moved_span_after(&content, moved_start, moved.len())
    };
    Ok(outcome(
        document,
        SectionEditTarget::Move {
            source,
            destination,
            destination_mode,
            level_shift_applied: delta,
        },
        disposition,
        expect_source_etag.is_some() || expect_destination_etag.is_some(),
        EditPreservation {
            preserves_non_target_bytes: false,
            target_span_before: Some(source_span),
            target_span_after: Some(after),
        },
        content,
    ))
}

fn verify_unmoved_structure(
    document: &Document,
    output: &str,
    source: &SectionEntry,
    moved_start: u32,
    moved_len: usize,
) -> Result<(), CoreError> {
    let reparsed = document.reparse(output)?;
    let source_indices = source.block_indices.iter().copied().collect::<HashSet<_>>();
    let moved_end = moved_start + moved_len as u32;
    let original = document
        .blocks()
        .iter()
        .filter(|block| !source_indices.contains(&block.index))
        .map(|block| (block.kind, document.slice_unchecked(&block.span)))
        .collect::<Vec<_>>();
    let moved_block_count = reparsed
        .blocks()
        .iter()
        .filter(|block| block.span.byte_start >= moved_start && block.span.byte_start < moved_end)
        .count();
    let remaining = reparsed
        .blocks()
        .iter()
        .filter(|block| {
            !(block.span.byte_start >= moved_start && block.span.byte_start < moved_end)
        })
        .map(|block| (block.kind, reparsed.slice_unchecked(&block.span)))
        .collect::<Vec<_>>();
    if moved_block_count != source.block_indices.len() || remaining != original {
        return Err(CoreError::InvalidSelector(
            "cannot move-section: relocation would change blocks outside the source section".into(),
        ));
    }
    Ok(())
}

fn verify_guard(
    document: &Document,
    section: &SectionEntry,
    expected: Option<&TargetEtagGuard>,
    role: Option<GuardRole>,
) -> Result<(), CoreError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = TargetEtag::for_bytes(document.slice_unchecked(&section.span).as_bytes());
    if expected.as_str() != actual.as_str() {
        if let Some(role) = role {
            return Err(CoreError::SectionMoveEtagMismatch {
                role,
                selector: describe_selector(section),
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        return Err(CoreError::TargetEtagMismatch {
            target: EtagTarget::Section(describe_selector(section)),
            expected: expected.to_string(),
            actual: actual.to_string(),
        });
    }
    let count = SectionIndex::new(document)
        .all_etags()
        .into_iter()
        .filter(|etag| etag == expected.as_str())
        .count();
    if count > 1 {
        if let Some(role) = role {
            Err(CoreError::SectionMoveEtagAmbiguous {
                role,
                expected: expected.to_string(),
                count,
            })
        } else {
            Err(CoreError::TargetEtagAmbiguous {
                target_kind: "section",
                expected: expected.to_string(),
                count,
            })
        }
    } else {
        Ok(())
    }
}

fn validate_relevel(
    parsed: &ParsedDocument,
    source: &SectionEntry,
    delta: i32,
) -> Result<(), CoreError> {
    for index in &source.block_indices {
        if let Some(heading) = &parsed.blocks[*index as usize].heading {
            if heading.kind == HeadingSourceKind::Setext {
                return Err(CoreError::InvalidSelector(format!(
                    "setext heading {:?} (line {}) cannot be re-leveled; convert to ATX (## {}) first or use --keep-level",
                    heading.text, parsed.blocks[*index as usize].span.line_start, heading.text
                )));
            }
            let level = heading.level as i32 + delta;
            if !(1..=6).contains(&level) {
                return Err(CoreError::InvalidSelector(format!(
                    "cannot move-section: descendant {:?} would land at heading level {} (max is 6); reduce destination depth or use --keep-level",
                    heading.text, level
                )));
            }
        }
    }
    Ok(())
}

fn splice(
    source: &str,
    moved: &str,
    leading: &str,
    insert: u32,
    source_start: u32,
    source_end: u32,
) -> (String, u32) {
    let mut output = String::with_capacity(source.len() + moved.len() + leading.len());
    if insert <= source_start {
        output.push_str(&source[..insert as usize]);
        output.push_str(leading);
        let moved_start = output.len() as u32;
        output.push_str(moved);
        output.push_str(&source[insert as usize..source_start as usize]);
        output.push_str(&source[source_end as usize..]);
        (output, moved_start)
    } else {
        output.push_str(&source[..source_start as usize]);
        output.push_str(&source[source_end as usize..insert as usize]);
        output.push_str(leading);
        let moved_start = output.len() as u32;
        output.push_str(moved);
        output.push_str(&source[insert as usize..]);
        (output, moved_start)
    }
}

#[allow(clippy::too_many_arguments)]
fn choose_setext_leading_separators(
    document: &Document,
    source: &str,
    moved: &str,
    separator: &str,
    insert: u32,
    source_start: u32,
    source_end: u32,
    text: &str,
    level: u8,
) -> usize {
    (0..=2)
        .find(|count| {
            let (candidate, start) = splice(
                source,
                moved,
                &separator.repeat(*count),
                insert,
                source_start,
                source_end,
            );
            moved_section_reparses_at(
                document,
                &candidate,
                start as usize,
                moved.len(),
                text,
                level,
            )
        })
        .unwrap_or(2)
}

fn following_setext_heading(
    parsed: &ParsedDocument,
    insert: u32,
    source_start: u32,
    source_end: u32,
) -> Option<(String, u8)> {
    let following = if insert == source_start {
        source_end
    } else {
        insert
    };
    if following as usize >= parsed.source.len() {
        return None;
    }
    parsed.blocks.iter().find_map(|block| {
        let heading = block.heading.as_ref()?;
        (heading.kind == HeadingSourceKind::Setext
            && line_start(parsed.source.as_bytes(), block.span.byte_start as usize)
                == following as usize)
            .then(|| (heading.text.clone(), heading.level))
    })
}

#[allow(clippy::too_many_arguments)]
fn choose_trailing_separators(
    document: &Document,
    source: &str,
    moved: &str,
    separator: &str,
    insert: u32,
    source_start: u32,
    source_end: u32,
    text: &str,
    level: u8,
) -> usize {
    (0..=2)
        .find(|count| {
            let candidate_moved = format!("{}{}", moved, separator.repeat(*count));
            let (candidate, start) = splice(
                source,
                &candidate_moved,
                "",
                insert,
                source_start,
                source_end,
            );
            setext_heading_reparses_at(
                document,
                &candidate,
                start as usize + candidate_moved.len(),
                text,
                level,
            )
        })
        .unwrap_or(2)
}

fn setext_heading_reparses_at(
    document: &Document,
    output: &str,
    start: usize,
    text: &str,
    level: u8,
) -> bool {
    let Ok(parsed) = document.reparse(output.to_string()) else {
        return false;
    };
    parsed.blocks().iter().any(|block| {
        block.heading.as_ref().is_some_and(|heading| {
            heading.kind == HeadingSourceKind::Setext
                && heading.text == text
                && heading.level == level
                && line_start(output.as_bytes(), block.span.byte_start as usize) == start
        })
    })
}

fn moved_section_reparses_at(
    document: &Document,
    output: &str,
    moved_start: usize,
    moved_len: usize,
    text: &str,
    level: u8,
) -> bool {
    let Ok(parsed) = document.reparse(output.to_string()) else {
        return false;
    };
    let moved_end = moved_start + moved_len;
    for (index, block) in parsed.blocks().iter().enumerate() {
        let Some(heading) = &block.heading else {
            continue;
        };
        if heading.text != text
            || heading.level != level
            || line_start(output.as_bytes(), block.span.byte_start as usize) != moved_start
        {
            continue;
        }
        let section_end = parsed
            .blocks()
            .iter()
            .skip(index + 1)
            .find_map(|next| {
                next.heading.as_ref().and_then(|next_heading| {
                    (next_heading.level <= heading.level)
                        .then(|| line_start(output.as_bytes(), next.span.byte_start as usize))
                })
            })
            .unwrap_or(output.len());
        return section_end == moved_end;
    }
    false
}

fn rewrite_atx_levels(
    moved: String,
    section: &SectionEntry,
    parsed: &ParsedDocument,
    source_start: u32,
    delta: i8,
) -> Result<String, CoreError> {
    let mut bytes = moved.into_bytes();
    let mut edits = Vec::new();
    for index in &section.block_indices {
        let block = &parsed.blocks[*index as usize];
        let Some(heading) = &block.heading else {
            continue;
        };
        if heading.kind != HeadingSourceKind::Atx {
            continue;
        }
        let marker = heading.marker_span;
        edits.push((
            (marker.byte_start - source_start) as usize,
            (marker.byte_end - source_start) as usize,
            (heading.level as i32 + delta as i32) as usize,
        ));
    }
    edits.sort_by(|left, right| right.0.cmp(&left.0));
    for (start, end, level) in edits {
        bytes.splice(start..end, vec![b'#'; level]);
    }
    String::from_utf8(bytes).map_err(|_| {
        CoreError::InvalidSelector("internal: ATX rewrite produced invalid UTF-8".into())
    })
}

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

fn moved_span_after(content: &str, start: u32, length: usize) -> SourceSpan {
    let end = start + length as u32;
    let bytes = content.as_bytes();
    let line_start = bytes[..start as usize]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count() as u32
        + 1;
    let line_end = if end as usize >= content.len() {
        bytes.iter().filter(|byte| **byte == b'\n').count() as u32 + 1
    } else {
        let line_at_end = bytes[..end as usize]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count() as u32
            + 1;
        line_at_end - u32::from(end > 0 && bytes[end as usize - 1] == b'\n')
    };
    SourceSpan {
        line_start,
        line_end,
        byte_start: start,
        byte_end: end,
    }
}

pub(crate) fn preserve_following_boundary(
    section: &str,
    replacement: &str,
    follows: bool,
) -> String {
    if replacement.is_empty() || !follows {
        return replacement.to_string();
    }
    let boundary = trailing_line_endings(section);
    let existing = trailing_line_endings(replacement).len();
    let mut completed = replacement.to_string();
    for ending in boundary.iter().skip(existing) {
        completed.push_str(ending);
    }
    completed
}

fn trailing_line_endings(content: &str) -> Vec<&str> {
    let bytes = content.as_bytes();
    let mut end = bytes.len();
    let mut spans = Vec::new();
    while end > 0 && bytes[end - 1] == b'\n' {
        let start = if end > 1 && bytes[end - 2] == b'\r' {
            end - 2
        } else {
            end - 1
        };
        spans.push((start, end));
        end = start;
    }
    spans.reverse();
    spans
        .into_iter()
        .map(|(start, end)| &content[start..end])
        .collect()
}

fn outcome(
    document: &Document,
    target: SectionEditTarget,
    disposition: MutationDisposition,
    guarded: bool,
    preservation: EditPreservation,
    content: String,
) -> EditOutcome<SectionEditTarget> {
    EditOutcome {
        base_revision: document.revision().clone(),
        target,
        disposition,
        guarded,
        line_endings: document.line_ending_style(),
        preservation,
        content,
    }
}

fn describe_selector(section: &SectionEntry) -> String {
    let selector = &section.selector;
    if selector.heading_text.is_none() {
        ":preamble".into()
    } else {
        match selector.occurrence {
            Some(occurrence) => format!(
                "{:?} occurrence {}",
                selector.heading_text.as_deref().unwrap_or(""),
                occurrence
            ),
            None => format!("{:?}", selector.heading_text.as_deref().unwrap_or("")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::HeadingMatchMode;
    use crate::section::{SectionIndex, SectionTarget};

    #[test]
    fn whole_document_closure_rejects_unrelated_block_changes() {
        let document = Document::parse("# A\n\na\n\n# B\n\nb\n").unwrap();
        let source = SectionIndex::new(&document)
            .resolve(&SectionTarget::heading("A", None, HeadingMatchMode::Exact).unwrap())
            .unwrap();
        let corrupted = "# A\n\na\n\n# B\n\nchanged\n";

        assert!(matches!(
            verify_unmoved_structure(
                &document,
                corrupted,
                source.entry(),
                source.span.byte_start,
                source.span.byte_end as usize - source.span.byte_start as usize,
            ),
            Err(CoreError::InvalidSelector(_))
        ));
    }
}
