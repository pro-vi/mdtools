//! Position-to-target resolution.
//!
//! Every other selector in this library is by name: heading text, block index,
//! task loc, table row index. A reading UI has none of those — it has a click,
//! which is a byte offset or a line. `locate` turns one position into the
//! targets that contain it, each carrying the same etag the matching read path
//! produces, so the result feeds a guarded mutation without a second reading of
//! the source.
//!
//! Positions between blocks (a blank line) are not an error: they resolve to
//! `Ok` with `block: None`, because a click on whitespace is a meaningful
//! position for a UI and erroring would put the same `match` in every consumer.
//! Only a position outside the document is an error.

use crate::block::{self, BlockRecord};
use crate::core_error::CoreError;
use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::model::{BlockKind, SectionEntry, SourceSpan};
use crate::parser::{extract_table_projection, BlockInfo};
use crate::section::SectionIndex;
use crate::task::{self, TaskLoc, TaskRecord};

/// The targets containing one position. Every field is `None` when the position
/// falls outside that kind of target.
#[derive(Clone, Debug)]
pub struct Located {
    /// The enclosing top-level block, absent between blocks and in frontmatter.
    pub block: Option<BlockRecord>,
    /// The innermost section containing the position: the preamble, or the
    /// deepest heading section whose blocks or span cover it. Absent in
    /// frontmatter, which precedes every section.
    pub section: Option<SectionEntry>,
    /// The innermost task item containing the position.
    pub task: Option<TaskRecord>,
    /// The table data row containing the position. Absent on the header row
    /// and the separator line, where `block` is still the table.
    pub table_row: Option<LocatedTableRow>,
}

/// A data row of a table, addressed the way the table-row mutations address it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedTableRow {
    /// Block index of the enclosing table, the first argument of
    /// [`crate::table::prepare_replace_row`] and its siblings.
    pub table_block_index: u32,
    /// 0-based index into the table's data rows, header excluded — the same
    /// base the row mutations use.
    pub row_index: u32,
    /// Source span of the row's bytes, newline excluded.
    pub span: SourceSpan,
    /// The fingerprint the row mutations accept as their guard. Row edits are
    /// guarded by the *whole table block's* bytes, not the row's, so this is
    /// the table block's etag — identical to `block.etag` on the same result.
    pub etag: TargetEtag,
}

/// Resolve a 0-based byte offset to its enclosing targets.
///
/// Errors only when `byte_offset` is at or past the end of the source.
pub fn locate(document: &Document, byte_offset: u32) -> Result<Located, CoreError> {
    let source_len = document.source().len();
    if byte_offset as usize >= source_len {
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
        .and_then(|info| innermost_task(info, byte_offset))
        .map(|loc| task::task(document, &loc).map(|read| read.task))
        .transpose()?;
    let table_row = info
        .filter(|info| info.kind == BlockKind::Table)
        .map(|info| table_row_at(document, info, byte_offset))
        .transpose()?
        .flatten();
    let index = SectionIndex::new(document);
    let section = match info {
        Some(info) => index.section_for_block(info.index),
        None => index.section_for_byte(byte_offset),
    };

    Ok(Located {
        block,
        section,
        task,
        table_row,
    })
}

/// Resolve a 1-based line to its enclosing targets, at the line's first byte.
///
/// Errors with [`CoreError::LineOutOfRange`] for line 0 or a line past the
/// document, and with [`CoreError::ByteOffsetOutOfRange`] for the empty line
/// after a trailing newline, whose first byte is the end of the source.
pub fn locate_line(document: &Document, line: u32) -> Result<Located, CoreError> {
    let byte_offset = document
        .line_to_byte(line)
        .ok_or(CoreError::LineOutOfRange {
            line,
            line_count: document.line_count(),
        })?;
    locate(document, byte_offset)
}

/// Binary search the sorted, non-overlapping top-level block spans.
fn block_at(blocks: &[BlockInfo], byte_offset: u32) -> Option<&BlockInfo> {
    let after = blocks.partition_point(|block| block.span.byte_start <= byte_offset);
    let candidate = blocks.get(after.checked_sub(1)?)?;
    (byte_offset < candidate.span.byte_end).then_some(candidate)
}

/// The data row containing the offset, or `None` on the header row or the
/// separator line.
fn table_row_at(
    document: &Document,
    block: &BlockInfo,
    byte_offset: u32,
) -> Result<Option<LocatedTableRow>, CoreError> {
    let source = document.slice_unchecked(&block.span);
    let projection = extract_table_projection(source, block.span)?;
    let etag = TargetEtag::for_bytes(source.as_bytes());
    Ok(projection
        .rows
        .iter()
        .position(|row| row.span.byte_start <= byte_offset && byte_offset < row.span.byte_end)
        .map(|row_index| LocatedTableRow {
            table_block_index: block.index,
            row_index: row_index as u32,
            span: projection.rows[row_index].span,
            etag: etag.clone(),
        }))
}

/// Nested task spans are contained in their parents, so the deepest containing
/// item is the innermost one.
fn innermost_task(block: &BlockInfo, byte_offset: u32) -> Option<TaskLoc> {
    block
        .task_items
        .iter()
        .filter(|item| item.span.byte_start <= byte_offset && byte_offset < item.span.byte_end)
        .max_by_key(|item| item.depth)
        .map(|item| TaskLoc::new(block.index, item.child_path.clone()))
}
