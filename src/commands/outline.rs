use std::path::Path;

use crate::cli::OutlineArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;
use mdtools::section::SectionIndex;

pub fn run(args: &OutlineArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, json, |file| process_file(file, json, multi))
}

fn process_file(file: &Path, json: bool, multi: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = ParsedDocument::parse(source)?;
    let file_str = file.to_string_lossy();
    let result = build_outline(&doc, &file_str);

    if json {
        output::write_json(&result)?;
    } else {
        for entry in &result.entries {
            let h = &entry.heading;
            let depth_marker = "#".repeat(h.level as usize);
            let text = output::escape_text_field(&h.text);
            if multi {
                println!(
                    "{}:\t{} {}\t{}-{}\tblock:{}",
                    file_str, depth_marker, text, h.span.line_start, h.span.line_end, h.block_index
                );
            } else {
                println!(
                    "{} {}\t{}-{}\tblock:{}",
                    depth_marker, text, h.span.line_start, h.span.line_end, h.block_index
                );
            }
        }
    }
    Ok(())
}

fn build_outline(doc: &ParsedDocument, file: &str) -> OutlineResult {
    OutlineResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string(),
        entries: SectionIndex::new(doc).outline(),
    }
}
