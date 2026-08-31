// Parser boundary: all comrak interaction is contained here.
// The rest of the codebase sees only model types, never comrak types.

use crate::core_error::CoreError;
use crate::model::*;
use crate::source::{DocumentSource, LineIndex, ParsePolicy};
use comrak::{
    nodes::{AstNode, NodeValue, Sourcepos},
    parse_document, Arena, Options,
};

fn source_position_error(line: usize, column: usize, reason: &'static str) -> CoreError {
    CoreError::InvalidSourcePosition {
        line,
        column,
        reason,
    }
}

fn sourcepos_to_span(
    line_index: &LineIndex,
    source: &str,
    sourcepos: Sourcepos,
) -> Result<SourceSpan, CoreError> {
    let byte_start = line_index
        .to_byte(sourcepos.start.line, sourcepos.start.column)
        .ok_or_else(|| {
            source_position_error(
                sourcepos.start.line,
                sourcepos.start.column,
                "start is outside the source",
            )
        })?;
    if sourcepos.end.column == 0 {
        if sourcepos.end.line <= 1 {
            return Err(source_position_error(
                sourcepos.end.line,
                sourcepos.end.column,
                "zero-column end has no preceding content line",
            ));
        }
        let sentinel_line = u32::try_from(sourcepos.end.line).map_err(|_| {
            source_position_error(
                sourcepos.end.line,
                sourcepos.end.column,
                "line cannot be represented",
            )
        })?;
        let byte_end = line_index.line_start_byte(sentinel_line).ok_or_else(|| {
            source_position_error(
                sourcepos.end.line,
                sourcepos.end.column,
                "sentinel end line is outside the source",
            )
        })?;
        let adjusted = Sourcepos {
            start: sourcepos.start,
            end: comrak::nodes::LineColumn {
                line: sourcepos.end.line - 1,
                column: 1,
            },
        };
        return validate_parser_span(source, adjusted, byte_start, byte_end);
    }
    let byte_end = line_index
        .to_byte_end(sourcepos.end.line, sourcepos.end.column)
        .ok_or_else(|| {
            source_position_error(
                sourcepos.end.line,
                sourcepos.end.column,
                "end is outside the source",
            )
        })?;
    validate_parser_span(source, sourcepos, byte_start, byte_end)
}

fn validate_parser_span(
    source: &str,
    sourcepos: Sourcepos,
    byte_start: usize,
    byte_end: usize,
) -> Result<SourceSpan, CoreError> {
    if byte_start > byte_end {
        return Err(source_position_error(
            sourcepos.start.line,
            sourcepos.start.column,
            "start is after end",
        ));
    }
    if !source.is_char_boundary(byte_start) || !source.is_char_boundary(byte_end) {
        return Err(source_position_error(
            sourcepos.start.line,
            sourcepos.start.column,
            "offset is not a UTF-8 character boundary",
        ));
    }
    Ok(SourceSpan {
        line_start: u32::try_from(sourcepos.start.line).map_err(|_| {
            source_position_error(
                sourcepos.start.line,
                sourcepos.start.column,
                "line cannot be represented",
            )
        })?,
        line_end: u32::try_from(sourcepos.end.line).map_err(|_| {
            source_position_error(
                sourcepos.end.line,
                sourcepos.end.column,
                "line cannot be represented",
            )
        })?,
        byte_start: byte_start as u32,
        byte_end: byte_end as u32,
    })
}

fn frontmatter_sourcepos_to_span(
    line_index: &LineIndex,
    source: &str,
    sourcepos: Sourcepos,
) -> Result<SourceSpan, CoreError> {
    let mut span = sourcepos_to_span(line_index, source, sourcepos)?;
    let suffix = &source[span.byte_end as usize..];
    if suffix.starts_with("\r\n") {
        span.byte_end += 2;
    } else if suffix.starts_with('\n') {
        span.byte_end += 1;
    }
    Ok(span)
}

fn sourcepos_to_span_fixup(
    line_index: &LineIndex,
    source: &str,
    sourcepos: Sourcepos,
    kind: BlockKind,
    heading_line: Option<usize>,
) -> Result<SourceSpan, CoreError> {
    let mut span = if kind == BlockKind::IndentedCode {
        let byte_start = line_index.to_byte(sourcepos.start.line, 1).ok_or_else(|| {
            source_position_error(sourcepos.start.line, 1, "start line is outside the source")
        })?;
        let (line_end, byte_end) = if sourcepos.end.column == 0 && sourcepos.end.line > 1 {
            let content_line = sourcepos.end.line - 1;
            let next_line = u32::try_from(content_line + 1).map_err(|_| {
                source_position_error(content_line, 1, "line cannot be represented")
            })?;
            (
                content_line,
                line_index
                    .line_start_byte(next_line)
                    .unwrap_or(line_index.source_len()),
            )
        } else {
            (
                sourcepos.end.line,
                line_index
                    .to_byte_end(sourcepos.end.line, sourcepos.end.column)
                    .ok_or_else(|| {
                        source_position_error(
                            sourcepos.end.line,
                            sourcepos.end.column,
                            "end is outside the source",
                        )
                    })?,
            )
        };
        let adjusted = Sourcepos {
            start: comrak::nodes::LineColumn {
                line: sourcepos.start.line,
                column: 1,
            },
            end: comrak::nodes::LineColumn {
                line: line_end,
                column: sourcepos.end.column.max(1),
            },
        };
        validate_parser_span(source, adjusted, byte_start, byte_end)?
    } else {
        sourcepos_to_span(line_index, source, sourcepos)?
    };

    if kind == BlockKind::ThematicBreak && sourcepos.end.column == 0 && sourcepos.end.line > 1 {
        let content_line = sourcepos.start.line;
        span.line_end = content_line as u32;
        span.byte_end = line_content_end(line_index, source, content_line)? as u32;
    }

    if let Some(line) = heading_line {
        span.byte_start = heading_line_start(line_index, source, line)?;
    }

    Ok(span)
}

