use std::path::Path;

use crate::cli::StatsArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &StatsArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    multifile::for_each_file(&file_set, |file| process_file(file, json, multi))
}

fn process_file(file: &Path, json: bool, multi: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(file)?;
    let doc = ParsedDocument::parse(source)?;
    let file_str = file.to_string_lossy();
    let stats = compute_stats(&doc);

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

fn compute_stats(doc: &ParsedDocument) -> DocumentStats {
    let heading_count = doc
        .blocks
        .iter()
        .filter(|b| b.heading.is_some())
        .count() as u32;

    let block_count = doc.blocks.len() as u32;

    let link_count: u32 = doc.blocks.iter().map(|b| b.links.len() as u32).sum();

    let section_count = compute_section_count(doc);

    let line_count = doc.line_count();

    let word_count = compute_word_count(doc);

    DocumentStats {
        word_count,
        heading_count,
        block_count,
        link_count,
        section_count,
        line_count,
    }
}

/// Count sections: each heading is a section, plus preamble if non-empty.
fn compute_section_count(doc: &ParsedDocument) -> u32 {
    let heading_sections = doc
        .blocks
        .iter()
        .filter(|b| b.heading.is_some())
        .count() as u32;

    // Preamble counts if there are blocks before the first heading
    let has_preamble = doc
        .blocks
        .first()
        .map_or(false, |b| b.heading.is_none());

    heading_sections + if has_preamble { 1 } else { 0 }
}

/// Count whitespace-delimited tokens in plaintext rendering of block bodies.
/// Spec: headings, paragraphs, list items, table cells contribute.
/// Code fence contents, HTML block contents, and frontmatter do not.
fn compute_word_count(doc: &ParsedDocument) -> u32 {
    let mut count = 0u32;
    for block in &doc.blocks {
        match block.kind {
            BlockKind::Heading
            | BlockKind::Paragraph
            | BlockKind::List
            | BlockKind::BlockQuote
            | BlockKind::Table => {
                let content = doc.slice(&block.span);
                count += count_words_in_content(content, block.kind);
            }
            _ => {}
        }
    }
    count
}

fn count_words_in_content(content: &str, kind: BlockKind) -> u32 {
    match kind {
        BlockKind::Heading => {
            // Strip leading # markers
            let text = content.trim_start_matches('#').trim();
            text.split_whitespace().count() as u32
        }
        BlockKind::List => {
            // Strip list markers and count words in items
            content
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    // Strip bullet markers: -, *, +, or numbered
                    let text = if trimmed.starts_with("- ")
                        || trimmed.starts_with("* ")
                        || trimmed.starts_with("+ ")
                    {
                        &trimmed[2..]
                    } else if let Some(rest) = strip_ordered_marker(trimmed) {
                        rest
                    } else {
                        trimmed
                    };
                    // Strip checkbox markers
                    let text = text
                        .strip_prefix("[ ] ")
                        .or_else(|| text.strip_prefix("[x] "))
                        .or_else(|| text.strip_prefix("[X] "))
                        .unwrap_or(text);
                    text.split_whitespace().count() as u32
                })
                .sum()
        }
        BlockKind::Table => {
            // Count words in cell content, skip separator rows
            content
                .lines()
                .filter(|line| !line.trim().starts_with("|---") && !line.trim().starts_with("| ---"))
                .filter(|line| {
                    // Skip separator rows like |---|---|
                    !line.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
                })
                .map(|line| {
                    line.split('|')
                        .map(|cell| cell.trim().split_whitespace().count() as u32)
                        .sum::<u32>()
                })
                .sum()
        }
        _ => content.split_whitespace().count() as u32,
    }
}

fn strip_ordered_marker(s: &str) -> Option<&str> {
    let mut chars = s.chars();
    // Must start with digit
    if !chars.next().map_or(false, |c| c.is_ascii_digit()) {
        return None;
    }
    // Consume remaining digits
    let rest = s.trim_start_matches(|c: char| c.is_ascii_digit());
    // Must be followed by . or ) then space
    if rest.starts_with(". ") || rest.starts_with(") ") {
        Some(&rest[2..])
    } else {
        None
    }
}
