use crate::cli::{DeleteTableRowArgs, InsertTableRowArgs, ReplaceTableRowArgs, TableArgs};
use crate::commands::replace::{
    emit_mutation, inserted_span_after, strip_one_trailing_newline, verify_expected_etag,
    MutationEmission,
};
use crate::errors::CommandError;
use crate::model::*;
use crate::output;
use crate::parser::{self, ParsedDocument};

pub fn run(args: &TableArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let table_blocks: Vec<_> = doc
        .blocks
        .iter()
        .filter(|b| b.kind == BlockKind::Table)
        .collect();

    if table_blocks.is_empty() {
        return Err(CommandError::no_tables());
    }

    match args.index {
        None if args.select.is_empty() && args.filters.is_empty() => {
            run_list_tables(&doc, &table_blocks, args, json)
        }
        None => {
            if table_blocks.len() == 1 {
                run_read_table(&doc, table_blocks[0], args, json)
            } else {
                Err(CommandError::new(
                    crate::errors::DiagnosticCode::InvalidSelector,
                    format!(
                        "document has {} tables; use --index to select one",
                        table_blocks.len()
                    ),
                ))
            }
        }
        Some(idx) => {
            let block = doc
                .blocks
                .get(idx as usize)
                .ok_or_else(|| CommandError::block_out_of_range(idx, doc.blocks.len() as u32))?;
            if block.kind != BlockKind::Table {
                return Err(CommandError::table_not_found(idx));
            }
            run_read_table(&doc, block, args, json)
        }
    }
}