fn line_content_end(line_index: &LineIndex, source: &str, line: usize) -> Result<usize, CoreError> {
    let start = line_index
        .to_byte(line, 1)
        .ok_or_else(|| source_position_error(line, 1, "content line is outside the source"))?;
    let mut end = source.as_bytes()[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(source.len(), |offset| start + offset);
    if end > start && source.as_bytes()[end - 1] == b'\r' {
        end -= 1;
    }
    Ok(end)
}

fn heading_line_start(line_index: &LineIndex, source: &str, line: usize) -> Result<u32, CoreError> {
    let line_start = line_index
        .to_byte(line, 1)
        .ok_or_else(|| source_position_error(line, 1, "heading line is outside the source"))?;
    if line_start >= source.len() {
        return Err(source_position_error(
            line,
            1,
            "heading line has no source bytes",
        ));
    }
    Ok(line_start as u32)
}

// --- Comrak options ---

fn comrak_opts(delimiter: Option<&str>) -> Options<'static> {
    // Every extension enabled here that can create a top-level block must have
    // an explicit arm in `node_value_to_block_kind`. The classifier fails
    // closed so an options change cannot silently reinterpret a new node kind.
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    if let Some(d) = delimiter {
        options.extension.front_matter_delimiter = Some(d.to_string());
    }
    options
}

/// Strip frontmatter delimiters (--- or +++) and trailing whitespace.
/// comrak's FrontMatter.raw includes "---\ncontent\n---\n\n"
pub fn strip_frontmatter_delimiters(raw: &str) -> String {
    let trimmed = raw.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.len() < 2 {
        return String::new();
    }
    let content_lines = &lines[1..lines.len() - 1];
    content_lines.join("\n")
}

/// Detect which frontmatter delimiter to use by inspecting the first line.
pub(crate) fn detect_frontmatter_delimiter(source: &str) -> Option<&'static str> {
    let first_line = source.lines().next().unwrap_or("");
    if first_line == "---" {
        Some("---")
    } else if first_line == "+++" {
        Some("+++")
    } else {
        None
    }
}

// --- Ephemeral parser facts ---

/// Source-free parser facts consumed during index construction.
pub(crate) struct ParsedFacts {
    pub(crate) blocks: Vec<BlockFact>,
    pub(crate) frontmatter: Option<FrontmatterFact>,
}

/// One parser-retained top-level block.
#[derive(Clone, Debug)]
pub(crate) struct BlockFact {
    pub index: u32,
    pub kind: BlockKind,
    pub span: SourceSpan,
    pub heading: Option<HeadingFact>,
    pub links: Vec<LinkFact>,
    pub task_items: Vec<TaskItemFact>,
    pub table: Option<TableFact>,
}

#[derive(Clone, Debug)]
pub(crate) struct TaskItemFact {
    pub child_path: Vec<u32>,
    pub task_index: u32,
    pub status: TaskStatus,
    pub depth: u32,
    pub span: SourceSpan,
    pub symbol_byte_offset: u32,
    pub summary_text: String,
}

