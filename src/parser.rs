// Parser boundary: all comrak interaction is contained here.
// The rest of the codebase sees only model types, never comrak types.

use comrak::{
    nodes::{AstNode, NodeValue, Sourcepos},
    parse_document, Arena, Options,
};

use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;

// --- Line-start index for byte offset derivation ---

/// Byte offset of the start of each 1-based line.
/// `line_starts[0]` = byte offset of line 1 (always 0).
struct LineIndex {
    starts: Vec<usize>,
    source_len: usize,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self {
            starts,
            source_len: source.len(),
        }
    }

    /// Convert comrak's 1-based line + 1-based column to 0-based byte offset.
    fn to_byte(&self, line: usize, col: usize) -> Option<usize> {
        if line == 0 || col == 0 {
            return None;
        }
        let idx = line - 1;
        if idx >= self.starts.len() {
            return None;
        }
        Some(self.starts[idx] + col - 1)
    }

    /// Convert comrak's inclusive end position to an exclusive byte offset.
    fn to_byte_end(&self, line: usize, col: usize) -> Option<usize> {
        if line == 0 || col == 0 {
            return None;
        }
        let idx = line - 1;
        if idx >= self.starts.len() {
            return None;
        }
        Some((self.starts[idx] + col).min(self.source_len))
    }

    fn sourcepos_to_span(&self, sp: Sourcepos) -> SourceSpan {
        let byte_start = self.to_byte(sp.start.line, sp.start.column).unwrap_or(0) as u32;
        let byte_end = self
            .to_byte_end(sp.end.line, sp.end.column)
            .unwrap_or(self.source_len) as u32;
        SourceSpan {
            line_start: sp.start.line as u32,
            line_end: sp.end.line as u32,
            byte_start,
            byte_end,
        }
    }

    fn frontmatter_sourcepos_to_span(&self, sp: Sourcepos, source: &str) -> SourceSpan {
        let mut span = self.sourcepos_to_span(sp);
        let suffix = &source[span.byte_end as usize..];
        if suffix.starts_with("\r\n") {
            span.byte_end += 2;
        } else if suffix.starts_with('\n') {
            span.byte_end += 1;
        }
        span
    }

    /// Fix parser-reported spans where the exact source-owned block starts earlier
    /// than comrak's position metadata indicates.
    fn sourcepos_to_span_fixup(
        &self,
        sp: Sourcepos,
        source: &str,
        is_indented_code: bool,
        heading_line: Option<usize>,
    ) -> SourceSpan {
        let mut span = if is_indented_code {
            // For indented code blocks, comrak reports start.column=5 (after the
            // 4-space indent) and end.column=0 (sentinel for blank-line termination).
            let byte_start = self.to_byte(sp.start.line, 1).unwrap_or(0) as u32;
            let byte_end = if sp.end.column == 0 && sp.end.line > 1 {
                let prev_line = sp.end.line - 1;
                let prev_idx = prev_line - 1;
                if prev_idx + 1 < self.starts.len() {
                    (self.starts[prev_idx + 1]).min(self.source_len) as u32
                } else {
                    self.source_len as u32
                }
            } else {
                self.to_byte_end(sp.end.line, sp.end.column)
                    .unwrap_or(self.source_len) as u32
            };

            SourceSpan {
                line_start: sp.start.line as u32,
                line_end: if sp.end.column == 0 && sp.end.line > 1 {
                    (sp.end.line - 1) as u32
                } else {
                    sp.end.line as u32
                },
                byte_start,
                byte_end,
            }
        } else {
            self.sourcepos_to_span(sp)
        };

        if let Some(line) = heading_line {
            if let Some(byte_start) = self.heading_line_start(source, line) {
                span.byte_start = byte_start;
            }
        }

        span
    }

    fn heading_line_start(&self, source: &str, line: usize) -> Option<u32> {
        let line_start = self.to_byte(line, 1)?;
        let bytes = source.as_bytes();
        if line_start >= bytes.len() {
            return None;
        }
        Some(line_start as u32)
    }

    fn line_count(&self) -> u32 {
        self.starts.len() as u32
    }

    /// Find the 1-based line number for a byte offset using binary search.
    fn byte_to_line(&self, byte_offset: usize) -> usize {
        match self.starts.binary_search(&byte_offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx, // idx is the line whose start is > byte_offset, so we're on line idx
        }
    }
}