pub fn run_replace_table_row(args: &ReplaceTableRowArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let block = doc
        .blocks
        .get(args.table_block_index as usize)
        .ok_or_else(|| {
            CommandError::block_out_of_range(args.table_block_index, doc.blocks.len() as u32)
        })?;
    if block.kind != BlockKind::Table {
        return Err(CommandError::table_not_found(args.table_block_index));
    }

    let table_source = doc.slice(&block.span);
    verify_expected_etag(
        args.etag_guard.expect_etag.as_deref(),
        table_source,
        |expected, actual| {
            CommandError::table_etag_mismatch(args.table_block_index, expected, actual)
        },
    )?;

    let table = parser::extract_table_projection(table_source, block.span)?;
    let row = table.rows.get(args.row_index as usize).ok_or_else(|| {
        CommandError::table_row_not_found(
            args.table_block_index,
            args.row_index,
            table.rows.len() as u32,
        )
    })?;

    let replacement = output::read_content(args.from.as_deref())?;
    let replacement = strip_one_trailing_newline(replacement);
    parser::validate_table_row_payload(&replacement, table.headers.len())?;

    let original_row = doc.slice(&row.span);
    let disposition = if replacement == original_row {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let changed = disposition != MutationDisposition::NoChange;

    let before = &doc.source[..row.span.byte_start as usize];
    let after = &doc.source[row.span.byte_end as usize..];
    let output_doc = format!("{}{}{}", before, replacement, after);

    let target = MutationTargetRef::TableRow(TableRowTargetRef {
        kind: MutationTargetKind::TableRow,
        table_block_index: args.table_block_index,
        row_index: args.row_index,
        span: row.span,
    });

    emit_mutation(MutationEmission {
        in_place: args.in_place,
        json,
        file: &args.file,
        command: MutationCommandKind::ReplaceTableRow,
        target,
        disposition,
        changed,
        line_endings: doc.line_ending_style(),
        span_before: Some(row.span),
        span_after: match disposition {
            MutationDisposition::NoChange => Some(row.span),
            MutationDisposition::Replaced => Some(
                crate::commands::replace::replacement_span_after(row.span, &replacement),
            ),
            MutationDisposition::Deleted | MutationDisposition::Inserted => None,
        },
        output_doc: &output_doc,
    })
}

pub fn run_insert_table_row(args: &InsertTableRowArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let block = doc
        .blocks
        .get(args.table_block_index as usize)
        .ok_or_else(|| {
            CommandError::block_out_of_range(args.table_block_index, doc.blocks.len() as u32)
        })?;
    if block.kind != BlockKind::Table {
        return Err(CommandError::table_not_found(args.table_block_index));
    }

    let table_source = doc.slice(&block.span);
    verify_expected_etag(
        args.etag_guard.expect_etag.as_deref(),
        table_source,
        |expected, actual| {
            CommandError::table_etag_mismatch(args.table_block_index, expected, actual)
        },
    )?;

    let table = parser::extract_table_projection(table_source, block.span)?;
    let row_count = table.rows.len() as u32;
    if args.row_index > row_count {
        return Err(CommandError::table_row_insertion_out_of_range(
            args.table_block_index,
            args.row_index,
            row_count,
        ));
    }

    let payload = output::read_content(args.from.as_deref())?;
    let payload = strip_one_trailing_newline(payload);
    parser::validate_table_row_payload(&payload, table.headers.len())?;

    let target = MutationTargetRef::TableRowInsertion(TableRowInsertionTargetRef {
        kind: MutationTargetKind::TableRowInsertion,
        table_block_index: args.table_block_index,
        row_index: args.row_index,
        table_span: block.span,
    });

    let insertion = resolve_table_row_insertion(&doc, block.span, &table, args.row_index)?;
    let before = &doc.source[..insertion.insert_byte];
    let after = &doc.source[insertion.insert_byte..];

    let mut output_doc =
        String::with_capacity(doc.source.len() + payload.len() + insertion.separator.len());
    output_doc.push_str(before);
    let payload_start = match insertion.separator_placement {
        SeparatorPlacement::BeforePayload => {
            output_doc.push_str(insertion.separator);
            let start = output_doc.len();
            output_doc.push_str(&payload);
            start
        }
        SeparatorPlacement::AfterPayload => {
            let start = output_doc.len();
            output_doc.push_str(&payload);
            output_doc.push_str(insertion.separator);
            start
        }
    };
    let payload_end = payload_start + payload.len();
    output_doc.push_str(after);
    let span_after = inserted_span_after(&output_doc, payload_start, payload_end)?;

    emit_mutation(MutationEmission {
        in_place: args.in_place,
        json,
        file: &args.file,
        command: MutationCommandKind::InsertTableRow,
        target,
        disposition: MutationDisposition::Inserted,
        changed: true,
        line_endings: doc.line_ending_style(),
        span_before: None,
        span_after: Some(span_after),
        output_doc: &output_doc,
    })
}

pub fn run_delete_table_row(args: &DeleteTableRowArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let block = doc
        .blocks
        .get(args.table_block_index as usize)
        .ok_or_else(|| {
            CommandError::block_out_of_range(args.table_block_index, doc.blocks.len() as u32)
        })?;
    if block.kind != BlockKind::Table {
        return Err(CommandError::table_not_found(args.table_block_index));
    }

    let table_source = doc.slice(&block.span);
    verify_expected_etag(
        args.etag_guard.expect_etag.as_deref(),
        table_source,
        |expected, actual| {
            CommandError::table_etag_mismatch(args.table_block_index, expected, actual)
        },
    )?;

    let table = parser::extract_table_projection(table_source, block.span)?;
    let row = table.rows.get(args.row_index as usize).ok_or_else(|| {
        CommandError::table_row_not_found(
            args.table_block_index,
            args.row_index,
            table.rows.len() as u32,
        )
    })?;
    let deletion_span = resolve_table_row_deletion_span(&doc, row.span);

    let before = &doc.source[..deletion_span.byte_start as usize];
    let after = &doc.source[deletion_span.byte_end as usize..];
    let output_doc = format!("{}{}", before, after);

    let target = MutationTargetRef::TableRow(TableRowTargetRef {
        kind: MutationTargetKind::TableRow,
        table_block_index: args.table_block_index,
        row_index: args.row_index,
        span: row.span,
    });

    emit_mutation(MutationEmission {
        in_place: args.in_place,
        json,
        file: &args.file,
        command: MutationCommandKind::DeleteTableRow,
        target,
        disposition: MutationDisposition::Deleted,
        changed: true,
        line_endings: doc.line_ending_style(),
        span_before: Some(deletion_span),
        span_after: None,
        output_doc: &output_doc,
    })
}

struct TableRowInsertionPlan<'a> {
    insert_byte: usize,
    separator: &'a str,
    separator_placement: SeparatorPlacement,
}

enum SeparatorPlacement {
    BeforePayload,
    AfterPayload,
}

