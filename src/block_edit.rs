use crate::core_error::{CoreError, EtagTarget};
use crate::document::Document;
use crate::edit::{
    normalize_line_endings, replacement_span_after, strip_one_trailing_newline, EditOutcome,
    EditPreservation,
};
use crate::fingerprint::{TargetEtag, TargetEtagGuard};
use crate::model::{
    BlockMoveMode, InsertLocation, LineEndingStyle, MutationDisposition, SourceSpan,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardRole {
    Source,
    Destination,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockEditTarget {
    Block {
        block_index: u32,
        span: SourceSpan,
    },
    Insertion {
        location: InsertLocation,
        anchor_span: Option<SourceSpan>,
    },
    Move {
        source_index: u32,
        source_span: SourceSpan,
        destination_index: u32,
        destination_span: SourceSpan,
        destination_mode: BlockMoveMode,
    },
}

pub struct PreparedBlockReplace<'a> {
    document: &'a Document,
    block_index: u32,
    span: SourceSpan,
    guarded: bool,
}

pub struct PreparedBlockInsert<'a> {
    document: &'a Document,
    location: InsertLocation,
    insert_byte: usize,
    anchor_span: Option<SourceSpan>,
    guarded: bool,
}

pub fn prepare_replace<'a>(
    document: &'a Document,
    block_index: u32,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<PreparedBlockReplace<'a>, CoreError> {
    let block = resolve_block(document, block_index)?;
    verify_block_guard(document, block_index, block.span, expect_etag)?;
    Ok(PreparedBlockReplace {
        document,
        block_index,
        span: block.span,
        guarded: expect_etag.is_some(),
    })
}

impl PreparedBlockReplace<'_> {
    pub fn apply(self, content: impl Into<String>) -> EditOutcome<BlockEditTarget> {
        let original = self.document.slice_unchecked(&self.span);
        let normalized = normalize_line_endings(&content.into(), self.document.line_ending_style());
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
        let content = format!(
            "{}{}{}",
            &self.document.source()[..self.span.byte_start as usize],
            replacement,
            &self.document.source()[self.span.byte_end as usize..]
        );
        let after = match disposition {
            MutationDisposition::NoChange => Some(self.span),
            MutationDisposition::Deleted => None,
            MutationDisposition::Replaced => Some(replacement_span_after(self.span, &replacement)),
            MutationDisposition::Inserted => unreachable!(),
        };
        outcome(
            self.document,
            BlockEditTarget::Block {
                block_index: self.block_index,
                span: self.span,
            },
            disposition,
            self.guarded,
            Some(self.span),
            after,
            content,
        )
    }
}

pub fn prepare_insert<'a>(
    document: &'a Document,
    location: InsertLocation,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<PreparedBlockInsert<'a>, CoreError> {
    let (insert_byte, anchor_span) = resolve_insert_location(document, location)?;
    match (expect_etag, location) {
        (Some(_), InsertLocation::Start | InsertLocation::End) => {
            return Err(CoreError::InvalidSelector(
                "expected etag requires a before or after anchor".into(),
            ));
        }
        (Some(expected), InsertLocation::Before(index) | InsertLocation::After(index)) => {
            verify_block_guard(
                document,
                index,
                anchor_span.expect("anchor span"),
                Some(expected),
            )?;
        }
        _ => {}
    }
    Ok(PreparedBlockInsert {
        document,
        location,
        insert_byte,
        anchor_span,
        guarded: expect_etag.is_some(),
    })
}

impl PreparedBlockInsert<'_> {
    pub fn apply(
        self,
        content: impl Into<String>,
    ) -> Result<EditOutcome<BlockEditTarget>, CoreError> {
        let content = normalize_line_endings(&content.into(), self.document.line_ending_style());
        let target = BlockEditTarget::Insertion {
            location: self.location,
            anchor_span: self.anchor_span,
        };
        if content.is_empty() {
            return Ok(outcome(
                self.document,
                target,
                MutationDisposition::NoChange,
                self.guarded,
                None,
                None,
                self.document.source().to_string(),
            ));
        }
        let before = &self.document.source()[..self.insert_byte];
        let after = &self.document.source()[self.insert_byte..];
        let needs_leading = !before.is_empty() && !before.ends_with('\n');
        let needs_separator =
            !before.is_empty() && !before.ends_with("\n\n") && !before.ends_with("\r\n\r\n");
        let needs_trailing =
            !after.is_empty() && !after.starts_with('\n') && !after.starts_with("\r\n");
        let newline = if self.document.line_ending_style() == LineEndingStyle::Crlf {
            "\r\n"
        } else {
            "\n"
        };
        let mut output = String::with_capacity(self.document.source().len() + content.len() + 4);
        output.push_str(before);
        if needs_leading || needs_separator {
            output.push_str(newline);
        }
        let payload_start = output.len();
        output.push_str(&content);
        let payload_end = output.len();
        if needs_trailing {
            output.push_str(newline);
            output.push_str(newline);
        } else if !after.is_empty() && !content.ends_with('\n') {
            output.push_str(newline);
        }
        output.push_str(after);
        let reparsed = Document::parse(output.clone())?;
        let after_span = reparsed.span_for_byte_range(payload_start as u32, payload_end as u32);
        Ok(outcome(
            self.document,
            target,
            MutationDisposition::Inserted,
            self.guarded,
            None,
            Some(after_span),
            output,
        ))
    }
}