// --- Comrak options ---

fn comrak_opts(delimiter: Option<&str>) -> Options<'static> {
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
fn detect_frontmatter_delimiter(source: &str) -> Option<&'static str> {
    let first_line = source.lines().next().unwrap_or("");
    if first_line == "---" {
        Some("---")
    } else if first_line == "+++" {
        Some("+++")
    } else {
        None
    }
}

// --- Parsed document projection ---

/// Fully projected document — no comrak types escape this boundary.
pub struct ParsedDocument {
    pub source: String,
    pub blocks: Vec<BlockInfo>,
    pub frontmatter: Option<FrontmatterInfo>,
    line_index: LineIndex,
}

/// A projected top-level block.
pub struct BlockInfo {
    pub index: u32,
    pub kind: BlockKind,
    pub span: SourceSpan,
    pub heading: Option<HeadingInfo>,
    pub links: Vec<LinkInfo>,
    pub task_items: Vec<TaskItemInfo>,
}

pub struct TaskItemInfo {
    pub child_path: Vec<u32>,
    pub task_index: u32,
    pub status: TaskStatus,
    pub depth: u32,
    pub span: SourceSpan,
    pub symbol_byte_offset: u32,
    pub summary_text: String,
}

pub struct HeadingInfo {
    pub level: u8,
    pub text: String,
    pub kind: HeadingSourceKind,
    /// Byte span covering only the `#` run for ATX headings. None for setext.
    pub marker_span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingSourceKind {
    Atx,
    Setext,
}

pub struct LinkInfo {
    pub kind: LinkKind,
    pub text: String,
    pub destination: Option<String>,
    pub title: Option<String>,
    pub span: SourceSpan,
}

pub struct FrontmatterInfo {
    pub raw: String,
    pub span: SourceSpan,
    pub format: FrontmatterFormat,
}

pub struct FrontmatterState<'a> {
    pub span: Option<SourceSpan>,
    pub raw: Option<&'a str>,
    pub format: Option<FrontmatterFormat>,
    pub etag: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrontmatterParseMode {
    Lenient,
    StrictRead,
    Mutation,
}

impl ParsedDocument {
    pub fn parse(source: String) -> Result<Self, CommandError> {
        Self::parse_inner(source, FrontmatterParseMode::Lenient)
    }

    /// Parse specifically for the frontmatter command, which should error on malformed frontmatter
    /// rather than falling back to treating it as plain content.
    pub fn parse_for_frontmatter(source: String) -> Result<Self, CommandError> {
        Self::parse_inner(source, FrontmatterParseMode::StrictRead)
    }

    /// Parse for frontmatter mutation commands, which must fail closed on malformed
    /// or unclosed leading frontmatter instead of treating it as absent.
    pub fn parse_for_frontmatter_mutation(source: String) -> Result<Self, CommandError> {
        Self::parse_inner(source, FrontmatterParseMode::Mutation)
    }

