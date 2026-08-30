//! Position-to-target resolution.
//!
//! Every other selector in this library is by name: heading text, block index,
//! task loc, table row index. A reading UI has none of those — it has a click,
//! which is a byte offset or a line. `locate` turns one position into the
//! targets that contain it, each carrying the same etag the matching read path
//! produces, so the result feeds a guarded mutation without a second reading of
//! the source.
//!
//! A position that lands between two blocks is not an error: it resolves to
//! `Ok` with `block: None` and the surrounding section still filled in, because
//! a click on whitespace is a meaningful position for a UI and erroring would
//! put the same `match` in every consumer. Only a byte offset outside the
//! document is an error. A blank line *inside* a block — between two items of a
//! loose list, or within a fenced code block — is inside that block's span and
//! resolves to it.

use crate::block::{self, BlockRecord};
use crate::core_error::CoreError;
use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::model::{BlockKind, SectionEntry, SourceSpan};
use crate::parser::BlockInfo;
use crate::section::SectionIndex;
use crate::task::{self, TaskLoc, TaskRecord};

/// The targets containing one position. Every field is `None` when the position
/// falls outside that kind of target.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Located {
    /// The enclosing top-level block. Absent in frontmatter, between two
    /// blocks, and on the newline that terminates a block's last line — block
    /// spans exclude it.
    pub block: Option<BlockRecord>,
    /// The innermost section containing the position: the preamble, or the
    /// deepest heading section covering it. Absent only in frontmatter, which
    /// precedes every section.
    pub section: Option<SectionEntry>,
    /// The innermost task item containing the position.
    pub task: Option<TaskRecord>,
    /// The table data row containing the position. Absent on the header row and
    /// the separator line, and for a table nested inside another block — only a
    /// top-level table is resolved, and a nested one reports its outer block.
    pub table_row: Option<LocatedTableRow>,
}

/// A data row of a table, addressed the way the table-row mutations address it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocatedTableRow {
    /// Block index of the enclosing table, the first argument of
    /// [`crate::table::prepare_replace_row`] and its siblings.
    pub table_block_index: u32,
    /// 0-based index into the table's data rows, header excluded. Replace and
    /// delete address *this* row; insert places a new row **before** it.
    pub row_index: u32,
    /// Source span of the row's bytes, its line ending excluded.
    pub span: SourceSpan,
    /// The fingerprint the row mutations accept as their guard. It is named for
    /// the table because that is what it hashes: row edits are guarded by the
    /// *whole table block's* bytes, not the row's, so this is the same value as
    /// `block.etag` on the same result.
    pub table_etag: TargetEtag,
}

/// Resolve a 0-based byte offset to its enclosing targets.
///
/// Errors only when `byte_offset` is at or past the end of the source. An
/// offset that lands inside a multibyte character is fine — nothing slices
/// there — and resolves to the same targets as that character's first byte.
pub fn locate(document: &Document, byte_offset: u32) -> Result<Located, CoreError> {
    let source_len = document.source().len() as u32;
    if byte_offset >= source_len {
        return Err(CoreError::ByteOffsetOutOfRange {
            byte_offset,
            source_len,
        });
    }

    let info = block_at(document.blocks(), byte_offset);
    let block = info
        .map(|info| block::block(document, info.index).map(|read| read.block))
        .transpose()?;
    let task = info
        .and_then(|info| innermost_task_loc(info, byte_offset))
        .map(|loc| task::task(document, &loc).map(|read| read.task))
        .transpose()?;
    let table_row = match (info, &block) {
        (Some(info), Some(record)) if info.kind == BlockKind::Table => {
            table_row_at(document, info, record, byte_offset)?
        }
        _ => None,
    };
    // By source position in both the block and the no-block case. The
    // block-index route would have to agree with this one anyway, and only this
    // one is right for a block the parser emits out of source order.
    let section = SectionIndex::new(document).section_for_byte(byte_offset);

    Ok(Located {
        block,
        section,
        task,
        table_row,
    })
}

