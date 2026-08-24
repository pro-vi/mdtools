use std::path::Path;

use crate::cli::StatsArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use mdtools::document::Document;

pub fn run(args: &StatsArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, json, |file| process_file(file, json, multi))
}

fn process_file(file: &Path, json: bool, multi: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = Document::parse(source)?;
    let file_str = file.to_string_lossy();
    let stats = mdtools::stats::document_stats(&doc);

    if json {
        let result = StatsResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: file_str.to_string(),
            stats,
        };
        output::write_json(&result)?;
    } else if multi {
        println!("{}:\twords={}", file_str, stats.word_count);
        println!("{}:\theadings={}", file_str, stats.heading_count);
        println!("{}:\tblocks={}", file_str, stats.block_count);
        println!("{}:\tlinks={}", file_str, stats.link_count);
        println!("{}:\tsections={}", file_str, stats.section_count);
        println!("{}:\tlines={}", file_str, stats.line_count);
    } else {
        println!("words={}", stats.word_count);
        println!("headings={}", stats.heading_count);
        println!("blocks={}", stats.block_count);
        println!("links={}", stats.link_count);
        println!("sections={}", stats.section_count);
        println!("lines={}", stats.line_count);
    }
    Ok(())
}
