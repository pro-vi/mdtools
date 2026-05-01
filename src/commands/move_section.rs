use crate::cli::MoveSectionArgs;
use crate::commands::section::{build_selector, find_section};
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use crate::parser::{HeadingSourceKind, ParsedDocument};

pub fn run_move_section(args: &MoveSectionArgs, json: bool) -> Result<(), CommandError> {
    if args.in_place && args.file.is_none() {
        return Err(CommandError::new(
            DiagnosticCode::InvalidSelector,
            "--in-place requires a FILE argument",
        ));
    }

    let source = match &args.file {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    let file_label = args
        .file
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "-".to_string());
    let doc = ParsedDocument::parse(source)?;

    // --- Resolve source + destination selectors ---
    let source_selector = build_selector(&args.source, args.source_occurrence, args.ignore_case)?;
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
    let dest_inside_src =
        dest_span.byte_start >= src_byte_start && dest_span.byte_end <= src_byte_end;
    let src_inside_dest =
        src_byte_start >= dest_span.byte_start && src_byte_end <= dest_span.byte_end;

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
    let mut moved = doc.source[src_byte_start as usize..src_byte_end as usize].to_string();

    if level_delta != 0 {
        moved = rewrite_atx_levels(moved, &source_section, &doc, src_byte_start, level_delta)?;
    }

    // Compute separator-injection against the POST-removal layout, not the
    // original document. Two distinct concerns at the leading boundary:
    //   - ATX source needs at least 1 newline between preceding content and
    //     the heading marker.
    //   - Setext source needs a blank-line block boundary (≥2 newlines) — a
    //     single newline lets CommonMark fold the setext text + underline
    //     into the preceding paragraph as a multi-line setext heading.
    let bytes_view = doc.source.as_bytes();
    let doc_len = doc.source.len();

    // Will any content follow the moved bytes in the post-removal output?
    let content_follows_insert = if insert_byte_raw <= src_byte_start {
        insert_byte_raw < src_byte_start || (src_byte_end as usize) < doc_len
    } else {
        (insert_byte_raw as usize) < doc_len
    };

    if content_follows_insert && !moved.ends_with('\n') {
        moved.push_str(separator);
    }

    let source_starts_with_setext = doc
        .blocks
        .get(source_section.block_indices[0] as usize)
        .and_then(|b| b.heading.as_ref())
        .map_or(false, |h| h.kind == HeadingSourceKind::Setext);

    // Walk backward from the post-removal insert position counting
    // consecutive line breaks. In the backward-splice case where the insert sits
    // past `src_byte_end`, the chunk doc[src_byte_start..src_byte_end] has
    // been removed, so we clamp the walk at src_byte_end to avoid counting
    // newlines from inside the removed source.
    let (walk_start, walk_lower_bound) = if insert_byte_raw <= src_byte_start {
        (insert_byte_raw as usize, 0usize)
    } else if insert_byte_raw == src_byte_end {
        (src_byte_start as usize, 0usize)
    } else {
        (insert_byte_raw as usize, src_byte_end as usize)
    };
    let (preceding_trailing_breaks, has_preceding_content) =
        count_trailing_line_breaks(bytes_view, walk_start, walk_lower_bound);
    let moved_leading_breaks = count_leading_line_breaks(moved.as_bytes());

    let source_heading_text = source_section
        .heading
        .as_ref()
        .map(|h| h.text.clone())
        .unwrap_or_default();

    let leading_separators_needed = if source_starts_with_setext {
        choose_setext_leading_separators(
            &doc.source,
            &moved,
            separator,
            insert_byte_raw,
            src_byte_start,
            src_byte_end,
            &source_heading_text,
            source_top_level,
        )
    } else if has_preceding_content {
        1usize.saturating_sub(preceding_trailing_breaks + moved_leading_breaks)
    } else {
        0
    };

    // Capture the moved section's byte_start in the OUTPUT document while
    // building it — needed for the JSON envelope's target_span_after.
    let leading_pad: String = separator.repeat(leading_separators_needed);
    let (output_doc, moved_byte_start_in_output) = build_spliced_output(
        &doc.source,
        &moved,
        &leading_pad,
        insert_byte_raw,
        src_byte_start,
        src_byte_end,
    );

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

    let span_before = SourceSpan {
        line_start: src_span.line_start,
        line_end: src_span.line_end,
        byte_start: src_byte_start,
        byte_end: src_byte_end,
    };

    // The post-move span describes where the moved section now lives in the
    // OUTPUT doc. For NoChange this is identical to span_before; for Replaced
    // we derive line_start/line_end from the output bytes using the same
    // convention as find_heading_section (subtract one when byte_end follows
    // a newline, since byte_end points to the start of the next thing).
    let span_after = match disposition {
        MutationDisposition::NoChange => Some(span_before),
        MutationDisposition::Replaced => {
            let output_bytes = output_doc.as_bytes();
            let moved_byte_end_in_output = moved_byte_start_in_output + moved.len() as u32;
            let line_start = (output_bytes[..moved_byte_start_in_output as usize]
                .iter()
                .filter(|&&b| b == b'\n')
                .count()
                + 1) as u32;
            let total_lines = (output_bytes.iter().filter(|&&b| b == b'\n').count() + 1) as u32;
            let byte_end_pos = moved_byte_end_in_output as usize;
            let line_end = if byte_end_pos >= output_doc.len() {
                total_lines
            } else {
                let line_at_end = (output_bytes[..byte_end_pos]
                    .iter()
                    .filter(|&&b| b == b'\n')
                    .count()
                    + 1) as u32;
                if byte_end_pos > 0 && output_bytes[byte_end_pos - 1] == b'\n' {
                    line_at_end - 1
                } else {
                    line_at_end
                }
            };
            Some(SourceSpan {
                line_start,
                line_end,
                byte_start: moved_byte_start_in_output,
                byte_end: moved_byte_end_in_output,
            })
        }
        _ => Some(span_before),
    };

    let invariant = SourcePreservationInvariant {
        preserves_non_target_bytes: false,
        target_span_before: Some(span_before),
        target_span_after: span_after,
    };

    let make_result = |content: Option<String>| MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file_label.clone(),
        command: MutationCommandKind::MoveSection,
        target: target.clone(),
        disposition,
        changed,
        line_endings,
        invariant: invariant.clone(),
        content,
    };

    if args.in_place {
        // Already validated above that args.file is Some when --in-place.
        let path = args.file.as_ref().expect("validated above");
        if changed {
            std::fs::write(path, &output_doc)?;
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

fn build_spliced_output(
    source: &str,
    moved: &str,
    leading_pad: &str,
    insert_byte_raw: u32,
    src_byte_start: u32,
    src_byte_end: u32,
) -> (String, u32) {
    let cap = source.len() + moved.len() + leading_pad.len();
    if insert_byte_raw <= src_byte_start {
        let mut out = String::with_capacity(cap);
        out.push_str(&source[..insert_byte_raw as usize]);
        out.push_str(leading_pad);
        let moved_start = out.len() as u32;
        out.push_str(moved);
        out.push_str(&source[insert_byte_raw as usize..src_byte_start as usize]);
        out.push_str(&source[src_byte_end as usize..]);
        (out, moved_start)
    } else {
        let mut out = String::with_capacity(cap);
        out.push_str(&source[..src_byte_start as usize]);
        out.push_str(&source[src_byte_end as usize..insert_byte_raw as usize]);
        out.push_str(leading_pad);
        let moved_start = out.len() as u32;
        out.push_str(moved);
        out.push_str(&source[insert_byte_raw as usize..]);
        (out, moved_start)
    }
}

fn choose_setext_leading_separators(
    source: &str,
    moved: &str,
    separator: &str,
    insert_byte_raw: u32,
    src_byte_start: u32,
    src_byte_end: u32,
    heading_text: &str,
    expected_level: u8,
) -> usize {
    for count in 0..=2 {
        let leading_pad = separator.repeat(count);
        let (candidate, moved_start) = build_spliced_output(
            source,
            moved,
            &leading_pad,
            insert_byte_raw,
            src_byte_start,
            src_byte_end,
        );
        if moved_section_reparses_at(
            &candidate,
            moved_start as usize,
            moved.len(),
            heading_text,
            expected_level,
        ) {
            return count;
        }
    }

    2
}

fn moved_section_reparses_at(
    output: &str,
    moved_start: usize,
    moved_len: usize,
    heading_text: &str,
    expected_level: u8,
) -> bool {
    let parsed = match ParsedDocument::parse(output.to_string()) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };
    let bytes = output.as_bytes();
    let moved_end = moved_start + moved_len;

    for (idx, block) in parsed.blocks.iter().enumerate() {
        let Some(heading) = &block.heading else {
            continue;
        };
        if heading.text != heading_text || heading.level != expected_level {
            continue;
        }
        if line_start(bytes, block.span.byte_start as usize) != moved_start {
            continue;
        }

        let section_end = parsed
            .blocks
            .iter()
            .skip(idx + 1)
            .find_map(|next| {
                next.heading.as_ref().and_then(|next_heading| {
                    if next_heading.level <= heading.level {
                        Some(line_start(bytes, next.span.byte_start as usize))
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(output.len());

        return section_end == moved_end;
    }

    false
}

fn count_trailing_line_breaks(bytes: &[u8], start: usize, lower_bound: usize) -> (usize, bool) {
    let mut count = 0usize;
    let mut p = start;
    while p > lower_bound && bytes[p - 1] == b'\n' {
        count += 1;
        p -= 1;
        if p > lower_bound && bytes[p - 1] == b'\r' {
            p -= 1;
        }
    }

    (count, p > 0)
}

fn count_leading_line_breaks(bytes: &[u8]) -> usize {
    let mut count = 0usize;
    let mut p = 0usize;
    while p < bytes.len() {
        if bytes[p] == b'\n' {
            count += 1;
            p += 1;
        } else if p + 1 < bytes.len() && bytes[p] == b'\r' && bytes[p + 1] == b'\n' {
            count += 1;
            p += 2;
        } else {
            break;
        }
    }
    count
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
