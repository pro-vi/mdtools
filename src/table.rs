use crate::core_error::{CoreError, EtagTarget};
use crate::document::Document;
use crate::edit::{
    replacement_span_after, strip_one_trailing_newline, EditOutcome, EditPreservation, SourceEdit,
};
use crate::fingerprint::{TargetEtag, TargetEtagGuard};
use crate::model::{BlockKind, ColumnAlignment, MutationDisposition, SourceSpan};
use crate::parser::{validate_table_row_payload, BlockInfo, TableProjection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnSelector {
    Index(usize),
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TablePredicate {
    Equal(ColumnSelector, String),
    NotEqual(ColumnSelector, String),
    Contains(ColumnSelector, String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableQuery {
    pub columns: Vec<ColumnSelector>,
    pub predicates: Vec<TablePredicate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableSummary {
    pub block_index: u32,
    pub span: SourceSpan,
    pub etag: TargetEtag,
    pub headers: Vec<String>,
    pub row_count: u32,
    pub column_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRead {
    pub block_index: u32,
    pub span: SourceSpan,
    pub etag: TargetEtag,
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEditTarget {
    Row {
        table_block_index: u32,
        row_index: u32,
        span: SourceSpan,
    },
    Insertion {
        table_block_index: u32,
        row_index: u32,
        table_span: SourceSpan,
    },
}

pub struct PreparedTableRowEdit<'a> {
    document: &'a Document,
    table_block_index: u32,
    row_index: u32,
    block_span: SourceSpan,
    table: TableProjection,
    guarded: bool,
}

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
    let prepared = prepare_replace_row(document, table_block_index, row_index, None)?;
    let payload = strip_one_trailing_newline(payload.into());
    validate_table_row_payload(&payload, prepared.table.headers.len())?;
    let row = &prepared.table.rows[row_index as usize];
    if payload == document.slice_unchecked(&row.span) {
        Ok(TableMutationPlan {
            edit: None,
            disposition: MutationDisposition::NoChange,
            result: TableResultLocation::Base(row.span),
        })
    } else {
        let payload_len = payload.len();
        Ok(TableMutationPlan {
            edit: Some(SourceEdit {
                start: row.span.byte_start as usize,
                end: row.span.byte_end as usize,
                replacement: payload,
            }),
            disposition: MutationDisposition::Replaced,
            result: TableResultLocation::Replacement(0..payload_len),
        })
    }
}

pub(crate) fn plan_insert_row(
    document: &Document,
    table_block_index: u32,
    row_index: u32,
    payload: impl Into<String>,
) -> Result<TableMutationPlan, CoreError> {
    let prepared = prepare_insert_row(document, table_block_index, row_index, None)?;
    let payload = strip_one_trailing_newline(payload.into());
    validate_table_row_payload(&payload, prepared.table.headers.len())?;
    let insertion = resolve_insertion(
        document,
        prepared.block_span,
        &prepared.table,
        prepared.row_index,
    )?;
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
    let prepared = prepare_replace_row(document, table_block_index, row_index, None)?;
    let row = &prepared.table.rows[row_index as usize];
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

pub fn tables(document: &Document) -> Result<Vec<TableSummary>, CoreError> {
    let summaries = document
        .blocks()
        .iter()
        .filter(|block| block.kind == BlockKind::Table)
        .map(|block| {
            let source = document.slice_unchecked(&block.span);
            let table = cached_table(block)?;
            Ok(TableSummary {
                block_index: block.index,
                span: block.span,
                etag: TargetEtag::for_bytes(source.as_bytes()),
                headers: table.headers.clone(),
                row_count: table.rows.len() as u32,
                column_count: table.alignments.len() as u32,
            })
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    if summaries.is_empty() {
        Err(CoreError::NoTables)
    } else {
        Ok(summaries)
    }
}

pub fn table(
    document: &Document,
    block_index: u32,
    query: &TableQuery,
) -> Result<TableRead, CoreError> {
    let (span, projection, etag) = resolve_table(document, block_index)?;
    let predicate_indices = query
        .predicates
        .iter()
        .map(|predicate| match predicate {
            TablePredicate::Equal(column, value) => resolve_column(&projection.headers, column)
                .map(|index| ResolvedPredicate::Equal(index, value)),
            TablePredicate::NotEqual(column, value) => resolve_column(&projection.headers, column)
                .map(|index| ResolvedPredicate::NotEqual(index, value)),
            TablePredicate::Contains(column, value) => resolve_column(&projection.headers, column)
                .map(|index| ResolvedPredicate::Contains(index, value)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let rows = projection
        .rows
        .iter()
        .map(|row| row.cells.clone())
        .filter(|row| row_matches(row, &predicate_indices))
        .collect::<Vec<_>>();
    let indices = if query.columns.is_empty() {
        (0..projection.headers.len()).collect::<Vec<_>>()
    } else {
        query
            .columns
            .iter()
            .map(|column| resolve_column(&projection.headers, column))
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(TableRead {
        block_index,
        span,
        etag,
        headers: indices
            .iter()
            .map(|index| projection.headers[*index].clone())
            .collect(),
        alignments: indices
            .iter()
            .map(|index| projection.alignments[*index])
            .collect(),
        rows: rows
            .into_iter()
            .map(|row| {
                indices
                    .iter()
                    .map(|index| row.get(*index).cloned().unwrap_or_default())
                    .collect()
            })
            .collect(),
    })
}

pub fn prepare_replace_row<'a>(
    document: &'a Document,
    table_block_index: u32,
    row_index: u32,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<PreparedTableRowEdit<'a>, CoreError> {
    prepare_row_edit(document, table_block_index, row_index, expect_etag, false)
}

pub fn prepare_insert_row<'a>(
    document: &'a Document,
    table_block_index: u32,
    row_index: u32,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<PreparedTableRowEdit<'a>, CoreError> {
    prepare_row_edit(document, table_block_index, row_index, expect_etag, true)
}

fn prepare_row_edit<'a>(
    document: &'a Document,
    table_block_index: u32,
    row_index: u32,
    expect_etag: Option<&TargetEtagGuard>,
    insertion: bool,
) -> Result<PreparedTableRowEdit<'a>, CoreError> {
    let (block_span, table, actual) = resolve_table(document, table_block_index)?;
    verify_table_guard(document, table_block_index, expect_etag, &actual)?;
    let row_count = table.rows.len() as u32;
    if (insertion && row_index > row_count) || (!insertion && row_index >= row_count) {
        return Err(CoreError::TableRowOutOfRange {
            table_block_index,
            row_index,
            row_count,
            insertion,
        });
    }
    Ok(PreparedTableRowEdit {
        document,
        table_block_index,
        row_index,
        block_span,
        table,
        guarded: expect_etag.is_some(),
    })
}

impl PreparedTableRowEdit<'_> {
    pub fn replace(
        self,
        payload: impl Into<String>,
    ) -> Result<EditOutcome<TableEditTarget>, CoreError> {
        let payload = strip_one_trailing_newline(payload.into());
        validate_table_row_payload(&payload, self.table.headers.len())?;
        let row = &self.table.rows[self.row_index as usize];
        let original = self.document.slice_unchecked(&row.span);
        let disposition = if payload == original {
            MutationDisposition::NoChange
        } else {
            MutationDisposition::Replaced
        };
        let content = format!(
            "{}{}{}",
            &self.document.source()[..row.span.byte_start as usize],
            payload,
            &self.document.source()[row.span.byte_end as usize..]
        );
        let span_after = if disposition == MutationDisposition::NoChange {
            row.span
        } else {
            replacement_span_after(row.span, &payload)
        };
        Ok(self.outcome(
            TableEditTarget::Row {
                table_block_index: self.table_block_index,
                row_index: self.row_index,
                span: row.span,
            },
            disposition,
            Some(row.span),
            Some(span_after),
            content,
        ))
    }

    pub fn insert(
        self,
        payload: impl Into<String>,
    ) -> Result<EditOutcome<TableEditTarget>, CoreError> {
        let payload = strip_one_trailing_newline(payload.into());
        validate_table_row_payload(&payload, self.table.headers.len())?;
        let insertion =
            resolve_insertion(self.document, self.block_span, &self.table, self.row_index)?;
        let mut content = String::with_capacity(
            self.document.source().len() + payload.len() + insertion.separator.len(),
        );
        content.push_str(&self.document.source()[..insertion.insert_byte]);
        let payload_start = match insertion.placement {
            SeparatorPlacement::BeforePayload => {
                content.push_str(insertion.separator);
                let start = content.len();
                content.push_str(&payload);
                start
            }
            SeparatorPlacement::AfterPayload => {
                let start = content.len();
                content.push_str(&payload);
                content.push_str(insertion.separator);
                start
            }
        };
        let payload_end = payload_start + payload.len();
        content.push_str(&self.document.source()[insertion.insert_byte..]);
        let span_after = span_for_inserted_bytes(&content, payload_start, payload_end);
        Ok(self.outcome(
            TableEditTarget::Insertion {
                table_block_index: self.table_block_index,
                row_index: self.row_index,
                table_span: self.block_span,
            },
            MutationDisposition::Inserted,
            None,
            Some(span_after),
            content,
        ))
    }

    fn outcome(
        &self,
        target: TableEditTarget,
        disposition: MutationDisposition,
        before: Option<SourceSpan>,
        after: Option<SourceSpan>,
        content: String,
    ) -> EditOutcome<TableEditTarget> {
        EditOutcome {
            base_revision: self.document.revision().clone(),
            target,
            disposition,
            guarded: self.guarded,
            line_endings: self.document.line_ending_style(),
            preservation: EditPreservation {
                preserves_non_target_bytes: true,
                target_span_before: before,
                target_span_after: after,
            },
            content,
        }
    }
}

pub fn delete_row(
    document: &Document,
    table_block_index: u32,
    row_index: u32,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<EditOutcome<TableEditTarget>, CoreError> {
    let prepared = prepare_replace_row(document, table_block_index, row_index, expect_etag)?;
    let row = &prepared.table.rows[row_index as usize];
    let deletion = deletion_span(document, row.span);
    let content = format!(
        "{}{}",
        &document.source()[..deletion.byte_start as usize],
        &document.source()[deletion.byte_end as usize..]
    );
    Ok(prepared.outcome(
        TableEditTarget::Row {
            table_block_index,
            row_index,
            span: row.span,
        },
        MutationDisposition::Deleted,
        Some(deletion),
        None,
        content,
    ))
}

fn resolve_table(
    document: &Document,
    block_index: u32,
) -> Result<(SourceSpan, TableProjection, TargetEtag), CoreError> {
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
    let source = document.slice_unchecked(&block.span);
    Ok((
        block.span,
        cached_table(block)?.clone(),
        TargetEtag::for_bytes(source.as_bytes()),
    ))
}

fn cached_table(block: &BlockInfo) -> Result<&TableProjection, CoreError> {
    block.table.as_ref().ok_or_else(|| {
        CoreError::ParseFailed(format!(
            "table block {} is missing its cached projection",
            block.index
        ))
    })
}

fn verify_table_guard(
    document: &Document,
    block_index: u32,
    expected: Option<&TargetEtagGuard>,
    actual: &TargetEtag,
) -> Result<(), CoreError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    if expected.as_str() != actual.as_str() {
        return Err(CoreError::TargetEtagMismatch {
            target: EtagTarget::Table(block_index),
            expected: expected.to_string(),
            actual: actual.to_string(),
        });
    }
    let count = document
        .blocks()
        .iter()
        .filter(|block| block.kind == BlockKind::Table)
        .filter(|block| {
            TargetEtag::for_bytes(document.slice_unchecked(&block.span).as_bytes()).as_str()
                == expected.as_str()
        })
        .count();
    if count > 1 {
        return Err(CoreError::TargetEtagAmbiguous {
            target_kind: "table",
            expected: expected.to_string(),
            count,
        });
    }
    Ok(())
}

enum ResolvedPredicate<'a> {
    Equal(usize, &'a str),
    NotEqual(usize, &'a str),
    Contains(usize, &'a str),
}

fn row_matches(row: &[String], predicates: &[ResolvedPredicate<'_>]) -> bool {
    predicates.iter().all(|predicate| match predicate {
        ResolvedPredicate::Equal(index, value) => {
            row.get(*index).map(String::as_str) == Some(*value)
        }
        ResolvedPredicate::NotEqual(index, value) => {
            row.get(*index).map(String::as_str) != Some(*value)
        }
        ResolvedPredicate::Contains(index, value) => {
            row.get(*index).is_some_and(|cell| cell.contains(value))
        }
    })
}

fn resolve_column(headers: &[String], column: &ColumnSelector) -> Result<usize, CoreError> {
    match column {
        ColumnSelector::Index(index) if *index < headers.len() => Ok(*index),
        ColumnSelector::Index(index) => Err(CoreError::ColumnNotFound {
            column: index.to_string(),
            headers: headers.to_vec(),
        }),
        ColumnSelector::Name(name) => {
            headers
                .iter()
                .position(|header| header == name)
                .ok_or_else(|| CoreError::ColumnNotFound {
                    column: name.clone(),
                    headers: headers.to_vec(),
                })
        }
    }
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
    table: &TableProjection,
    row_index: u32,
) -> Result<Insertion<'a>, CoreError> {
    let source = document.source();
    if row_index < table.rows.len() as u32 {
        let row = &table.rows[row_index as usize];
        let separator =
            line_boundary_before(source, row.span.byte_start as usize).ok_or_else(|| {
                CoreError::ParseFailed(
                    "could not resolve line boundary for table row insertion".into(),
                )
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
        .ok_or_else(|| {
            CoreError::ParseFailed("could not resolve line boundary for table row insertion".into())
        })?;
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

fn span_for_inserted_bytes(content: &str, start: usize, end: usize) -> SourceSpan {
    let line = |offset: usize| {
        content[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count() as u32
            + 1
    };
    SourceSpan {
        line_start: line(start),
        line_end: if end > start {
            line(end - 1)
        } else {
            line(start)
        },
        byte_start: start as u32,
        byte_end: end as u32,
    }
}