/// Resolve a 1-based line to the targets containing its first content byte.
///
/// Leading indentation is skipped, because a block's span starts at its first
/// non-whitespace byte — up to three spaces of indentation are legal Markdown
/// and belong to no block. A blank line resolves from its own start, so the
/// between-blocks rule still holds.
///
/// The line after a document's final newline is counted by
/// [`Document::line_count`] but holds nothing, so it resolves to `Ok` with
/// every field `None`. That keeps `for line in 1..=document.line_count()` free
/// of a special case. Only a line outside `1..=line_count` is an error.
pub fn locate_line(document: &Document, line: u32) -> Result<Located, CoreError> {
    let line_start = document
        .line_to_byte(line)
        .ok_or(CoreError::LineOutOfRange {
            line,
            line_count: document.line_count(),
        })?;
    if line_start as usize >= document.source().len() {
        return Ok(Located::default());
    }
    locate(document, first_content_byte(document.source(), line_start))
}

/// Scan the top-level block spans for the one containing the offset.
///
/// The spans do not overlap, but they are **not** in source order: comrak emits
/// every footnote definition after the blocks that reference it, so
/// `document.blocks()` is two ascending runs concatenated. A binary search over
/// that answers wrongly — it reports "no block here" for ordinary content,
/// which a caller cannot tell apart from a click on a blank line. The scan
/// costs no more than the `SectionIndex` this call already rebuilds.
fn block_at(blocks: &[BlockInfo], byte_offset: u32) -> Option<&BlockInfo> {
    blocks
        .iter()
        .find(|block| block.span.byte_start <= byte_offset && byte_offset < block.span.byte_end)
}

/// First byte of the line's content, or the line's own start when it is blank.
fn first_content_byte(source: &str, line_start: u32) -> u32 {
    let bytes = source.as_bytes();
    let mut offset = line_start as usize;
    while bytes
        .get(offset)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        offset += 1;
    }
    match bytes.get(offset) {
        Some(b'\n') | Some(b'\r') | None => line_start,
        Some(_) => offset as u32,
    }
}

/// The data row containing the offset, or `None` on the header row or the
/// separator line.
///
/// A row's span excludes its line ending, because that is the span the row
/// mutations splice over. Containment has to include it: end-of-line is an
/// ordinary click position, and without it the caller's only fallback is a
/// whole-table edit.
fn table_row_at(
    document: &Document,
    block: &BlockInfo,
    record: &BlockRecord,
    byte_offset: u32,
) -> Result<Option<LocatedTableRow>, CoreError> {
    let source = document.source();
    let projection = block.table.as_ref().ok_or_else(|| {
        CoreError::ParseFailed(format!(
            "table block {} is missing its cached projection",
            block.index
        ))
    })?;
    Ok(projection
        .rows
        .iter()
        .position(|row| {
            row.span.byte_start <= byte_offset
                && byte_offset < row.span.byte_end + terminator_len(source, row.span.byte_end)
        })
        .map(|row_index| LocatedTableRow {
            table_block_index: block.index,
            row_index: row_index as u32,
            span: projection.rows[row_index].span,
            // The block record's own fingerprint rather than a second call to
            // `for_bytes` over the same slice, so this value cannot drift from
            // what the guard re-hashes.
            table_etag: record.etag.clone(),
        }))
}

/// Length of the line ending at `byte_end`, or 0 when none follows.
fn terminator_len(source: &str, byte_end: u32) -> u32 {
    let tail = &source.as_bytes()[byte_end as usize..];
    if tail.starts_with(b"\r\n") {
        2
    } else if tail.starts_with(b"\n") {
        1
    } else {
        0
    }
}

/// Nested task spans are contained in their parents, so the deepest containing
/// item is the innermost one.
fn innermost_task_loc(block: &BlockInfo, byte_offset: u32) -> Option<TaskLoc> {
    block
        .task_items
        .iter()
        .filter(|item| item.span.byte_start <= byte_offset && byte_offset < item.span.byte_end)
        .max_by_key(|item| item.depth)
        .map(|item| TaskLoc::new(block.index, item.child_path.clone()))
}
