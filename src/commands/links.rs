use std::path::Path;

use crate::cli::LinksArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use mdtools::document::Document;
use mdtools::link;

pub fn run(args: &LinksArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, json, |file| process_file(file, json, multi))
}

fn process_file(file: &Path, json: bool, multi: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = Document::parse(source)?;
    let file_str = file.to_string_lossy();

    let links: Vec<LinkEntry> = link::links(&doc)
        .into_iter()
        .map(|link| LinkEntry {
            kind: link.kind,
            text: link.text,
            destination: link.destination,
            title: link.title,
            source_block_index: link.source_block_index,
            span: link.span,
        })
        .collect();

    if json {
        let result = LinksResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: file_str.to_string(),
            links,
        };
        output::write_json(&result)?;
    } else {
        for link in &links {
            let dest = link.destination.as_deref().unwrap_or("");
            let dest = output::escape_text_field(dest);
            if multi {
                println!(
                    "{}:\t{}\t{}\tblock:{}\t{}-{}",
                    file_str,
                    link.kind,
                    dest,
                    link.source_block_index,
                    link.span.line_start,
                    link.span.line_end
                );
            } else {
                println!(
                    "{}\t{}\tblock:{}\t{}-{}",
                    link.kind,
                    dest,
                    link.source_block_index,
                    link.span.line_start,
                    link.span.line_end
                );
            }
        }
    }
    Ok(())
}
