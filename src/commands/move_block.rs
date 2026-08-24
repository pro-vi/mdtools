use crate::cli::MoveBlockArgs;
use crate::commands::{edit, replace::target_to_wire};
use crate::errors::CommandError;
use crate::model::{BlockMoveMode, MutationCommandKind};
use crate::output;
use mdtools::block_edit;
use mdtools::fingerprint::TargetEtagGuard;

pub fn run_move_block(args: &MoveBlockArgs, json: bool) -> Result<(), CommandError> {
    let (document, edit_target) = output::read_edit_document(&args.file)?;
    let (destination_index, destination_mode) = if let Some(index) = args.before {
        (index, BlockMoveMode::Before)
    } else {
        (
            args.after.expect("clap requires one destination"),
            BlockMoveMode::After,
        )
    };
    let source_etag = args.expect_source_etag.as_deref().map(TargetEtagGuard::new);
    let destination_etag = args.expect_dest_etag.as_deref().map(TargetEtagGuard::new);
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