    fn parse_inner(source: String, mode: FrontmatterParseMode) -> Result<Self, CommandError> {
        let delimiter = detect_frontmatter_delimiter(&source);
        let line_index = LineIndex::new(&source);
        let opts = comrak_opts(delimiter);
        let arena = Arena::new();
        let root = parse_document(&arena, &source, &opts);

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

        if matches!(mode, FrontmatterParseMode::Mutation)
            && delimiter.is_some()
            && !has_frontmatter_node
        {
            let delimiter = delimiter.expect("checked is_some above");
            return Err(CommandError::new(
                DiagnosticCode::FrontmatterParseFailed,
                format!("unclosed frontmatter (no closing '{}')", delimiter),
            ));
        }

        // If frontmatter exists, validate it
        if has_frontmatter_node {
            if let Some(ref raw) = frontmatter_raw {
                let content = strip_frontmatter_delimiters(raw);
                let valid = if content.trim().is_empty() {
                    true
                } else {
                    match frontmatter_format {
                        FrontmatterFormat::Yaml => {
                            serde_yaml::from_str::<serde_json::Value>(&content).is_ok()
                        }
                        FrontmatterFormat::Toml => content.parse::<toml::Value>().is_ok(),
                    }
                };
                if !valid && matches!(mode, FrontmatterParseMode::Lenient) {
                    // Re-parse without frontmatter delimiter — treat as plain content
                    let _ = root;
                    return Self::parse_without_frontmatter(source);
                }
            }
        }

        let mut blocks = Vec::new();
        let mut frontmatter = None;

        for node in root.children() {
            let data = node.data.borrow();
            let sp = data.sourcepos;

            match &data.value {
                NodeValue::FrontMatter(_) => {
                    let fm_span = line_index.frontmatter_sourcepos_to_span(sp, &source);
                    let fm_raw =
                        source[fm_span.byte_start as usize..fm_span.byte_end as usize].to_string();
                    frontmatter = Some(FrontmatterInfo {
                        raw: fm_raw,
                        span: fm_span,
                        format: frontmatter_format,
                    });
                    // Frontmatter is NOT a block — no index increment
                }
                _ => {
                    let heading_meta = if let NodeValue::Heading(h) = &data.value {
                        Some((h.level, h.setext, sp.start.line))
                    } else {
                        None
                    };
                    let kind = node_value_to_block_kind(&data.value);
                    let is_indented = matches!(kind, BlockKind::IndentedCode);
                    let heading_line = heading_meta.map(|(_, _, line)| line);
                    let span =
                        line_index.sourcepos_to_span_fixup(sp, &source, is_indented, heading_line);
                    drop(data);

                    let heading = heading_meta.map(|(level, setext, line)| {
                        let text = collect_heading_text(node);
                        let kind = if setext {
                            HeadingSourceKind::Setext
                        } else {
                            HeadingSourceKind::Atx
                        };
                        let marker_span = if setext {
                            None
                        } else {
                            compute_atx_marker_span(&line_index, &source, line)
                        };
                        HeadingInfo {
                            level,
                            text,
                            kind,
                            marker_span,
                        }
                    });

                    let links = collect_links(node, &line_index, &source);
                    let task_items = collect_all_task_items(node, &line_index);

                    blocks.push(BlockInfo {
                        index: blocks.len() as u32,
                        kind,
                        span,
                        heading,
                        links,
                        task_items,
                    });
                }
            }
        }

        Ok(ParsedDocument {
            source,
            blocks,
            frontmatter,
            line_index,
        })
    }

    fn parse_without_frontmatter(source: String) -> Result<Self, CommandError> {
        let line_index = LineIndex::new(&source);
        let opts = comrak_opts(None); // No frontmatter delimiter
        let arena = Arena::new();
        let root = parse_document(&arena, &source, &opts);

        let mut blocks = Vec::new();

        for (block_index, node) in root.children().enumerate() {
            let data = node.data.borrow();
            let sp = data.sourcepos;
            let heading_meta = if let NodeValue::Heading(h) = &data.value {
                Some((h.level, h.setext, sp.start.line))
            } else {
                None
            };
            let kind = node_value_to_block_kind(&data.value);
            let is_indented = matches!(kind, BlockKind::IndentedCode);
            let heading_line = heading_meta.map(|(_, _, line)| line);
            let span = line_index.sourcepos_to_span_fixup(sp, &source, is_indented, heading_line);
            drop(data);

            let heading = heading_meta.map(|(level, setext, line)| {
                let text = collect_heading_text(node);
                let kind = if setext {
                    HeadingSourceKind::Setext
                } else {
                    HeadingSourceKind::Atx
                };
                let marker_span = if setext {
                    None
                } else {
                    compute_atx_marker_span(&line_index, &source, line)
                };
                HeadingInfo {
                    level,
                    text,
                    kind,
                    marker_span,
                }
            });

            let links = collect_links(node, &line_index, &source);
            let task_items = collect_all_task_items(node, &line_index);

            blocks.push(BlockInfo {
                index: block_index as u32,
                kind,
                span,
                heading,
                links,
                task_items,
            });
        }

        Ok(ParsedDocument {
            source,
            blocks,
            frontmatter: None,
            line_index,
        })
    }

