use crate::cli::{DeleteSectionArgs, ReplaceSectionArgs, SectionArgs};
use crate::commands::edit;
use crate::errors::{CommandError, DiagnosticCode, SelectorRole};
use crate::model::*;
use crate::output;
use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtagGuard;
use mdtools::section::{ResolvedSection, SectionIndex, SectionTarget};
use mdtools::section_edit::{self, SectionEditTarget};

pub fn run_section(args: &SectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(source)?;
    let target = build_selector(
        &args.selector,
        args.occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let section = find_section(&document, &target)?;
    let content = document.slice(&section.span)?.to_string();
    if json {
        output::write_json(&SectionReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            section: section.entry().clone(),
            content,
        })?;
    } else {
        print!("{content}");
    }
    Ok(())
}

pub fn run_replace_section(args: &ReplaceSectionArgs, json: bool) -> Result<(), CommandError> {
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let target = build_selector(
        &args.selector,
        args.occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let section = find_section(&document, &target)?;
    let expected = parse_etag(args.etag_guard.expect_etag.as_deref())?;
    let prepared = section_edit::prepare_replace(&document, section, expected.as_ref())?;
    let payload = output::read_content(args.from.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::ReplaceSection,
        prepared.apply(payload),
        target_to_wire,
    )
}

pub fn run_delete_section(args: &DeleteSectionArgs, json: bool) -> Result<(), CommandError> {
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let target = build_selector(
        &args.selector,
        args.occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let section = find_section(&document, &target)?;
    let expected = parse_etag(args.etag_guard.expect_etag.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::DeleteSection,
        section_edit::delete(&document, section, expected.as_ref())?,
        target_to_wire,
    )
}

pub fn build_selector(
    selector: &str,
    occurrence: Option<u32>,
    contains: bool,
    ignore_case: bool,
) -> Result<SectionTarget, CommandError> {
    if contains && selector.is_empty() {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "empty selector cannot be used with --contains",
        )
        .with_hint("pass a non-empty heading substring with --contains, or drop --contains for an exact match"));
    }
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
            Ok(SectionTarget::preamble())
        }
    } else {
        SectionTarget::heading(
            selector,
            occurrence,
            match (contains, ignore_case) {
                (false, false) => HeadingMatchMode::Exact,
                (false, true) => HeadingMatchMode::ExactIgnoreCase,
                (true, false) => HeadingMatchMode::Contains,
                (true, true) => HeadingMatchMode::ContainsIgnoreCase,
            },
        )
        .map_err(CommandError::from)
    }
}

pub fn find_section(
    document: &Document,
    target: &SectionTarget,
) -> Result<ResolvedSection, CommandError> {
    find_section_as(document, target, SelectorRole::Target)
}

pub fn find_section_as(
    document: &Document,
    target: &SectionTarget,
    role: SelectorRole,
) -> Result<ResolvedSection, CommandError> {
    SectionIndex::new(document)
        .resolve(target)
        .map_err(|error| map_section_error(error, role))
}

fn map_section_error(error: CoreError, role: SelectorRole) -> CommandError {
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

pub(crate) fn target_to_wire(target: &SectionEditTarget) -> MutationTargetRef {
    match target {
        SectionEditTarget::Section(section) => MutationTargetRef::Section(SectionTargetRef {
            kind: MutationTargetKind::Section,
            selector: section.selector.clone(),
            section: section.clone(),
        }),
        SectionEditTarget::Move {
            source,
            destination,
            destination_mode,
            level_shift_applied,
        } => MutationTargetRef::SectionMove(SectionMoveTargetRef {
            kind: MutationTargetKind::Section,
            source: SectionTargetRef {
                kind: MutationTargetKind::Section,
                selector: source.selector.clone(),
                section: source.clone(),
            },
            destination: SectionTargetRef {
                kind: MutationTargetKind::Section,
                selector: destination.selector.clone(),
                section: destination.clone(),
            },
            destination_mode: *destination_mode,
            level_shift_applied: *level_shift_applied,
        }),
    }
}

fn parse_etag(value: Option<&str>) -> Result<Option<TargetEtagGuard>, CommandError> {
    Ok(value.map(TargetEtagGuard::new))
}
