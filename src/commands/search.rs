use std::path::Path;

use crate::cli::SearchArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

type SourceScalarRange = (usize, usize);

pub fn run(args: &SearchArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, |file| process_file(file, args, json, multi))
}

fn process_file(
    file: &Path,
    args: &SearchArgs,
    json: bool,
    multi: bool,
) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = ParsedDocument::parse(source)?;
    let file_str = file.to_string_lossy();

    let match_mode = if args.ignore_case {
        SearchMatchMode::LiteralIgnoreCase
    } else {
        SearchMatchMode::Literal
    };

    let filter_kinds: Vec<BlockKind> = if args.kinds.is_empty() {
        vec![
            BlockKind::Heading,
            BlockKind::Paragraph,
            BlockKind::List,
            BlockKind::BlockQuote,
            BlockKind::CodeFence,
            BlockKind::IndentedCode,
            BlockKind::ThematicBreak,
            BlockKind::Table,
            BlockKind::HtmlBlock,
            BlockKind::FootnoteDefinition,
        ]
    } else {
        args.kinds.clone()
    };

    let mut matches = Vec::new();

    for block in &doc.blocks {
        if !filter_kinds.contains(&block.kind) {
            continue;
        }

        let content = doc.slice(&block.span);
        let found = find_matches_in_content(
            content,
            &args.query,
            args.ignore_case,
            block.index,
            block.kind,
            block.span.byte_start,
            block.span.line_start,
        );
        matches.extend(found);
    }

    if json {
        let result = SearchResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: file_str.to_string(),
            query: args.query.clone(),
            match_mode,
            block_kinds: filter_kinds,
            matches,
        };
        output::write_json(&result)?;
    } else {
        for m in &matches {
            let preview = output::escape_text_field(&m.preview);
            if multi {
                println!(
                    "{}:\t{}\t{}\t{}-{}\t{}",
                    file_str,
                    m.block_index,
                    m.block_kind,
                    m.match_span.line_start,
                    m.match_span.line_end,
                    preview
                );
            } else {
                println!(
                    "{}\t{}\t{}-{}\t{}",
                    m.block_index,
                    m.block_kind,
                    m.match_span.line_start,
                    m.match_span.line_end,
                    preview
                );
            }
        }
    }
    Ok(())
}

fn find_matches_in_content(
    content: &str,
    query: &str,
    ignore_case: bool,
    block_index: u32,
    block_kind: BlockKind,
    block_byte_start: u32,
    block_line_start: u32,
) -> Vec<SearchMatch> {
    let mut results = Vec::new();

    if query.is_empty() {
        return results;
    }

    if ignore_case {
        let (haystack, provenance) = lowercase_with_provenance(content);
        let needle = query.to_lowercase();
        let mut search_start = 0usize;
        while search_start < haystack.len() {
            if let Some(pos) = haystack[search_start..].find(&needle) {
                let match_start = search_start + pos;
                let match_end = match_start + needle.len();

                let (orig_start, orig_end) =
                    map_lowercase_match_to_original(&provenance, match_start, match_end);

                push_match(
                    &mut results,
                    content,
                    orig_start,
                    orig_end,
                    block_byte_start,
                    block_line_start,
                    block_index,
                    block_kind,
                );

                // Advance past this match, respecting char boundaries in haystack
                search_start = next_char_boundary(&haystack, match_start + 1);
            } else {
                break;
            }
        }
    } else {
        let mut search_start = 0usize;
        while search_start < content.len() {
            if let Some(pos) = content[search_start..].find(query) {
                let match_start = search_start + pos;
                let match_end = match_start + query.len();

                push_match(
                    &mut results,
                    content,
                    match_start,
                    match_end,
                    block_byte_start,
                    block_line_start,
                    block_index,
                    block_kind,
                );

                // Advance past this match, respecting char boundaries
                search_start = next_char_boundary(content, match_start + 1);
            } else {
                break;
            }
        }
    }

    results
}

fn push_match(
    results: &mut Vec<SearchMatch>,
    content: &str,
    match_start: usize,
    match_end: usize,
    block_byte_start: u32,
    block_line_start: u32,
    block_index: u32,
    block_kind: BlockKind,
) {
    let abs_byte_start = block_byte_start as usize + match_start;
    let abs_byte_end = block_byte_start as usize + match_end;

    let lines_before = content[..match_start].matches('\n').count() as u32;
    let match_line_start = block_line_start + lines_before;
    let lines_in_match = content[match_start..match_end].matches('\n').count() as u32;
    let match_line_end = match_line_start + lines_in_match;

    let line_start_in_content = content[..match_start]
        .rfind('\n')
        .map(|p| p + 1)
        .unwrap_or(0);
    let line_end_in_content = content[match_end..]
        .find('\n')
        .map(|p| match_end + p)
        .unwrap_or(content.len());
    let preview_line = &content[line_start_in_content..line_end_in_content];

    results.push(SearchMatch {
        block_index,
        block_kind,
        match_span: SourceSpan {
            line_start: match_line_start,
            line_end: match_line_end,
            byte_start: abs_byte_start as u32,
            byte_end: abs_byte_end as u32,
        },
        preview: output::truncate_preview(preview_line, 80),
    });
}

/// Find the next valid char boundary at or after `pos`.
fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut p = pos;
    while !s.is_char_boundary(p) && p < s.len() {
        p += 1;
    }
    p
}

/// Lowercase content while recording, for each emitted byte, the original
/// source-scalar byte range that produced it.
fn lowercase_with_provenance(original: &str) -> (String, Vec<SourceScalarRange>) {
    let mut lowered = String::new();
    let mut provenance = Vec::new();

    for (orig_start, ch) in original.char_indices() {
        let orig_end = orig_start + ch.len_utf8();
        for lowered_ch in ch.to_lowercase() {
            let mut buf = [0; 4];
            let lowered_fragment = lowered_ch.encode_utf8(&mut buf);
            lowered.push_str(lowered_fragment);
            provenance.extend(
                std::iter::repeat((orig_start, orig_end)).take(lowered_fragment.len()),
            );
        }
    }

    (lowered, provenance)
}

fn map_lowercase_match_to_original(
    provenance: &[SourceScalarRange],
    match_start: usize,
    match_end: usize,
) -> (usize, usize) {
    debug_assert!(match_start < match_end);
    let orig_start = provenance[match_start].0;
    let orig_end = provenance[match_end - 1].1;
    (orig_start, orig_end)
}