#[derive(Clone, Debug)]
pub(crate) struct HeadingFact {
    pub level: u8,
    pub text: String,
    pub kind: HeadingSourceKind,
    /// Byte span covering the ATX `#` run or the setext underline marker.
    pub marker_span: SourceSpan,
    pub(crate) line_breaks: Vec<HeadingLineBreak>,
    pub(crate) multiline_code_spans: Vec<HeadingCodeSpan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HeadingLineBreak {
    pub(crate) span: SourceSpan,
    pub(crate) kind: HeadingLineBreakKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeadingCodeSpan {
    pub(crate) span: SourceSpan,
    pub(crate) literal: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeadingLineBreakKind {
    Soft,
    Hard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingSourceKind {
    Atx,
    Setext,
}

#[derive(Clone, Debug)]
pub(crate) struct LinkFact {
    pub kind: LinkKind,
    pub text: String,
    pub destination: Option<String>,
    pub title: Option<String>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub(crate) struct FrontmatterFact {
    pub span: SourceSpan,
    pub format: FrontmatterFormat,
}

impl ParsedFacts {
    pub(crate) fn parse(source: &DocumentSource) -> Result<Self, CoreError> {
        Self::parse_inner(source)
    }

    fn parse_inner(document_source: &DocumentSource) -> Result<Self, CoreError> {
        let source = document_source.text();
        let mode = document_source.policy();
        let delimiter = detect_frontmatter_delimiter(source);
        let line_index = document_source.lines();
        let opts = comrak_opts(delimiter);
        let arena = Arena::new();
        let root = parse_document(&arena, source, &opts);
        reject_excessive_ast_depth(root)?;

        // Check if frontmatter exists and is valid
        let mut has_frontmatter_node = false;
        let mut frontmatter_raw: Option<String> = None;
        let mut frontmatter_format = FrontmatterFormat::Yaml;

        for node in root.children() {
            let data = node.data.borrow();
            if let NodeValue::FrontMatter(raw) = &data.value {
                has_frontmatter_node = true;
                frontmatter_raw = Some(raw.clone());
                frontmatter_format = match delimiter {
                    Some("+++") => FrontmatterFormat::Toml,
                    _ => FrontmatterFormat::Yaml,
                };
            }
        }

        if matches!(mode, ParsePolicy::Mutation) && !has_frontmatter_node {
            if let Some(delimiter) = delimiter {
                return Err(CoreError::FrontmatterParseFailed(format!(
                    "unclosed frontmatter (no closing '{}')",
                    delimiter
                )));
            }
        }

        if matches!(mode, ParsePolicy::Lenient)
            && has_frontmatter_node
            && frontmatter_raw.as_ref().is_some_and(|raw| {
                !frontmatter_content_is_semantically_valid(raw, frontmatter_format)
            })
        {
            // Re-parse without frontmatter delimiter — treat malformed frontmatter as plain content.
            let _ = root;
            return Self::parse_without_frontmatter(document_source);
        }

        let mut blocks = Vec::new();
        let mut frontmatter = None;

        for node in root.children() {
            let data = node.data.borrow();
            let sp = data.sourcepos;

            match &data.value {
                NodeValue::FrontMatter(_) => {
                    let fm_span = frontmatter_sourcepos_to_span(line_index, source, sp)?;
                    frontmatter = Some(FrontmatterFact {
                        span: fm_span,
                        format: frontmatter_format,
                    });
                    // Frontmatter is NOT a block — no index increment
                }
                _ => {
                    let heading_meta = if let NodeValue::Heading(h) = &data.value {
                        Some((h.level, h.setext, sp.start.line, sp.end.line))
                    } else {
                        None
                    };
                    let kind = node_value_to_block_kind(&data.value)?;
                    let heading_line = heading_meta.map(|(_, _, line, _)| line);
                    let span = sourcepos_to_span_fixup(line_index, source, sp, kind, heading_line)?;
                    drop(data);

                    let heading = heading_meta
                        .map(|(level, setext, line, end_line)| {
                            let text = collect_heading_text(node);
                            let kind = if setext {
                                HeadingSourceKind::Setext
                            } else {
                                HeadingSourceKind::Atx
                            };
                            let marker_span = if setext {
                                compute_setext_marker_span(line_index, source, end_line)?
                            } else {
                                compute_atx_marker_span(line_index, source, line)?
                            }
                            .ok_or_else(|| {
                                CoreError::ParseFailed(format!(
                                    "heading at line {line} has no recognized syntax marker"
                                ))
                            })?;
                            Ok::<_, CoreError>(HeadingFact {
                                level,
                                text,
                                kind,
                                marker_span,
                                line_breaks: collect_heading_line_breaks(node, line_index, source)?,
                                multiline_code_spans: collect_multiline_heading_code_spans(
                                    node, line_index, source,
                                )?,
                            })
                        })
                        .transpose()?;

                    let links = collect_links(node, line_index, source)?;
                    let task_items = collect_all_task_items(node, line_index, source, span)?;
                    let table = project_table_node(node, line_index, source, None)?;

                    blocks.push(BlockFact {
                        index: blocks.len() as u32,
                        kind,
                        span,
                        heading,
                        links,
                        task_items,
                        table,
                    });
                }
            }
        }

        Ok(ParsedFacts {
            blocks,
            frontmatter,
        })
    }

    pub(crate) fn parse_without_frontmatter(
        document_source: &DocumentSource,
    ) -> Result<Self, CoreError> {
        let source = document_source.text();
        let line_index = document_source.lines();
        let opts = comrak_opts(None); // No frontmatter delimiter
        let arena = Arena::new();
        let root = parse_document(&arena, source, &opts);
        reject_excessive_ast_depth(root)?;

        let mut blocks = Vec::new();

        for (block_index, node) in root.children().enumerate() {
            let data = node.data.borrow();
            let sp = data.sourcepos;
            let heading_meta = if let NodeValue::Heading(h) = &data.value {
                Some((h.level, h.setext, sp.start.line, sp.end.line))
            } else {
                None
            };
            let kind = node_value_to_block_kind(&data.value)?;
            let heading_line = heading_meta.map(|(_, _, line, _)| line);
            let span = sourcepos_to_span_fixup(line_index, source, sp, kind, heading_line)?;
            drop(data);

            let heading = heading_meta
                .map(|(level, setext, line, end_line)| {
                    let text = collect_heading_text(node);
                    let kind = if setext {
                        HeadingSourceKind::Setext
                    } else {
                        HeadingSourceKind::Atx
                    };
                    let marker_span = if setext {
                        compute_setext_marker_span(line_index, source, end_line)?
                    } else {
                        compute_atx_marker_span(line_index, source, line)?
                    }
                    .ok_or_else(|| {
                        CoreError::ParseFailed(format!(
                            "heading at line {line} has no recognized syntax marker"
                        ))
                    })?;
                    Ok::<_, CoreError>(HeadingFact {
                        level,
                        text,
                        kind,
                        marker_span,
                        line_breaks: collect_heading_line_breaks(node, line_index, source)?,
                        multiline_code_spans: collect_multiline_heading_code_spans(
                            node, line_index, source,
                        )?,
                    })
                })
                .transpose()?;

            let links = collect_links(node, line_index, source)?;
            let task_items = collect_all_task_items(node, line_index, source, span)?;
            let table = project_table_node(node, line_index, source, None)?;

            blocks.push(BlockFact {
                index: block_index as u32,
                kind,
                span,
                heading,
                links,
                task_items,
                table,
            });
        }

        Ok(ParsedFacts {
            blocks,
            frontmatter: None,
        })
    }
}

fn reject_excessive_ast_depth<'a>(root: &'a AstNode<'a>) -> Result<(), CoreError> {
    const MAX_AST_DEPTH: usize = 2_048;
    let mut pending = root
        .children()
        .map(|node| (node, 1usize))
        .collect::<Vec<_>>();
    while let Some((node, depth)) = pending.pop() {
        if depth > MAX_AST_DEPTH {
            return Err(CoreError::ParseFailed(format!(
                "Markdown AST exceeds the supported depth of {MAX_AST_DEPTH}"
            )));
        }
        pending.extend(node.children().map(|child| (child, depth + 1)));
    }
    Ok(())
}

fn frontmatter_content_is_semantically_valid(raw: &str, format: FrontmatterFormat) -> bool {
    let content = strip_frontmatter_delimiters(raw);
    if content.trim().is_empty() {
        return true;
    }

    match format {
        FrontmatterFormat::Yaml => serde_yaml::from_str::<serde_json::Value>(&content).is_ok(),
        FrontmatterFormat::Toml => content.parse::<toml::Value>().is_ok(),
    }
}

/// Compute the byte span covering an ATX heading's `#` run.
///
/// CommonMark allows 0-3 leading spaces before the `#`s. Returns None if the
/// line does not begin (after that indentation) with `#` — i.e. a setext
/// heading or anything else.
fn compute_atx_marker_span(
    line_index: &LineIndex,
    source: &str,
    line: usize,
) -> Result<Option<SourceSpan>, CoreError> {
    let line_start_byte = line_index.to_byte(line, 1).ok_or_else(|| {
        source_position_error(line, 1, "heading marker line is outside the source")
    })?;
    let bytes = source.as_bytes();
    if line_start_byte > bytes.len() {
        return Err(source_position_error(
            line,
            1,
            "heading marker starts outside the source",
        ));
    }
    let mut p = line_start_byte;
    let indent_limit = (line_start_byte + 3).min(bytes.len());
    while p < indent_limit && bytes[p] == b' ' {
        p += 1;
    }
    if p >= bytes.len() || bytes[p] != b'#' {
        return Ok(None);
    }
    let marker_start = p;
    while p < bytes.len() && bytes[p] == b'#' {
        p += 1;
    }
    Ok(Some(SourceSpan {
        line_start: line as u32,
        line_end: line as u32,
        byte_start: marker_start as u32,
        byte_end: p as u32,
    }))
}

/// Compute the byte span covering a setext heading's `=` or `-` underline.
fn compute_setext_marker_span(
    line_index: &LineIndex,
    source: &str,
    line: usize,
) -> Result<Option<SourceSpan>, CoreError> {
    let line_start = line_index.to_byte(line, 1).ok_or_else(|| {
        source_position_error(line, 1, "setext marker line is outside the source")
    })?;
    let line_end = source.as_bytes()[line_start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(source.len(), |offset| line_start + offset);
    let line_bytes = &source.as_bytes()[line_start..line_end];
    let mut start = 0usize;
    while start < line_bytes.len() && start < 3 && line_bytes[start] == b' ' {
        start += 1;
    }
    let Some(marker) = line_bytes.get(start).copied() else {
        return Ok(None);
    };
    if !matches!(marker, b'=' | b'-') {
        return Ok(None);
    }
    let mut end = start;
    while line_bytes.get(end) == Some(&marker) {
        end += 1;
    }
    Ok(Some(SourceSpan {
        line_start: line as u32,
        line_end: line as u32,
        byte_start: (line_start + start) as u32,
        byte_end: (line_start + end) as u32,
    }))
}

// --- Task item extraction ---

/// Recursively collect task items from a list node.
fn collect_task_items<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    prefix: &[u32],
    depth: u32,
) -> Result<Vec<TaskItemFact>, CoreError> {
    let mut items = Vec::new();
    let mut task_counter = 0u32;

    for (child_counter, child) in node.children().enumerate() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::TaskItem(task) => {
                let sp = data.sourcepos;
                let span = sourcepos_to_span(line_index, source, sp)?;
                let symbol_byte_offset = line_index
                    .to_byte(
                        task.symbol_sourcepos.start.line,
                        task.symbol_sourcepos.start.column,
                    )
                    .ok_or_else(|| {
                        source_position_error(
                            task.symbol_sourcepos.start.line,
                            task.symbol_sourcepos.start.column,
                            "task marker is outside the source",
                        )
                    })? as u32;
                let status = if task.symbol.is_some() {
                    TaskStatus::Done
                } else {
                    TaskStatus::Pending
                };
                drop(data);

                let mut path = prefix.to_vec();
                path.push(child_counter as u32);

                let summary_text = collect_task_item_text(child);

                items.push(TaskItemFact {
                    child_path: path.clone(),
                    task_index: task_counter,
                    status,
                    depth,
                    span,
                    symbol_byte_offset,
                    summary_text,
                });

                items.extend(collect_nested_task_items(
                    child,
                    line_index,
                    source,
                    &path,
                    depth + 1,
                )?);

                task_counter += 1;
            }
            NodeValue::Item(_) => {
                drop(data);

                let mut path = prefix.to_vec();
                path.push(child_counter as u32);

                items.extend(collect_nested_task_items(
                    child,
                    line_index,
                    source,
                    &path,
                    depth + 1,
                )?);
            }
            _ => {
                drop(data);
            }
        }
    }

    Ok(items)
}

fn collect_nested_task_items<'a>(
    item: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    item_path: &[u32],
    depth: u32,
) -> Result<Vec<TaskItemFact>, CoreError> {
    let nested_lists = item
        .children()
        .filter(|child| matches!(child.data.borrow().value, NodeValue::List(_)))
        .collect::<Vec<_>>();
    let needs_list_ordinal = nested_lists.len() > 1;
    let mut items = Vec::new();
    for (list_ordinal, nested_list) in nested_lists.into_iter().enumerate() {
        let mut prefix = item_path.to_vec();
        if needs_list_ordinal {
            // A single nested list keeps the existing compact loc. Multiple
            // sibling lists need their own segment or their item ordinals
            // collide (for example, bullet item 0 and ordered item 0).
            prefix.push(list_ordinal as u32);
        }
        items.extend(collect_task_items(
            nested_list,
            line_index,
            source,
            &prefix,
            depth,
        )?);
    }
    Ok(items)
}

/// Find all task items under any block node by walking descendants for List nodes.
/// Handles task lists inside blockquotes, callouts, and other containers.
fn collect_all_task_items<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    block_span: SourceSpan,
) -> Result<Vec<TaskItemFact>, CoreError> {
    let data = node.data.borrow();
    if matches!(data.value, NodeValue::List(_)) {
        drop(data);
        return Ok(clamp_task_spans(
            collect_task_items(node, line_index, source, &[], 0)?,
            block_span,
        ));
    }
    drop(data);

    // Walk children looking for List descendants (e.g. inside BlockQuote).
    // Use a counter so sibling lists inside the same container get distinct
    // path prefixes (e.g. first list → prefix [0], second → prefix [1]).
    let mut items = Vec::new();
    let mut list_counter = 0u32;
    find_list_descendants(node, line_index, source, &mut items, &mut list_counter)?;
    Ok(clamp_task_spans(items, block_span))
}

