use crate::cli::MoveSectionArgs;
use crate::commands::section::{build_selector, find_section};
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use crate::parser::{HeadingSourceKind, ParsedDocument};

pub fn run_move_section(args: &MoveSectionArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    // --- Resolve source + destination selectors ---
    let source_selector =
        build_selector(&args.source, args.source_occurrence, args.ignore_case)?;
    let source_section = find_section(&doc, &source_selector)?;

    let (dest_text, dest_mode) = pick_destination(args)?;
    let dest_selector = build_selector(dest_text, args.dest_occurrence, args.ignore_case)?;
    let dest_section = find_section(&doc, &dest_selector)?;

    let src_span = source_section.span;
    let dest_span = dest_section.span;

    // --- Line-start alignment ---
    //
    // Comrak reports sourcepos for ATX headings starting at the `#`, not at
    // the start of the line, so 0-3 leading spaces are excluded from the
    // block span. Without this fix-up, indented headings would orphan their
    // leading spaces at the source location and lose them at the destination.
    let bytes = doc.source.as_bytes();
    let src_byte_start = line_start(bytes, src_span.byte_start as usize) as u32;
    let src_byte_end = line_start(bytes, src_span.byte_end as usize) as u32;

    // --- Compute insert byte (raw — before src removal) and new top level ---
    let insert_byte_raw = match dest_mode {
        InsertMode::AfterSibling | InsertMode::IntoAsChild => {
            line_start(bytes, dest_span.byte_end as usize) as u32
        }
        InsertMode::BeforeSibling => line_start(bytes, dest_span.byte_start as usize) as u32,
    };

    let dest_heading_level = dest_section
        .heading
        .as_ref()
        .map(|h| h.level)
        .ok_or_else(|| {
            CommandError::new(
                DiagnosticCode::InvalidSelector,
                "destination must be a heading section, not the preamble",
            )
        })?;

    let new_top_level = match dest_mode {
        InsertMode::AfterSibling | InsertMode::BeforeSibling => dest_heading_level,
        InsertMode::IntoAsChild => dest_heading_level + 1,
    };

    let source_top_level = source_section
        .heading
        .as_ref()
        .map(|h| h.level)
        .ok_or_else(|| {
            CommandError::new(
                DiagnosticCode::InvalidSelector,
                "source must be a heading section, not the preamble",
            )
        })?;

    // --- Validate ancestor / descendant containment ---
    let dest_inside_src = dest_span.byte_start >= src_byte_start
        && dest_span.byte_end <= src_byte_end;
    let src_inside_dest = src_byte_start >= dest_span.byte_start
        && src_byte_end <= dest_span.byte_end;

    if dest_inside_src {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "cannot move-section: destination is inside source",
        ));
    }
    if src_inside_dest
        && matches!(
            dest_mode,
            InsertMode::AfterSibling | InsertMode::IntoAsChild
        )
    {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "cannot move-section: destination contains source; insert position is ambiguous",
        ));
    }

    // --- Compute level delta + setext check + clamp ---
    let keep_level = args.keep_level;
    let level_delta_raw: i32 = if keep_level {
        0
    } else {
        new_top_level as i32 - source_top_level as i32
    };

    if level_delta_raw != 0 {
        // Reject any setext heading in the section — top OR descendant. The
        // top heading wouldn't survive the re-level (its underline stays put);
        // a setext descendant would silently keep its old level while ATX
        // siblings shift around it, breaking the hierarchy.
        for &idx in &source_section.block_indices {
            let block = &doc.blocks[idx as usize];
            if let Some(h) = &block.heading {
                if h.kind == HeadingSourceKind::Setext {
                    let line = block.span.line_start;
                    return Err(CommandError::new(
                        DiagnosticCode::InvalidSelector,
                        format!(
                            "setext heading {:?} (line {}) cannot be re-leveled; \
                             convert to ATX (## {}) first or use --keep-level",
                            h.text, line, h.text
                        ),
                    ));
                }
            }
        }

        // Clamp: ensure no descendant lands outside [1, 6].
        for &idx in &source_section.block_indices {
            let block = &doc.blocks[idx as usize];
            if let Some(h) = &block.heading {
                let new_level = h.level as i32 + level_delta_raw;
                if !(1..=6).contains(&new_level) {
                    return Err(CommandError::new(
                        DiagnosticCode::InvalidSelector,
                        format!(
                            "cannot move-section: descendant {:?} would land at heading level {} \
                             (max is 6); reduce destination depth or use --keep-level",
                            h.text, new_level
                        ),
                    ));
                }
            }
        }
    }

    let level_delta: i8 = level_delta_raw as i8;

    // --- Splice ---
    let line_endings = doc.line_ending_style();
    let separator: &str = match line_endings {
        LineEndingStyle::Crlf => "\r\n",
        _ => "\n",
    };
    let mut moved =
        doc.source[src_byte_start as usize..src_byte_end as usize].to_string();

    if level_delta != 0 {
        moved = rewrite_atx_levels(moved, &source_section, &doc, src_byte_start, level_delta)?;
    }

    // Trailing-edge fix-up: if the moved bytes don't end with a line break
    // and they'll be followed by content, synthesize one so the following
    // block doesn't get glued onto the moved heading text. Use the document's
    // detected line-ending style so a CRLF doc stays CRLF.
    let src_ends_with_newline =
        src_byte_end > 0 && doc.source.as_bytes()[src_byte_end as usize - 1] == b'\n';
    let dest_followed_by_content = (insert_byte_raw as usize) < doc.source.len();
    if !src_ends_with_newline && dest_followed_by_content && !moved.ends_with('\n') {
        moved.push_str(separator);
    }

    // Leading-edge fix-up: if we're inserting at EOF (or after a position
    // whose preceding byte isn't `\n`), the moved bytes will glue onto the
    // previous line — e.g. moving A `--after` a final no-newline section B
    // would produce "...b## A...". Prepend a separator in that case.
    let needs_leading_separator = insert_byte_raw > 0
        && doc.source.as_bytes()[insert_byte_raw as usize - 1] != b'\n'
        && !moved.starts_with('\n');

    let output_doc = if insert_byte_raw <= src_byte_start {
        // dest comes before src in the document
        let mut out = String::with_capacity(doc.source.len() + moved.len() + separator.len());
        out.push_str(&doc.source[..insert_byte_raw as usize]);
        if needs_leading_separator {
            out.push_str(separator);
        }
        out.push_str(&moved);
        out.push_str(&doc.source[insert_byte_raw as usize..src_byte_start as usize]);
        out.push_str(&doc.source[src_byte_end as usize..]);
        out
    } else {
        // dest comes after src in the document
        let mut out = String::with_capacity(doc.source.len() + moved.len() + separator.len());
        out.push_str(&doc.source[..src_byte_start as usize]);
        out.push_str(&doc.source[src_byte_end as usize..insert_byte_raw as usize]);
        if needs_leading_separator {
            out.push_str(separator);
        }
        out.push_str(&moved);
        out.push_str(&doc.source[insert_byte_raw as usize..]);
        out
    };

    let changed = output_doc != doc.source;
    let disposition = if changed {
        MutationDisposition::Replaced
    } else {
        MutationDisposition::NoChange
    };

    // --- Build envelope ---
    let target = MutationTargetRef::SectionMove(SectionMoveTargetRef {
        kind: MutationTargetKind::Section,
        source: SectionTargetRef {
            kind: MutationTargetKind::Section,
            selector: source_section.selector.clone(),
            section: source_section,
        },
        destination: SectionTargetRef {
            kind: MutationTargetKind::Section,
            selector: dest_section.selector.clone(),
            section: dest_section,
        },
        destination_mode: dest_mode,
        level_shift_applied: level_delta,
    });

    let invariant = SourcePreservationInvariant {
        preserves_non_target_bytes: false,
        target_span_before: Some(SourceSpan {
            line_start: src_span.line_start,
            line_end: src_span.line_end,
            byte_start: src_byte_start,
            byte_end: src_byte_end,
        }),
        target_span_after: None,
    };

    let make_result = |content: Option<String>| MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: args.file.to_string_lossy().to_string(),
        command: MutationCommandKind::MoveSection,
        target: target.clone(),
        disposition,
        changed,
        line_endings,
        invariant: invariant.clone(),
        content,
    };

    if args.in_place {
        if changed {
            std::fs::write(&args.file, &output_doc)?;
        }
        if json {
            output::write_json(&make_result(None))?;
        }
    } else if json {
        output::write_json(&make_result(Some(output_doc)))?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}