    pub fn line_count(&self) -> u32 {
        self.line_index.line_count()
    }

    /// Find the line number for a given byte offset.
    pub fn byte_to_line(&self, byte_offset: u32) -> u32 {
        self.line_index.byte_to_line(byte_offset as usize) as u32
    }

    pub fn span_for_byte_range(&self, byte_start: u32, byte_end: u32) -> SourceSpan {
        let line_start = self.byte_to_line(byte_start);
        let line_end = if byte_end > byte_start {
            self.byte_to_line(byte_end - 1)
        } else {
            line_start
        };
        SourceSpan {
            line_start,
            line_end,
            byte_start,
            byte_end,
        }
    }

    /// Extract the source text for a given span.
    pub fn slice(&self, span: &SourceSpan) -> &str {
        &self.source[span.byte_start as usize..span.byte_end as usize]
    }

    pub fn frontmatter_state(&self) -> FrontmatterState<'_> {
        match &self.frontmatter {
            Some(frontmatter) => {
                let raw = self.slice(&frontmatter.span);
                FrontmatterState {
                    span: Some(frontmatter.span),
                    raw: Some(raw),
                    format: Some(frontmatter.format),
                    etag: frontmatter_state_etag(Some(raw)),
                }
            }
            None => FrontmatterState {
                span: None,
                raw: None,
                format: None,
                etag: frontmatter_state_etag(None),
            },
        }
    }

    /// Detect the document's line ending style.
    pub fn line_ending_style(&self) -> LineEndingStyle {
        let has_crlf = self.source.contains("\r\n");
        let has_bare_lf = self
            .source
            .bytes()
            .enumerate()
            .any(|(i, b)| b == b'\n' && (i == 0 || self.source.as_bytes()[i - 1] != b'\r'));
        match (has_crlf, has_bare_lf) {
            (true, false) => LineEndingStyle::Crlf,
            (false, true) | (false, false) => LineEndingStyle::Lf,
            (true, true) => LineEndingStyle::Mixed,
        }
    }
}

fn frontmatter_state_etag(raw: Option<&str>) -> String {
    const ABSENT_DOMAIN: &[u8] = b"mdtools.frontmatter.absent";
    const PRESENT_DOMAIN: &[u8] = b"mdtools.frontmatter.present\0";

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let bytes = raw.map(str::as_bytes);
    let domain = if bytes.is_some() {
        PRESENT_DOMAIN
    } else {
        ABSENT_DOMAIN
    };

    for &b in domain {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    if let Some(bytes) = bytes {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    format!("{:016x}", hash)
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
) -> Option<SourceSpan> {
    let line_start_byte = line_index.to_byte(line, 1)?;
    let bytes = source.as_bytes();
    if line_start_byte > bytes.len() {
        return None;
    }
    let mut p = line_start_byte;
    let indent_limit = (line_start_byte + 3).min(bytes.len());
    while p < indent_limit && bytes[p] == b' ' {
        p += 1;
    }
    if p >= bytes.len() || bytes[p] != b'#' {
        return None;
    }
    let marker_start = p;
    while p < bytes.len() && bytes[p] == b'#' {
        p += 1;
    }
    Some(SourceSpan {
        line_start: line as u32,
        line_end: line as u32,
        byte_start: marker_start as u32,
        byte_end: p as u32,
    })
}

// --- Task item extraction ---

/// Recursively collect task items from a list node.
fn collect_task_items<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    prefix: &[u32],
    depth: u32,
) -> Vec<TaskItemInfo> {
    let mut items = Vec::new();
    let mut task_counter = 0u32;

    for (child_counter, child) in node.children().enumerate() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::TaskItem(task) => {
                let sp = data.sourcepos;
                let span = line_index.sourcepos_to_span(sp);
                let symbol_byte_offset = line_index
                    .to_byte(
                        task.symbol_sourcepos.start.line,
                        task.symbol_sourcepos.start.column,
                    )
                    .unwrap_or(0) as u32;
                let status = if task.symbol.is_some() {
                    TaskStatus::Done
                } else {
                    TaskStatus::Pending
                };
                drop(data);

                let mut path = prefix.to_vec();
                path.push(child_counter as u32);

                let summary_text = collect_task_item_text(child);

                items.push(TaskItemInfo {
                    child_path: path.clone(),
                    task_index: task_counter,
                    status,
                    depth,
                    span,
                    symbol_byte_offset,
                    summary_text,
                });

                // Recurse into nested lists within this task item
                for grandchild in child.children() {
                    let gc_data = grandchild.data.borrow();
                    if matches!(gc_data.value, NodeValue::List(_)) {
                        drop(gc_data);
                        items.extend(collect_task_items(grandchild, line_index, &path, depth + 1));
                    }
                }

                task_counter += 1;
            }
            NodeValue::Item(_) => {
                drop(data);

                let mut path = prefix.to_vec();
                path.push(child_counter as u32);

                // Regular items can contain nested lists with task items
                for grandchild in child.children() {
                    let gc_data = grandchild.data.borrow();
                    if matches!(gc_data.value, NodeValue::List(_)) {
                        drop(gc_data);
                        items.extend(collect_task_items(grandchild, line_index, &path, depth + 1));
                    }
                }
            }
            _ => {
                drop(data);
            }
        }
    }

    items
}

