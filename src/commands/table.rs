use crate::cli::{DeleteTableRowArgs, InsertTableRowArgs, ReplaceTableRowArgs, TableArgs};
use crate::commands::edit;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;
use mdtools::table::{self, ColumnSelector, TableEditTarget, TablePredicate, TableQuery};

pub fn run(args: &TableArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(source)?;
    let summaries = table::tables(&document)?;
    if args.index.is_none() && args.select.is_empty() && args.filters.is_empty() {
        return emit_table_list(args, json, summaries);
    }
    let block_index = match args.index {
        Some(index) => index,
        None if summaries.len() == 1 => summaries[0].block_index,
        None => {
            return Err(CommandError::new(
                DiagnosticCode::InvalidSelector,
                format!(
                    "document has {} tables; use --index to select one",
                    summaries.len()
                ),
            )
            .with_hint(
                "run `md table --json <FILE>` (no --index) to list the table block indices, then pass --index <BLOCK_INDEX>",
            ));
        }
    };
    let query = TableQuery {
        columns: args.select.iter().map(|value| column(value)).collect(),
        predicates: parse_filters(&args.filters)?,
    };
    let read = table::table(&document, block_index, &query)?;
    if json {
        output::write_json(&TableReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            block_index: read.block_index,
            span: read.span,
            etag: read.etag.into_string(),
            headers: read.headers,
            alignments: read.alignments,
            rows: read.rows,
        })?;
    } else {
        println!(
            "{}",
            read.headers
                .iter()
                .map(|value| output::escape_text_field(value))
                .collect::<Vec<_>>()
                .join("\t")
        );
        for row in read.rows {
            println!(
                "{}",
                row.iter()
                    .map(|value| output::escape_text_field(value))
                    .collect::<Vec<_>>()
                    .join("\t")
            );
        }
    }
    Ok(())
}

pub fn run_replace_table_row(args: &ReplaceTableRowArgs, json: bool) -> Result<(), CommandError> {
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let expected = parse_etag(args.etag_guard.expect_etag.as_deref())?;
    let prepared = table::prepare_replace_row(
        &document,
        args.table_block_index,
        args.row_index,
        expected.as_ref(),
    )?;
    let payload = output::read_content(args.from.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::ReplaceTableRow,
        prepared.replace(payload)?,
        table_target_to_wire,
    )
}

pub fn run_insert_table_row(args: &InsertTableRowArgs, json: bool) -> Result<(), CommandError> {
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let expected = parse_etag(args.etag_guard.expect_etag.as_deref())?;
    let prepared = table::prepare_insert_row(
        &document,
        args.table_block_index,
        args.row_index,
        expected.as_ref(),
    )?;
    let payload = output::read_content(args.from.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::InsertTableRow,
        prepared.insert(payload)?,
        table_target_to_wire,
    )
}

pub fn run_delete_table_row(args: &DeleteTableRowArgs, json: bool) -> Result<(), CommandError> {
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let expected = parse_etag(args.etag_guard.expect_etag.as_deref())?;
    let outcome = table::delete_row(
        &document,
        args.table_block_index,
        args.row_index,
        expected.as_ref(),
    )?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::DeleteTableRow,
        outcome,
        table_target_to_wire,
    )
}

fn emit_table_list(
    args: &TableArgs,
    json: bool,
    summaries: Vec<table::TableSummary>,
) -> Result<(), CommandError> {
    let entries = summaries
        .into_iter()
        .map(|summary| TableEntry {
            block_index: summary.block_index,
            span: summary.span,
            etag: summary.etag.into_string(),
            headers: summary.headers,
            row_count: summary.row_count,
            column_count: summary.column_count,
        })
        .collect::<Vec<_>>();
    if json {
        output::write_json(&TablesResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            tables: entries,
        })?;
    } else {
        for entry in entries {
            println!(
                "{}\t{}\t{} rows\t{} cols",
                entry.block_index,
                entry.headers.join(", "),
                entry.row_count,
                entry.column_count
            );
        }
    }
    Ok(())
}

fn table_target_to_wire(target: &TableEditTarget) -> MutationTargetRef {
    match target {
        TableEditTarget::Row {
            table_block_index,
            row_index,
            span,
        } => MutationTargetRef::TableRow(TableRowTargetRef {
            kind: MutationTargetKind::TableRow,
            table_block_index: *table_block_index,
            row_index: *row_index,
            span: *span,
        }),
        TableEditTarget::Insertion {
            table_block_index,
            row_index,
            table_span,
        } => MutationTargetRef::TableRowInsertion(TableRowInsertionTargetRef {
            kind: MutationTargetKind::TableRowInsertion,
            table_block_index: *table_block_index,
            row_index: *row_index,
            table_span: *table_span,
        }),
    }
}

fn parse_etag(value: Option<&str>) -> Result<Option<TargetEtag>, CommandError> {
    Ok(value.map(mdtools::fingerprint::cli_compat::target_etag))
}

fn column(value: &str) -> ColumnSelector {
    value
        .parse::<usize>()
        .map(ColumnSelector::Index)
        .unwrap_or_else(|_| ColumnSelector::Name(value.to_string()))
}

fn parse_filters(filters: &[String]) -> Result<Vec<TablePredicate>, CommandError> {
    filters
        .iter()
        .map(|filter| {
            let Some((name, operator, value)) = find_first_operator(filter) else {
                return Err(CommandError::new(
                    DiagnosticCode::InvalidSelector,
                    format!(
                        "invalid filter: {:?} (use col=val, col!=val, or col~=substr)",
                        filter
                    ),
                )
                .with_hint("write each --where filter as col=val, col!=val, or col~=substr"));
            };
            let column = column(name.trim());
            let value = value.trim().to_string();
            Ok(match operator {
                "!=" => TablePredicate::NotEqual(column, value),
                "~=" => TablePredicate::Contains(column, value),
                _ => TablePredicate::Equal(column, value),
            })
        })
        .collect()
}

fn find_first_operator(value: &str) -> Option<(&str, &str, &str)> {
    let bytes = value.as_bytes();
    for index in 0..bytes.len() {
        if index + 1 < bytes.len() && bytes[index] == b'!' && bytes[index + 1] == b'=' {
            return Some((&value[..index], "!=", &value[index + 2..]));
        }
        if index + 1 < bytes.len() && bytes[index] == b'~' && bytes[index + 1] == b'=' {
            return Some((&value[..index], "~=", &value[index + 2..]));
        }
        if bytes[index] == b'=' {
            return Some((&value[..index], "=", &value[index + 1..]));
        }
    }
    None
}
