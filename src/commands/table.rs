use crate::cli::TableArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::output;
use crate::parser::{self, ParsedDocument};

pub fn run(args: &TableArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let table_blocks: Vec<_> = doc.blocks.iter().filter(|b| b.kind == BlockKind::Table).collect();

    if table_blocks.is_empty() {
        return Err(CommandError::no_tables());
    }

    match args.index {
        None if args.select.is_empty() => run_list_tables(&doc, &table_blocks, args, json),
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

fn run_list_tables(
    doc: &ParsedDocument,
    table_blocks: &[&crate::parser::BlockInfo],
    args: &TableArgs,
    json: bool,
) -> Result<(), CommandError> {
    let mut entries = Vec::new();
    for block in table_blocks {
        let table_source = doc.slice(&block.span);
        let data = parser::extract_table_data(&table_source)?;
        entries.push(TableEntry {
            block_index: block.index,
            span: block.span,
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
    let data = parser::extract_table_data(&table_source)?;

    let (headers, rows) = if args.select.is_empty() {
        (data.headers.clone(), data.rows.clone())
    } else {
        let indices = resolve_columns(&data.headers, &args.select)?;
        let projected_headers: Vec<String> = indices.iter().map(|&i| data.headers[i].clone()).collect();
        let projected_rows: Vec<Vec<String>> = data
            .rows
            .iter()
            .map(|row| indices.iter().map(|&i| row.get(i).cloned().unwrap_or_default()).collect())
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
            headers,
            alignments,
            rows,
        })?;
    } else {
        // TSV output
        println!("{}", headers.iter().map(|h| output::escape_text_field(h)).collect::<Vec<_>>().join("\t"));
        for row in &rows {
            println!("{}", row.iter().map(|c| output::escape_text_field(c)).collect::<Vec<_>>().join("\t"));
        }
    }
    Ok(())
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
