use crate::core_error::CoreError;
use crate::document::Document;
use crate::edit::{strip_one_trailing_newline, SourceEdit};
use crate::model::{BlockKind, MutationDisposition, SourceSpan};
use crate::parser::{validate_table_row_payload, TableFact};

pub(crate) enum TableResultLocation {
    None,
    Base(SourceSpan),
    Replacement(std::ops::Range<usize>),
}

pub(crate) struct TableMutationPlan {
    pub(crate) edit: Option<SourceEdit>,
    pub(crate) disposition: MutationDisposition,
    pub(crate) result: TableResultLocation,
}

pub(crate) fn plan_replace_row(
    document: &Document,
    table_block_index: u32,
    row_index: u32,
    payload: impl Into<String>,
) -> Result<TableMutationPlan, CoreError> {
    let (block_span, table) = prepare(document, table_block_index, row_index, false)?;
    let _ = block_span;
    let payload = strip_one_trailing_newline(payload.into());
    validate_table_row_payload(&payload, table.headers.len())?;
    let row = &table.rows[row_index as usize];
    if payload == document.slice_unchecked(&row.span) {
        Ok(TableMutationPlan {
            edit: None,
            disposition: MutationDisposition::NoChange,
            result: TableResultLocation::Base(row.span),
        })
    } else {
        let length = payload.len();
        Ok(TableMutationPlan {
            edit: Some(SourceEdit {
                start: row.span.byte_start as usize,
                end: row.span.byte_end as usize,
                replacement: payload,
            }),
            disposition: MutationDisposition::Replaced,
            result: TableResultLocation::Replacement(0..length),
        })
    }
}

pub(crate) fn plan_insert_row(
    document: &Document,
    table_block_index: u32,
    row_index: u32,
    payload: impl Into<String>,
) -> Result<TableMutationPlan, CoreError> {
    let (block_span, table) = prepare(document, table_block_index, row_index, true)?;
    let payload = strip_one_trailing_newline(payload.into());
    validate_table_row_payload(&payload, table.headers.len())?;
    let insertion = resolve_insertion(document, block_span, &table, row_index)?;
    let (replacement, result) = match insertion.placement {
        SeparatorPlacement::BeforePayload => {
            let mut replacement = String::with_capacity(insertion.separator.len() + payload.len());
            replacement.push_str(insertion.separator);
            let start = replacement.len();
            replacement.push_str(&payload);
            let end = replacement.len();
            (replacement, start..end)
        }
        SeparatorPlacement::AfterPayload => {
            let mut replacement = String::with_capacity(payload.len() + insertion.separator.len());
            replacement.push_str(&payload);
            let end = replacement.len();
            replacement.push_str(insertion.separator);
            (replacement, 0..end)
        }
    };
    Ok(TableMutationPlan {
        edit: Some(SourceEdit {
            start: insertion.insert_byte,
            end: insertion.insert_byte,
            replacement,
        }),
        disposition: MutationDisposition::Inserted,
        result: TableResultLocation::Replacement(result),
    })
}

pub(crate) fn plan_delete_row(
    document: &Document,
    table_block_index: u32,
    row_index: u32,
) -> Result<TableMutationPlan, CoreError> {
    let (_, table) = prepare(document, table_block_index, row_index, false)?;
    let row = &table.rows[row_index as usize];
    let deletion = deletion_span(document, row.span);
    Ok(TableMutationPlan {
        edit: Some(SourceEdit {
            start: deletion.byte_start as usize,
            end: deletion.byte_end as usize,
            replacement: String::new(),
        }),
        disposition: MutationDisposition::Deleted,
        result: TableResultLocation::None,
    })
}

fn prepare(
    document: &Document,
    block_index: u32,
    row_index: u32,
    insertion: bool,
) -> Result<(SourceSpan, TableFact), CoreError> {
    let block =
        document
            .blocks()
            .get(block_index as usize)
            .ok_or(CoreError::BlockIndexOutOfRange {
                index: block_index,
                block_count: document.blocks().len() as u32,
            })?;
    if block.kind != BlockKind::Table {
        return Err(CoreError::NotTable { block_index });
    }
    let table = block.table.clone().ok_or_else(|| {
        CoreError::ParseFailed(format!(
            "table block {block_index} is missing its cached projection"
        ))
    })?;
    let row_count = table.rows.len() as u32;
    if (insertion && row_index > row_count) || (!insertion && row_index >= row_count) {
        return Err(CoreError::TableRowOutOfRange {
            table_block_index: block_index,
            row_index,
            row_count,
            insertion,
        });
    }
    Ok((block.span, table))
}

struct Insertion<'a> {
    insert_byte: usize,
    separator: &'a str,
    placement: SeparatorPlacement,
}

enum SeparatorPlacement {
    BeforePayload,
    AfterPayload,
}

fn resolve_insertion<'a>(
    document: &'a Document,
    block_span: SourceSpan,
    table: &TableFact,
    row_index: u32,
) -> Result<Insertion<'a>, CoreError> {
    let source = document.source();
    if row_index < table.rows.len() as u32 {
        let row = &table.rows[row_index as usize];
        let separator =
            line_boundary_before(source, row.span.byte_start as usize).ok_or_else(|| {
                CoreError::ParseFailed("could not resolve table row insertion boundary".into())
            })?;
        return Ok(Insertion {
            insert_byte: row.span.byte_start as usize,
            separator,
            placement: SeparatorPlacement::AfterPayload,
        });
    }
    let separator = line_boundary_after(source, block_span.byte_end as usize)
        .or_else(|| {
            table
                .rows
                .last()
                .and_then(|row| line_boundary_before(source, row.span.byte_start as usize))
        })
        .or_else(|| last_line_boundary_within(document.slice_unchecked(&block_span)))
        .ok_or_else(|| CoreError::ParseFailed("could not resolve table row boundary".into()))?;
    Ok(Insertion {
        insert_byte: block_span.byte_end as usize,
        separator,
        placement: SeparatorPlacement::BeforePayload,
    })
}

fn line_boundary_after(source: &str, index: usize) -> Option<&str> {
    let tail = source.get(index..)?;
    if tail.starts_with("\r\n") {
        Some(&tail[..2])
    } else if tail.starts_with('\n') {
        Some(&tail[..1])
    } else {
        None
    }
}

fn line_boundary_before(source: &str, index: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    if index > bytes.len() {
        None
    } else if index >= 2 && &bytes[index - 2..index] == b"\r\n" {
        source.get(index - 2..index)
    } else if index >= 1 && bytes[index - 1] == b'\n' {
        source.get(index - 1..index)
    } else {
        None
    }
}

fn last_line_boundary_within(source: &str) -> Option<&str> {
    let newline = source.rfind('\n')?;
    if newline > 0 && source.as_bytes()[newline - 1] == b'\r' {
        Some(&source[newline - 1..newline + 1])
    } else {
        Some(&source[newline..newline + 1])
    }
}

fn deletion_span(document: &Document, row: SourceSpan) -> SourceSpan {
    let source = document.source().as_bytes();
    let start = row.byte_start as usize;
    let end = row.byte_end as usize;
    let (start, end) = if source[end..].starts_with(b"\r\n") {
        (start, end + 2)
    } else if source[end..].starts_with(b"\n") {
        (start, end + 1)
    } else if end == source.len() && start >= 2 && &source[start - 2..start] == b"\r\n" {
        (start - 2, end)
    } else if end == source.len() && start >= 1 && source[start - 1] == b'\n' {
        (start - 1, end)
    } else {
        (start, end)
    };
    document.span_for_byte_range(start as u32, end as u32)
}
