use crate::cli::{DeleteBlockArgs, InsertBlockArgs, ReplaceBlockArgs};
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run_replace_block(args: &ReplaceBlockArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let block = doc
        .blocks
        .get(args.index as usize)
        .ok_or_else(|| CommandError::block_out_of_range(args.index, doc.blocks.len() as u32))?;

    let block_span = block.span;

    verify_etag(
        args.expect_etag.as_deref(),
        args.index,
        doc.slice(&block_span),
    )?;

    let replacement = output::read_content(args.from.as_deref())?;

    let line_endings = doc.line_ending_style();
    let replacement = normalize_line_endings(&replacement, &line_endings);

    let original = doc.slice(&block_span);
    // Bug B (editor-bench finding #5): `cat <<'EOF' > f`, editors, and `echo`
    // all append a trailing newline the agent never intended as block content.
    // Most block spans exclude their trailing newline, but some include a
    // significant one (notably blank-line-terminated indented code via the
    // parser fixup), so gate on the ACTUAL span content rather than block kind:
    // strip one trailing line-ending only when the original slice does not
    // already end in one. Removes the spurious blank line and makes content
    // round-trips register as NoChange, without truncating a span whose
    // trailing newline is real.
    //
    // Known limits (acceptable for the dominant `--from` case): content with
    // >1 trailing newline still leaves an artifact, and you cannot use
    // replace-block to *add* a final newline to a last block that lacks one.
    let replacement = if original.ends_with('\n') {
        replacement
    } else {
        strip_one_trailing_newline(replacement)
    };

    let disposition = if replacement == original {
        MutationDisposition::NoChange
    } else if replacement.is_empty() {
        MutationDisposition::Deleted
    } else {
        MutationDisposition::Replaced
    };
    let changed = disposition != MutationDisposition::NoChange;

    let before = &doc.source[..block_span.byte_start as usize];
    let after = &doc.source[block_span.byte_end as usize..];
    let output_doc = format!("{}{}{}", before, replacement, after);

    let target = MutationTargetRef::Block(BlockTargetRef {
        kind: MutationTargetKind::Block,
        block_index: args.index,
        span: block_span,
    });

    emit_mutation(
        args.in_place,
        json,
        &args.file,
        MutationCommandKind::ReplaceBlock,
        target,
        disposition,
        changed,
        line_endings,
        Some(block_span),
        &replacement,
        &output_doc,
    )
}

pub fn run_insert_block(args: &InsertBlockArgs, json: bool) -> Result<(), CommandError> {
    // Validate exactly one location flag
    let location = parse_insert_location(args)?;

    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    verify_insert_etag(args.expect_etag.as_deref(), &location, &doc)?;

    let content = output::read_content(args.from.as_deref())?;

    let line_endings = doc.line_ending_style();
    let content = normalize_line_endings(&content, &line_endings);

    let (insert_byte, anchor_span) = resolve_insert_location(&doc, &location)?;

    let before = &doc.source[..insert_byte];
    let after = &doc.source[insert_byte..];

    // Ensure proper block separation
    let needs_leading_newline =
        !before.is_empty() && !before.ends_with('\n') && !before.ends_with("\r\n");
    let needs_separator = !before.is_empty()
        && !before.ends_with("\n\n")
        && !before.ends_with("\r\n\r\n")
        && !content.is_empty();
    let needs_trailing_separator = !after.is_empty()
        && !after.starts_with('\n')
        && !after.starts_with("\r\n")
        && !content.is_empty();

    let nl = match line_endings {
        LineEndingStyle::Crlf => "\r\n",
        _ => "\n",
    };

    let mut output_doc = String::with_capacity(doc.source.len() + content.len() + 4);
    output_doc.push_str(before);
    if needs_leading_newline {
        output_doc.push_str(nl);
    }
    if needs_separator && !needs_leading_newline {
        output_doc.push_str(nl);
    }
    output_doc.push_str(&content);
    if needs_trailing_separator {
        output_doc.push_str(nl);
        output_doc.push_str(nl);
    } else if !after.is_empty() && !content.is_empty() && !content.ends_with('\n') {
        output_doc.push_str(nl);
    }
    output_doc.push_str(after);

    let target = MutationTargetRef::Insert(InsertTargetRef {
        kind: MutationTargetKind::InsertLocation,
        location,
        anchor_span,
    });

    let (disposition, changed) = if content.is_empty() {
        (MutationDisposition::NoChange, false)
    } else {
        (MutationDisposition::Inserted, true)
    };

    emit_mutation(
        args.in_place,
        json,
        &args.file,
        MutationCommandKind::InsertBlock,
        target,
        disposition,
        changed,
        line_endings,
        None,
        &content,
        &output_doc,
    )
}