fn resolve_table_row_insertion<'a>(
    doc: &'a ParsedDocument,
    block_span: SourceSpan,
    table: &parser::TableProjection,
    row_index: u32,
) -> Result<TableRowInsertionPlan<'a>, CommandError> {
    let source = doc.source.as_str();
    let row_count = table.rows.len() as u32;

    if row_index < row_count {
        let row = &table.rows[row_index as usize];
        let separator =
            line_boundary_before(source, row.span.byte_start as usize).ok_or_else(|| {
                CommandError::new(
                    crate::errors::DiagnosticCode::ParseFailed,
                    "could not resolve line boundary for table row insertion",
                )
            })?;
        return Ok(TableRowInsertionPlan {
            insert_byte: row.span.byte_start as usize,
            separator,
            separator_placement: SeparatorPlacement::AfterPayload,
        });
    }

    let separator = line_boundary_after(source, block_span.byte_end as usize)
        .or_else(|| {
            table
                .rows
                .last()
                .and_then(|row| line_boundary_before(source, row.span.byte_start as usize))
        })
        .or_else(|| last_line_boundary_within(doc.slice(&block_span)))
        .ok_or_else(|| {
            CommandError::new(
                crate::errors::DiagnosticCode::ParseFailed,
                "could not resolve line boundary for table row insertion",
            )
        })?;

    Ok(TableRowInsertionPlan {
        insert_byte: block_span.byte_end as usize,
        separator,
        separator_placement: SeparatorPlacement::BeforePayload,
    })
}

fn line_boundary_after(source: &str, byte_index: usize) -> Option<&str> {
    let tail = source.get(byte_index..)?;
    if tail.starts_with("\r\n") {
        Some(&tail[..2])
    } else if tail.starts_with('\n') {
        Some(&tail[..1])
    } else {
        None
    }
}

fn line_boundary_before(source: &str, byte_index: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    if byte_index > bytes.len() {
        None
    } else if byte_index >= 2 && bytes[byte_index - 2] == b'\r' && bytes[byte_index - 1] == b'\n' {
        source.get(byte_index - 2..byte_index)
    } else if byte_index >= 1 && bytes[byte_index - 1] == b'\n' {
        source.get(byte_index - 1..byte_index)
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

fn resolve_table_row_deletion_span(doc: &ParsedDocument, row_span: SourceSpan) -> SourceSpan {
    let source = doc.source.as_bytes();
    let start = row_span.byte_start as usize;
    let end = row_span.byte_end as usize;
    let (byte_start, byte_end) = if source[end..].starts_with(b"\r\n") {
        (start, end + 2)
    } else if source[end..].starts_with(b"\n") {
        (start, end + 1)
    } else if end == source.len() {
        if start >= 2 && &source[start - 2..start] == b"\r\n" {
            (start - 2, end)
        } else if start >= 1 && source[start - 1] == b'\n' {
            (start - 1, end)
        } else {
            (start, end)
        }
    } else {
        (start, end)
    };

    doc.span_for_byte_range(byte_start as u32, byte_end as u32)
}

fn run_list_tables(
    doc: &ParsedDocument,
    table_blocks: &[&crate::parser::BlockInfo],
    args: &TableArgs,
    json: bool,
) -> Result<(), CommandError> {
    let mut entries = Vec::new();
    for block in table_blocks {
        let table_source = doc.slice(&block.span);
        let data = parser::extract_table_projection(table_source, block.span)?;
        entries.push(TableEntry {
            block_index: block.index,
            span: block.span,
            etag: output::content_etag(table_source.as_bytes()),
            headers: data.headers,
            row_count: data.rows.len() as u32,
            column_count: data.alignments.len() as u32,
        });
    }

    if json {
        output::write_json(&TablesResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            tables: entries,
        })?;
    } else {
        for entry in &entries {
            let header_preview = entry.headers.join(", ");
            println!(
                "{}\t{}\t{} rows\t{} cols",
                entry.block_index, header_preview, entry.row_count, entry.column_count
            );
        }
    }
    Ok(())
}

fn run_read_table(
    doc: &ParsedDocument,
    block: &crate::parser::BlockInfo,
    args: &TableArgs,
    json: bool,
) -> Result<(), CommandError> {
    let table_source = doc.slice(&block.span);
    let data = parser::extract_table_projection(table_source, block.span)?;
    let table_etag = output::content_etag(table_source.as_bytes());

    // Filter rows (before projection, so --where can reference any column)
    let filtered_rows: Vec<Vec<String>> = if args.filters.is_empty() {
        data.rows.iter().map(|row| row.cells.clone()).collect()
    } else {
        let filters = parse_filters(&data.headers, &args.filters)?;
        data.rows
            .iter()
            .map(|row| row.cells.clone())
            .filter(|row| row_matches(row, &filters))
            .collect()
    };

    let (headers, rows) = if args.select.is_empty() {
        (data.headers.clone(), filtered_rows)
    } else {
        let indices = resolve_columns(&data.headers, &args.select)?;
        let projected_headers: Vec<String> =
            indices.iter().map(|&i| data.headers[i].clone()).collect();
        let projected_rows: Vec<Vec<String>> = filtered_rows
            .iter()
            .map(|row: &Vec<String>| {
                indices
                    .iter()
                    .map(|&i| row.get(i).cloned().unwrap_or_default())
                    .collect()
            })
            .collect();
        (projected_headers, projected_rows)
    };

    if json {
        let alignments = if args.select.is_empty() {
            data.alignments.clone()
        } else {
            let indices = resolve_columns(&data.headers, &args.select)?;
            indices.iter().map(|&i| data.alignments[i]).collect()
        };
        output::write_json(&TableReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            block_index: block.index,
            span: block.span,
            etag: table_etag,
            headers,
            alignments,
            rows,
        })?;
    } else {
        // TSV output
        println!(
            "{}",
            headers
                .iter()
                .map(|h| output::escape_text_field(h))
                .collect::<Vec<_>>()
                .join("\t")
        );
        for row in &rows {
            println!(
                "{}",
                row.iter()
                    .map(|c| output::escape_text_field(c))
                    .collect::<Vec<_>>()
                    .join("\t")
            );
        }
    }
    Ok(())
}