/// Find all task items under any block node by walking descendants for List nodes.
/// Handles task lists inside blockquotes, callouts, and other containers.
fn collect_all_task_items<'a>(node: &'a AstNode<'a>, line_index: &LineIndex) -> Vec<TaskItemInfo> {
    let data = node.data.borrow();
    if matches!(data.value, NodeValue::List(_)) {
        drop(data);
        return collect_task_items(node, line_index, &[], 0);
    }
    drop(data);

    // Walk children looking for List descendants (e.g. inside BlockQuote).
    // Use a counter so sibling lists inside the same container get distinct
    // path prefixes (e.g. first list → prefix [0], second → prefix [1]).
    let mut items = Vec::new();
    let mut list_counter = 0u32;
    find_list_descendants(node, line_index, &mut items, &mut list_counter);
    items
}

fn find_list_descendants<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    out: &mut Vec<TaskItemInfo>,
    list_counter: &mut u32,
) {
    for child in node.children() {
        let data = child.data.borrow();
        if matches!(data.value, NodeValue::List(_)) {
            drop(data);
            let prefix = [*list_counter];
            out.extend(collect_task_items(child, line_index, &prefix, 0));
            *list_counter += 1;
        } else {
            drop(data);
            find_list_descendants(child, line_index, out, list_counter);
        }
    }
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

fn node_value_to_block_kind(value: &NodeValue) -> BlockKind {
    match value {
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
        // Anything else at top level is unexpected — treat as paragraph
        _ => BlockKind::Paragraph,
    }
}

/// Collect plaintext heading content by walking inline children.
fn collect_heading_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    collect_text_recursive(node, &mut text);
    text
}

fn collect_text_recursive<'a>(node: &'a AstNode<'a>, out: &mut String) {
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => out.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
            _ => {
                drop(data);
                collect_text_recursive(child, out);
            }
        }
    }
}

/// Collect links from a block node by walking all descendants.
fn collect_links<'a>(node: &'a AstNode<'a>, line_index: &LineIndex, source: &str) -> Vec<LinkInfo> {
    let mut links = Vec::new();
    collect_links_recursive(node, line_index, source, &mut links);
    links
}

