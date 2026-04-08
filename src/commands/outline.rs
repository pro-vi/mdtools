use std::path::Path;

use crate::cli::OutlineArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &OutlineArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, |file| process_file(file, json, multi))
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
    let mut entries = Vec::new();

    let heading_blocks: Vec<_> = doc.blocks.iter().filter(|b| b.heading.is_some()).collect();

    for block in &heading_blocks {
        let h = block.heading.as_ref().unwrap();

        // Section span: from this heading to the next heading of same or higher level,
        // or to the next heading block, or to end of document.
        let section_byte_end = find_section_end(doc, block.index, h.level);
        let section_line_end = if section_byte_end as usize >= doc.source.len() {
            doc.line_count()
        } else {
            let line_at_end = doc.byte_to_line(section_byte_end);
            if section_byte_end > 0
                && doc.source.as_bytes().get(section_byte_end as usize - 1) == Some(&b'\n')
            {
                line_at_end - 1
            } else {
                line_at_end
            }
        };

        let heading_ref = HeadingRef {
            level: h.level,
            text: h.text.clone(),
            block_index: block.index,
            span: block.span,
        };

        let section_span = SourceSpan {
            line_start: block.span.line_start,
            line_end: section_line_end,
            byte_start: block.span.byte_start,
            byte_end: section_byte_end,
        };

        entries.push(OutlineEntry {
            heading: heading_ref,
            section_span,
        });
    }

    OutlineResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string(),
        entries,
    }
}

/// Find the byte offset where a section ends.
/// A section ends before the next heading at same or higher level (<=),
/// or at the end of the document.
fn find_section_end(doc: &ParsedDocument, heading_block_index: u32, heading_level: u8) -> u32 {
    for block in &doc.blocks {
        if block.index <= heading_block_index {
            continue;
        }
        if let Some(h) = &block.heading {
            if h.level <= heading_level {
                // Section ends before this heading block
                return block.span.byte_start;
            }
        }
    }
    // Section extends to end of document
    doc.source.len() as u32
}