fn pick_destination(args: &MoveSectionArgs) -> Result<(&str, InsertMode), CommandError> {
    match (
        args.after.as_deref(),
        args.before.as_deref(),
        args.into.as_deref(),
    ) {
        (Some(a), None, None) => Ok((a, InsertMode::AfterSibling)),
        (None, Some(b), None) => Ok((b, InsertMode::BeforeSibling)),
        (None, None, Some(i)) => Ok((i, InsertMode::IntoAsChild)),
        _ => Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "exactly one of --after, --before, or --into is required",
        )),
    }
}

/// Rewrite ATX `#` runs inside `moved` so every heading shifts by `level_delta`.
///
/// Walks blocks in reverse byte order so earlier offsets stay valid as later
/// markers are spliced. Caller has already validated that no level lands
/// outside [1, 6] and that no setext heading would need re-leveling.
fn rewrite_atx_levels(
    moved: String,
    src_section: &SectionEntry,
    doc: &ParsedDocument,
    src_byte_start: u32,
    level_delta: i8,
) -> Result<String, CommandError> {
    let mut buf = moved.into_bytes();

    // Collect ATX heading marker spans (relative to `buf`) in reverse byte order.
    let mut marker_edits: Vec<(usize, usize, usize)> = Vec::new(); // (start, end, new_level)
    for &idx in &src_section.block_indices {
        let block = &doc.blocks[idx as usize];
        let h = match &block.heading {
            Some(h) => h,
            None => continue,
        };
        if h.kind != HeadingSourceKind::Atx {
            continue;
        }
        let m = h.marker_span.ok_or_else(|| {
            CommandError::new(
                DiagnosticCode::InvalidSelector,
                "internal: ATX heading missing marker_span",
            )
        })?;
        let rel_start = (m.byte_start - src_byte_start) as usize;
        let rel_end = (m.byte_end - src_byte_start) as usize;
        let new_level = (h.level as i32 + level_delta as i32) as usize;
        marker_edits.push((rel_start, rel_end, new_level));
    }
    marker_edits.sort_by(|a, b| b.0.cmp(&a.0));

    for (start, end, new_level) in marker_edits {
        let new_marker = vec![b'#'; new_level];
        buf.splice(start..end, new_marker);
    }

    String::from_utf8(buf).map_err(|_| {
        CommandError::new(
            DiagnosticCode::InvalidSelector,
            "internal: ATX rewrite produced invalid UTF-8",
        )
    })
}

/// Walk backward from `pos` to the byte after the previous newline (or 0).
/// Used to capture leading-space indentation that comrak's ATX sourcepos skips.
///
/// No-op at EOF: a position at or past `bytes.len()` is always treated as
/// already-line-aligned, so a section ending mid-line at EOF (file with no
/// trailing newline) does not lose its trailing content.
fn line_start(bytes: &[u8], pos: usize) -> usize {
    if pos >= bytes.len() {
        return bytes.len();
    }
    let mut p = pos;
    while p > 0 && bytes[p - 1] != b'\n' {
        p -= 1;
    }
    p
}