fn clamp_task_spans(mut items: Vec<TaskItemFact>, block_span: SourceSpan) -> Vec<TaskItemFact> {
    for item in &mut items {
        item.span.byte_start = item.span.byte_start.max(block_span.byte_start);
        item.span.byte_end = item.span.byte_end.min(block_span.byte_end);
        item.span.line_start = item.span.line_start.max(block_span.line_start);
        item.span.line_end = item.span.line_end.min(block_span.line_end);
    }
    items
}

fn find_list_descendants<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    out: &mut Vec<TaskItemFact>,
    list_counter: &mut u32,
) -> Result<(), CoreError> {
    for child in node.children() {
        let data = child.data.borrow();
        if matches!(data.value, NodeValue::List(_)) {
            drop(data);
            let prefix = [*list_counter];
            out.extend(collect_task_items(child, line_index, source, &prefix, 0)?);
            *list_counter += 1;
        } else {
            drop(data);
            find_list_descendants(child, line_index, source, out, list_counter)?;
        }
    }
    Ok(())
}

/// Extract the first paragraph's inline text from a task item node.
fn collect_task_item_text<'a>(node: &'a AstNode<'a>) -> String {
    for child in node.children() {
        let data = child.data.borrow();
        if matches!(data.value, NodeValue::Paragraph) {
            drop(data);
            let mut text = String::new();
            collect_text_recursive(child, &mut text);
            return text;
        }
    }
    String::new()
}

