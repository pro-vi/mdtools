use std::path::Path;

use crate::cli::FrontmatterArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use mdtools::document::Document;
use mdtools::frontmatter;

pub fn run(args: &FrontmatterArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    if args.fields.is_empty() {
        multifile::for_each_file(&file_set, json, |file| process_file(file))
    } else {
        multifile::for_each_file(&file_set, json, |file| {
            run_field_projection(file, &args.fields, json, multi)
        })
    }
}

fn process_file(file: &Path) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let document = Document::parse_for_frontmatter(source)?;
    let record = frontmatter::read(&document)?;
    let frontmatter = match (record.format, record.span, record.raw) {
        (Some(format), Some(span), Some(raw)) => Some(FrontmatterEnvelope {
            format,
            span,
            raw,
            data: record.data,
        }),
        _ => None,
    };
    output::write_json(&FrontmatterReadResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string_lossy().to_string(),
        present: record.present,
        etag: record.etag.into_string(),
        frontmatter,
    })?;
    Ok(())
}

fn run_field_projection(
    file: &Path,
    fields: &[String],
    json: bool,
    multi: bool,
) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let document = Document::parse_for_frontmatter(source)?;
    let record = frontmatter::read(&document)?;
    let file_name = file.to_string_lossy();
    if json {
        let projected = fields
            .iter()
            .map(|field| {
                (
                    field.clone(),
                    frontmatter::project_field(&record.data, field),
                )
            })
            .collect();
        output::write_json(&FrontmatterFieldProjectionResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: file_name.to_string(),
            present: record.present,
            etag: record.etag.into_string(),
            fields: projected,
        })?;
    } else {
        let values = fields
            .iter()
            .map(|field| format_field_value(&frontmatter::project_field(&record.data, field)))
            .collect::<Vec<_>>();
        if multi {
            println!("{}\t{}", file_name, values.join("\t"));
        } else {
            println!("{}", values.join("\t"));
        }
    }
    Ok(())
}

pub(crate) fn format_field_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        other => other.to_string(),
    }
}
