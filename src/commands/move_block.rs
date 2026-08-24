use crate::cli::MoveBlockArgs;
use crate::commands::{edit, replace::target_to_wire};
use crate::errors::CommandError;
use crate::model::{BlockMoveMode, MutationCommandKind};
use crate::output;
use mdtools::block_edit;
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;

pub fn run_move_block(args: &MoveBlockArgs, json: bool) -> Result<(), CommandError> {
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let (destination_index, destination_mode) = if let Some(index) = args.before {
        (index, BlockMoveMode::Before)
    } else {
        (
            args.after.expect("clap requires one destination"),
            BlockMoveMode::After,
        )
    };
    let source_etag = parse_etag(args.expect_source_etag.as_deref())?;
    let destination_etag = parse_etag(args.expect_dest_etag.as_deref())?;
    let outcome = block_edit::move_block(
        &document,
        args.source_index,
        destination_index,
        destination_mode,
        source_etag.as_ref(),
        destination_etag.as_ref(),
    )?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::MoveBlock,
        outcome,
        target_to_wire,
    )
}

fn parse_etag(value: Option<&str>) -> Result<Option<TargetEtag>, CommandError> {
    Ok(value.map(mdtools::fingerprint::cli_compat::target_etag))
}