// --- Node projection helpers ---

/// Keep this projection exhaustive with the top-level nodes enabled by
/// [`comrak_opts`]. Unknown nodes are an options/projection contract failure.
fn node_value_to_block_kind(value: &NodeValue) -> Result<BlockKind, CoreError> {
    let kind = match value {
        NodeValue::Heading(_) => BlockKind::Heading,
        NodeValue::Paragraph => BlockKind::Paragraph,
        NodeValue::List(_) => BlockKind::List,
        NodeValue::BlockQuote => BlockKind::BlockQuote,
        NodeValue::CodeBlock(cb) if cb.fenced => BlockKind::CodeFence,
        NodeValue::CodeBlock(_) => BlockKind::IndentedCode,
        NodeValue::ThematicBreak => BlockKind::ThematicBreak,
        NodeValue::Table(_) => BlockKind::Table,
        NodeValue::HtmlBlock(_) => BlockKind::HtmlBlock,
        NodeValue::FootnoteDefinition(_) => BlockKind::FootnoteDefinition,
        unexpected => {
            let debug = format!("{unexpected:?}");
            let name = debug
                .split(['(', '{'])
                .next()
                .unwrap_or("unknown parser node");
            return Err(CoreError::ParseFailed(format!(
                "unclassified top-level parser node: {name}"
            )));
        }
    };
    Ok(kind)
}

