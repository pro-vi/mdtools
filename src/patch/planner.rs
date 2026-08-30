use std::collections::HashMap;
use std::ops::Range;

use super::*;
use crate::edit::{normalize_line_endings, strip_one_trailing_newline, SourceEdit};
use crate::frontmatter::{FrontmatterAction, FrontmatterPathMutation};
use crate::index::IndexNode;
use crate::model::{BlockKind, InsertMode, LineEndingStyle, TaskStatus};
use crate::table::TableResultLocation;
use crate::target::TargetSummary;

pub(super) fn apply(patch: &Patch, document: &Document) -> Result<PatchOutcome, CoreError> {
    if patch.base_revision != *document.revision() {
        return Err(CoreError::DocumentRevisionMismatch {
            expected: patch.base_revision.to_string(),
            actual: document.revision().to_string(),
        });
    }
    if patch.operations.is_empty() {
        return Err(CoreError::InvalidPatch(
            "patch must contain at least one operation".into(),
        ));
    }
    for operation in &patch.operations {
        super::preflight_operation_evidence(document, operation)?;
    }

    let mutations = plan_operations(document, &patch.operations)?;
    if mutations
        .iter()
        .any(|mutation| mutation.results.len() != mutation.receipts.len())
    {
        return Err(CoreError::PatchInvariant(
            "planned result and receipt counts diverged".into(),
        ));
    }
    reject_claim_overlaps(&mutations)?;
    reject_byte_overlaps(&mutations)?;

    let applied_edits = flatten_edits(&mutations);
    let mut source = document.source().to_string();
    let mut edit_order = applied_edits.iter().collect::<Vec<_>>();
    edit_order.sort_by_key(|edit| std::cmp::Reverse((edit.edit.start, edit.edit.end)));
    for edit in edit_order {
        source.replace_range(edit.edit.start..edit.edit.end, &edit.edit.replacement);
    }

    let candidate = document.reparse(source)?;
    let result_targets = ResultTargetIndex::new(&candidate)?;
    let verified = mutations
        .iter()
        .enumerate()
        .map(|(index, mutation)| {
            mutation
                .results
                .iter()
                .map(|result| {
                    verify_result(
                        &candidate,
                        &result_targets,
                        index,
                        &applied_edits,
                        &result.expectation,
                    )
                    .map(|verified| (result.operation, verified))
                })
                .collect::<Result<Vec<_>, CoreError>>()
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    drop(applied_edits);
    let mut receipts = Vec::with_capacity(patch.operations.len());
    for (mutation, verified) in mutations.into_iter().zip(verified) {
        for (receipt, (operation, result)) in mutation.receipts.into_iter().zip(verified) {
            if receipt.operation != operation {
                return Err(CoreError::PatchInvariant(
                    "planned result and receipt operation order diverged".into(),
                ));
            }
            receipts.push((operation, receipt.draft.finish(&candidate, result)?));
        }
    }
    receipts.sort_by_key(|(operation, _)| *operation);
    let receipts = receipts.into_iter().map(|(_, receipt)| receipt).collect();
    Ok(PatchOutcome {
        document: candidate,
        receipts,
    })
}

struct PlannedMutation {
    claims: Vec<IndexedClaim>,
    edits: Vec<ByteEdit>,
    results: Vec<IndexedResultExpectation>,
    receipts: Vec<IndexedReceiptDraft>,
}

struct IndexedClaim {
    operation: usize,
    region: ConflictRegion,
}

struct IndexedResultExpectation {
    operation: usize,
    expectation: ResultExpectation,
}

struct IndexedReceiptDraft {
    operation: usize,
    draft: ReceiptDraft,
}

#[derive(Clone, Debug)]
struct ByteEdit {
    start: usize,
    end: usize,
    replacement: String,
}

impl From<SourceEdit> for ByteEdit {
    fn from(edit: SourceEdit) -> Self {
        Self {
            start: edit.start,
            end: edit.end,
            replacement: edit.replacement,
        }
    }
}

#[derive(Clone, Debug)]
enum ConflictRegion {
    Source { start: usize, end: usize },
    FrontmatterPath(Vec<String>),
}

impl ConflictRegion {
    fn span(span: SourceSpan) -> Self {
        Self::Source {
            start: span.byte_start as usize,
            end: span.byte_end as usize,
        }
    }

    fn point(byte: usize) -> Self {
        Self::Source {
            start: byte,
            end: byte,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExpectedKind {
    Block,
    HeadingSection,
    Preamble,
    Task,
    TableRow,
}

enum ResultExpectation {
    None,
    Target {
        kind: ExpectedKind,
        location: ResultLocation,
    },
    TargetWithBlockClosure {
        kind: ExpectedKind,
        location: ResultLocation,
        blocks: Vec<ParserBlockExpectation>,
        target_block_kinds: Option<Vec<BlockKind>>,
    },
    ParserBlockClosure {
        blocks: Vec<ParserBlockExpectation>,
    },
    BlockFragment {
        location: ResultLocation,
        fragment: BlockFragmentShape,
    },
    Preamble {
        location: ResultLocation,
    },
    Section(SectionResultExpectation),
    FrontmatterField {
        path: Vec<String>,
        expected: FrontmatterExpectedValue,
    },
}

enum FrontmatterExpectedValue {
    Missing,
    Value(serde_json::Value),
}

enum ResultLocation {
    Base(SourceSpan),
    Edit { edit: usize, range: Range<usize> },
}

struct BlockFragmentShape {
    operation: &'static str,
    kind: BlockKind,
    block_start: usize,
    block_end: usize,
}

struct SectionResultExpectation {
    location: ResultLocation,
    parent_path: Vec<crate::target::HeadingAddressSegment>,
    canonical: String,
}

struct ParserBlockExpectation {
    kind: BlockKind,
    markdown: String,
    location: ResultLocation,
}

enum VerifiedResult {
    None,
    Target(TargetSnapshot),
}

struct AppliedEdit<'a> {
    mutation: usize,
    local: usize,
    operation: usize,
    edit: &'a ByteEdit,
}

enum ReceiptDraft {
    ReplaceBlock {
        before: ReplaceBlockState,
        disposition: MutationDisposition,
    },
    DeleteBlock {
        before: BlockIdentity,
    },
    InsertBlock {
        target: BlockInsertionEvidence,
        disposition: MutationDisposition,
    },
    MoveBlock {
        before: BlockIdentity,
        destination_before: BlockIdentity,
        disposition: MutationDisposition,
    },
    ReplaceSection {
        before: HeadingSectionIdentity,
        disposition: MutationDisposition,
    },
    InsertSection {
        parent_before: HeadingSectionIdentity,
    },
    ReplacePreamble {
        before: PreambleIdentity,
        disposition: MutationDisposition,
    },
    DeleteSection {
        before: SectionIdentity,
        disposition: MutationDisposition,
    },
    MoveSection {
        before: HeadingSectionIdentity,
        destination_before: HeadingSectionIdentity,
        disposition: MutationDisposition,
    },
    SetTaskStatus {
        before: TaskIdentity,
        disposition: MutationDisposition,
    },
    SetFrontmatter {
        before: FrontmatterFieldIdentity,
        disposition: MutationDisposition,
    },
    DeleteFrontmatter {
        before: FrontmatterFieldIdentity,
        disposition: MutationDisposition,
    },
    ReplaceTableRow {
        before: TableRowIdentity,
        disposition: MutationDisposition,
    },
    InsertTableRow {
        table_before: BlockIdentity,
    },
    DeleteTableRow {
        before: TableRowIdentity,
    },
}

fn move_interval(source: SourceSpan, destination: SourceSpan) -> ConflictRegion {
    ConflictRegion::Source {
        start: source.byte_start.min(destination.byte_start) as usize,
        end: source.byte_end.max(destination.byte_end) as usize,
    }
}

fn reject_claim_overlaps(mutations: &[PlannedMutation]) -> Result<(), CoreError> {
    let claims = mutations
        .iter()
        .flat_map(|mutation| mutation.claims.iter())
        .collect::<Vec<_>>();
    for (offset, left) in claims.iter().enumerate() {
        for right in &claims[offset + 1..] {
            if left.operation != right.operation && claims_overlap(&left.region, &right.region) {
                return Err(overlap_error(
                    left.operation,
                    right.operation,
                    "semantically",
                ));
            }
        }
    }
    Ok(())
}

fn claims_overlap(left: &ConflictRegion, right: &ConflictRegion) -> bool {
    match (left, right) {
        (
            ConflictRegion::Source {
                start: left_start,
                end: left_end,
            },
            ConflictRegion::Source {
                start: right_start,
                end: right_end,
            },
        ) => ranges_overlap(*left_start, *left_end, *right_start, *right_end),
        (ConflictRegion::FrontmatterPath(left), ConflictRegion::FrontmatterPath(right)) => {
            left.starts_with(right) || right.starts_with(left)
        }
        _ => false,
    }
}

fn ranges_overlap(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    if left_start == left_end && right_start == right_end {
        left_start == right_start
    } else if left_start == left_end {
        right_start <= left_start && left_start <= right_end
    } else if right_start == right_end {
        left_start <= right_start && right_start <= left_end
    } else {
        left_start < right_end && right_start < left_end
    }
}

fn overlap_error(left: usize, right: usize, qualifier: &str) -> CoreError {
    let (left, right) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    CoreError::PatchInvariant(format!("operations {left} and {right} overlap {qualifier}"))
}

fn plan_operations(
    document: &Document,
    operations: &[PatchOp],
) -> Result<Vec<PlannedMutation>, CoreError> {
    let mut planned = Vec::with_capacity(operations.len());
    let mut frontmatter = Vec::new();
    for (index, operation) in operations.iter().enumerate() {
        match operation {
            PatchOp::SetFrontmatter { .. } | PatchOp::DeleteFrontmatter { .. } => {
                frontmatter.push((index, operation));
            }
            _ => planned.push(plan_operation(document, index, operation)?),
        }
    }
    if !frontmatter.is_empty() {
        planned.push(plan_frontmatter_group(document, &frontmatter)?);
    }
    Ok(planned)
}

fn plan_operation(
    document: &Document,
    operation_index: usize,
    operation: &PatchOp,
) -> Result<PlannedMutation, CoreError> {
    match operation {
        PatchOp::ReplaceBlock { target, markdown } => {
            let current = document.resolve(&TargetAddress::Block {
                block: target.address.clone(),
            })?;
            let before = ReplaceBlockState::try_from(current.snapshot())?;
            let original = document.slice(&target.guard.span)?;
            let normalized = normalize_line_endings(markdown, document.line_ending_style());
            let replacement = if original.ends_with('\n') {
                normalized
            } else {
                strip_one_trailing_newline(normalized)
            };
            let disposition = if replacement == original {
                MutationDisposition::NoChange
            } else if replacement.is_empty() {
                MutationDisposition::Deleted
            } else {
                MutationDisposition::Replaced
            };
            let (edits, result) = match disposition {
                MutationDisposition::NoChange => (
                    Vec::new(),
                    target_result(ExpectedKind::Block, ResultLocation::Base(target.guard.span)),
                ),
                MutationDisposition::Replaced => {
                    let len = replacement.len();
                    let fragment = parse_block_fragment("replace_block", &replacement)?;
                    (
                        vec![ByteEdit {
                            start: target.guard.span.byte_start as usize,
                            end: target.guard.span.byte_end as usize,
                            replacement,
                        }],
                        ResultExpectation::BlockFragment {
                            location: ResultLocation::Edit {
                                edit: 0,
                                range: 0..len,
                            },
                            fragment,
                        },
                    )
                }
                MutationDisposition::Deleted => (
                    vec![ByteEdit {
                        start: target.guard.span.byte_start as usize,
                        end: target.guard.span.byte_end as usize,
                        replacement: String::new(),
                    }],
                    ResultExpectation::None,
                ),
                MutationDisposition::Inserted => unreachable!(),
            };
            Ok(atomic_plan(
                operation_index,
                vec![ConflictRegion::span(target.guard.span)],
                edits,
                result,
                ReceiptDraft::ReplaceBlock {
                    before,
                    disposition,
                },
            ))
        }
        PatchOp::DeleteBlock { target } => {
            let current = document.resolve(&TargetAddress::Block {
                block: target.address.clone(),
            })?;
            Ok(atomic_plan(
                operation_index,
                vec![ConflictRegion::span(target.guard.span)],
                vec![ByteEdit {
                    start: target.guard.span.byte_start as usize,
                    end: target.guard.span.byte_end as usize,
                    replacement: String::new(),
                }],
                ResultExpectation::None,
                ReceiptDraft::DeleteBlock {
                    before: BlockIdentity::try_from(current.snapshot())?,
                },
            ))
        }
        PatchOp::InsertBlock { target, markdown } => {
            plan_insert_block(document, operation_index, target, markdown)
        }
        PatchOp::MoveBlock {
            source,
            destination,
            position,
        } => plan_move_block(document, operation_index, source, destination, *position),
        PatchOp::ReplaceSection { target, fragment } => {
            plan_replace_section(document, operation_index, target, fragment)
        }
        PatchOp::InsertSection { target, fragment } => {
            plan_insert_section(document, operation_index, target, fragment)
        }
        PatchOp::ReplacePreamble { target, markdown } => {
            plan_replace_preamble(document, operation_index, target, markdown)
        }
        PatchOp::DeleteSection { target } => plan_delete_section(document, operation_index, target),
        PatchOp::MoveSection {
            source,
            destination,
            position,
            keep_level,
        } => plan_move_section(
            document,
            operation_index,
            source,
            destination,
            *position,
            *keep_level,
        ),
        PatchOp::SetTaskStatus { target, status } => {
            plan_task(document, operation_index, target, *status)
        }
        PatchOp::ReplaceTableRow { target, markdown } => {
            plan_replace_table_row(document, operation_index, target, markdown)
        }
        PatchOp::InsertTableRow {
            target,
            row,
            markdown,
        } => plan_insert_table_row(document, operation_index, target, *row, markdown),
        PatchOp::DeleteTableRow { target } => {
            plan_delete_table_row(document, operation_index, target)
        }
        PatchOp::SetFrontmatter { .. } | PatchOp::DeleteFrontmatter { .. } => {
            unreachable!("frontmatter operations are grouped")
        }
    }
}

fn target_result(kind: ExpectedKind, location: ResultLocation) -> ResultExpectation {
    ResultExpectation::Target { kind, location }
}

fn section_expected_kind(address: &crate::target::SectionAddress) -> ExpectedKind {
    match address {
        crate::target::SectionAddress::Preamble => ExpectedKind::Preamble,
        crate::target::SectionAddress::Heading { .. } => ExpectedKind::HeadingSection,
    }
}

fn atomic_plan(
    operation: usize,
    mut claims: Vec<ConflictRegion>,
    edits: Vec<ByteEdit>,
    result: ResultExpectation,
    receipt: ReceiptDraft,
) -> PlannedMutation {
    claims.extend(edits.iter().map(|edit| ConflictRegion::Source {
        start: edit.start,
        end: edit.end,
    }));
    PlannedMutation {
        claims: claims
            .into_iter()
            .map(|region| IndexedClaim { operation, region })
            .collect(),
        edits,
        results: vec![IndexedResultExpectation {
            operation,
            expectation: result,
        }],
        receipts: vec![IndexedReceiptDraft {
            operation,
            draft: receipt,
        }],
    }
}

fn plan_insert_block(
    document: &Document,
    operation: usize,
    target: &BlockInsertionTarget,
    markdown: &str,
) -> Result<PlannedMutation, CoreError> {
    if markdown.is_empty() {
        return Err(CoreError::InvalidPatch(
            "insert_block markdown must not be empty".into(),
        ));
    }
    let claims = vec![ConflictRegion::point(super::insertion_base_anchor(
        document, target,
    ))];
    let content = normalize_line_endings(markdown, document.line_ending_style());
    let fragment = parse_block_fragment("insert_block", &content)?;
    let insert_byte = super::insertion_base_anchor(document, target);
    let before = &document.source()[..insert_byte];
    let after = &document.source()[insert_byte..];
    let newline = newline(document.line_ending_style());
    let mut replacement = String::with_capacity(content.len() + 8);
    let leading_present = trailing_line_breaks(before) + leading_line_breaks(&content);
    let leading_required = usize::from(!before.is_empty()) * 2;
    replacement.push_str(&newline.repeat(leading_required.saturating_sub(leading_present)));
    let payload_start = replacement.len();
    replacement.push_str(&content);
    let payload_end = replacement.len();
    let trailing_present = trailing_line_breaks(&content) + leading_line_breaks(after);
    let trailing_required = usize::from(!after.is_empty()) * 2;
    replacement.push_str(&newline.repeat(trailing_required.saturating_sub(trailing_present)));
    Ok(atomic_plan(
        operation,
        claims,
        vec![ByteEdit {
            start: insert_byte,
            end: insert_byte,
            replacement,
        }],
        ResultExpectation::BlockFragment {
            location: ResultLocation::Edit {
                edit: 0,
                range: payload_start..payload_end,
            },
            fragment,
        },
        ReceiptDraft::InsertBlock {
            target: BlockInsertionEvidence::from(target),
            disposition: MutationDisposition::Inserted,
        },
    ))
}

fn parse_block_fragment(
    operation: &'static str,
    source: &str,
) -> Result<BlockFragmentShape, CoreError> {
    let fragment = Document::parse_fragment(source.to_string())?;
    if fragment.index().source_block_indices().len() != 1 {
        return Err(CoreError::InvalidPatch(format!(
            "{operation} payload must parse as exactly one body block"
        )));
    }
    let blocks = fragment
        .map()?
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    let [block] = blocks.as_slice() else {
        return Err(CoreError::InvalidPatch(format!(
            "{operation} payload must parse as exactly one body block"
        )));
    };
    let TargetSummary::Block { kind, .. } = block.summary else {
        unreachable!("filtered block target has block summary")
    };
    let span = block.selection_span.expect("body block has selection span");
    Ok(BlockFragmentShape {
        operation,
        kind,
        block_start: span.byte_start as usize,
        block_end: span.byte_end as usize,
    })
}

fn plan_move_block(
    document: &Document,
    operation: usize,
    source: &ReplaceBlockTarget,
    destination: &ReplaceBlockTarget,
    position: RelativePosition,
) -> Result<PlannedMutation, CoreError> {
    let claims = vec![move_interval(source.guard.span, destination.guard.span)];
    let source_snapshot = document.resolve(&TargetAddress::Block {
        block: source.address.clone(),
    })?;
    let destination_snapshot = document.resolve(&TargetAddress::Block {
        block: destination.address.clone(),
    })?;
    let source_index = super::block_index(document, &source.address)?;
    let destination_index = super::block_index(document, &destination.address)?;
    if source_index == destination_index {
        return Err(CoreError::InvalidPatch(
            "source and destination block addresses must differ".into(),
        ));
    }
    let source_order = document.index().source_block_indices();
    let source_position = source_order
        .iter()
        .position(|index| *index == source_index)
        .expect("resolved source is source ordered");
    let mut order = source_order.clone();
    let moved = order.remove(source_position);
    let destination_position = order
        .iter()
        .position(|index| *index == destination_index)
        .expect("resolved destination remains after source removal");
    let insertion = match position {
        RelativePosition::Before => destination_position,
        RelativePosition::After => destination_position + 1,
    };
    order.insert(insertion, moved);
    let moved_position = order
        .iter()
        .position(|index| *index == source_index)
        .expect("moved block remains in order");
    let low = source_position.min(moved_position);
    let high = source_position.max(moved_position);
    let interval_start = document.blocks()[source_order[low] as usize]
        .span
        .byte_start as usize;
    let interval_end = source_order
        .get(high + 1)
        .map(|index| document.blocks()[*index as usize].span.byte_start as usize)
        .unwrap_or(document.source().len());
    let mut replacement = String::with_capacity(interval_end - interval_start);
    let mut moved_range = 0..0;
    let mut block_expectations = Vec::with_capacity(high - low + 1);
    for slot in low..=high {
        let parser_index = order[slot];
        let block = &document.blocks()[parser_index as usize];
        let block_start = replacement.len();
        replacement.push_str(document.slice_unchecked(&block.span));
        let block_end = replacement.len();
        if parser_index == source_index {
            moved_range = block_start..block_end;
        }
        block_expectations.push(ParserBlockExpectation {
            kind: block.kind,
            markdown: document.slice_unchecked(&block.span).to_string(),
            location: ResultLocation::Edit {
                edit: 1,
                range: block_start..block_end,
            },
        });
        let gap_start = document.blocks()[source_order[slot] as usize].span.byte_end as usize;
        let gap_end = source_order
            .get(slot + 1)
            .map(|index| document.blocks()[*index as usize].span.byte_start as usize)
            .unwrap_or(document.source().len());
        replacement.push_str(&document.source()[gap_start..gap_end]);
    }
    let unchanged = replacement == document.source()[interval_start..interval_end];
    let (edits, result, disposition) = if unchanged {
        (
            Vec::new(),
            target_result(ExpectedKind::Block, ResultLocation::Base(source.guard.span)),
            MutationDisposition::NoChange,
        )
    } else {
        (
            vec![
                ByteEdit {
                    start: interval_start,
                    end: interval_end,
                    replacement: String::new(),
                },
                ByteEdit {
                    start: interval_start,
                    end: interval_start,
                    replacement,
                },
            ],
            ResultExpectation::TargetWithBlockClosure {
                kind: ExpectedKind::Block,
                location: ResultLocation::Edit {
                    edit: 1,
                    range: moved_range,
                },
                blocks: block_expectations,
                target_block_kinds: None,
            },
            MutationDisposition::Replaced,
        )
    };
    Ok(atomic_plan(
        operation,
        claims,
        edits,
        result,
        ReceiptDraft::MoveBlock {
            before: BlockIdentity::try_from(source_snapshot.snapshot())?,
            destination_before: BlockIdentity::try_from(destination_snapshot.snapshot())?,
            disposition,
        },
    ))
}

fn plan_replace_section(
    document: &Document,
    operation: usize,
    target: &HeadingPatchTarget,
    fragment: &crate::fragment::SectionFragment,
) -> Result<PlannedMutation, CoreError> {
    let claims = vec![ConflictRegion::span(target.guard.span)];
    let address = TargetAddress::Section {
        path: target.path.clone(),
    };
    let current = document.resolve(&address)?;
    let before = HeadingSectionIdentity::try_from(current.snapshot())?;
    let current_read = current.read_section(document)?;
    let prepared = fragment.prepare()?;
    let original = document.slice_unchecked(&target.guard.span);
    let semantic_unchanged = prepared.is_semantic()
        && current_read.fragment
            == crate::fragment::SectionFragment::Semantic {
                markdown: prepared.canonical().to_string(),
            };
    let literal_unchanged = matches!(fragment, crate::fragment::SectionFragment::Literal { markdown } if markdown == original);
    let disposition = if semantic_unchanged || literal_unchanged {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let parent_path = target.path[..target.path.len() - 1].to_vec();
    let parent_level = heading_level_for_path(document, &parent_path)?;
    let (edits, result) = if disposition == MutationDisposition::NoChange {
        (
            Vec::new(),
            ResultExpectation::Section(SectionResultExpectation {
                location: ResultLocation::Base(target.guard.span),
                parent_path,
                canonical: prepared.canonical().to_string(),
            }),
        )
    } else {
        let mut replacement = prepared.render(parent_level, document.line_ending_style())?;
        if prepared.rendered_root_level(parent_level)? <= parent_level {
            return Err(CoreError::InvalidPatch(
                "literal replacement root must remain beneath the structural parent".into(),
            ));
        }
        let follows = target.guard.span.byte_end as usize != document.source().len();
        if prepared.is_semantic() && follows {
            let suffix = &document.source()[target.guard.span.byte_end as usize..];
            let present = trailing_line_breaks(&replacement) + leading_line_breaks(suffix);
            replacement.push_str(
                &newline(document.line_ending_style()).repeat(2usize.saturating_sub(present)),
            );
        }
        let len = replacement.len();
        (
            vec![ByteEdit {
                start: target.guard.span.byte_start as usize,
                end: target.guard.span.byte_end as usize,
                replacement,
            }],
            ResultExpectation::Section(SectionResultExpectation {
                location: ResultLocation::Edit {
                    edit: 0,
                    range: 0..len,
                },
                parent_path,
                canonical: prepared.canonical().to_string(),
            }),
        )
    };
    Ok(atomic_plan(
        operation,
        claims,
        edits,
        result,
        ReceiptDraft::ReplaceSection {
            before,
            disposition,
        },
    ))
}

fn plan_insert_section(
    document: &Document,
    operation: usize,
    target: &SectionInsertionTarget,
    fragment: &crate::fragment::SectionFragment,
) -> Result<PlannedMutation, CoreError> {
    let parent_address = TargetAddress::Section {
        path: target.parent.path.clone(),
    };
    let parent = document.resolve(&parent_address)?;
    let parent_before = HeadingSectionIdentity::try_from(parent.snapshot())?;
    let parent_level = heading_level_for_path(document, &target.parent.path)?;
    let prepared = fragment.prepare()?;
    if prepared.rendered_root_level(parent_level)? <= parent_level {
        return Err(CoreError::InvalidPatch(
            "literal insertion root must be deeper than its structural parent".into(),
        ));
    }
    let content = prepared.render(parent_level, document.line_ending_style())?;
    let insert_byte = target.parent.guard.span.byte_end as usize;
    let mut replacement = content;
    if prepared.is_semantic() {
        let before = &document.source()[..insert_byte];
        let after = &document.source()[insert_byte..];
        let line_ending = newline(document.line_ending_style());
        let leading_present = trailing_line_breaks(before) + leading_line_breaks(&replacement);
        let trailing_present = trailing_line_breaks(&replacement) + leading_line_breaks(after);
        let mut bounded = String::new();
        bounded.push_str(&line_ending.repeat(2usize.saturating_sub(leading_present)));
        bounded.push_str(&replacement);
        bounded.push_str(
            &line_ending
                .repeat((usize::from(!after.is_empty()) * 2).saturating_sub(trailing_present)),
        );
        replacement = bounded;
    }
    let len = replacement.len();
    Ok(atomic_plan(
        operation,
        vec![ConflictRegion::point(insert_byte)],
        vec![ByteEdit {
            start: insert_byte,
            end: insert_byte,
            replacement,
        }],
        ResultExpectation::Section(SectionResultExpectation {
            location: ResultLocation::Edit {
                edit: 0,
                range: 0..len,
            },
            parent_path: target.parent.path.clone(),
            canonical: prepared.canonical().to_string(),
        }),
        ReceiptDraft::InsertSection { parent_before },
    ))
}

fn plan_replace_preamble(
    document: &Document,
    operation: usize,
    target: &PreamblePatchTarget,
    markdown: &str,
) -> Result<PlannedMutation, CoreError> {
    if markdown.is_empty() {
        return Err(CoreError::InvalidPatch(
            "replace_preamble payload cannot be empty; use delete_section for removal".into(),
        ));
    }
    let current = document.resolve(&TargetAddress::Preamble)?;
    let before = PreambleIdentity::try_from(current.snapshot())?;
    let original = document.slice_unchecked(&target.guard.span);
    let disposition = if markdown == original {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let (edits, result) = if disposition == MutationDisposition::NoChange {
        (
            Vec::new(),
            ResultExpectation::Preamble {
                location: ResultLocation::Base(target.guard.span),
            },
        )
    } else {
        (
            vec![ByteEdit {
                start: target.guard.span.byte_start as usize,
                end: target.guard.span.byte_end as usize,
                replacement: markdown.to_string(),
            }],
            ResultExpectation::Preamble {
                location: ResultLocation::Edit {
                    edit: 0,
                    range: 0..markdown.len(),
                },
            },
        )
    };
    Ok(atomic_plan(
        operation,
        vec![ConflictRegion::span(target.guard.span)],
        edits,
        result,
        ReceiptDraft::ReplacePreamble {
            before,
            disposition,
        },
    ))
}

fn heading_level_for_path(
    document: &Document,
    path: &[crate::target::HeadingAddressSegment],
) -> Result<u8, CoreError> {
    if path.is_empty() {
        return Ok(0);
    }
    let snapshot = document
        .resolve(&TargetAddress::Section {
            path: path.to_vec(),
        })?
        .snapshot()
        .clone();
    let TargetSummary::Section { level, .. } = snapshot.summary else {
        return Err(CoreError::PatchInvariant(
            "heading address resolved without section summary".into(),
        ));
    };
    Ok(level)
}

fn plan_delete_section(
    document: &Document,
    operation: usize,
    target: &SectionPatchTarget,
) -> Result<PlannedMutation, CoreError> {
    let claims = vec![ConflictRegion::span(target.guard.span)];
    let current = super::resolve_section_snapshot(document, &target.address)?;
    let before = SectionIdentity::try_from(&current)?;
    let disposition = if target.guard.span.byte_start == target.guard.span.byte_end {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Deleted
    };
    let surviving_blocks = document
        .index()
        .source_block_indices()
        .into_iter()
        .filter_map(|index| {
            let block = &document.blocks()[index as usize];
            let deleted = block.span.byte_start >= target.guard.span.byte_start
                && block.span.byte_end <= target.guard.span.byte_end;
            (!deleted).then(|| ParserBlockExpectation {
                kind: block.kind,
                markdown: document.slice_unchecked(&block.span).to_string(),
                location: ResultLocation::Base(block.span),
            })
        })
        .collect::<Vec<_>>();
    Ok(atomic_plan(
        operation,
        claims,
        if disposition == MutationDisposition::Deleted {
            vec![ByteEdit {
                start: target.guard.span.byte_start as usize,
                end: target.guard.span.byte_end as usize,
                replacement: String::new(),
            }]
        } else {
            Vec::new()
        },
        if disposition == MutationDisposition::NoChange {
            target_result(
                section_expected_kind(&target.address),
                ResultLocation::Base(target.guard.span),
            )
        } else {
            ResultExpectation::ParserBlockClosure {
                blocks: surviving_blocks,
            }
        },
        ReceiptDraft::DeleteSection {
            before,
            disposition,
        },
    ))
}

fn plan_move_section(
    document: &Document,
    operation: usize,
    source: &HeadingPatchTarget,
    destination: &HeadingPatchTarget,
    position: SectionMovePosition,
    keep_level: bool,
) -> Result<PlannedMutation, CoreError> {
    let claims = vec![move_interval(source.guard.span, destination.guard.span)];
    let source_address = source.address();
    let destination_address = destination.address();
    let before = HeadingSectionIdentity::try_from(&super::resolve_section_snapshot(
        document,
        &source_address,
    )?)?;
    let destination_before = HeadingSectionIdentity::try_from(&super::resolve_section_snapshot(
        document,
        &destination_address,
    )?)?;
    let interval_start = source
        .guard
        .span
        .byte_start
        .min(destination.guard.span.byte_start);
    let interval_end = source
        .guard
        .span
        .byte_end
        .max(destination.guard.span.byte_end);
    let block_expectations = document
        .index()
        .source_block_indices()
        .into_iter()
        .filter_map(|index| {
            let block = &document.blocks()[index as usize];
            let in_interval =
                block.span.byte_start >= interval_start && block.span.byte_end <= interval_end;
            let in_source = block.span.byte_start >= source.guard.span.byte_start
                && block.span.byte_end <= source.guard.span.byte_end;
            (in_interval && !in_source).then(|| ParserBlockExpectation {
                kind: block.kind,
                markdown: document.slice_unchecked(&block.span).to_string(),
                location: ResultLocation::Base(block.span),
            })
        })
        .collect::<Vec<_>>();
    let mode = match position {
        SectionMovePosition::BeforeSibling => InsertMode::BeforeSibling,
        SectionMovePosition::AfterSibling => InsertMode::AfterSibling,
        SectionMovePosition::IntoAsChild => InsertMode::IntoAsChild,
    };
    let resolved_source = crate::section::resolve_address(document, &source_address)?;
    let source_block_kinds = resolved_source
        .block_indices
        .iter()
        .map(|index| document.blocks()[*index as usize].kind)
        .collect::<Vec<_>>();
    let planned = crate::section_edit::plan_section_move(
        document,
        resolved_source,
        crate::section::resolve_address(document, &destination_address)?,
        mode,
        keep_level,
    )?;
    let edits = planned
        .edits
        .into_iter()
        .map(ByteEdit::from)
        .collect::<Vec<_>>();
    let changed = edits_change_source(document.source(), &edits)?;
    let (edits, result, disposition) = if changed {
        (
            edits,
            ResultExpectation::TargetWithBlockClosure {
                kind: ExpectedKind::HeadingSection,
                location: ResultLocation::Edit {
                    edit: planned.result_edit,
                    range: planned.result_range,
                },
                blocks: block_expectations,
                target_block_kinds: Some(source_block_kinds),
            },
            MutationDisposition::Replaced,
        )
    } else {
        (
            Vec::new(),
            target_result(
                ExpectedKind::HeadingSection,
                ResultLocation::Base(source.guard.span),
            ),
            MutationDisposition::NoChange,
        )
    };
    Ok(atomic_plan(
        operation,
        claims,
        edits,
        result,
        ReceiptDraft::MoveSection {
            before,
            destination_before,
            disposition,
        },
    ))
}

fn plan_task(
    document: &Document,
    operation: usize,
    target: &TaskPatchTarget,
    status: TaskStatus,
) -> Result<PlannedMutation, CoreError> {
    let address = TargetAddress::Task {
        block: target.block.clone(),
        path: target.path.clone(),
    };
    let current = document.resolve(&address)?;
    let before = TaskIdentity::try_from(current.snapshot())?;
    let (symbol, observed) = super::task_edit_site(document, target)?;
    let claims = vec![ConflictRegion::Source {
        start: symbol,
        end: symbol + 1,
    }];
    let disposition = if observed == status {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    Ok(atomic_plan(
        operation,
        claims,
        if disposition == MutationDisposition::Replaced {
            vec![ByteEdit {
                start: symbol,
                end: symbol + 1,
                replacement: match status {
                    TaskStatus::Done => "x",
                    TaskStatus::Pending => " ",
                }
                .into(),
            }]
        } else {
            Vec::new()
        },
        target_result(ExpectedKind::Task, ResultLocation::Base(target.guard.span)),
        ReceiptDraft::SetTaskStatus {
            before,
            disposition,
        },
    ))
}

fn plan_replace_table_row(
    document: &Document,
    operation: usize,
    target: &TableRowPatchTarget,
    markdown: &str,
) -> Result<PlannedMutation, CoreError> {
    let current = document.resolve(&TargetAddress::TableRow {
        table: target.table.clone(),
        row: target.row,
    })?;
    let claims = vec![ConflictRegion::span(
        current.snapshot().selection_span.ok_or_else(|| {
            CoreError::PatchInvariant("table row has no semantic selection".into())
        })?,
    )];
    let before = TableRowIdentity::try_from(current.snapshot())?;
    let plan = crate::table::plan_replace_row(
        document,
        super::block_index(document, &target.table)?,
        target.row,
        markdown.to_string(),
    )?;
    let result = table_result(plan.result, ExpectedKind::TableRow);
    Ok(atomic_plan(
        operation,
        claims,
        plan.edit.into_iter().map(ByteEdit::from).collect(),
        result,
        ReceiptDraft::ReplaceTableRow {
            before,
            disposition: plan.disposition,
        },
    ))
}

fn plan_insert_table_row(
    document: &Document,
    operation: usize,
    target: &TablePatchTarget,
    row: u32,
    markdown: &str,
) -> Result<PlannedMutation, CoreError> {
    let claims = vec![ConflictRegion::span(target.guard.span)];
    let table = document.resolve(&TargetAddress::Block {
        block: target.table.clone(),
    })?;
    let plan = crate::table::plan_insert_row(
        document,
        super::block_index(document, &target.table)?,
        row,
        markdown.to_string(),
    )?;
    Ok(atomic_plan(
        operation,
        claims,
        plan.edit.into_iter().map(ByteEdit::from).collect(),
        table_result(plan.result, ExpectedKind::TableRow),
        ReceiptDraft::InsertTableRow {
            table_before: BlockIdentity::try_from(table.snapshot())?,
        },
    ))
}

fn plan_delete_table_row(
    document: &Document,
    operation: usize,
    target: &TableRowPatchTarget,
) -> Result<PlannedMutation, CoreError> {
    let current = document.resolve(&TargetAddress::TableRow {
        table: target.table.clone(),
        row: target.row,
    })?;
    let claims = vec![ConflictRegion::span(
        current.snapshot().selection_span.ok_or_else(|| {
            CoreError::PatchInvariant("table row has no semantic selection".into())
        })?,
    )];
    let plan = crate::table::plan_delete_row(
        document,
        super::block_index(document, &target.table)?,
        target.row,
    )?;
    Ok(atomic_plan(
        operation,
        claims,
        plan.edit.into_iter().map(ByteEdit::from).collect(),
        ResultExpectation::None,
        ReceiptDraft::DeleteTableRow {
            before: TableRowIdentity::try_from(current.snapshot())?,
        },
    ))
}

fn table_result(result: TableResultLocation, kind: ExpectedKind) -> ResultExpectation {
    match result {
        TableResultLocation::None => ResultExpectation::None,
        TableResultLocation::Base(span) => target_result(kind, ResultLocation::Base(span)),
        TableResultLocation::Replacement(range) => {
            target_result(kind, ResultLocation::Edit { edit: 0, range })
        }
    }
}

fn plan_frontmatter_group(
    document: &Document,
    operations: &[(usize, &PatchOp)],
) -> Result<PlannedMutation, CoreError> {
    let grouped = operations
        .iter()
        .map(|(_, operation)| match operation {
            PatchOp::SetFrontmatter { target, value } => Some(FrontmatterPathMutation {
                path: target.path.clone(),
                action: FrontmatterAction::Set(value.clone()),
            }),
            PatchOp::DeleteFrontmatter { target } => Some(FrontmatterPathMutation {
                path: target.path.clone(),
                action: FrontmatterAction::Delete,
            }),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            CoreError::PatchInvariant("non-frontmatter operation entered group".into())
        })?;
    let mut claims = operations
        .iter()
        .map(|(operation, patch_op)| {
            let path = match patch_op {
                PatchOp::SetFrontmatter { target, .. } | PatchOp::DeleteFrontmatter { target } => {
                    target.path.clone()
                }
                _ => unreachable!("frontmatter group was refined"),
            };
            IndexedClaim {
                operation: *operation,
                region: ConflictRegion::FrontmatterPath(path),
            }
        })
        .collect::<Vec<_>>();
    for (offset, left) in claims.iter().enumerate() {
        for right in &claims[offset + 1..] {
            if claims_overlap(&left.region, &right.region) {
                return Err(overlap_error(
                    left.operation,
                    right.operation,
                    "semantically",
                ));
            }
        }
    }
    let plan = crate::frontmatter::plan_path_batch(document, &grouped)?;
    if plan.dispositions.len() != operations.len() {
        return Err(CoreError::PatchInvariant(
            "frontmatter group lost an operation disposition".into(),
        ));
    }
    let mut results = Vec::with_capacity(operations.len());
    let mut receipts = Vec::with_capacity(operations.len());
    for ((operation, patch_op), disposition) in operations.iter().zip(plan.dispositions) {
        let (path, expected, draft) = match patch_op {
            PatchOp::SetFrontmatter { target, value } => (
                target.path.clone(),
                FrontmatterExpectedValue::Value(value.clone()),
                ReceiptDraft::SetFrontmatter {
                    before: super::frontmatter_identity(document, &target.path)?,
                    disposition,
                },
            ),
            PatchOp::DeleteFrontmatter { target } => (
                target.path.clone(),
                FrontmatterExpectedValue::Missing,
                ReceiptDraft::DeleteFrontmatter {
                    before: super::frontmatter_identity(document, &target.path)?,
                    disposition,
                },
            ),
            _ => unreachable!("frontmatter group was refined"),
        };
        results.push(IndexedResultExpectation {
            operation: *operation,
            expectation: ResultExpectation::FrontmatterField { path, expected },
        });
        receipts.push(IndexedReceiptDraft {
            operation: *operation,
            draft,
        });
    }
    let edits = plan
        .edit
        .into_iter()
        .map(ByteEdit::from)
        .collect::<Vec<_>>();
    if let (Some((operation, _)), Some(edit)) = (operations.first(), edits.first()) {
        claims.push(IndexedClaim {
            operation: *operation,
            region: ConflictRegion::Source {
                start: edit.start,
                end: edit.end,
            },
        });
    }
    Ok(PlannedMutation {
        claims,
        edits,
        results,
        receipts,
    })
}

fn reject_byte_overlaps(mutations: &[PlannedMutation]) -> Result<(), CoreError> {
    let edits = flatten_edits(mutations);
    for (offset, left) in edits.iter().enumerate() {
        for right in &edits[offset + 1..] {
            if left.mutation != right.mutation
                && ranges_overlap(
                    left.edit.start,
                    left.edit.end,
                    right.edit.start,
                    right.edit.end,
                )
            {
                return Err(overlap_error(
                    left.operation,
                    right.operation,
                    "in byte edits",
                ));
            }
        }
    }
    Ok(())
}

fn flatten_edits(mutations: &[PlannedMutation]) -> Vec<AppliedEdit<'_>> {
    mutations
        .iter()
        .enumerate()
        .flat_map(|(mutation, plan)| {
            let operation = plan
                .receipts
                .iter()
                .map(|receipt| receipt.operation)
                .min()
                .expect("complete plan has a receipt");
            plan.edits
                .iter()
                .enumerate()
                .map(move |(local, edit)| AppliedEdit {
                    mutation,
                    local,
                    operation,
                    edit,
                })
        })
        .collect()
}

fn edits_change_source(source: &str, edits: &[ByteEdit]) -> Result<bool, CoreError> {
    if edits.is_empty() {
        return Ok(false);
    }
    let start = edits.iter().map(|edit| edit.start).min().unwrap();
    let end = edits.iter().map(|edit| edit.end).max().unwrap();
    let mut local = source
        .get(start..end)
        .ok_or_else(|| CoreError::PatchInvariant("planned edit interval is invalid".into()))?
        .to_string();
    let mut order = edits.iter().collect::<Vec<_>>();
    order.sort_by_key(|edit| std::cmp::Reverse((edit.start, edit.end)));
    for edit in order {
        local.replace_range(edit.start - start..edit.end - start, &edit.replacement);
    }
    Ok(local != source[start..end])
}

fn newline(style: LineEndingStyle) -> &'static str {
    if style == LineEndingStyle::Crlf {
        "\r\n"
    } else {
        "\n"
    }
}

fn leading_line_breaks(value: &str) -> usize {
    let mut count = 0;
    let mut bytes = value.as_bytes();
    while let Some(rest) = bytes
        .strip_prefix(b"\r\n")
        .or_else(|| bytes.strip_prefix(b"\n"))
    {
        count += 1;
        bytes = rest;
    }
    count
}

fn trailing_line_breaks(value: &str) -> usize {
    let mut count = 0;
    let mut bytes = value.as_bytes();
    while let Some(rest) = bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
    {
        count += 1;
        bytes = rest;
    }
    count
}

fn verify_result(
    candidate: &Document,
    targets: &ResultTargetIndex,
    mutation: usize,
    edits: &[AppliedEdit<'_>],
    expectation: &ResultExpectation,
) -> Result<VerifiedResult, CoreError> {
    match expectation {
        ResultExpectation::None => Ok(VerifiedResult::None),
        ResultExpectation::FrontmatterField { path, expected } => {
            let record = crate::frontmatter::read(candidate)?;
            let actual = crate::target::project_frontmatter_field(&record.data, path);
            let matches = match expected {
                FrontmatterExpectedValue::Missing => actual.is_none(),
                FrontmatterExpectedValue::Value(value) => actual == Some(value),
            };
            if !matches {
                return Err(CoreError::PatchInvariant(format!(
                    "frontmatter result at path {path:?} differs from its planned value"
                )));
            }
            Ok(VerifiedResult::Target(
                candidate
                    .resolve(&TargetAddress::FrontmatterField { path: path.clone() })?
                    .snapshot()
                    .clone(),
            ))
        }
        ResultExpectation::Target { kind, location } => {
            let (start, end) = resolve_location(mutation, edits, location)?;
            let snapshot = targets.get(*kind, start, end).cloned().ok_or_else(|| {
                CoreError::PatchInvariant(format!(
                    "operation result lost its promised {:?} target at bytes {start}..{end}",
                    kind
                ))
            })?;
            Ok(VerifiedResult::Target(snapshot))
        }
        ResultExpectation::TargetWithBlockClosure {
            kind,
            location,
            blocks,
            target_block_kinds,
        } => {
            for block in blocks {
                let (start, end) = resolve_location(mutation, edits, &block.location)?;
                targets.verify_parser_block(start, end, block)?;
            }
            let (start, end) = resolve_location(mutation, edits, location)?;
            let snapshot = targets.get(*kind, start, end).cloned().ok_or_else(|| {
                CoreError::PatchInvariant(format!(
                    "operation result lost its promised {:?} target at bytes {start}..{end}",
                    kind
                ))
            })?;
            if let Some(expected_kinds) = target_block_kinds {
                let span = snapshot.selection_span.ok_or_else(|| {
                    CoreError::PatchInvariant("move closure target has no selection span".into())
                })?;
                let actual_kinds = targets.parser_block_kinds_within(span);
                if &actual_kinds != expected_kinds {
                    return Err(CoreError::PatchInvariant(format!(
                        "move closure changed target block sequence from {expected_kinds:?} to {actual_kinds:?}"
                    )));
                }
            }
            Ok(VerifiedResult::Target(snapshot))
        }
        ResultExpectation::ParserBlockClosure { blocks } => {
            for block in blocks {
                let (start, end) = resolve_location(mutation, edits, &block.location)?;
                targets.verify_parser_block(start, end, block)?;
            }
            Ok(VerifiedResult::None)
        }
        ResultExpectation::BlockFragment { location, fragment } => {
            let (start, end) = resolve_location(mutation, edits, location)?;
            Ok(VerifiedResult::Target(
                targets.block_fragment(start, end, fragment)?,
            ))
        }
        ResultExpectation::Preamble { location } => {
            let (start, end) = resolve_location(mutation, edits, location)?;
            Ok(VerifiedResult::Target(targets.preamble_within(
                candidate.source(),
                start,
                end,
            )?))
        }
        ResultExpectation::Section(expectation) => {
            let (start, end) = resolve_location(mutation, edits, &expectation.location)?;
            Ok(VerifiedResult::Target(targets.section_within(
                candidate,
                start,
                end,
                expectation,
            )?))
        }
    }
}

fn resolve_location(
    mutation: usize,
    edits: &[AppliedEdit<'_>],
    location: &ResultLocation,
) -> Result<(u32, u32), CoreError> {
    match location {
        ResultLocation::Base(span) => Ok((
            transform_base_offset(span.byte_start as usize, edits, true)? as u32,
            transform_base_offset(span.byte_end as usize, edits, false)? as u32,
        )),
        ResultLocation::Edit { edit, range } => {
            let applied = edits
                .iter()
                .find(|applied| applied.mutation == mutation && applied.local == *edit)
                .ok_or_else(|| {
                    CoreError::PatchInvariant("result expectation names a missing edit".into())
                })?;
            let output_start = transform_edit_start(applied, edits)?;
            Ok((
                (output_start + range.start) as u32,
                (output_start + range.end) as u32,
            ))
        }
    }
}

fn transform_base_offset(
    offset: usize,
    edits: &[AppliedEdit<'_>],
    include_insertion_at_boundary: bool,
) -> Result<usize, CoreError> {
    let mut transformed = offset as i64;
    for edit in edits {
        let before = edit.edit.end < offset
            || (edit.edit.end == offset
                && (edit.edit.start != edit.edit.end || include_insertion_at_boundary));
        if before {
            transformed +=
                edit.edit.replacement.len() as i64 - (edit.edit.end - edit.edit.start) as i64;
        }
    }
    usize::try_from(transformed).map_err(|_| {
        CoreError::PatchInvariant("combined edits moved a target before the document start".into())
    })
}

fn transform_edit_start(
    target: &AppliedEdit<'_>,
    edits: &[AppliedEdit<'_>],
) -> Result<usize, CoreError> {
    let mut transformed = target.edit.start as i64;
    for edit in edits {
        if edit.mutation == target.mutation && edit.local == target.local {
            continue;
        }
        if edit.edit.end <= target.edit.start {
            transformed +=
                edit.edit.replacement.len() as i64 - (edit.edit.end - edit.edit.start) as i64;
        }
    }
    usize::try_from(transformed).map_err(|_| {
        CoreError::PatchInvariant("combined edits moved a result before the document start".into())
    })
}

struct ResultTargetIndex {
    snapshots: HashMap<(ExpectedKind, u32, u32), TargetSnapshot>,
    blocks: Vec<TargetSnapshot>,
    preamble: Option<TargetSnapshot>,
    sections: Vec<TargetSnapshot>,
    parser_blocks: HashMap<(u32, u32), (BlockKind, String)>,
}

impl ResultTargetIndex {
    fn new(document: &Document) -> Result<Self, CoreError> {
        let mut snapshots = HashMap::new();
        let mut blocks = Vec::new();
        let mut preamble = None;
        let mut sections = Vec::new();
        let parser_blocks = document
            .index()
            .source_block_indices()
            .into_iter()
            .map(|index| {
                let block = &document.blocks()[index as usize];
                (
                    (block.span.byte_start, block.span.byte_end),
                    (
                        block.kind,
                        document.slice_unchecked(&block.span).to_string(),
                    ),
                )
            })
            .collect();
        for entry in document.index().entries_in_source_order() {
            let kind = match entry.node {
                IndexNode::BodyBlock { .. } => ExpectedKind::Block,
                IndexNode::Preamble { .. } => ExpectedKind::Preamble,
                IndexNode::Section { .. } => ExpectedKind::HeadingSection,
                IndexNode::TaskItem { .. } => ExpectedKind::Task,
                IndexNode::TableRow { .. } => ExpectedKind::TableRow,
                _ => continue,
            };
            let Some(address) = document.index().address_for_node(entry.id) else {
                continue;
            };
            let snapshot = document.resolve(address)?.snapshot().clone();
            let Some(span) = snapshot.selection_span else {
                continue;
            };
            if kind == ExpectedKind::Block {
                blocks.push(snapshot.clone());
            } else if kind == ExpectedKind::Preamble {
                preamble = Some(snapshot.clone());
            } else if kind == ExpectedKind::HeadingSection {
                sections.push(snapshot.clone());
            }
            snapshots.insert((kind, span.byte_start, span.byte_end), snapshot);
        }
        Ok(Self {
            snapshots,
            blocks,
            preamble,
            sections,
            parser_blocks,
        })
    }

    fn get(&self, kind: ExpectedKind, start: u32, end: u32) -> Option<&TargetSnapshot> {
        self.snapshots.get(&(kind, start, end))
    }

    fn verify_parser_block(
        &self,
        start: u32,
        end: u32,
        expected: &ParserBlockExpectation,
    ) -> Result<(), CoreError> {
        let Some((kind, markdown)) = self.parser_blocks.get(&(start, end)) else {
            return Err(CoreError::PatchInvariant(format!(
                "move closure lost parser block at bytes {start}..{end}"
            )));
        };
        if *kind != expected.kind || markdown != &expected.markdown {
            return Err(CoreError::PatchInvariant(format!(
                "move closure changed parser block at bytes {start}..{end}"
            )));
        }
        Ok(())
    }

    fn parser_block_kinds_within(&self, span: SourceSpan) -> Vec<BlockKind> {
        let mut blocks = self
            .parser_blocks
            .iter()
            .filter_map(|((start, end), (kind, _))| {
                (*start >= span.byte_start && *end <= span.byte_end).then_some((*start, *kind))
            })
            .collect::<Vec<_>>();
        blocks.sort_by_key(|(start, _)| *start);
        blocks.into_iter().map(|(_, kind)| kind).collect()
    }

    fn block_fragment(
        &self,
        region_start: u32,
        region_end: u32,
        fragment: &BlockFragmentShape,
    ) -> Result<TargetSnapshot, CoreError> {
        let matches = self
            .blocks
            .iter()
            .filter(|snapshot| {
                snapshot.selection_span.is_some_and(|span| {
                    span.byte_start >= region_start && span.byte_end <= region_end
                })
            })
            .collect::<Vec<_>>();
        let [snapshot] = matches.as_slice() else {
            return Err(CoreError::PatchInvariant(format!(
                "{} must produce exactly one body block inside bytes {region_start}..{region_end}",
                fragment.operation
            )));
        };
        let span = snapshot.selection_span.expect("body block selection span");
        let TargetSummary::Block { kind, .. } = snapshot.summary else {
            unreachable!("block snapshot has block summary")
        };
        if kind != fragment.kind
            || span.byte_start - region_start != fragment.block_start as u32
            || span.byte_end - region_start != fragment.block_end as u32
        {
            return Err(CoreError::PatchInvariant(format!(
                "{} result does not preserve its parsed fragment shape",
                fragment.operation
            )));
        }
        Ok((*snapshot).clone())
    }

    fn preamble_within(
        &self,
        source: &str,
        region_start: u32,
        region_end: u32,
    ) -> Result<TargetSnapshot, CoreError> {
        let snapshot = self
            .preamble
            .as_ref()
            .ok_or_else(|| CoreError::PatchInvariant("candidate has no preamble target".into()))?;
        let span = snapshot.selection_span.ok_or_else(|| {
            CoreError::PatchInvariant("candidate preamble has no selection span".into())
        })?;
        if span.byte_start < region_start || span.byte_end > region_end {
            return Err(CoreError::PatchInvariant(
                "preamble result escaped its declared replacement region".into(),
            ));
        }
        let leading = &source[region_start as usize..span.byte_start as usize];
        let trailing = &source[span.byte_end as usize..region_end as usize];
        if !leading.chars().all(char::is_whitespace) || !trailing.chars().all(char::is_whitespace) {
            return Err(CoreError::PatchInvariant(
                "preamble result contains non-whitespace outside its selection".into(),
            ));
        }
        Ok(snapshot.clone())
    }

    fn section_within(
        &self,
        document: &Document,
        region_start: u32,
        region_end: u32,
        expectation: &SectionResultExpectation,
    ) -> Result<TargetSnapshot, CoreError> {
        let matches = self
            .sections
            .iter()
            .filter(|snapshot| {
                let TargetAddress::Section { path } = &snapshot.address else {
                    return false;
                };
                path.len() == expectation.parent_path.len() + 1
                    && path.starts_with(&expectation.parent_path)
                    && snapshot.selection_span.is_some_and(|span| {
                        span.byte_start >= region_start && span.byte_end <= region_end
                    })
            })
            .collect::<Vec<_>>();
        let [snapshot] = matches.as_slice() else {
            return Err(CoreError::PatchInvariant(format!(
                "operation must produce exactly one declared section inside bytes {region_start}..{region_end}"
            )));
        };
        let span = snapshot.selection_span.ok_or_else(|| {
            CoreError::PatchInvariant("result section has no selection span".into())
        })?;
        let leading = &document.source()[region_start as usize..span.byte_start as usize];
        let trailing = &document.source()[span.byte_end as usize..region_end as usize];
        if !leading.chars().all(char::is_whitespace) || !trailing.chars().all(char::is_whitespace) {
            return Err(CoreError::PatchInvariant(
                "section result has non-whitespace outside its selected subtree".into(),
            ));
        }
        let read = document
            .resolve(&snapshot.address)?
            .read_section(document)?;
        if read.fragment
            != (crate::fragment::SectionFragment::Semantic {
                markdown: expectation.canonical.clone(),
            })
        {
            return Err(CoreError::PatchInvariant(
                "section result differs from its planned semantic subtree".into(),
            ));
        }
        Ok((*snapshot).clone())
    }
}

impl ReceiptDraft {
    fn finish(
        self,
        candidate: &Document,
        result: VerifiedResult,
    ) -> Result<PatchReceipt, CoreError> {
        let revision = candidate.revision().clone();
        let target = |result: VerifiedResult| match result {
            VerifiedResult::Target(snapshot) => Ok(snapshot),
            VerifiedResult::None => Err(CoreError::PatchInvariant(
                "receipt requires a verified result target".into(),
            )),
        };
        Ok(match self {
            Self::ReplaceBlock {
                before,
                disposition,
            } => {
                let outcome = match disposition {
                    MutationDisposition::NoChange | MutationDisposition::Replaced => {
                        let after = ReplaceBlockState::try_from(&target(result)?)?;
                        if disposition == MutationDisposition::NoChange {
                            ReplaceBlockOutcome::NoChange { before, after }
                        } else {
                            ReplaceBlockOutcome::Replaced { before, after }
                        }
                    }
                    MutationDisposition::Deleted => ReplaceBlockOutcome::Deleted {
                        before,
                        result_revision: revision,
                    },
                    other => return Err(impossible("replace_block", other)),
                };
                PatchReceipt::ReplaceBlock { outcome }
            }
            Self::DeleteBlock { before } => PatchReceipt::DeleteBlock {
                before,
                result_revision: revision,
            },
            Self::InsertBlock {
                target: insertion,
                disposition,
            } => {
                let outcome = match disposition {
                    MutationDisposition::Inserted => InsertBlockOutcome::Inserted {
                        target: insertion,
                        after: BlockIdentity::try_from(&target(result)?)?,
                    },
                    other => return Err(impossible("insert_block", other)),
                };
                PatchReceipt::InsertBlock { outcome }
            }
            Self::MoveBlock {
                before,
                destination_before,
                disposition,
            } => {
                let after = BlockIdentity::try_from(&target(result)?)?;
                let outcome = if disposition == MutationDisposition::NoChange {
                    MoveBlockOutcome::NoChange { before, after }
                } else if disposition == MutationDisposition::Replaced {
                    MoveBlockOutcome::Replaced { before, after }
                } else {
                    return Err(impossible("move_block", disposition));
                };
                PatchReceipt::MoveBlock {
                    destination_before,
                    outcome,
                }
            }
            Self::ReplaceSection {
                before,
                disposition,
            } => {
                let after = HeadingSectionIdentity::try_from(&target(result)?)?;
                let outcome = match disposition {
                    MutationDisposition::NoChange => {
                        ReplaceSectionOutcome::NoChange { before, after }
                    }
                    MutationDisposition::Replaced => {
                        ReplaceSectionOutcome::Replaced { before, after }
                    }
                    other => return Err(impossible("replace_section", other)),
                };
                PatchReceipt::ReplaceSection { outcome }
            }
            Self::InsertSection { parent_before } => PatchReceipt::InsertSection {
                outcome: InsertSectionOutcome::Inserted {
                    parent_before,
                    after: HeadingSectionIdentity::try_from(&target(result)?)?,
                },
            },
            Self::ReplacePreamble {
                before,
                disposition,
            } => {
                let after = PreambleIdentity::try_from(&target(result)?)?;
                let outcome = if disposition == MutationDisposition::NoChange {
                    ReplacePreambleOutcome::NoChange { before, after }
                } else if disposition == MutationDisposition::Replaced {
                    ReplacePreambleOutcome::Replaced { before, after }
                } else {
                    return Err(impossible("replace_preamble", disposition));
                };
                PatchReceipt::ReplacePreamble { outcome }
            }
            Self::DeleteSection {
                before,
                disposition,
            } => {
                let outcome = if disposition == MutationDisposition::NoChange {
                    DeleteSectionOutcome::NoChange {
                        before,
                        after: SectionIdentity::try_from(&target(result)?)?,
                    }
                } else if disposition == MutationDisposition::Deleted {
                    DeleteSectionOutcome::Deleted {
                        before,
                        result_revision: revision,
                    }
                } else {
                    return Err(impossible("delete_section", disposition));
                };
                PatchReceipt::DeleteSection { outcome }
            }
            Self::MoveSection {
                before,
                destination_before,
                disposition,
            } => {
                let after = HeadingSectionIdentity::try_from(&target(result)?)?;
                let outcome = if disposition == MutationDisposition::NoChange {
                    MoveSectionOutcome::NoChange { before, after }
                } else if disposition == MutationDisposition::Replaced {
                    MoveSectionOutcome::Replaced { before, after }
                } else {
                    return Err(impossible("move_section", disposition));
                };
                PatchReceipt::MoveSection {
                    destination_before,
                    outcome,
                }
            }
            Self::SetTaskStatus {
                before,
                disposition,
            } => {
                let after = TaskIdentity::try_from(&target(result)?)?;
                let outcome = if disposition == MutationDisposition::NoChange {
                    SetTaskOutcome::NoChange { before, after }
                } else if disposition == MutationDisposition::Replaced {
                    SetTaskOutcome::Replaced { before, after }
                } else {
                    return Err(impossible("set_task_status", disposition));
                };
                PatchReceipt::SetTaskStatus { outcome }
            }
            Self::SetFrontmatter {
                before,
                disposition,
            } => {
                let after = FrontmatterFieldIdentity::try_from(&target(result)?)?;
                let outcome = match disposition {
                    MutationDisposition::NoChange => {
                        SetFrontmatterOutcome::NoChange { before, after }
                    }
                    MutationDisposition::Inserted => {
                        SetFrontmatterOutcome::Inserted { before, after }
                    }
                    MutationDisposition::Replaced => {
                        SetFrontmatterOutcome::Replaced { before, after }
                    }
                    other => return Err(impossible("set_frontmatter", other)),
                };
                PatchReceipt::SetFrontmatter { outcome }
            }
            Self::DeleteFrontmatter {
                before,
                disposition,
            } => {
                let after = FrontmatterFieldIdentity::try_from(&target(result)?)?;
                let outcome = if disposition == MutationDisposition::NoChange {
                    DeleteFrontmatterOutcome::NoChange { before, after }
                } else if disposition == MutationDisposition::Deleted {
                    DeleteFrontmatterOutcome::Deleted { before, after }
                } else {
                    return Err(impossible("delete_frontmatter", disposition));
                };
                PatchReceipt::DeleteFrontmatter { outcome }
            }
            Self::ReplaceTableRow {
                before,
                disposition,
            } => {
                let after = TableRowIdentity::try_from(&target(result)?)?;
                let outcome = if disposition == MutationDisposition::NoChange {
                    ReplaceTableRowOutcome::NoChange { before, after }
                } else if disposition == MutationDisposition::Replaced {
                    ReplaceTableRowOutcome::Replaced { before, after }
                } else {
                    return Err(impossible("replace_table_row", disposition));
                };
                PatchReceipt::ReplaceTableRow { outcome }
            }
            Self::InsertTableRow { table_before } => PatchReceipt::InsertTableRow {
                table_before,
                after: TableRowIdentity::try_from(&target(result)?)?,
            },
            Self::DeleteTableRow { before } => PatchReceipt::DeleteTableRow {
                before,
                result_revision: revision,
            },
        })
    }
}

fn impossible(operation: &str, disposition: MutationDisposition) -> CoreError {
    CoreError::PatchInvariant(format!(
        "{operation} operation produced impossible {disposition:?} receipt disposition"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representative_complete_plans_are_reviewable() {
        let document = Document::parse_for_frontmatter_mutation(
            "---\na: old\nb: old\n---\n\none\n\ntwo\n\nthree\n",
        )
        .unwrap();
        let blocks = document
            .map()
            .unwrap()
            .into_iter()
            .filter(|snapshot| snapshot.kind == TargetKind::Block)
            .collect::<Vec<_>>();
        let field = |name: &str| {
            let resolved = document
                .resolve(&TargetAddress::FrontmatterField {
                    path: vec![name.into()],
                })
                .unwrap();
            FrontmatterPatchTarget::try_from(resolved.snapshot()).unwrap()
        };
        let operations = vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&blocks[0]).unwrap(),
                markdown: "changed".into(),
            },
            PatchOp::MoveBlock {
                source: ReplaceBlockTarget::try_from(&blocks[2]).unwrap(),
                destination: ReplaceBlockTarget::try_from(&blocks[1]).unwrap(),
                position: RelativePosition::Before,
            },
            PatchOp::SetFrontmatter {
                target: field("a"),
                value: serde_json::json!("new_a"),
            },
            PatchOp::SetFrontmatter {
                target: field("b"),
                value: serde_json::json!("new_b"),
            },
        ];
        let plans = plan_operations(&document, &operations).unwrap();
        assert_eq!(plans.len(), 3);
        assert!(plans.iter().all(|plan| {
            !plan.claims.is_empty()
                && !plan.results.is_empty()
                && plan.results.len() == plan.receipts.len()
        }));
        assert_eq!(plans[0].edits.len(), 1);
        assert_eq!(plans[0].results.len(), 1);
        assert_eq!(plans[0].receipts.len(), 1);
        assert_eq!(plans[1].edits.len(), 2);
        assert_eq!(plans[1].results.len(), 1);
        assert_eq!(plans[1].receipts.len(), 1);
        assert_eq!(plans[2].edits.len(), 1);
        assert_eq!(plans[2].results.len(), 2);
        assert_eq!(plans[2].receipts.len(), 2);
        assert_eq!(
            plans[2]
                .results
                .iter()
                .map(|result| result.operation)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        for plan in &plans {
            assert_edits_are_claimed(plan);
        }
        assert!(matches!(
            &plans[2].results[0].expectation,
            ResultExpectation::FrontmatterField {
                path,
                expected: FrontmatterExpectedValue::Value(value)
            } if path == &["a".to_string()] && value == &serde_json::json!("new_a")
        ));
        assert!(matches!(
            &plans[2].results[1].expectation,
            ResultExpectation::FrontmatterField {
                path,
                expected: FrontmatterExpectedValue::Value(value)
            } if path == &["b".to_string()] && value == &serde_json::json!("new_b")
        ));
        assert_eq!(
            plans[2]
                .receipts
                .iter()
                .map(|receipt| receipt.operation)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );

        println!("{}", render_plans(&plans));
    }

    #[test]
    fn semantic_section_plans_are_complete_at_construction() {
        let document = Document::parse("lead\n\n# Parent\n\n## Child\n\nbody\n").unwrap();
        let snapshots = document.map().unwrap();
        let by_heading = |heading: &str| {
            snapshots
                .iter()
                .find(|snapshot| {
                    matches!(&snapshot.summary, TargetSummary::Section { heading: value, .. } if value == heading)
                })
                .unwrap()
        };
        let preamble = snapshots
            .iter()
            .find(|snapshot| snapshot.kind == TargetKind::Preamble)
            .unwrap();
        let operations = vec![
            PatchOp::ReplaceSection {
                target: HeadingPatchTarget::try_from(by_heading("Child")).unwrap(),
                fragment: crate::fragment::SectionFragment::Semantic {
                    markdown: "# Renamed".into(),
                },
            },
            PatchOp::InsertSection {
                target: SectionInsertionTarget::try_from(by_heading("Parent")).unwrap(),
                fragment: crate::fragment::SectionFragment::Semantic {
                    markdown: "# Inserted".into(),
                },
            },
            PatchOp::ReplacePreamble {
                target: PreamblePatchTarget::try_from(preamble).unwrap(),
                markdown: "new lead".into(),
            },
        ];
        let plans = plan_operations(&document, &operations).unwrap();
        assert_eq!(plans.len(), 3);
        for (operation, plan) in plans.iter().enumerate() {
            assert_eq!(plan.edits.len(), 1);
            assert_eq!(plan.results.len(), 1);
            assert_eq!(plan.receipts.len(), 1);
            assert_eq!(plan.results[0].operation, operation);
            assert_eq!(plan.receipts[0].operation, operation);
            assert_edits_are_claimed(plan);
        }
        assert!(matches!(
            plans[0].results[0].expectation,
            ResultExpectation::Section(_)
        ));
        assert!(matches!(
            plans[1].results[0].expectation,
            ResultExpectation::Section(_)
        ));
        assert!(matches!(
            plans[2].results[0].expectation,
            ResultExpectation::Preamble { .. }
        ));

        println!("{}", render_plans(&plans));
    }

    fn assert_edits_are_claimed(plan: &PlannedMutation) {
        for edit in &plan.edits {
            assert!(plan.claims.iter().any(|claim| {
                matches!(
                    claim.region,
                    ConflictRegion::Source { start, end }
                        if start == edit.start && end == edit.end
                )
            }));
        }
    }

    fn render_plans(plans: &[PlannedMutation]) -> String {
        let mut rendered = String::new();
        for (index, plan) in plans.iter().enumerate() {
            rendered.push_str(&format!(
                "plan {index}: claims={} edits={} results={} receipts={}\n",
                plan.claims.len(),
                plan.edits.len(),
                plan.results.len(),
                plan.receipts.len()
            ));
            for claim in &plan.claims {
                rendered.push_str(&format!(
                    "  claim operation={} {}\n",
                    claim.operation,
                    claim_name(&claim.region)
                ));
            }
            for (edit, value) in plan.edits.iter().enumerate() {
                rendered.push_str(&format!(
                    "  edit {edit}: {}..{} replacement-bytes={}\n",
                    value.start,
                    value.end,
                    value.replacement.len()
                ));
            }
            for result in &plan.results {
                rendered.push_str(&format!(
                    "  result operation={} {}\n",
                    result.operation,
                    result_name(&result.expectation)
                ));
            }
            for receipt in &plan.receipts {
                rendered.push_str(&format!(
                    "  receipt operation={} {}\n",
                    receipt.operation,
                    receipt_name(&receipt.draft)
                ));
            }
        }
        rendered
    }

    fn claim_name(claim: &ConflictRegion) -> &'static str {
        match claim {
            ConflictRegion::Source { .. } => "source",
            ConflictRegion::FrontmatterPath(_) => "frontmatter_path",
        }
    }

    fn result_name(result: &ResultExpectation) -> &'static str {
        match result {
            ResultExpectation::None => "none",
            ResultExpectation::Target { .. } => "target",
            ResultExpectation::TargetWithBlockClosure { .. } => "target_with_block_closure",
            ResultExpectation::ParserBlockClosure { .. } => "parser_block_closure",
            ResultExpectation::BlockFragment { .. } => "block_fragment",
            ResultExpectation::Preamble { .. } => "preamble",
            ResultExpectation::Section(_) => "section",
            ResultExpectation::FrontmatterField { .. } => "frontmatter_field",
        }
    }

    fn receipt_name(receipt: &ReceiptDraft) -> &'static str {
        match receipt {
            ReceiptDraft::ReplaceBlock { .. } => "replace_block",
            ReceiptDraft::DeleteBlock { .. } => "delete_block",
            ReceiptDraft::InsertBlock { .. } => "insert_block",
            ReceiptDraft::MoveBlock { .. } => "move_block",
            ReceiptDraft::ReplaceSection { .. } => "replace_section",
            ReceiptDraft::InsertSection { .. } => "insert_section",
            ReceiptDraft::ReplacePreamble { .. } => "replace_preamble",
            ReceiptDraft::DeleteSection { .. } => "delete_section",
            ReceiptDraft::MoveSection { .. } => "move_section",
            ReceiptDraft::SetTaskStatus { .. } => "set_task_status",
            ReceiptDraft::SetFrontmatter { .. } => "set_frontmatter",
            ReceiptDraft::DeleteFrontmatter { .. } => "delete_frontmatter",
            ReceiptDraft::ReplaceTableRow { .. } => "replace_table_row",
            ReceiptDraft::InsertTableRow { .. } => "insert_table_row",
            ReceiptDraft::DeleteTableRow { .. } => "delete_table_row",
        }
    }
}
