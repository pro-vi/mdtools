use crate::cli::{DeleteSectionArgs, ReplaceSectionArgs, SectionArgs};
use crate::errors::CommandError;
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run_section(args: &SectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;
    let selector = build_selector(&args.selector, args.occurrence, args.ignore_case)?;
    let section = find_section(&doc, &selector)?;
    let content = doc.slice(&section.span).to_string();

    if json {
        let result = SectionReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            section,
            content,
        };
        output::write_json(&result)?;
    } else {
        print!("{}", content);
    }
    Ok(())
}

pub fn run_replace_section(args: &ReplaceSectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;
    let selector = build_selector(&args.selector, args.occurrence, args.ignore_case)?;
    let section = find_section(&doc, &selector)?;
    let section_span = section.span;

    let replacement = output::read_content(args.content_file.as_deref())?;

    let line_endings = doc.line_ending_style();
    let replacement = normalize_line_endings(&replacement, &line_endings);

    let before = &doc.source[..section_span.byte_start as usize];
    let after = &doc.source[section_span.byte_end as usize..];
    let output_doc = format!("{}{}{}", before, replacement, after);

    let disposition = if replacement == doc.slice(&section_span) {
        MutationDisposition::NoChange
    } else if replacement.is_empty() {
        MutationDisposition::Deleted
    } else {
        MutationDisposition::Replaced
    };

    let changed = disposition != MutationDisposition::NoChange;

    if args.in_place {
        if changed {
            std::fs::write(&args.file, &output_doc)?;
        }
        if json {
            let result = build_section_mutation_result(
                &args.file.to_string_lossy(),
                section,
                disposition,
                changed,
                line_endings,
                section_span,
                &replacement,
                None,
                MutationCommandKind::ReplaceSection,
            );
            output::write_json(&result)?;
        }
    } else if json {
        let result = build_section_mutation_result(
            &args.file.to_string_lossy(),
            section,
            disposition,
            changed,
            line_endings,
            section_span,
            &replacement,
            Some(output_doc),
            MutationCommandKind::ReplaceSection,
        );
        output::write_json(&result)?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}

pub fn run_delete_section(args: &DeleteSectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;
    let selector = build_selector(&args.selector, args.occurrence, args.ignore_case)?;
    let section = find_section(&doc, &selector)?;
    let section_span = section.span;
    let line_endings = doc.line_ending_style();

    let before = &doc.source[..section_span.byte_start as usize];
    let after = &doc.source[section_span.byte_end as usize..];
    let output_doc = format!("{}{}", before, after);

    let changed = true;
    let disposition = MutationDisposition::Deleted;

    if args.in_place {
        std::fs::write(&args.file, &output_doc)?;
        if json {
            let result = build_section_mutation_result(
                &args.file.to_string_lossy(),
                section,
                disposition,
                changed,
                line_endings,
                section_span,
                "",
                None,
                MutationCommandKind::DeleteSection,
            );
            output::write_json(&result)?;
        }
    } else if json {
        let result = build_section_mutation_result(
            &args.file.to_string_lossy(),
            section,
            disposition,
            changed,
            line_endings,
            section_span,
            "",
            Some(output_doc),
            MutationCommandKind::DeleteSection,
        );
        output::write_json(&result)?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}

pub fn build_selector(
    selector: &str,
    occurrence: Option<u32>,
    ignore_case: bool,
) -> Result<SectionSelector, CommandError> {
    if selector == ":preamble" {
        Ok(SectionSelector {
            kind: SectionSelectorKind::Preamble,
            heading_text: None,
            occurrence: None,
            match_mode: HeadingMatchMode::Exact,
        })
    } else {
        Ok(SectionSelector {
            kind: SectionSelectorKind::HeadingText,
            heading_text: Some(selector.to_string()),
            occurrence,
            match_mode: if ignore_case {
                HeadingMatchMode::ExactIgnoreCase
            } else {
                HeadingMatchMode::Exact
            },
        })
    }
}

pub fn find_section(
    doc: &ParsedDocument,
    selector: &SectionSelector,
) -> Result<SectionEntry, CommandError> {
    match selector.kind {
        SectionSelectorKind::Preamble => find_preamble(doc),
        SectionSelectorKind::HeadingText => {
            let heading_text = selector.heading_text.as_deref().unwrap_or("");
            find_heading_section(doc, heading_text, selector)
        }
    }
}

fn find_preamble(doc: &ParsedDocument) -> Result<SectionEntry, CommandError> {
    // Preamble: all blocks before the first heading
    let mut block_indices = Vec::new();
    for block in &doc.blocks {
        if block.heading.is_some() {
            break;
        }
        block_indices.push(block.index);
    }

    let span = if block_indices.is_empty() {
        // Empty preamble — span starts after frontmatter or at doc start
        let byte_start = doc
            .frontmatter
            .as_ref()
            .map(|fm| fm.span.byte_end)
            .unwrap_or(0);
        SourceSpan {
            line_start: if byte_start > 0 {
                doc.blocks.first().map(|b| b.span.line_start).unwrap_or(1)
            } else {
                1
            },
            line_end: if byte_start > 0 {
                doc.blocks.first().map(|b| b.span.line_start).unwrap_or(1)
            } else {
                1
            },
            byte_start,
            byte_end: byte_start,
        }
    } else {
        let first = &doc.blocks[block_indices[0] as usize];
        let last = &doc.blocks[*block_indices.last().unwrap() as usize];
        SourceSpan {
            line_start: first.span.line_start,
            line_end: last.span.line_end,
            byte_start: first.span.byte_start,
            byte_end: last.span.byte_end,
        }
    };

    Ok(SectionEntry {
        kind: SectionKind::Preamble,
        heading: None,
        selector: SectionSelector {
            kind: SectionSelectorKind::Preamble,
            heading_text: None,
            occurrence: None,
            match_mode: HeadingMatchMode::Exact,
        },
        depth: 0,
        block_indices,
        span,
    })
}

fn find_heading_section(
    doc: &ParsedDocument,
    heading_text: &str,
    selector: &SectionSelector,
) -> Result<SectionEntry, CommandError> {
    let ignore_case = selector.match_mode == HeadingMatchMode::ExactIgnoreCase;

    // Find all matching headings
    let matches: Vec<_> = doc
        .blocks
        .iter()
        .filter(|b| {
            b.heading.as_ref().map_or(false, |h| {
                if ignore_case {
                    h.text.eq_ignore_ascii_case(heading_text)
                } else {
                    h.text == heading_text
                }
            })
        })
        .collect();

    if matches.is_empty() {
        return Err(CommandError::not_found_heading(heading_text));
    }

    if matches.len() > 1 && selector.occurrence.is_none() {
        return Err(CommandError::duplicate_heading(heading_text, matches.len()));
    }

    let selected = if let Some(occ) = selector.occurrence {
        matches
            .get(occ.saturating_sub(1) as usize)
            .ok_or_else(|| CommandError::not_found_heading(heading_text))?
    } else {
        matches[0]
    };

    let h = selected.heading.as_ref().unwrap();
    let level = h.level;

    // Collect all blocks in this section
    let mut block_indices = vec![selected.index];
    for block in &doc.blocks {
        if block.index <= selected.index {
            continue;
        }
        if let Some(bh) = &block.heading {
            if bh.level <= level {
                break;
            }
        }
        block_indices.push(block.index);
    }

    // Section span — line_end must be consistent with byte_end
    let section_byte_end = find_section_byte_end(doc, selected.index, level);
    let section_line_end = if section_byte_end as usize >= doc.source.len() {
        doc.line_count()
    } else {
        // byte_end points to start of next section; line_end is the line before that
        let line_at_end = doc.byte_to_line(section_byte_end);
        if section_byte_end > 0 && doc.source.as_bytes().get(section_byte_end as usize - 1) == Some(&b'\n') {
            line_at_end - 1
        } else {
            line_at_end
        }
    };

    let span = SourceSpan {
        line_start: selected.span.line_start,
        line_end: section_line_end,
        byte_start: selected.span.byte_start,
        byte_end: section_byte_end,
    };

    let heading_ref = HeadingRef {
        level,
        text: h.text.clone(),
        block_index: selected.index,
        span: selected.span,
    };

    Ok(SectionEntry {
        kind: SectionKind::Heading,
        heading: Some(heading_ref),
        selector: selector.clone(),
        depth: level,
        block_indices,
        span,
    })
}

fn find_section_byte_end(doc: &ParsedDocument, heading_index: u32, level: u8) -> u32 {
    // Find the next heading at same or higher level
    for block in &doc.blocks {
        if block.index <= heading_index {
            continue;
        }
        if let Some(h) = &block.heading {
            if h.level <= level {
                return block.span.byte_start;
            }
        }
    }
    doc.source.len() as u32
}

fn build_section_mutation_result(
    file: &str,
    section: SectionEntry,
    disposition: MutationDisposition,
    changed: bool,
    line_endings: LineEndingStyle,
    span_before: SourceSpan,
    replacement: &str,
    content: Option<String>,
    command: MutationCommandKind,
) -> MutationResult {
    let span_after = match disposition {
        MutationDisposition::Deleted => None,
        MutationDisposition::NoChange => Some(span_before),
        MutationDisposition::Replaced => {
            let byte_end = span_before.byte_start + replacement.len() as u32;
            let line_count = replacement.matches('\n').count() as u32;
            Some(SourceSpan {
                line_start: span_before.line_start,
                line_end: span_before.line_start + line_count,
                byte_start: span_before.byte_start,
                byte_end,
            })
        }
        _ => Some(span_before),
    };

    MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string(),
        command,
        target: MutationTargetRef::Section(SectionTargetRef {
            kind: MutationTargetKind::Section,
            selector: section.selector.clone(),
            section,
        }),
        disposition,
        changed,
        line_endings,
        invariant: SourcePreservationInvariant {
            preserves_non_target_bytes: true,
            target_span_before: Some(span_before),
            target_span_after: span_after,
        },
        content,
    }
}

fn normalize_line_endings(content: &str, style: &LineEndingStyle) -> String {
    crate::output::normalize_line_endings(content, style)
}
