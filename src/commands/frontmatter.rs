use std::path::Path;

use crate::cli::FrontmatterArgs;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &FrontmatterArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();

    if args.fields.is_empty() {
        multifile::for_each_file(&file_set, |file| process_file(file, json))
    } else {
        multifile::for_each_file(&file_set, |file| {
            run_field_projection(file, &args.fields, json, multi)
        })
    }
}

fn process_file(file: &Path, _json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = ParsedDocument::parse_for_frontmatter(source)?;

    let result = match &doc.frontmatter {
        Some(fm) => {
            let parsed_data = parse_frontmatter_data(&fm.raw, fm.format)?;
            FrontmatterReadResult {
                schema_version: SCHEMA_VERSION.to_string(),
                file: file.to_string_lossy().to_string(),
                present: true,
                frontmatter: Some(FrontmatterEnvelope {
                    format: fm.format,
                    span: fm.span,
                    raw: fm.raw.clone(),
                    data: parsed_data,
                }),
            }
        }
        None => FrontmatterReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: file.to_string_lossy().to_string(),
            present: false,
            frontmatter: None,
        },
    };

    // Frontmatter always emits JSON (both text and --json modes)
    output::write_json(&result)?;
    Ok(())
}

fn run_field_projection(
    file: &Path,
    fields: &[String],
    json: bool,
    multi: bool,
) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = ParsedDocument::parse_for_frontmatter(source)?;
    let file_str = file.to_string_lossy();

    let data = match &doc.frontmatter {
        Some(fm) => parse_frontmatter_data(&fm.raw, fm.format)?,
        None => serde_json::Value::Object(serde_json::Map::new()),
    };

    if json {
        let mut field_map = serde_json::Map::new();
        for field in fields {
            let val = extract_field(&data, field);
            field_map.insert(field.clone(), val);
        }
        let proj = serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "file": file_str,
            "fields": field_map,
        });
        output::write_json(&proj)?;
    } else {
        let values: Vec<String> = fields
            .iter()
            .map(|f| format_field_value(&extract_field(&data, f)))
            .collect();
        if multi {
            println!("{}\t{}", file_str, values.join("\t"));
        } else {
            println!("{}", values.join("\t"));
        }
    }
    Ok(())
}

fn extract_field(data: &serde_json::Value, field: &str) -> serde_json::Value {
    let mut current = data;
    for segment in field.split('.') {
        match current.get(segment) {
            Some(v) => current = v,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

fn format_field_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn parse_frontmatter_data(
    raw: &str,
    format: FrontmatterFormat,
) -> Result<serde_json::Value, CommandError> {
    let content = strip_frontmatter_delimiters(raw);

    match format {
        FrontmatterFormat::Yaml => {
            serde_yaml::from_str::<serde_json::Value>(&content).map_err(|e| {
                CommandError::new(
                    DiagnosticCode::FrontmatterParseFailed,
                    format!("invalid YAML frontmatter: {}", e),
                )
            })
        }
        FrontmatterFormat::Toml => {
            let toml_value: toml::Value = content.parse().map_err(|e: toml::de::Error| {
                CommandError::new(
                    DiagnosticCode::FrontmatterParseFailed,
                    format!("invalid TOML frontmatter: {}", e),
                )
            })?;
            serde_json::to_value(&toml_value).map_err(|e| {
                CommandError::new(
                    DiagnosticCode::FrontmatterParseFailed,
                    format!("TOML to JSON conversion failed: {}", e),
                )
            })
        }
    }
}

fn strip_frontmatter_delimiters(raw: &str) -> String {
    crate::parser::strip_frontmatter_delimiters(raw)
}
