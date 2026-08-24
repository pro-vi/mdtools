use std::path::Path;

use crate::cli::SearchArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use mdtools::document::Document;
use mdtools::search::{self, SearchQuery};

pub fn run(args: &SearchArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, json, |file| {
        process_file(file, args, json, multi)
    })
}

fn process_file(
    file: &Path,
    args: &SearchArgs,
    json: bool,
    multi: bool,
) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let document = Document::parse(source)?;
    let file_name = file.to_string_lossy();
    let match_mode = if args.ignore_case {
        SearchMatchMode::LiteralIgnoreCase
    } else {
        SearchMatchMode::Literal
    };
    let block_kinds = if args.kinds.is_empty() {
        search::ALL_BLOCK_KINDS.to_vec()
    } else {
        args.kinds.clone()
    };
    let query = SearchQuery {
        text: args.query.clone(),
        match_mode,
        block_kinds: block_kinds.clone(),
    };
    let matches = search::search(&document, &query);

    if json {
        output::write_json(&SearchResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: file_name.to_string(),
            query: args.query.clone(),
            match_mode,
            block_kinds,
            matches,
        })?;
    } else {
        for found in &matches {
            let preview = output::escape_text_field(&found.preview);
            if multi {
                println!(
                    "{}:\t{}\t{}\t{}-{}\t{}",
                    file_name,
                    found.block_index,
                    found.block_kind,
                    found.match_span.line_start,
                    found.match_span.line_end,
                    preview
                );
            } else {
                println!(
                    "{}\t{}\t{}-{}\t{}",
                    found.block_index,
                    found.block_kind,
                    found.match_span.line_start,
                    found.match_span.line_end,
                    preview
                );
            }
        }
    }
    Ok(())
}
