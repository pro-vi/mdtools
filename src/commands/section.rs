use crate::cli::{DeleteSectionArgs, ReplaceSectionArgs, SectionArgs};
use crate::commands::replace::{replacement_span_after, verify_expected_etag_unique};
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;
use mdtools::core_error::CoreError;
use mdtools::section::SectionIndex;

pub fn run_section(args: &SectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;
    let selector = build_selector(
        &args.selector,
        args.occurrence,
        args.contains,
        args.ignore_case,
    )?;
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
    let selector = build_selector(
        &args.selector,
        args.occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let section = find_section(&doc, &selector)?;
    let section_span = section.span;
    verify_expected_etag_unique(
        args.etag_guard.expect_etag.as_deref(),
        doc.slice(&section_span),
        "section",
        Some(crate::errors::SelectorRole::Target),
        || all_section_etags(&doc),
        |expected, actual| {
            CommandError::section_etag_mismatch(
                &describe_selector(&section.selector),
                expected,
                actual,
            )
        },
    )?;

    let replacement = output::read_content(args.from.as_deref())?;

    let line_endings = doc.line_ending_style();
    let replacement = normalize_line_endings(&replacement, &line_endings);
    let effective_replacement = preserve_following_section_boundary(
        doc.slice(&section_span),
        &replacement,
        (section_span.byte_end as usize) < doc.source.len(),
    );
    let before = &doc.source[..section_span.byte_start as usize];
    let after = &doc.source[section_span.byte_end as usize..];
    let output_doc = format!("{}{}{}", before, effective_replacement, after);

    let disposition = if effective_replacement == doc.slice(&section_span) {
        MutationDisposition::NoChange
    } else if effective_replacement.is_empty() {
        MutationDisposition::Deleted
    } else {
        MutationDisposition::Replaced
    };

    let changed = disposition != MutationDisposition::NoChange;

    if args.in_place {
        if changed {
            output::write_file_atomic(args.file.as_ref(), &output_doc)?;
        }
        if json {
            let result = build_section_mutation_result(
                &args.file.to_string_lossy(),
                args.etag_guard.expect_etag.is_some(),
                section,
                disposition,
                changed,
                line_endings,
                section_span,
                &effective_replacement,
                None,
                MutationCommandKind::ReplaceSection,
            );
            output::write_json(&result)?;
        }
    } else if json {
        let result = build_section_mutation_result(
            &args.file.to_string_lossy(),
            args.etag_guard.expect_etag.is_some(),
            section,
            disposition,
            changed,
            line_endings,
            section_span,
            &effective_replacement,
            Some(output_doc),
            MutationCommandKind::ReplaceSection,
        );
        output::write_json(&result)?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}

fn preserve_following_section_boundary(
    section_content: &str,
    replacement: &str,
    has_following_section: bool,
) -> String {
    if replacement.is_empty() || !has_following_section {
        return replacement.to_string();
    }

    let boundary_tokens = trailing_line_ending_tokens(section_content);
    if boundary_tokens.is_empty() {
        return replacement.to_string();
    }

    let replacement_trailing_count = trailing_line_ending_tokens(replacement).len();
    if replacement_trailing_count >= boundary_tokens.len() {
        return replacement.to_string();
    }

    let extra_len: usize = boundary_tokens
        .iter()
        .skip(replacement_trailing_count)
        .map(|token| token.len())
        .sum();
    let mut completed = String::with_capacity(replacement.len() + extra_len);
    completed.push_str(replacement);
    for token in boundary_tokens.iter().skip(replacement_trailing_count) {
        completed.push_str(token);
    }
    completed
}

pub fn run_delete_section(args: &DeleteSectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;
    let selector = build_selector(
        &args.selector,
        args.occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let section = find_section(&doc, &selector)?;
    let section_span = section.span;
    verify_expected_etag_unique(
        args.etag_guard.expect_etag.as_deref(),
        doc.slice(&section_span),
        "section",
        Some(crate::errors::SelectorRole::Target),
        || all_section_etags(&doc),
        |expected, actual| {
            CommandError::section_etag_mismatch(
                &describe_selector(&section.selector),
                expected,
                actual,
            )
        },
    )?;
    let line_endings = doc.line_ending_style();

    let before = &doc.source[..section_span.byte_start as usize];
    let after = &doc.source[section_span.byte_end as usize..];
    let output_doc = format!("{}{}", before, after);

    let changed = true;
    let disposition = MutationDisposition::Deleted;

    if args.in_place {
        output::write_file_atomic(args.file.as_ref(), &output_doc)?;
        if json {
            let result = build_section_mutation_result(
                &args.file.to_string_lossy(),
                args.etag_guard.expect_etag.is_some(),
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
            args.etag_guard.expect_etag.is_some(),
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

/// Content etags of every section in the document (preamble + one per
/// heading), for section-guard ambiguity checks: identical duplicate
/// sections share a fingerprint, and a guard hash matching more than one
/// section cannot prove identity.
pub fn all_section_etags(doc: &ParsedDocument) -> Vec<String> {
    SectionIndex::new(doc).all_etags()
}

pub fn build_selector(
    selector: &str,
    occurrence: Option<u32>,
    contains: bool,
    ignore_case: bool,
) -> Result<SectionSelector, CommandError> {
    if contains && selector.is_empty() {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "empty selector cannot be used with --contains",
        )
        .with_hint("pass a non-empty heading substring with --contains, or drop --contains for an exact match"));
    }

    // Occurrence is a 1-based contract; 0 must never silently select the
    // first match (wrong-target hazard on mutations).
    if occurrence == Some(0) {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "occurrence is 1-based; 0 is not a valid occurrence",
        )
        .with_hint("pass a 1-based occurrence (1 selects the first match)"));
    }

    if selector == ":preamble" {
        if contains {
            Err(CommandError::new(
                DiagnosticCode::InvalidSelector,
                "--contains cannot be used with :preamble",
            )
            .with_hint("use :preamble alone to select content before the first heading, or a heading selector with --contains"))
        } else if occurrence.is_some() {
            Err(CommandError::new(
                DiagnosticCode::InvalidSelector,
                ":preamble is unique; occurrence cannot be used with it",
            )
            .with_hint("drop the occurrence flag: :preamble selects the single pre-heading region"))
        } else {
            Ok(SectionSelector {
                kind: SectionSelectorKind::Preamble,
                heading_text: None,
                occurrence: None,
                match_mode: HeadingMatchMode::Exact,
            })
        }
    } else {
        Ok(SectionSelector {
            kind: SectionSelectorKind::HeadingText,
            heading_text: Some(selector.to_string()),
            occurrence,
            match_mode: match (contains, ignore_case) {
                (false, false) => HeadingMatchMode::Exact,
                (false, true) => HeadingMatchMode::ExactIgnoreCase,
                (true, false) => HeadingMatchMode::Contains,
                (true, true) => HeadingMatchMode::ContainsIgnoreCase,
            },
        })
    }
}

pub fn find_section(
    doc: &ParsedDocument,
    selector: &SectionSelector,
) -> Result<SectionEntry, CommandError> {
    find_section_as(doc, selector, crate::errors::SelectorRole::Target)
}

/// Like find_section, but selector errors carry `role` so adapters can
/// recommend the right disambiguation flag (move-section passes source /
/// destination).
pub fn find_section_as(
    doc: &ParsedDocument,
    selector: &SectionSelector,
    role: crate::errors::SelectorRole,
) -> Result<SectionEntry, CommandError> {
    SectionIndex::new(doc)
        .resolve(selector)
        .map_err(|error| map_section_error(error, role))
}

fn map_section_error(error: CoreError, role: crate::errors::SelectorRole) -> CommandError {
    match error {
        CoreError::HeadingNotFound { heading } => {
            CommandError::not_found_heading_as(&heading, role)
        }
        CoreError::DuplicateHeading { heading, matches } => {
            let matches = matches
                .into_iter()
                .map(|item| crate::errors::MatchRef {
                    block_index: item.block_index,
                    occurrence: item.occurrence,
                    line: item.line,
                })
                .collect::<Vec<_>>();
            CommandError::duplicate_heading_as(&heading, matches.len(), &matches, role)
        }
        CoreError::OccurrenceOutOfRange {
            heading,
            requested,
            matches,
        } => {
            let matches = matches
                .into_iter()
                .map(|item| crate::errors::MatchRef {
                    block_index: item.block_index,
                    occurrence: item.occurrence,
                    line: item.line,
                })
                .collect::<Vec<_>>();
            CommandError::occurrence_out_of_range(&heading, requested, &matches, role)
        }
        other => other.into(),
    }
}

fn build_section_mutation_result(
    file: &str,
    guarded: bool,
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
        MutationDisposition::Replaced => Some(replacement_span_after(span_before, replacement)),
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
        guarded,
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

fn trailing_line_ending_tokens(content: &str) -> Vec<&str> {
    let bytes = content.as_bytes();
    let mut end = bytes.len();
    let mut spans = Vec::new();

    while end > 0 && bytes[end - 1] == b'\n' {
        let start = if end > 1 && bytes[end - 2] == b'\r' {
            end - 2
        } else {
            end - 1
        };
        spans.push((start, end));
        end = start;
    }

    spans.reverse();
    spans
        .into_iter()
        .map(|(start, end)| &content[start..end])
        .collect()
}

pub(crate) fn describe_selector(selector: &SectionSelector) -> String {
    match selector.kind {
        SectionSelectorKind::Preamble => ":preamble".to_string(),
        SectionSelectorKind::HeadingText => {
            let heading = selector.heading_text.as_deref().unwrap_or("");
            match selector.occurrence {
                Some(occurrence) => format!("{:?} occurrence {}", heading, occurrence),
                None => format!("{:?}", heading),
            }
        }
    }
}
