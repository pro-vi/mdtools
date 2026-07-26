use crate::cli::MoveBlockArgs;
use crate::commands::replace::{
    all_block_etags, emit_mutation, verify_expected_etag_unique_with, MutationEmission,
};
use crate::errors::{CommandError, DiagnosticCode, SelectorRole};
use crate::model::*;
use crate::parser::ParsedDocument;

pub fn run_move_block(args: &MoveBlockArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let source_block = doc.blocks.get(args.source_index as usize).ok_or_else(|| {
        CommandError::block_out_of_range(args.source_index, doc.blocks.len() as u32)
    })?;
    let (dest_index, destination_mode) = destination_from_args(args);
    let dest_block = doc
        .blocks
        .get(dest_index as usize)
        .ok_or_else(|| CommandError::block_out_of_range(dest_index, doc.blocks.len() as u32))?;

    if args.source_index == dest_index {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "source and destination block indices must be different",
        ));
    }

    verify_expected_block_move_etag(
        args.expect_source_etag.as_deref(),
        args.source_index,
        SelectorRole::Source,
        &doc,
        doc.slice(&source_block.span),
    )?;
    verify_expected_block_move_etag(
        args.expect_dest_etag.as_deref(),
        dest_index,
        SelectorRole::Destination,
        &doc,
        doc.slice(&dest_block.span),
    )?;

    let order = build_block_order(
        doc.blocks.len(),
        args.source_index,
        dest_index,
        destination_mode,
    );
    let changed = order
        .iter()
        .enumerate()
        .any(|(position, &block_index)| position != block_index as usize);
    let disposition = if changed {
        MutationDisposition::Replaced
    } else {
        MutationDisposition::NoChange
    };

    let output_doc = reconstruct_document(&doc, &order);
    let span_after = verify_structural_closure(&doc, &output_doc, &order, args.source_index)?;

    let target = MutationTargetRef::BlockMove(BlockMoveTargetRef {
        kind: MutationTargetKind::Block,
        source: BlockTargetRef {
            kind: MutationTargetKind::Block,
            block_index: args.source_index,
            span: source_block.span,
        },
        destination: BlockTargetRef {
            kind: MutationTargetKind::Block,
            block_index: dest_index,
            span: dest_block.span,
        },
        destination_mode,
    });

    emit_mutation(MutationEmission {
        in_place: args.in_place,
        json,
        file: &args.file,
        command: MutationCommandKind::MoveBlock,
        target,
        disposition,
        changed,
        guarded: args.expect_source_etag.is_some() || args.expect_dest_etag.is_some(),
        line_endings: doc.line_ending_style(),
        span_before: Some(source_block.span),
        span_after: Some(if disposition == MutationDisposition::NoChange {
            source_block.span
        } else {
            span_after
        }),
        output_doc: &output_doc,
    })
}

fn destination_from_args(args: &MoveBlockArgs) -> (u32, BlockMoveMode) {
    if let Some(index) = args.before {
        (index, BlockMoveMode::Before)
    } else {
        (
            args.after.expect("clap requires one destination"),
            BlockMoveMode::After,
        )
    }
}

fn build_block_order(
    block_count: usize,
    source_index: u32,
    dest_index: u32,
    destination_mode: BlockMoveMode,
) -> Vec<u32> {
    let mut order: Vec<u32> = (0..block_count as u32).collect();
    let moved = order.remove(source_index as usize);
    let dest_position = order
        .iter()
        .position(|&index| index == dest_index)
        .expect("destination index must remain after source removal");
    let insert_position = match destination_mode {
        BlockMoveMode::Before => dest_position,
        BlockMoveMode::After => dest_position + 1,
    };
    order.insert(insert_position, moved);
    order
}

fn reconstruct_document(doc: &ParsedDocument, order: &[u32]) -> String {
    let prefix_end = doc
        .blocks
        .first()
        .map(|block| block.span.byte_start as usize)
        .unwrap_or(doc.source.len());
    let prefix = &doc.source[..prefix_end];
    let gaps = gap_slots(doc);

    let mut output_doc = String::with_capacity(doc.source.len());
    output_doc.push_str(prefix);
    for (slot_index, &block_index) in order.iter().enumerate() {
        output_doc.push_str(doc.slice(&doc.blocks[block_index as usize].span));
        output_doc.push_str(gaps[slot_index]);
    }
    output_doc
}

fn gap_slots(doc: &ParsedDocument) -> Vec<&str> {
    doc.blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let gap_start = block.span.byte_end as usize;
            let gap_end = doc
                .blocks
                .get(index + 1)
                .map(|next| next.span.byte_start as usize)
                .unwrap_or(doc.source.len());
            &doc.source[gap_start..gap_end]
        })
        .collect()
}

fn verify_structural_closure(
    doc: &ParsedDocument,
    output_doc: &str,
    order: &[u32],
    source_index: u32,
) -> Result<SourceSpan, CommandError> {
    let reparsed = ParsedDocument::parse(output_doc.to_string())?;
    if reparsed.blocks.len() != doc.blocks.len() {
        return Err(structural_closure_error());
    }

    for (position, &original_index) in order.iter().enumerate() {
        let expected = &doc.blocks[original_index as usize];
        let actual = reparsed
            .blocks
            .get(position)
            .ok_or_else(structural_closure_error)?;
        if actual.kind != expected.kind {
            return Err(structural_closure_error());
        }
        if reparsed.slice(&actual.span) != doc.slice(&expected.span) {
            return Err(structural_closure_error());
        }
    }

    let moved_position = order
        .iter()
        .position(|&index| index == source_index)
        .expect("source index must remain in permutation");
    Ok(reparsed.blocks[moved_position].span)
}

fn structural_closure_error() -> CommandError {
    CommandError::new(
        DiagnosticCode::InvalidSelector,
        "move-block would change the parsed top-level block sequence; choose a destination that preserves block boundaries",
    )
}

fn verify_expected_block_move_etag(
    expect: Option<&str>,
    index: u32,
    role: SelectorRole,
    doc: &ParsedDocument,
    current: &str,
) -> Result<(), CommandError> {
    verify_expected_etag_unique_with(
        expect,
        current,
        || all_block_etags(doc),
        |expected, actual| CommandError::move_block_etag_mismatch(role, index, expected, actual),
        |expected, count| CommandError::move_block_etag_ambiguous(role, index, expected, count),
    )
}