pub fn run_delete_block(args: &DeleteBlockArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let block = doc
        .blocks
        .get(args.index as usize)
        .ok_or_else(|| CommandError::block_out_of_range(args.index, doc.blocks.len() as u32))?;

    let block_span = block.span;

    verify_etag(
        args.expect_etag.as_deref(),
        args.index,
        doc.slice(&block_span),
    )?;

    let line_endings = doc.line_ending_style();

    // Delete exactly the block's byte span
    let before = &doc.source[..block_span.byte_start as usize];
    let after = &doc.source[block_span.byte_end as usize..];
    let output_doc = format!("{}{}", before, after);

    let target = MutationTargetRef::Block(BlockTargetRef {
        kind: MutationTargetKind::Block,
        block_index: args.index,
        span: block_span,
    });

    emit_mutation(
        args.in_place,
        json,
        &args.file,
        MutationCommandKind::DeleteBlock,
        target,
        MutationDisposition::Deleted,
        true,
        line_endings,
        Some(block_span),
        "",
        &output_doc,
    )
}

fn parse_insert_location(args: &InsertBlockArgs) -> Result<InsertLocation, CommandError> {
    let mut count = 0;
    if args.before.is_some() {
        count += 1;
    }
    if args.after.is_some() {
        count += 1;
    }
    if args.at_start {
        count += 1;
    }
    if args.at_end {
        count += 1;
    }

    if count == 0 {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "exactly one of --before, --after, --at-start, --at-end must be provided",
        ));
    }
    if count > 1 {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "exactly one of --before, --after, --at-start, --at-end must be provided",
        ));
    }

    if let Some(idx) = args.before {
        Ok(InsertLocation::Before(idx))
    } else if let Some(idx) = args.after {
        Ok(InsertLocation::After(idx))
    } else if args.at_start {
        Ok(InsertLocation::Start)
    } else {
        Ok(InsertLocation::End)
    }
}

fn resolve_insert_location(
    doc: &ParsedDocument,
    location: &InsertLocation,
) -> Result<(usize, Option<SourceSpan>), CommandError> {
    match location {
        InsertLocation::Before(idx) => {
            let block = doc
                .blocks
                .get(*idx as usize)
                .ok_or_else(|| CommandError::block_out_of_range(*idx, doc.blocks.len() as u32))?;
            Ok((block.span.byte_start as usize, Some(block.span)))
        }
        InsertLocation::After(idx) => {
            let block = doc
                .blocks
                .get(*idx as usize)
                .ok_or_else(|| CommandError::block_out_of_range(*idx, doc.blocks.len() as u32))?;
            Ok((block.span.byte_end as usize, Some(block.span)))
        }
        InsertLocation::Start => {
            // After frontmatter, before first block
            let start = doc
                .frontmatter
                .as_ref()
                .map(|fm| {
                    // Find the byte after the frontmatter raw content
                    // The raw includes trailing newlines, but sourcepos doesn't
                    // Use the first block's start, or end of frontmatter span
                    doc.blocks
                        .first()
                        .map(|b| b.span.byte_start as usize)
                        .unwrap_or(fm.span.byte_end as usize)
                })
                .unwrap_or(0);
            Ok((start, None))
        }
        InsertLocation::End => Ok((doc.source.len(), None)),
    }
}

fn normalize_line_endings(content: &str, style: &LineEndingStyle) -> String {
    crate::output::normalize_line_endings(content, style)
}

/// Verify the target block's current content fingerprint matches the agent's
/// `--expect-etag`. Fails-closed (Conflict) on mismatch, so a stale index never
/// silently mutates the wrong block. No-op when no etag was supplied.
fn verify_etag(expect: Option<&str>, index: u32, current: &str) -> Result<(), CommandError> {
    verify_expected_etag(expect, current, |expected, actual| {
        CommandError::etag_mismatch(index, expected, actual)
    })
}

