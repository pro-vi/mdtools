use crate::cli::CollectArgs;
use crate::commands::frontmatter::{extract_field, format_field_value, parse_frontmatter_data};
use crate::errors::{CommandError, MdExitCode};
use crate::model::{CollectResult, SCHEMA_VERSION};
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &CollectArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let mut paths = file_set.paths.clone();
    paths.sort();
    let aggregate_partial_failures = paths.len() > 1;

    let mut rows = Vec::new();
    let mut error_count = 0u32;
    let mut worst_code = MdExitCode::Success;

    for path in &paths {
        match collect_row(path, &args.fields) {
            Ok(row) => rows.push(row),
            Err(err) => {
                if !aggregate_partial_failures {
                    return Err(err);
                }
                multifile::report_file_error(path, &err);
                if (err.exit_code as u8) > (worst_code as u8) {
                    worst_code = err.exit_code;
                }
                error_count += 1;
            }
        }
    }

    let headers = collect_headers(&args.fields);
    if json {
        output::write_json(&CollectResult {
            schema_version: SCHEMA_VERSION.to_string(),
            headers,
            rows,
        })?;
    } else {
        write_tsv(&headers, &rows);
    }

    if error_count == 0 {
        Ok(())
    } else {
        Err(CommandError {
            exit_code: worst_code,
            message: format!("{} file(s) failed", error_count),
        })
    }
}

fn collect_headers(fields: &[String]) -> Vec<String> {
    let mut headers = Vec::with_capacity(fields.len() + 1);
    headers.push("path".to_string());
    headers.extend(fields.iter().cloned());
    headers
}

fn collect_row(
    path: &std::path::Path,
    fields: &[String],
) -> Result<Vec<serde_json::Value>, CommandError> {
    let source = std::fs::read_to_string(path)?;
    let doc = ParsedDocument::parse_for_frontmatter(source)?;
    let data = frontmatter_data(&doc)?;

    let mut row = Vec::with_capacity(fields.len() + 1);
    row.push(serde_json::Value::String(
        path.to_string_lossy().to_string(),
    ));
    for field in fields {
        row.push(extract_field(&data, field));
    }
    Ok(row)
}

fn frontmatter_data(doc: &ParsedDocument) -> Result<serde_json::Value, CommandError> {
    match &doc.frontmatter {
        Some(fm) => parse_frontmatter_data(&fm.raw, fm.format),
        None => Ok(empty_frontmatter_object()),
    }
}

fn empty_frontmatter_object() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

fn write_tsv(headers: &[String], rows: &[Vec<serde_json::Value>]) {
    println!(
        "{}",
        headers
            .iter()
            .map(|header| output::escape_text_field(header))
            .collect::<Vec<_>>()
            .join("\t")
    );
    for row in rows {
        println!(
            "{}",
            row.iter()
                .map(format_tsv_cell)
                .collect::<Vec<_>>()
                .join("\t")
        );
    }
}

fn format_tsv_cell(value: &serde_json::Value) -> String {
    output::escape_text_field(&format_field_value(value))
}