// --- Row filtering ---

enum FilterOp {
    Eq(usize, String),
    NotEq(usize, String),
    Contains(usize, String),
}

fn parse_filters(headers: &[String], filters: &[String]) -> Result<Vec<FilterOp>, CommandError> {
    let mut ops = Vec::new();
    for f in filters {
        // Find the first operator boundary. Try multi-char operators first
        // so that "col=a!=b" parses as col / = / a!=b, not col=a / != / b.
        let parsed = find_first_operator(f);
        match parsed {
            Some((col, "!=", val)) => {
                let idx = resolve_column(headers, col.trim())?;
                ops.push(FilterOp::NotEq(idx, val.trim().to_string()));
            }
            Some((col, "~=", val)) => {
                let idx = resolve_column(headers, col.trim())?;
                ops.push(FilterOp::Contains(idx, val.trim().to_string()));
            }
            Some((col, _, val)) => {
                let idx = resolve_column(headers, col.trim())?;
                ops.push(FilterOp::Eq(idx, val.trim().to_string()));
            }
            None => {
                return Err(CommandError::new(
                    crate::errors::DiagnosticCode::InvalidSelector,
                    format!(
                        "invalid filter: {:?} (use col=val, col!=val, or col~=substr)",
                        f
                    ),
                ));
            }
        }
    }
    Ok(ops)
}

/// Find the first operator in a filter string by scanning left-to-right.
/// Returns (column, operator, value). Checks for != and ~= before bare =
/// at each position so the operator is always the leftmost match.
fn find_first_operator(s: &str) -> Option<(&str, &str, &str)> {
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if i + 1 < bytes.len() {
            if bytes[i] == b'!' && bytes[i + 1] == b'=' {
                return Some((&s[..i], "!=", &s[i + 2..]));
            }
            if bytes[i] == b'~' && bytes[i + 1] == b'=' {
                return Some((&s[..i], "~=", &s[i + 2..]));
            }
        }
        if bytes[i] == b'=' {
            return Some((&s[..i], "=", &s[i + 1..]));
        }
    }
    None
}

fn resolve_column(headers: &[String], col: &str) -> Result<usize, CommandError> {
    if let Ok(idx) = col.parse::<usize>() {
        if idx >= headers.len() {
            return Err(CommandError::column_not_found(col, headers));
        }
        Ok(idx)
    } else {
        headers
            .iter()
            .position(|h| h == col)
            .ok_or_else(|| CommandError::column_not_found(col, headers))
    }
}

fn row_matches(row: &[String], filters: &[FilterOp]) -> bool {
    filters.iter().all(|f| match f {
        FilterOp::Eq(idx, val) => row.get(*idx) == Some(val),
        FilterOp::NotEq(idx, val) => row.get(*idx) != Some(val),
        FilterOp::Contains(idx, val) => row.get(*idx).is_some_and(|c| c.contains(val.as_str())),
    })
}

fn resolve_columns(headers: &[String], select: &[String]) -> Result<Vec<usize>, CommandError> {
    let mut indices = Vec::new();
    for s in select {
        if let Ok(idx) = s.parse::<usize>() {
            if idx >= headers.len() {
                return Err(CommandError::column_not_found(s, headers));
            }
            indices.push(idx);
        } else {
            match headers.iter().position(|h| h == s) {
                Some(idx) => indices.push(idx),
                None => return Err(CommandError::column_not_found(s, headers)),
            }
        }
    }
    Ok(indices)
}