pub(crate) fn verify_expected_etag<F>(
    expect: Option<&str>,
    current: &str,
    mismatch: F,
) -> Result<(), CommandError>
where
    F: FnOnce(&str, &str) -> CommandError,
{
    if let Some(expected) = expect {
        let actual = output::content_etag(current.as_bytes());
        if expected != actual {
            return Err(mismatch(expected, &actual));
        }
    }
    Ok(())
}

/// For insert-block, `--expect-etag` verifies the anchor block (--before/--after).
/// --at-start/--at-end have no anchor block, so a supplied etag is a usage error
/// rather than a silently-ignored guard.
fn verify_insert_etag(
    expect: Option<&str>,
    location: &InsertLocation,
    doc: &ParsedDocument,
) -> Result<(), CommandError> {
    let Some(expected) = expect else {
        return Ok(());
    };
    let anchor = match location {
        InsertLocation::Before(idx) | InsertLocation::After(idx) => *idx,
        InsertLocation::Start | InsertLocation::End => {
            return Err(CommandError::new(
                DiagnosticCode::InvalidSelector,
                "--expect-etag requires --before or --after (--at-start/--at-end have no anchor block)",
            ));
        }
    };
    let block = doc
        .blocks
        .get(anchor as usize)
        .ok_or_else(|| CommandError::block_out_of_range(anchor, doc.blocks.len() as u32))?;
    verify_etag(Some(expected), anchor, doc.slice(&block.span))
}

/// Strip at most one trailing line-ending from `--from`/stdin replacement
/// content. Block spans exclude the trailing newline, so the trailing `\n` that
/// `cat <<'EOF' > f`, editors, and `echo` universally append would otherwise
/// inject a spurious blank line and defeat the no-op check on round-trips.
fn strip_one_trailing_newline(mut s: String) -> String {
    if s.ends_with("\r\n") {
        s.truncate(s.len() - 2);
    } else if s.ends_with('\n') {
        s.truncate(s.len() - 1);
    }
    s
}

/// Compute target_span_after based on disposition and replacement content.
fn compute_span_after(
    disposition: MutationDisposition,
    span_before: Option<SourceSpan>,
    replacement: &str,
) -> Option<SourceSpan> {
    match disposition {
        MutationDisposition::Deleted => None,
        MutationDisposition::NoChange => span_before,
        MutationDisposition::Replaced => {
            if let Some(before) = span_before {
                let byte_start = before.byte_start;
                let byte_end = byte_start + replacement.len() as u32;
                // Count lines in replacement to get line_end
                let line_count = replacement.matches('\n').count() as u32;
                Some(SourceSpan {
                    line_start: before.line_start,
                    line_end: before.line_start + line_count,
                    byte_start,
                    byte_end,
                })
            } else {
                None
            }
        }
        MutationDisposition::Inserted => {
            // For insert, span_before is None; compute from insertion point in output
            // Find where the content was inserted
            None // Simplified — insert span_after would require knowing the insert byte
        }
    }
}

fn emit_mutation(
    in_place: bool,
    json: bool,
    file: &std::path::Path,
    command: MutationCommandKind,
    target: MutationTargetRef,
    disposition: MutationDisposition,
    changed: bool,
    line_endings: LineEndingStyle,
    span_before: Option<SourceSpan>,
    replacement: &str,
    output_doc: &str,
) -> Result<(), CommandError> {
    let span_after = compute_span_after(disposition, span_before, replacement);

    let build_result = |content: Option<String>| MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string_lossy().to_string(),
        command,
        target: target.clone(),
        disposition,
        changed,
        line_endings,
        invariant: SourcePreservationInvariant {
            preserves_non_target_bytes: true,
            target_span_before: span_before,
            target_span_after: span_after,
        },
        content,
    };

    if in_place {
        if changed {
            std::fs::write(file, output_doc)?;
        }
        if json {
            output::write_json(&build_result(None))?;
        }
    } else if json {
        output::write_json(&build_result(Some(output_doc.to_string())))?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}
