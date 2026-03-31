use std::path::Path;

use crate::cli::SearchArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &SearchArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, |file| process_file(file, args, json, multi))
}

fn process_file(file: &Path, args: &SearchArgs, json: bool, multi: bool) -> Result<(), CommandError> {
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
                    file_str, m.block_index, m.block_kind, m.match_span.line_start, m.match_span.line_end, preview
                );
            } else {
                println!(
                    "{}\t{}\t{}-{}\t{}",
                    m.block_index, m.block_kind, m.match_span.line_start, m.match_span.line_end, preview
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

    // For case-insensitive search, we search the lowercased content but report
    // positions in the original content. This works because to_lowercase() preserves
    // char count for the scripts we care about (Latin, CJK, etc.), and we use
    // char-based iteration to map between the two.
    //
    // For case-sensitive search, haystack IS content, so positions map directly.

    if ignore_case {
        let haystack = content.to_lowercase();
        let needle = query.to_lowercase();
        let mut search_start = 0usize;
        while search_start < haystack.len() {
            if let Some(pos) = haystack[search_start..].find(&needle) {
                let match_start = search_start + pos;
                let match_end = match_start + needle.len();

                // Map lowercased positions back to original content positions
                // Since to_lowercase can change byte lengths, we map via char indices
                let orig_start = map_lowercase_pos_to_original(content, &haystack, match_start);
                let orig_end = map_lowercase_pos_to_original(content, &haystack, match_end);

                push_match(
                    &mut results, content, orig_start, orig_end,
                    block_byte_start, block_line_start, block_index, block_kind,
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
                    &mut results, content, match_start, match_end,
                    block_byte_start, block_line_start, block_index, block_kind,
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

    let line_start_in_content = content[..match_start].rfind('\n').map(|p| p + 1).unwrap_or(0);
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

/// Map a byte position in the lowercased string back to the corresponding
/// byte position in the original string, using char-by-char alignment.
fn map_lowercase_pos_to_original(original: &str, lowered: &str, lowered_pos: usize) -> usize {
    let mut orig_byte = 0;
    let mut low_byte = 0;

    let mut orig_chars = original.chars();
    let mut low_chars = lowered.chars();

    loop {
        if low_byte >= lowered_pos {
            return orig_byte;
        }
        match (orig_chars.next(), low_chars.next()) {
            (Some(oc), Some(lc)) => {
                orig_byte += oc.len_utf8();
                low_byte += lc.len_utf8();
            }
            _ => return orig_byte,
        }
    }
}