/// Collect plaintext heading content by walking inline children.
fn collect_heading_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    collect_text_recursive(node, &mut text);
    text
}

fn collect_heading_line_breaks<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
) -> Result<Vec<HeadingLineBreak>, CoreError> {
    let mut breaks = Vec::new();
    collect_heading_line_breaks_recursive(node, line_index, source, &mut breaks)?;
    Ok(breaks)
}

fn collect_heading_line_breaks_recursive<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    breaks: &mut Vec<HeadingLineBreak>,
) -> Result<(), CoreError> {
    for child in node.children() {
        let data = child.data.borrow();
        let kind = match data.value {
            NodeValue::SoftBreak => Some(HeadingLineBreakKind::Soft),
            NodeValue::LineBreak => Some(HeadingLineBreakKind::Hard),
            _ => None,
        };
        if let Some(kind) = kind {
            breaks.push(HeadingLineBreak {
                span: sourcepos_to_span(line_index, source, data.sourcepos)?,
                kind,
            });
        } else {
            drop(data);
            collect_heading_line_breaks_recursive(child, line_index, source, breaks)?;
        }
    }
    Ok(())
}

fn collect_multiline_heading_code_spans<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
) -> Result<Vec<HeadingCodeSpan>, CoreError> {
    let mut spans = Vec::new();
    for child in node.descendants() {
        let data = child.data.borrow();
        let NodeValue::Code(code) = &data.value else {
            continue;
        };
        let span = sourcepos_to_span(line_index, source, data.sourcepos)?;
        if span.line_start != span.line_end {
            spans.push(HeadingCodeSpan {
                span,
                literal: normalize_inline_code_literal(&code.literal),
            });
        }
    }
    Ok(spans)
}

fn collect_text_recursive<'a>(node: &'a AstNode<'a>, out: &mut String) {
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => out.push_str(&normalize_inline_code_literal(&c.literal)),
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
            NodeValue::HtmlInline(html) if is_html_line_break(html) => out.push(' '),
            _ => {
                drop(data);
                collect_text_recursive(child, out);
            }
        }
    }
}

fn normalize_inline_code_literal(literal: &str) -> String {
    literal.replace('\r', "")
}

fn is_html_line_break(html: &str) -> bool {
    matches!(
        html.trim().to_ascii_lowercase().as_str(),
        "<br>" | "<br/>" | "<br />"
    )
}

/// Collect links from a block node by walking all descendants.
fn collect_links<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
) -> Result<Vec<LinkFact>, CoreError> {
    let mut links = Vec::new();
    collect_links_recursive(node, line_index, source, &mut links)?;
    Ok(links)
}

fn collect_links_recursive<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    out: &mut Vec<LinkFact>,
) -> Result<(), CoreError> {
    for child in node.descendants() {
        let data = child.data.borrow();
        if let NodeValue::Link(link) = &data.value {
            let sp = data.sourcepos;
            let span = sourcepos_to_span(line_index, source, sp)?;
            let url = link.url.clone();
            let title = link.title.clone();
            drop(data);

            let mut text = String::new();
            collect_text_recursive(child, &mut text);

            let kind = classify_link_kind(source, &span, &url);

            out.push(LinkFact {
                kind,
                text,
                destination: if url.is_empty() { None } else { Some(url) },
                title: if title.is_empty() { None } else { Some(title) },
                span,
            });
        }
    }
    Ok(())
}

/// Heuristic to distinguish inline, reference, and autolink kinds.
/// comrak resolves reference links, so we inspect the source text at the link span.
fn classify_link_kind(source: &str, span: &SourceSpan, _url: &str) -> LinkKind {
    let start = span.byte_start as usize;
    let end = span.byte_end as usize;
    if start >= source.len() || end > source.len() {
        return LinkKind::Inline;
    }
    let src = &source[start..end];

    // Angle-bracket autolink: <url>
    if src.starts_with('<') && src.ends_with('>') {
        return LinkKind::Autolink;
    }

    // Bare URL autolink (comrak autolink extension): no brackets
    if !src.starts_with('[') {
        return LinkKind::Autolink;
    }

    // Reference link: [text][ref] or [text][]
    // Inline link: [text](url)
    // After the closing ] of the link text, look for ( vs [
    if let Some(close_bracket) = find_link_text_close(src) {
        let after = &src[close_bracket + 1..];
        if after.starts_with('(') {
            return LinkKind::Inline;
        }
        if after.starts_with('[') {
            return LinkKind::Reference;
        }
    }

    LinkKind::Inline
}

