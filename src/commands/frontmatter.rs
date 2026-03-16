use crate::cli::FrontmatterArgs;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &FrontmatterArgs, _json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse_for_frontmatter(source)?;

    let result = match &doc.frontmatter {
        Some(fm) => {
            let parsed_data = parse_frontmatter_data(&fm.raw, fm.format)?;
            FrontmatterReadResult {
                schema_version: SCHEMA_VERSION.to_string(),
                file: args.file.to_string_lossy().to_string(),
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
            file: args.file.to_string_lossy().to_string(),
            present: false,
            frontmatter: None,
        },
    };

    // Frontmatter always emits JSON (both text and --json modes)
    output::write_json(&result)?;
    Ok(())
}

fn parse_frontmatter_data(
    raw: &str,
    format: FrontmatterFormat,
) -> Result<serde_json::Value, CommandError> {
    // Strip delimiters and trailing whitespace from raw content
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
            // Convert TOML value to JSON value
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
