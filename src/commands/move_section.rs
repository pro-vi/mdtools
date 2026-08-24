use std::io::Read;
use std::path::Path;

use crate::cli::MoveSectionArgs;
use crate::commands::edit;
use crate::commands::section::{build_selector, find_section_as, target_to_wire};
use crate::errors::{CommandError, DiagnosticCode, SelectorRole};
use crate::model::{InsertMode, MutationCommandKind};
use crate::output;
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtagGuard;
use mdtools::section_edit;

pub fn run_move_section(args: &MoveSectionArgs, json: bool) -> Result<(), CommandError> {
    if args.in_place && args.file.is_none() {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "--in-place requires a FILE argument",
        ));
    }
    let (source, edit_target) = match &args.file {
        Some(path) => {
            let (source, target) = output::read_edit_file(path)?.into_parts();
            (source, Some(target))
        }
        None => {
            let mut source = String::new();
            std::io::stdin().read_to_string(&mut source)?;
            (source, None)
        }
    };
    let document = Document::parse(source)?;
    let source_target = build_selector(
        &args.source,
        args.source_occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let (destination_text, destination_mode) = destination(args)?;
    let destination_target = build_selector(
        destination_text,
        args.dest_occurrence,
        args.contains,
        args.ignore_case,
    )?;
    let source = find_section_as(&document, &source_target, SelectorRole::Source)?;
    let destination = find_section_as(&document, &destination_target, SelectorRole::Destination)?;
    let source_etag = parse_etag(args.expect_source_etag.as_deref())?;
    let destination_etag = parse_etag(args.expect_dest_etag.as_deref())?;
    let outcome = section_edit::move_section(
        &document,
        source,
        destination,
        destination_mode,
        args.keep_level,
        source_etag.as_ref(),
        destination_etag.as_ref(),
    )?;
    let file = args.file.as_deref().unwrap_or_else(|| Path::new("-"));
    edit::emit(
        args.in_place,
        json,
        file,
        edit_target.as_ref(),
        MutationCommandKind::MoveSection,
        outcome,
        target_to_wire,
    )
}

fn destination(args: &MoveSectionArgs) -> Result<(&str, InsertMode), CommandError> {
    match (
        args.after.as_deref(),
        args.before.as_deref(),
        args.into.as_deref(),
    ) {
        (Some(value), None, None) => Ok((value, InsertMode::AfterSibling)),
        (None, Some(value), None) => Ok((value, InsertMode::BeforeSibling)),
        (None, None, Some(value)) => Ok((value, InsertMode::IntoAsChild)),
        _ => Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "exactly one of --after, --before, or --into is required",
        )),
    }
}

fn parse_etag(value: Option<&str>) -> Result<Option<TargetEtagGuard>, CommandError> {
    Ok(value.map(TargetEtagGuard::new))
}