/// Find the index of the `]` that closes the link text in `[text](url)` or `[text][ref]`.
/// Handles nested brackets.
fn find_link_text_close(src: &str) -> Option<usize> {
    if !src.starts_with('[') {
        return None;
    }
    let mut depth = 0;
    for (i, ch) in src.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

// --- Table extraction ---

#[derive(Clone, Debug)]
pub(crate) struct TableRowFact {
    pub cells: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub(crate) struct TableFact {
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<TableRowFact>,
}

fn project_table_node<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    block_span: Option<SourceSpan>,
) -> Result<Option<TableFact>, CoreError> {
    use comrak::nodes::TableAlignment;

    let data = node.data.borrow();
    let NodeValue::Table(table_meta) = &data.value else {
        return Ok(None);
    };
    let alignments = table_meta
        .alignments
        .iter()
        .map(|alignment| match alignment {
            TableAlignment::None => ColumnAlignment::None,
            TableAlignment::Left => ColumnAlignment::Left,
            TableAlignment::Center => ColumnAlignment::Center,
            TableAlignment::Right => ColumnAlignment::Right,
        })
        .collect();
    drop(data);

    let mut headers = Vec::new();
    let mut rows = Vec::new();
    for row_node in node.children() {
        let row_data = row_node.data.borrow();
        let NodeValue::TableRow(is_header) = row_data.value else {
            continue;
        };
        let local_span = physical_table_row_span(line_index, source, row_data.sourcepos)?;
        let row_span = block_span.map_or(local_span, |span| offset_span(local_span, span));
        drop(row_data);

        let cells = row_node
            .children()
            .map(|cell_node| {
                let mut text = String::new();
                collect_text_recursive(cell_node, &mut text);
                text.trim().to_string()
            })
            .collect::<Vec<_>>();
        if is_header {
            headers = cells;
        } else {
            rows.push(TableRowFact {
                cells,
                span: row_span,
            });
        }
    }

    Ok(Some(TableFact {
        headers,
        alignments,
        rows,
    }))
}

/// Span of one physical table-row line, including its legal 0-3 spaces of
/// indentation and excluding its line ending.
fn physical_table_row_span(
    line_index: &LineIndex,
    source: &str,
    sourcepos: Sourcepos,
) -> Result<SourceSpan, CoreError> {
    let line = sourcepos.start.line as u32;
    let byte_start = line_index.line_start_byte(line).ok_or_else(|| {
        source_position_error(
            sourcepos.start.line,
            sourcepos.start.column,
            "table row line is outside the source",
        )
    })?;
    let mut byte_end = line_index.line_start_byte(line + 1).unwrap_or(source.len());
    if source.as_bytes().get(byte_end.wrapping_sub(1)) == Some(&b'\n') {
        byte_end -= 1;
        if source.as_bytes().get(byte_end.wrapping_sub(1)) == Some(&b'\r') {
            byte_end -= 1;
        }
    }
    Ok(SourceSpan {
        line_start: line,
        line_end: line,
        byte_start: byte_start as u32,
        byte_end: byte_end as u32,
    })
}

fn count_table_row_columns(payload: &str) -> (usize, bool) {
    // Comrak's first-nonspace/table scanner trims ASCII space and tab here,
    // not arbitrary Unicode whitespace. A non-breaking space adjacent to an
    // outer pipe is cell content and must not turn that pipe into a boundary.
    let trimmed = payload.trim_matches(|ch| ch == ' ' || ch == '\t');
    let mut unescaped_pipes = Vec::new();
    for (byte_index, ch) in trimmed.char_indices() {
        // Match comrak 0.51's table-cell scanner: an ASCII pipe is literal
        // whenever its immediately preceding byte is a backslash. This is not
        // odd/even Markdown escape parity; a run of backslashes keeps the pipe
        // inside the cell. Backticks do not suppress table delimiters.
        if ch == '|' && (byte_index == 0 || trimmed.as_bytes()[byte_index - 1] != b'\\') {
            unescaped_pipes.push(byte_index);
        }
    }

    let leading = (unescaped_pipes.first() == Some(&0)) as usize;
    let trailing = unescaped_pipes
        .last()
        .is_some_and(|index| *index + 1 == trimmed.len()) as usize;
    (
        unescaped_pipes.len() + 1 - leading - trailing,
        !unescaped_pipes.is_empty(),
    )
}

fn offset_span(span: SourceSpan, block_span: SourceSpan) -> SourceSpan {
    SourceSpan {
        line_start: block_span.line_start + span.line_start - 1,
        line_end: block_span.line_start + span.line_end - 1,
        byte_start: block_span.byte_start + span.byte_start,
        byte_end: block_span.byte_start + span.byte_end,
    }
}

pub fn extract_table_projection(
    table_source: &str,
    block_span: SourceSpan,
) -> Result<TableFact, CoreError> {
    let temporary_source = DocumentSource::new(table_source.into(), ParsePolicy::Lenient)?;
    let line_index = temporary_source.lines();
    let arena = Arena::new();
    let opts = comrak_opts(None);
    let root = parse_document(&arena, table_source, &opts);
    reject_excessive_ast_depth(root)?;

    for node in root.children() {
        if let Some(table) = project_table_node(node, line_index, table_source, Some(block_span))? {
            return Ok(table);
        }
    }

    Err(CoreError::ParseFailed(
        "source does not contain a table".to_string(),
    ))
}

pub fn validate_table_row_payload(payload: &str, expected_columns: usize) -> Result<(), CoreError> {
    if payload.is_empty() {
        return Err(CoreError::InvalidTableRow(
            "table row payload must not be empty".to_string(),
        ));
    }
    if payload.contains('\n') || payload.contains('\r') {
        return Err(CoreError::InvalidTableRow(
            "table row payload must contain exactly one line".to_string(),
        ));
    }
    let (lexical_columns, has_unescaped_pipe) = count_table_row_columns(payload);
    if expected_columns > 1 && !has_unescaped_pipe {
        return Err(CoreError::InvalidTableRow(
            "table row payload must parse as exactly one GFM table data row".to_string(),
        ));
    }
    if lexical_columns != expected_columns {
        return Err(CoreError::InvalidTableRow(format!(
            "table row column count {} does not match table column count {}",
            lexical_columns, expected_columns
        )));
    }

    let headers = (0..expected_columns)
        .map(|idx| format!("c{}", idx))
        .collect::<Vec<_>>()
        .join(" | ");
    let delimiter = std::iter::repeat_n("---", expected_columns)
        .collect::<Vec<_>>()
        .join(" | ");
    let synthetic_table = format!("| {} |\n| {} |\n{}\n", headers, delimiter, payload);
    let projection = extract_table_projection(
        &synthetic_table,
        SourceSpan {
            line_start: 1,
            line_end: 1,
            byte_start: 0,
            byte_end: 0,
        },
    )?;

    if projection.rows.len() != 1 {
        return Err(CoreError::InvalidTableRow(
            "table row payload must parse as exactly one GFM table data row".to_string(),
        ));
    }

    let actual_columns = projection.rows[0].cells.len();
    if actual_columns != expected_columns {
        return Err(CoreError::InvalidTableRow(format!(
            "table row column count {} does not match table column count {}",
            actual_columns, expected_columns
        )));
    }

    Ok(())
}

#[cfg(test)]
mod projection_policy_tests {
    use super::*;

    fn sourcepos(
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Sourcepos {
        Sourcepos {
            start: comrak::nodes::LineColumn {
                line: start_line,
                column: start_column,
            },
            end: comrak::nodes::LineColumn {
                line: end_line,
                column: end_column,
            },
        }
    }

    #[test]
    fn unclassified_parser_nodes_fail_instead_of_becoming_paragraphs() {
        let error = node_value_to_block_kind(&NodeValue::Text("inline".into())).unwrap_err();
        assert_eq!(
            error,
            CoreError::ParseFailed("unclassified top-level parser node: Text".into())
        );
    }

    #[test]
    fn flat_sources_larger_than_one_mib_are_accepted() {
        let source =
            DocumentSource::new("x".repeat(1024 * 1024 + 1), ParsePolicy::Lenient).unwrap();
        let facts = ParsedFacts::parse(&source).unwrap();
        assert_eq!(source.len(), 1024 * 1024 + 1);
        assert_eq!(facts.blocks.len(), 1);
    }

    #[test]
    fn synthetic_table_projection_applies_ast_depth_limit() {
        let emphasis = "*".repeat(5_000);
        let payload = format!("| {emphasis}nested{emphasis} |");
        assert!(matches!(
            validate_table_row_payload(&payload, 1),
            Err(CoreError::ParseFailed(message)) if message.contains("AST exceeds")
        ));
    }

    #[test]
    fn parser_positions_fail_closed_instead_of_widening_spans() {
        let source = DocumentSource::new("one\n世界".into(), ParsePolicy::Lenient).unwrap();
        for invalid in [
            sourcepos(0, 1, 1, 1),
            sourcepos(1, 0, 1, 1),
            sourcepos(3, 1, 3, 1),
            sourcepos(2, 7, 2, 7),
            sourcepos(2, 1, 1, 1),
        ] {
            assert!(matches!(
                sourcepos_to_span(source.lines(), source.text(), invalid),
                Err(CoreError::InvalidSourcePosition { .. })
            ));
        }
    }

    #[test]
    fn parser_positions_reject_non_utf8_boundaries() {
        let source = DocumentSource::new("é".into(), ParsePolicy::Lenient).unwrap();
        assert!(matches!(
            sourcepos_to_span(source.lines(), source.text(), sourcepos(1, 2, 1, 2)),
            Err(CoreError::InvalidSourcePosition { reason, .. })
                if reason.contains("UTF-8")
        ));
    }

    #[test]
    fn zero_column_end_uses_the_start_of_its_sentinel_line() {
        let source = DocumentSource::new("one\n\n".into(), ParsePolicy::Lenient).unwrap();
        assert_eq!(
            sourcepos_to_span(source.lines(), source.text(), sourcepos(1, 1, 2, 0)).unwrap(),
            SourceSpan {
                line_start: 1,
                line_end: 1,
                byte_start: 0,
                byte_end: 4,
            }
        );
    }
}