pub fn delete(
    document: &Document,
    block_index: u32,
    expect_etag: Option<&TargetEtagGuard>,
) -> Result<EditOutcome<BlockEditTarget>, CoreError> {
    let prepared = prepare_replace(document, block_index, expect_etag)?;
    let content = format!(
        "{}{}",
        &document.source()[..prepared.span.byte_start as usize],
        &document.source()[prepared.span.byte_end as usize..]
    );
    Ok(outcome(
        document,
        BlockEditTarget::Block {
            block_index,
            span: prepared.span,
        },
        MutationDisposition::Deleted,
        prepared.guarded,
        Some(prepared.span),
        None,
        content,
    ))
}

pub fn move_block(
    document: &Document,
    source_index: u32,
    destination_index: u32,
    destination_mode: BlockMoveMode,
    expect_source_etag: Option<&TargetEtagGuard>,
    expect_destination_etag: Option<&TargetEtagGuard>,
) -> Result<EditOutcome<BlockEditTarget>, CoreError> {
    let source = resolve_block(document, source_index)?;
    let destination = resolve_block(document, destination_index)?;
    if source_index == destination_index {
        return Err(CoreError::InvalidSelector(
            "source and destination block indices must be different".into(),
        ));
    }
    verify_move_guard(
        document,
        source_index,
        source.span,
        GuardRole::Source,
        expect_source_etag,
    )?;
    verify_move_guard(
        document,
        destination_index,
        destination.span,
        GuardRole::Destination,
        expect_destination_etag,
    )?;
    let order = block_order(
        document.blocks().len(),
        source_index,
        destination_index,
        destination_mode,
    );
    let content = reconstruct(document, &order);
    let disposition = if content == document.source() {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let after = verify_structural_closure(document, &content, &order, source_index)?;
    Ok(outcome(
        document,
        BlockEditTarget::Move {
            source_index,
            source_span: source.span,
            destination_index,
            destination_span: destination.span,
            destination_mode,
        },
        disposition,
        expect_source_etag.is_some() || expect_destination_etag.is_some(),
        Some(source.span),
        Some(if disposition == MutationDisposition::NoChange {
            source.span
        } else {
            after
        }),
        content,
    ))
}

fn resolve_block(document: &Document, index: u32) -> Result<&crate::parser::BlockInfo, CoreError> {
    document
        .blocks()
        .get(index as usize)
        .ok_or(CoreError::BlockIndexOutOfRange {
            index,
            block_count: document.blocks().len() as u32,
        })
}

fn verify_block_guard(
    document: &Document,
    index: u32,
    span: SourceSpan,
    expected: Option<&TargetEtagGuard>,
) -> Result<(), CoreError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = TargetEtag::for_bytes(document.slice_unchecked(&span).as_bytes());
    if expected.as_str() != actual.as_str() {
        return Err(CoreError::TargetEtagMismatch {
            target: EtagTarget::Block(index),
            expected: expected.to_string(),
            actual: actual.to_string(),
        });
    }
    verify_unique(document, expected, None, index)
}

fn verify_move_guard(
    document: &Document,
    index: u32,
    span: SourceSpan,
    role: GuardRole,
    expected: Option<&TargetEtagGuard>,
) -> Result<(), CoreError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = TargetEtag::for_bytes(document.slice_unchecked(&span).as_bytes());
    if expected.as_str() != actual.as_str() {
        return Err(CoreError::BlockMoveEtagMismatch {
            role,
            index,
            expected: expected.to_string(),
            actual: actual.to_string(),
        });
    }
    verify_unique(document, expected, Some(role), index)
}

