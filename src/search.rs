use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::model::{BlockKind, SearchMatch, SearchMatchMode, SourceSpan};

pub const ALL_BLOCK_KINDS: &[BlockKind] = &[
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
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchQuery {
    pub text: String,
    pub match_mode: SearchMatchMode,
    pub block_kinds: Vec<BlockKind>,
}

impl SearchQuery {
    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: ALL_BLOCK_KINDS.to_vec(),
        }
    }

    pub fn literal_ignore_case(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            match_mode: SearchMatchMode::LiteralIgnoreCase,
            block_kinds: ALL_BLOCK_KINDS.to_vec(),
        }
    }
}

pub fn search(document: &Document, query: &SearchQuery) -> Vec<SearchMatch> {
    let block_kinds = if query.block_kinds.is_empty() {
        ALL_BLOCK_KINDS
    } else {
        &query.block_kinds
    };
    document
        .blocks()
        .iter()
        .filter(|block| block_kinds.contains(&block.kind))
        .flat_map(|block| {
            find_matches_in_content(
                document.slice_unchecked(&block.span),
                &query.text,
                query.match_mode == SearchMatchMode::LiteralIgnoreCase,
                block.index,
                block.kind,
                block.span.byte_start,
                block.span.line_start,
            )
        })
        .collect()
}

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
            let Some(position) = haystack[search_start..].find(&needle) else {
                break;
            };
            let match_start = search_start + position;
            let match_end = match_start + needle.len();
            let (original_start, original_end) =
                provenance.map_match_to_original(match_start, match_end);
            push_match(
                &mut results,
                content,
                original_start,
                original_end,
                block_byte_start,
                block_line_start,
                block_index,
                block_kind,
            );
            search_start = next_char_boundary(&haystack, match_start + 1);
        }
    } else {
        let mut search_start = 0usize;
        while search_start < content.len() {
            let Some(position) = content[search_start..].find(query) else {
                break;
            };
            let match_start = search_start + position;
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
            search_start = next_char_boundary(content, match_start + 1);
        }
    }
    results
}

#[allow(clippy::too_many_arguments)]
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
    let match_line_start = block_line_start + content[..match_start].matches('\n').count() as u32;
    let match_line_end =
        match_line_start + content[match_start..match_end].matches('\n').count() as u32;
    let preview_start = content[..match_start]
        .rfind('\n')
        .map(|position| position + 1)
        .unwrap_or(0);
    let preview_end = content[match_end..]
        .find('\n')
        .map(|position| match_end + position)
        .unwrap_or(content.len());

    results.push(SearchMatch {
        block_index,
        block_kind,
        match_span: SourceSpan {
            line_start: match_line_start,
            line_end: match_line_end,
            byte_start: block_byte_start + match_start as u32,
            byte_end: block_byte_start + match_end as u32,
        },
        etag: TargetEtag::for_bytes(&content.as_bytes()[match_start..match_end]),
        preview: preview(&content[preview_start..preview_end]),
    });
}

fn preview(content: &str) -> String {
    let escaped = content
        .chars()
        .map(|character| match character {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect::<String>();
    if escaped.chars().count() <= 80 {
        escaped
    } else {
        format!("{}...", escaped.chars().take(80).collect::<String>())
    }
}

fn next_char_boundary(value: &str, position: usize) -> usize {
    if position >= value.len() {
        return value.len();
    }
    let mut boundary = position;
    while boundary < value.len() && !value.is_char_boundary(boundary) {
        boundary += 1;
    }
    boundary
}

fn lowercase_with_provenance(original: &str) -> (String, SparseLowercaseProvenance) {
    let mut lowered = String::new();
    let mut provenance = SparseLowercaseProvenance::default();
    let mut cumulative_byte_delta_after = 0isize;
    for (original_start, character) in original.char_indices() {
        let original_end = original_start + character.len_utf8();
        let folded_start = lowered.len();
        let original_len = character.len_utf8();
        let mut lowered_scalar_count = 0usize;
        for lowered_character in character.to_lowercase() {
            lowered.push(lowered_character);
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
        let next = self
            .irregular_segments
            .partition_point(|segment| segment.folded_end <= folded_offset);
        if next == 0 {
            0
        } else {
            self.irregular_segments[next - 1].cumulative_byte_delta_after
        }
    }

    fn segment_covering_start_boundary(
        &self,
        folded_offset: usize,
    ) -> Option<&IrregularLowercaseSegment> {
        let next = self
            .irregular_segments
            .partition_point(|segment| segment.folded_end <= folded_offset);
        self.irregular_segments.get(next).filter(|segment| {
            segment.folded_start <= folded_offset && folded_offset < segment.folded_end
        })
    }

    fn segment_covering_end_boundary(
        &self,
        folded_offset: usize,
    ) -> Option<&IrregularLowercaseSegment> {
        let next = self
            .irregular_segments
            .partition_point(|segment| segment.folded_end < folded_offset);
        self.irregular_segments.get(next).filter(|segment| {
            segment.folded_start < folded_offset && folded_offset <= segment.folded_end
        })
    }
}

fn adjust_folded_offset(folded_offset: usize, cumulative_byte_delta: isize) -> usize {
    if cumulative_byte_delta >= 0 {
        folded_offset - cumulative_byte_delta as usize
    } else {
        folded_offset + cumulative_byte_delta.unsigned_abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_provenance_maps_mixed_byte_deltas() {
        let (lowered, provenance) = lowercase_with_provenance("AKİXZ");
        assert_eq!(lowered, "aki\u{307}xz");
        assert_eq!(provenance.map_match_to_original(1, 2), (1, 4));
        assert_eq!(provenance.map_match_to_original(2, 5), (4, 6));
        assert_eq!(provenance.map_match_to_original(5, 6), (6, 7));
    }
}