fn collect_links_recursive<'a>(
    node: &'a AstNode<'a>,
    line_index: &LineIndex,
    source: &str,
    out: &mut Vec<LinkInfo>,
) {
    for child in node.descendants() {
        let data = child.data.borrow();
        if let NodeValue::Link(link) = &data.value {
            let sp = data.sourcepos;
            let span = line_index.sourcepos_to_span(sp);
            let url = link.url.clone();
            let title = link.title.clone();
            drop(data);

            let mut text = String::new();
            collect_text_recursive(child, &mut text);

            let kind = classify_link_kind(source, &span, &url);

            out.push(LinkInfo {
                kind,
                text,
                destination: if url.is_empty() { None } else { Some(url) },
                title: if title.is_empty() { None } else { Some(title) },
                span,
            });
        }
    }
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

/// Find the index of the ] that closes the link text in [text](url) or [text][ref].
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
pub struct ProjectedTableRow {
    pub cells: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub struct TableProjection {
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<ProjectedTableRow>,
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
) -> Result<TableProjection, CommandError> {
    use comrak::nodes::TableAlignment;

    let line_index = LineIndex::new(table_source);
    let arena = Arena::new();
    let opts = comrak_opts(None);
    let root = parse_document(&arena, table_source, &opts);

    for node in root.children() {
        let data = node.data.borrow();
        if let NodeValue::Table(ref table_meta) = data.value {
            let alignments: Vec<ColumnAlignment> = table_meta
                .alignments
                .iter()
                .map(|a| match a {
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
                if let NodeValue::TableRow(is_header) = row_data.value {
                    let row_span =
                        offset_span(line_index.sourcepos_to_span(row_data.sourcepos), block_span);
                    drop(row_data);

                    let mut cells = Vec::new();
                    for cell_node in row_node.children() {
                        let mut text = String::new();
                        collect_text_recursive(cell_node, &mut text);
                        cells.push(text.trim().to_string());
                    }

                    if is_header {
                        headers = cells;
                    } else {
                        rows.push(ProjectedTableRow {
                            cells,
                            span: row_span,
                        });
                    }
                }
            }

            return Ok(TableProjection {
                headers,
                alignments,
                rows,
            });
        }
    }

    Err(CommandError::new(
        crate::errors::DiagnosticCode::ParseFailed,
        "source does not contain a table",
    ))
}

pub fn validate_table_row_payload(
    payload: &str,
    expected_columns: usize,
) -> Result<(), CommandError> {
    if payload.is_empty() {
        return Err(CommandError::invalid_table_row(
            "replacement row payload must not be empty",
        ));
    }
    if payload.contains('\n') || payload.contains('\r') {
        return Err(CommandError::invalid_table_row(
            "replacement row payload must contain exactly one line",
        ));
    }
    let (lexical_columns, has_unescaped_pipe) = count_table_row_columns(payload);
    if expected_columns > 1 && !has_unescaped_pipe {
        return Err(CommandError::invalid_table_row(
            "replacement row payload must parse as exactly one GFM table data row",
        ));
    }
    if lexical_columns != expected_columns {
        return Err(CommandError::invalid_table_row(format!(
            "replacement row column count {} does not match table column count {}",
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
    let projection = extract_table_data(&synthetic_table)?;

    if projection.rows.len() != 1 {
        return Err(CommandError::invalid_table_row(
            "replacement row payload must parse as exactly one GFM table data row",
        ));
    }

    let actual_columns = projection.rows[0].len();
    if actual_columns != expected_columns {
        return Err(CommandError::invalid_table_row(format!(
            "replacement row column count {} does not match table column count {}",
            actual_columns, expected_columns
        )));
    }

    Ok(())
}

/// Extract structured table data from a markdown source fragment
/// containing a single table. The source should be the sliced text
/// of a Table block (obtained via ParsedDocument::slice()).
pub fn extract_table_data(table_source: &str) -> Result<TableData, CommandError> {
    let projection = extract_table_projection(
        table_source,
        SourceSpan {
            line_start: 1,
            line_end: 1,
            byte_start: 0,
            byte_end: 0,
        },
    )?;

    Ok(TableData {
        headers: projection.headers,
        alignments: projection.alignments,
        rows: projection.rows.into_iter().map(|row| row.cells).collect(),
    })
}
