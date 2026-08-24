use crate::cli::{DeleteBlockArgs, InsertBlockArgs, ReplaceBlockArgs};
use crate::commands::edit;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use mdtools::block_edit::{self, BlockEditTarget};
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;

pub fn run_replace_block(args: &ReplaceBlockArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(source)?;
    let expected = parse_etag(args.expect_etag.as_deref())?;
    let prepared = block_edit::prepare_replace(&document, args.index, expected.as_ref())?;
    let payload = output::read_content(args.from.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        MutationCommandKind::ReplaceBlock,
        prepared.apply(payload),
        target_to_wire,
    )
}

pub fn run_insert_block(args: &InsertBlockArgs, json: bool) -> Result<(), CommandError> {
    let location = parse_insert_location(args)?;
    let source = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(source)?;
    let expected = parse_etag(args.expect_etag.as_deref())?;
    let prepared = block_edit::prepare_insert(&document, location, expected.as_ref()).map_err(
        |error| match error {
            mdtools::core_error::CoreError::InvalidSelector(message)
                if message == "expected etag requires a before or after anchor" =>
            {
                CommandError::new(
                    DiagnosticCode::InvalidSelector,
                    "--expect-etag requires --before or --after (--at-start/--at-end have no anchor block)",
                )
            }
            other => other.into(),
        },
    )?;
    let payload = output::read_content(args.from.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        MutationCommandKind::InsertBlock,
        prepared.apply(payload)?,
        target_to_wire,
    )
}

pub fn run_delete_block(args: &DeleteBlockArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(source)?;
    let expected = parse_etag(args.expect_etag.as_deref())?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        MutationCommandKind::DeleteBlock,
        block_edit::delete(&document, args.index, expected.as_ref())?,
        target_to_wire,
    )
}

fn parse_insert_location(args: &InsertBlockArgs) -> Result<InsertLocation, CommandError> {
    let count = usize::from(args.before.is_some())
        + usize::from(args.after.is_some())
        + usize::from(args.at_start)
        + usize::from(args.at_end);
    if count != 1 {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "exactly one of --before, --after, --at-start, --at-end must be provided",
        ));
    }
    Ok(if let Some(index) = args.before {
        InsertLocation::Before(index)
    } else if let Some(index) = args.after {
        InsertLocation::After(index)
    } else if args.at_start {
        InsertLocation::Start
    } else {
        InsertLocation::End
    })
}

pub(crate) fn target_to_wire(target: &BlockEditTarget) -> MutationTargetRef {
    match target {
        BlockEditTarget::Block { block_index, span } => MutationTargetRef::Block(BlockTargetRef {
            kind: MutationTargetKind::Block,
            block_index: *block_index,
            span: *span,
        }),
        BlockEditTarget::Insertion {
            location,
            anchor_span,
        } => MutationTargetRef::Insert(InsertTargetRef {
            kind: MutationTargetKind::InsertLocation,
            location: *location,
            anchor_span: *anchor_span,
        }),
        BlockEditTarget::Move {
            source_index,
            source_span,
            destination_index,
            destination_span,
            destination_mode,
        } => MutationTargetRef::BlockMove(BlockMoveTargetRef {
            kind: MutationTargetKind::Block,
            source: BlockTargetRef {
                kind: MutationTargetKind::Block,
                block_index: *source_index,
                span: *source_span,
            },
            destination: BlockTargetRef {
                kind: MutationTargetKind::Block,
                block_index: *destination_index,
                span: *destination_span,
            },
            destination_mode: *destination_mode,
        }),
    }
}

fn parse_etag(value: Option<&str>) -> Result<Option<TargetEtag>, CommandError> {
    value
        .map(str::parse::<TargetEtag>)
        .transpose()
        .map_err(CommandError::from)
}
