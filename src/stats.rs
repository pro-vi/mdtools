use crate::document::Document;
use crate::model::{BlockKind, DocumentStats};

pub fn document_stats(document: &Document) -> DocumentStats {
    let heading_count = document
        .blocks()
        .iter()
        .filter(|block| block.heading.is_some())
        .count() as u32;
    let block_count = document.blocks().len() as u32;
    let link_count = document
        .blocks()
        .iter()
        .map(|block| block.links.len() as u32)
        .sum();
    let has_preamble = !document.index().section_block_indices(None).is_empty();
    let section_count = heading_count + u32::from(has_preamble);
    let word_count = document
        .blocks()
        .iter()
        .map(|block| match block.kind {
            BlockKind::Heading => block
                .heading
                .as_ref()
                .map(|heading| heading.text.as_str())
                .unwrap_or_else(|| document.slice_unchecked(&block.span))
                .split_whitespace()
                .count() as u32,
            BlockKind::Paragraph => document
                .slice_unchecked(&block.span)
                .split_whitespace()
                .count() as u32,
            BlockKind::BlockQuote => document
                .slice_unchecked(&block.span)
                .lines()
                .map(|line| count_content_words(blockquote_content(line)))
                .sum(),
            BlockKind::List => count_list_words(document.slice_unchecked(&block.span)),
            BlockKind::Table => count_table_words(document.slice_unchecked(&block.span)),
            _ => 0,
        })
        .sum();

    DocumentStats {
        word_count,
        heading_count,
        block_count,
        link_count,
        section_count,
        line_count: document.line_count(),
    }
}

fn blockquote_content(mut line: &str) -> &str {
    line = line.trim_start();
    while let Some(rest) = line.strip_prefix('>') {
        line = rest.trim_start();
    }
    line
}

fn count_list_words(content: &str) -> u32 {
    content.lines().map(count_content_words).sum()
}

fn count_content_words(line: &str) -> u32 {
    let trimmed = line.trim_start();
    let text =
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            &trimmed[2..]
        } else if let Some(rest) = strip_ordered_marker(trimmed) {
            rest
        } else {
            trimmed
                .trim_start_matches('#')
                .strip_prefix(' ')
                .unwrap_or(trimmed)
        };
    text.strip_prefix("[ ] ")
        .or_else(|| text.strip_prefix("[x] "))
        .or_else(|| text.strip_prefix("[X] "))
        .unwrap_or(text)
        .split_whitespace()
        .count() as u32
}

fn count_table_words(content: &str) -> u32 {
    content
        .lines()
        .filter(|line| !line.trim().starts_with("|---") && !line.trim().starts_with("| ---"))
        .filter(|line| {
            !line
                .chars()
                .all(|character| matches!(character, '|' | '-' | ':' | ' '))
        })
        .map(|line| {
            line.split('|')
                .map(|cell| cell.split_whitespace().count() as u32)
                .sum::<u32>()
        })
        .sum()
}

fn strip_ordered_marker(value: &str) -> Option<&str> {
    if !value.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    let rest = value.trim_start_matches(|c: char| c.is_ascii_digit());
    (rest.starts_with(". ") || rest.starts_with(") ")).then(|| &rest[2..])
}
