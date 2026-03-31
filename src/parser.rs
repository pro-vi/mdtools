// Parser boundary: all comrak interaction is contained here.
// The rest of the codebase sees only model types, never comrak types.

use comrak::{
    nodes::{AstNode, NodeValue, Sourcepos},
    parse_document, Arena, Options,
};

use crate::errors::CommandError;
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
        let byte_end = self.to_byte_end(sp.end.line, sp.end.column)
            .unwrap_or(self.source_len) as u32;
        SourceSpan {
            line_start: sp.start.line as u32,
            line_end: sp.end.line as u32,
            byte_start,
            byte_end,
        }
    }

    /// For indented code blocks, comrak reports start.column=5 (after the 4-space indent)
    /// and end.column=0 (sentinel for blank-line termination). Fix both.
    fn sourcepos_to_span_fixup(&self, sp: Sourcepos, is_indented_code: bool) -> SourceSpan {
        if !is_indented_code {
            return self.sourcepos_to_span(sp);
        }

        // Fix start: use column 1 (beginning of line, including indent)
        let byte_start = self.to_byte(sp.start.line, 1).unwrap_or(0) as u32;

        // Fix end: if end.column == 0, use end of (end.line - 1)
        let byte_end = if sp.end.column == 0 && sp.end.line > 1 {
            // Find end of the previous line
            let prev_line = sp.end.line - 1;
            let prev_idx = prev_line - 1;
            if prev_idx + 1 < self.starts.len() {
                // End of prev_line = start of next line - 1 (the \n)
                // But we want to include the content up to the \n
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
pub(crate) fn strip_frontmatter_delimiters(raw: &str) -> String {
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
}

pub struct HeadingInfo {
    pub level: u8,
    pub text: String,
    pub setext: bool,
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

impl ParsedDocument {
    pub fn parse(source: String) -> Result<Self, CommandError> {
        Self::parse_inner(source, false)
    }

    /// Parse specifically for the frontmatter command, which should error on malformed frontmatter
    /// rather than falling back to treating it as plain content.
    pub fn parse_for_frontmatter(source: String) -> Result<Self, CommandError> {
        Self::parse_inner(source, true)
    }

    fn parse_inner(source: String, strict_frontmatter: bool) -> Result<Self, CommandError> {
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

        // If frontmatter exists, validate it
        if has_frontmatter_node {
            if let Some(ref raw) = frontmatter_raw {
                let content = strip_frontmatter_delimiters(raw);
                let valid = match frontmatter_format {
                    FrontmatterFormat::Yaml => {
                        serde_yaml::from_str::<serde_json::Value>(&content).is_ok()
                    }
                    FrontmatterFormat::Toml => content.parse::<toml::Value>().is_ok(),
                };
                if !valid && !strict_frontmatter {
                    // Re-parse without frontmatter delimiter — treat as plain content
                    let _ = root;
                    return Self::parse_without_frontmatter(source);
                }
            }
        }

        let mut blocks = Vec::new();
        let mut frontmatter = None;
        let mut block_index = 0u32;

        for node in root.children() {
            let data = node.data.borrow();
            let sp = data.sourcepos;

            match &data.value {
                NodeValue::FrontMatter(raw) => {
                    let fm_span = line_index.sourcepos_to_span(sp);
                    frontmatter = Some(FrontmatterInfo {
                        raw: raw.clone(),
                        span: fm_span,
                        format: frontmatter_format,
                    });
                    // Frontmatter is NOT a block — no index increment
                }
                _ => {
                    let kind = node_value_to_block_kind(&data.value);
                    let is_indented = matches!(kind, BlockKind::IndentedCode);
                    let span = line_index.sourcepos_to_span_fixup(sp, is_indented);

                    // Extract heading metadata while data is borrowed
                    let heading_meta = if let NodeValue::Heading(h) = &data.value {
                        Some((h.level, h.setext))
                    } else {
                        None
                    };
                    drop(data);

                    let heading = heading_meta.map(|(level, setext)| {
                        let text = collect_heading_text(node);
                        HeadingInfo { level, text, setext }
                    });

                    let links = collect_links(node, &line_index, &source);

                    blocks.push(BlockInfo {
                        index: block_index,
                        kind,
                        span,
                        heading,
                        links,
                    });
                    block_index += 1;
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
        let mut block_index = 0u32;

        for node in root.children() {
            let data = node.data.borrow();
            let sp = data.sourcepos;
            let kind = node_value_to_block_kind(&data.value);
            let is_indented = matches!(kind, BlockKind::IndentedCode);
            let span = line_index.sourcepos_to_span_fixup(sp, is_indented);

            let heading_meta = if let NodeValue::Heading(h) = &data.value {
                Some((h.level, h.setext))
            } else {
                None
            };
            drop(data);

            let heading = heading_meta.map(|(level, setext)| {
                let text = collect_heading_text(node);
                HeadingInfo { level, text, setext }
            });

            let links = collect_links(node, &line_index, &source);

            blocks.push(BlockInfo {
                index: block_index,
                kind,
                span,
                heading,
                links,
            });
            block_index += 1;
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

    /// Extract the source text for a given span.
    pub fn slice(&self, span: &SourceSpan) -> &str {
        &self.source[span.byte_start as usize..span.byte_end as usize]
    }

    /// Detect the document's line ending style.
    pub fn line_ending_style(&self) -> LineEndingStyle {
        let has_crlf = self.source.contains("\r\n");
        let has_bare_lf = self.source.bytes().enumerate().any(|(i, b)| {
            b == b'\n' && (i == 0 || self.source.as_bytes()[i - 1] != b'\r')
        });
        match (has_crlf, has_bare_lf) {
            (true, false) => LineEndingStyle::Crlf,
            (false, true) | (false, false) => LineEndingStyle::Lf,
            (true, true) => LineEndingStyle::Mixed,
        }
    }
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

/// Extract structured table data from a markdown source fragment
/// containing a single table. The source should be the sliced text
/// of a Table block (obtained via ParsedDocument::slice()).
pub fn extract_table_data(
    table_source: &str,
) -> Result<TableData, CommandError> {
    use comrak::nodes::TableAlignment;

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
                    let is_header = is_header;
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
                        rows.push(cells);
                    }
                }
            }

            return Ok(TableData {
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
