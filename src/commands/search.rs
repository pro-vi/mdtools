use std::path::Path;

use crate::cli::SearchArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SparseLowercaseProvenance {
    irregular_segments: Vec<IrregularLowercaseSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IrregularLowercaseSegment {
    folded_start: usize,
    folded_end: usize,
    original_start: usize,
    original_end: usize,
    cumulative_byte_delta_after: isize,
}

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

/// Lowercase content while recording only irregular source-scalars whose folded
/// projection expands or changes UTF-8 byte width.
fn lowercase_with_provenance(original: &str) -> (String, SparseLowercaseProvenance) {
    let mut lowered = String::new();
    let mut provenance = SparseLowercaseProvenance::default();
    let mut cumulative_byte_delta_after = 0isize;

    for (original_start, ch) in original.char_indices() {
        let original_end = original_start + ch.len_utf8();
        let folded_start = lowered.len();
        let original_len = ch.len_utf8();
        let mut lowered_scalar_count = 0usize;

        for lowered_ch in ch.to_lowercase() {
            lowered.push(lowered_ch);
            lowered_scalar_count += 1;
        }

        let folded_end = lowered.len();
        let folded_len = folded_end - folded_start;

        if lowered_scalar_count != 1 || folded_len != original_len {
            cumulative_byte_delta_after += folded_len as isize - original_len as isize;
            provenance
                .irregular_segments
                .push(IrregularLowercaseSegment {
                    folded_start,
                    folded_end,
                    original_start,
                    original_end,
                    cumulative_byte_delta_after,
                });
        }
    }

    (lowered, provenance)
}

fn map_lowercase_match_to_original(
    provenance: &SparseLowercaseProvenance,
    match_start: usize,
    match_end: usize,
) -> (usize, usize) {
    provenance.map_match_to_original(match_start, match_end)
}

impl SparseLowercaseProvenance {
    fn map_match_to_original(&self, match_start: usize, match_end: usize) -> (usize, usize) {
        debug_assert!(match_start < match_end);
        let original_start = self
            .segment_covering_start_boundary(match_start)
            .map(|segment| segment.original_start)
            .unwrap_or_else(|| {
                adjust_folded_offset(match_start, self.cumulative_byte_delta_before(match_start))
            });
        let original_end = self
            .segment_covering_end_boundary(match_end)
            .map(|segment| segment.original_end)
            .unwrap_or_else(|| {
                adjust_folded_offset(match_end, self.cumulative_byte_delta_before(match_end))
            });
        (original_start, original_end)
    }

    fn cumulative_byte_delta_before(&self, folded_offset: usize) -> isize {
        let next_segment = self
            .irregular_segments
            .partition_point(|segment| segment.folded_end <= folded_offset);
        if next_segment == 0 {
            0
        } else {
            self.irregular_segments[next_segment - 1].cumulative_byte_delta_after
        }
    }

    fn segment_covering_start_boundary(
        &self,
        folded_offset: usize,
    ) -> Option<&IrregularLowercaseSegment> {
        let next_segment = self
            .irregular_segments
            .partition_point(|segment| segment.folded_end <= folded_offset);
        self.irregular_segments.get(next_segment).filter(|segment| {
            segment.folded_start <= folded_offset && folded_offset < segment.folded_end
        })
    }

    fn segment_covering_end_boundary(
        &self,
        folded_offset: usize,
    ) -> Option<&IrregularLowercaseSegment> {
        let next_segment = self
            .irregular_segments
            .partition_point(|segment| segment.folded_end < folded_offset);
        self.irregular_segments.get(next_segment).filter(|segment| {
            segment.folded_start < folded_offset && folded_offset <= segment.folded_end
        })
    }
}

fn adjust_folded_offset(folded_offset: usize, cumulative_byte_delta: isize) -> usize {
    if cumulative_byte_delta >= 0 {
        debug_assert!(folded_offset >= cumulative_byte_delta as usize);
        folded_offset - cumulative_byte_delta as usize
    } else {
        folded_offset + cumulative_byte_delta.unsigned_abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_sparse_provenance_large_ascii_records_zero_irregular_segments_and_maps_match_exactly()
    {
        let original = "ABCD".repeat(1_024);
        let (lowered, provenance) = lowercase_with_provenance(&original);

        assert_eq!(lowered.len(), original.len());
        assert!(provenance.irregular_segments.is_empty());
        assert_eq!(
            map_lowercase_match_to_original(&provenance, 1_024, 2_048),
            (1_024, 2_048)
        );
    }

    #[test]
    fn search_sparse_provenance_maps_contraction_and_mixed_deltas() {
        let (lowered, provenance) = lowercase_with_provenance("AKİXZ");

        assert_eq!(lowered, "aki\u{307}xz");
        assert_eq!(provenance.irregular_segments.len(), 2);
        assert_eq!(map_lowercase_match_to_original(&provenance, 1, 2), (1, 4));
        assert_eq!(map_lowercase_match_to_original(&provenance, 2, 5), (4, 6));
        assert_eq!(map_lowercase_match_to_original(&provenance, 5, 6), (6, 7));
    }
}