fn verify_unique(
    document: &Document,
    expected: &TargetEtagGuard,
    role: Option<GuardRole>,
    index: u32,
) -> Result<(), CoreError> {
    let count = document
        .blocks()
        .iter()
        .filter(|block| {
            TargetEtag::for_bytes(document.slice_unchecked(&block.span).as_bytes()).as_str()
                == expected.as_str()
        })
        .count();
    if count > 1 {
        if let Some(role) = role {
            Err(CoreError::BlockMoveEtagAmbiguous {
                role,
                index,
                expected: expected.to_string(),
                count,
            })
        } else {
            Err(CoreError::TargetEtagAmbiguous {
                target_kind: "block",
                expected: expected.to_string(),
                count,
            })
        }
    } else {
        Ok(())
    }
}

fn resolve_insert_location(
    document: &Document,
    location: InsertLocation,
) -> Result<(usize, Option<SourceSpan>), CoreError> {
    match location {
        InsertLocation::Before(index) => {
            let block = resolve_block(document, index)?;
            Ok((block.span.byte_start as usize, Some(block.span)))
        }
        InsertLocation::After(index) => {
            let block = resolve_block(document, index)?;
            Ok((block.span.byte_end as usize, Some(block.span)))
        }
        InsertLocation::Start => Ok((
            document
                .frontmatter()
                .map(|frontmatter| {
                    document
                        .blocks()
                        .first()
                        .map(|block| block.span.byte_start as usize)
                        .unwrap_or(frontmatter.span.byte_end as usize)
                })
                .unwrap_or(0),
            None,
        )),
        InsertLocation::End => Ok((document.source().len(), None)),
    }
}

fn block_order(count: usize, source: u32, destination: u32, mode: BlockMoveMode) -> Vec<u32> {
    let mut order = (0..count as u32).collect::<Vec<_>>();
    let moved = order.remove(source as usize);
    let destination_position = order
        .iter()
        .position(|index| *index == destination)
        .expect("resolved destination remains after source removal");
    let insertion = match mode {
        BlockMoveMode::Before => destination_position,
        BlockMoveMode::After => destination_position + 1,
    };
    order.insert(insertion, moved);
    order
}

fn reconstruct(document: &Document, order: &[u32]) -> String {
    let prefix_end = document
        .blocks()
        .first()
        .map(|block| block.span.byte_start as usize)
        .unwrap_or(document.source().len());
    let mut output = String::with_capacity(document.source().len());
    output.push_str(&document.source()[..prefix_end]);
    let gaps = document
        .blocks()
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let end = document
                .blocks()
                .get(index + 1)
                .map(|next| next.span.byte_start as usize)
                .unwrap_or(document.source().len());
            &document.source()[block.span.byte_end as usize..end]
        })
        .collect::<Vec<_>>();
    for (slot, original) in order.iter().enumerate() {
        output.push_str(document.slice_unchecked(&document.blocks()[*original as usize].span));
        output.push_str(gaps[slot]);
    }
    output
}

fn verify_structural_closure(
    document: &Document,
    output: &str,
    order: &[u32],
    source_index: u32,
) -> Result<SourceSpan, CoreError> {
    let reparsed = Document::parse(output)?;
    if reparsed.blocks().len() != document.blocks().len() {
        return Err(structural_closure_error());
    }
    for (position, original_index) in order.iter().enumerate() {
        let expected = &document.blocks()[*original_index as usize];
        let actual = &reparsed.blocks()[position];
        if actual.kind != expected.kind
            || reparsed.slice_unchecked(&actual.span) != document.slice_unchecked(&expected.span)
        {
            return Err(structural_closure_error());
        }
    }
    let moved = order
        .iter()
        .position(|index| *index == source_index)
        .expect("source remains in permutation");
    Ok(reparsed.blocks()[moved].span)
}

fn structural_closure_error() -> CoreError {
    CoreError::InvalidSelector(
        "move-block would change the parsed top-level block sequence; choose a destination that preserves block boundaries".into(),
    )
}

fn outcome(
    document: &Document,
    target: BlockEditTarget,
    disposition: MutationDisposition,
    guarded: bool,
    before: Option<SourceSpan>,
    after: Option<SourceSpan>,
    content: String,
) -> EditOutcome<BlockEditTarget> {
    EditOutcome {
        base_revision: document.revision().clone(),
        target,
        disposition,
        guarded,
        line_endings: document.line_ending_style(),
        preservation: EditPreservation {
            preserves_non_target_bytes: true,
            target_span_before: before,
            target_span_after: after,
        },
        content,
    }
}
