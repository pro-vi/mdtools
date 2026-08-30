//! Immutable, source-ordered structural index for one Markdown document.
//!
//! Parser traversal order is deliberately not observable here. Comrak can move
//! footnote definitions away from their source position, while structural
//! ownership and later target addresses must follow the source bytes.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::model::{
    BlockKind, ColumnAlignment, FrontmatterFormat, LinkKind, SourceSpan, TaskStatus,
};
use crate::parser::{HeadingSourceKind, ParsedDocument, TableProjection};
use crate::target::TargetAddress;

/// Public structural kinds without exposing the index's internal node identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IndexNodeKind {
    Document,
    Frontmatter,
    Preamble,
    Section,
    Heading,
    HeadingMarker,
    BodyBlock,
    TaskItem,
    TableRow,
    Link,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct IndexNodeId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IndexInstanceId(u64);

static NEXT_INDEX_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub(crate) struct IndexEntry {
    pub(crate) id: IndexNodeId,
    pub(crate) parent: Option<IndexNodeId>,
    pub(crate) node: IndexNode,
}

#[derive(Clone, Debug)]
pub(crate) enum IndexNode {
    Document {
        span: SourceSpan,
    },
    Frontmatter {
        span: SourceSpan,
        format: FrontmatterFormat,
    },
    Preamble {
        span: SourceSpan,
    },
    Section {
        span: SourceSpan,
        heading_span: SourceSpan,
        level: u8,
        text: String,
    },
    Heading {
        span: SourceSpan,
        parser_index: u32,
        level: u8,
        text: String,
    },
    HeadingMarker {
        span: SourceSpan,
        level: u8,
        source_kind: HeadingSourceKind,
    },
    BodyBlock {
        span: SourceSpan,
        ordinal: u32,
        parser_index: u32,
        kind: BlockKind,
        table: Option<IndexedTable>,
    },
    TaskItem {
        span: SourceSpan,
        child_path: Vec<u32>,
        task_index: u32,
        status: TaskStatus,
        depth: u32,
        symbol_byte_offset: u32,
        summary_text: String,
    },
    TableRow {
        span: SourceSpan,
        ordinal: u32,
        cells: Vec<String>,
    },
    Link {
        span: SourceSpan,
        occurrence: u32,
        kind: LinkKind,
        text: String,
        destination: Option<String>,
        title: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct IndexedTable {
    pub(crate) headers: Vec<String>,
    pub(crate) alignments: Vec<ColumnAlignment>,
}

impl IndexNode {
    fn kind(&self) -> IndexNodeKind {
        match self {
            Self::Document { .. } => IndexNodeKind::Document,
            Self::Frontmatter { .. } => IndexNodeKind::Frontmatter,
            Self::Preamble { .. } => IndexNodeKind::Preamble,
            Self::Section { .. } => IndexNodeKind::Section,
            Self::Heading { .. } => IndexNodeKind::Heading,
            Self::HeadingMarker { .. } => IndexNodeKind::HeadingMarker,
            Self::BodyBlock { .. } => IndexNodeKind::BodyBlock,
            Self::TaskItem { .. } => IndexNodeKind::TaskItem,
            Self::TableRow { .. } => IndexNodeKind::TableRow,
            Self::Link { .. } => IndexNodeKind::Link,
        }
    }

    pub(crate) fn span(&self) -> SourceSpan {
        match self {
            Self::Document { span }
            | Self::Frontmatter { span, .. }
            | Self::Preamble { span }
            | Self::Section { span, .. }
            | Self::Heading { span, .. }
            | Self::HeadingMarker { span, .. }
            | Self::BodyBlock { span, .. }
            | Self::TaskItem { span, .. }
            | Self::TableRow { span, .. }
            | Self::Link { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug)]
struct SectionSpec {
    parent: Option<usize>,
    parser_index: u32,
    level: u8,
    text: String,
    heading_span: SourceSpan,
    marker_span: SourceSpan,
    source_kind: HeadingSourceKind,
    byte_end: u32,
}

/// One immutable index built with the parser projection and owned by [`Document`](crate::document::Document).
pub struct DocumentIndex {
    instance_id: IndexInstanceId,
    projection: ParsedDocument,
    nodes: Vec<IndexEntry>,
    source_order: Vec<IndexNodeId>,
    children_by_parent: HashMap<IndexNodeId, Vec<IndexNodeId>>,
    address_by_node: HashMap<IndexNodeId, TargetAddress>,
    node_by_address: HashMap<TargetAddress, IndexNodeId>,
    root: IndexNodeId,
}

impl DocumentIndex {
    pub(crate) fn build(projection: ParsedDocument) -> Self {
        let mut builder = IndexBuilder::default();
        let document_span = SourceSpan {
            line_start: 1,
            line_end: projection.line_count(),
            byte_start: 0,
            byte_end: projection.source.len() as u32,
        };
        let root = builder.push(
            None,
            IndexNode::Document {
                span: document_span,
            },
        );

        if let Some(frontmatter) = &projection.frontmatter {
            builder.push(
                Some(root),
                IndexNode::Frontmatter {
                    span: frontmatter.span,
                    format: frontmatter.format,
                },
            );
        }

        let mut block_order = (0..projection.blocks.len()).collect::<Vec<_>>();
        block_order.sort_by_key(|position| {
            let block = &projection.blocks[*position];
            (block.span.byte_start, block.span.byte_end, block.index)
        });

        let mut sections = Vec::<SectionSpec>::new();
        let mut section_stack = Vec::<usize>::new();
        let mut owner_by_block = vec![None; projection.blocks.len()];
        let mut section_by_heading_block = HashMap::<u32, usize>::new();

        for position in &block_order {
            let block = &projection.blocks[*position];
            let Some(heading) = &block.heading else {
                owner_by_block[*position] = section_stack.last().copied();
                continue;
            };

            while section_stack
                .last()
                .is_some_and(|section| sections[*section].level >= heading.level)
            {
                section_stack.pop();
            }
            let section_index = sections.len();
            sections.push(SectionSpec {
                parent: section_stack.last().copied(),
                parser_index: block.index,
                level: heading.level,
                text: heading.text.clone(),
                heading_span: block.span,
                marker_span: heading.marker_span,
                source_kind: heading.kind,
                byte_end: projection.source.len() as u32,
            });
            section_by_heading_block.insert(block.index, section_index);
            section_stack.push(section_index);
        }

        for section_index in 0..sections.len() {
            let level = sections[section_index].level;
            if let Some(next) = sections[section_index + 1..]
                .iter()
                .find(|candidate| candidate.level <= level)
            {
                sections[section_index].byte_end = next.heading_span.byte_start;
            }
        }

        let first_heading_start = sections
            .first()
            .map(|section| section.heading_span.byte_start)
            .unwrap_or(projection.source.len() as u32);
        let preamble_start = projection
            .frontmatter
            .as_ref()
            .map(|frontmatter| frontmatter.span.byte_end)
            .unwrap_or(0);
        let preamble_span = projection.span_for_byte_range(preamble_start, first_heading_start);
        let preamble = builder.push(
            Some(root),
            IndexNode::Preamble {
                span: preamble_span,
            },
        );

        let mut section_nodes = Vec::with_capacity(sections.len());
        let mut heading_nodes = Vec::with_capacity(sections.len());
        for section in &sections {
            let parent = section.parent.map_or(root, |parent| section_nodes[parent]);
            let span = section_span(&projection, section.heading_span, section.byte_end);
            let section_node = builder.push(
                Some(parent),
                IndexNode::Section {
                    span,
                    heading_span: section.heading_span,
                    level: section.level,
                    text: section.text.clone(),
                },
            );
            let heading_node = builder.push(
                Some(section_node),
                IndexNode::Heading {
                    span: section.heading_span,
                    parser_index: section.parser_index,
                    level: section.level,
                    text: section.text.clone(),
                },
            );
            builder.push(
                Some(heading_node),
                IndexNode::HeadingMarker {
                    span: section.marker_span,
                    level: section.level,
                    source_kind: section.source_kind,
                },
            );
            section_nodes.push(section_node);
            heading_nodes.push(heading_node);
        }

        let mut body_ordinals = HashMap::<IndexNodeId, u32>::new();
        for position in block_order {
            let block = &projection.blocks[position];
            if let Some(section_index) = section_by_heading_block.get(&block.index).copied() {
                add_links(&mut builder, heading_nodes[section_index], &block.links);
                continue;
            }

            let parent = owner_by_block[position].map_or(preamble, |owner| section_nodes[owner]);
            let ordinal = body_ordinals.entry(parent).or_default();
            let body = builder.push(
                Some(parent),
                IndexNode::BodyBlock {
                    span: block.span,
                    ordinal: *ordinal,
                    parser_index: block.index,
                    kind: block.kind,
                    table: block.table.as_ref().map(indexed_table),
                },
            );
            *ordinal += 1;

            add_tasks(&mut builder, body, &block.task_items);
            if let Some(table) = &block.table {
                for (row_index, row) in table.rows.iter().enumerate() {
                    builder.push(
                        Some(body),
                        IndexNode::TableRow {
                            span: row.span,
                            ordinal: row_index as u32,
                            cells: row.cells.clone(),
                        },
                    );
                }
            }
            add_links(&mut builder, body, &block.links);
        }

        let mut source_order = builder
            .nodes
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        source_order.sort_by(|left, right| {
            let left = &builder.nodes[left.0 as usize];
            let right = &builder.nodes[right.0 as usize];
            source_key(left).cmp(&source_key(right))
        });

        let mut children_by_parent = HashMap::<IndexNodeId, Vec<IndexNodeId>>::new();
        for entry in &builder.nodes {
            if let Some(parent) = entry.parent {
                children_by_parent.entry(parent).or_default().push(entry.id);
            }
        }

        let mut index = Self {
            instance_id: IndexInstanceId(NEXT_INDEX_INSTANCE_ID.fetch_add(1, Ordering::Relaxed)),
            projection,
            nodes: builder.nodes,
            source_order,
            children_by_parent,
            address_by_node: HashMap::new(),
            node_by_address: HashMap::new(),
            root,
        };
        let (address_by_node, node_by_address) = crate::target::build_index_addresses(&index);
        index.address_by_node = address_by_node;
        index.node_by_address = node_by_address;
        index
    }

    /// Count nodes in one declared Markdown domain.
    pub fn node_count(&self, kind: IndexNodeKind) -> usize {
        self.source_order
            .iter()
            .filter(|id| self.nodes[id.0 as usize].node.kind() == kind)
            .count()
    }

    /// Render the owned structural tree for review and fixture diagnostics.
    pub fn render_tree(&self) -> String {
        let mut rendered = String::new();
        self.render_node(self.root, 0, &mut rendered);
        rendered
    }

    pub(crate) fn projection(&self) -> &ParsedDocument {
        &self.projection
    }

    pub(crate) fn instance_id(&self) -> IndexInstanceId {
        self.instance_id
    }

    pub(crate) fn entries_in_source_order(&self) -> impl Iterator<Item = &IndexEntry> {
        self.source_order
            .iter()
            .map(|id| &self.nodes[id.0 as usize])
    }

    pub(crate) fn entry(&self, id: IndexNodeId) -> &IndexEntry {
        &self.nodes[id.0 as usize]
    }

    pub(crate) fn children(&self, parent: IndexNodeId) -> impl Iterator<Item = &IndexEntry> {
        self.children_by_parent
            .get(&parent)
            .into_iter()
            .flatten()
            .map(|id| &self.nodes[id.0 as usize])
    }

    pub(crate) fn address_for_node(&self, node: IndexNodeId) -> Option<&TargetAddress> {
        self.address_by_node.get(&node)
    }

    pub(crate) fn node_for_address(&self, address: &TargetAddress) -> Option<IndexNodeId> {
        self.node_by_address.get(address).copied()
    }

    pub(crate) fn section_span(&self, heading_span: SourceSpan, byte_end: u32) -> SourceSpan {
        section_span(&self.projection, heading_span, byte_end)
    }

    pub(crate) fn section_block_indices(&self, heading_block_index: Option<u32>) -> Vec<u32> {
        let owner = match heading_block_index {
            Some(block_index) => self.nodes.iter().find_map(|entry| {
                matches!(
                    entry.node,
                    IndexNode::Heading { parser_index, .. } if parser_index == block_index
                )
                .then_some(entry.parent)
                .flatten()
            }),
            None => self
                .nodes
                .iter()
                .find(|entry| entry.node.kind() == IndexNodeKind::Preamble)
                .map(|entry| entry.id),
        };
        let Some(owner) = owner else {
            return Vec::new();
        };

        self.source_order
            .iter()
            .filter_map(|id| {
                self.is_descendant_of(*id, owner)
                    .then(|| match self.nodes[id.0 as usize].node {
                        IndexNode::Heading { parser_index, .. }
                        | IndexNode::BodyBlock { parser_index, .. } => Some(parser_index),
                        _ => None,
                    })
                    .flatten()
            })
            .collect()
    }

    pub(crate) fn source_block_indices(&self) -> Vec<u32> {
        self.source_order
            .iter()
            .filter_map(|id| match self.nodes[id.0 as usize].node {
                IndexNode::Heading { parser_index, .. }
                | IndexNode::BodyBlock { parser_index, .. } => Some(parser_index),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn first_source_block_span(&self) -> Option<SourceSpan> {
        self.source_order.iter().find_map(|id| {
            let node = &self.nodes[id.0 as usize].node;
            matches!(
                node,
                IndexNode::Heading { .. } | IndexNode::BodyBlock { .. }
            )
            .then(|| node.span())
        })
    }

    #[cfg(test)]
    fn source_order(&self) -> impl Iterator<Item = IndexNodeId> + '_ {
        self.source_order.iter().copied()
    }

    #[cfg(test)]
    fn span(&self, id: IndexNodeId) -> SourceSpan {
        self.nodes[id.0 as usize].node.span()
    }

    fn render_node(&self, id: IndexNodeId, depth: usize, rendered: &mut String) {
        let entry = &self.nodes[id.0 as usize];
        let _ = write!(rendered, "{}", "  ".repeat(depth));
        render_label(&entry.node, rendered);
        render_span(entry.node.span(), rendered);
        rendered.push('\n');

        let mut children = self
            .nodes
            .iter()
            .filter(|candidate| candidate.parent == Some(id))
            .collect::<Vec<_>>();
        children.sort_by_key(|child| source_key(child));
        for child in children {
            self.render_node(child.id, depth + 1, rendered);
        }
    }

    fn is_descendant_of(&self, mut id: IndexNodeId, ancestor: IndexNodeId) -> bool {
        loop {
            if id == ancestor {
                return true;
            }
            let Some(parent) = self.nodes[id.0 as usize].parent else {
                return false;
            };
            id = parent;
        }
    }
}

fn section_span(
    projection: &ParsedDocument,
    heading_span: SourceSpan,
    byte_end: u32,
) -> SourceSpan {
    let line_end = if byte_end as usize >= projection.source.len() {
        projection.line_count()
    } else {
        let line_at_end = projection.byte_to_line(byte_end);
        if byte_end > 0 && projection.source.as_bytes().get(byte_end as usize - 1) == Some(&b'\n') {
            line_at_end - 1
        } else {
            line_at_end
        }
    };
    SourceSpan {
        line_start: heading_span.line_start,
        line_end,
        byte_start: heading_span.byte_start,
        byte_end,
    }
}

#[derive(Default)]
struct IndexBuilder {
    nodes: Vec<IndexEntry>,
}

impl IndexBuilder {
    fn push(&mut self, parent: Option<IndexNodeId>, node: IndexNode) -> IndexNodeId {
        let id = IndexNodeId(self.nodes.len() as u32);
        self.nodes.push(IndexEntry { id, parent, node });
        id
    }
}

fn indexed_table(table: &TableProjection) -> IndexedTable {
    IndexedTable {
        headers: table.headers.clone(),
        alignments: table.alignments.clone(),
    }
}

fn add_tasks(builder: &mut IndexBuilder, body: IndexNodeId, tasks: &[crate::parser::TaskItemInfo]) {
    let mut order = (0..tasks.len()).collect::<Vec<_>>();
    order.sort_by_key(|position| {
        let task = &tasks[*position];
        (task.span.byte_start, task.depth, task.child_path.clone())
    });
    let mut by_path = HashMap::<Vec<u32>, IndexNodeId>::new();
    for position in order {
        let task = &tasks[position];
        let parent = (1..task.child_path.len())
            .rev()
            .find_map(|prefix_len| by_path.get(&task.child_path[..prefix_len]).copied())
            .unwrap_or(body);
        let id = builder.push(
            Some(parent),
            IndexNode::TaskItem {
                span: task.span,
                child_path: task.child_path.clone(),
                task_index: task.task_index,
                status: task.status,
                depth: task.depth,
                symbol_byte_offset: task.symbol_byte_offset,
                summary_text: task.summary_text.clone(),
            },
        );
        by_path.insert(task.child_path.clone(), id);
    }
}

fn add_links(builder: &mut IndexBuilder, parent: IndexNodeId, links: &[crate::parser::LinkInfo]) {
    let mut links = links.iter().collect::<Vec<_>>();
    links.sort_by_key(|link| (link.span.byte_start, link.span.byte_end));
    for (occurrence, link) in links.into_iter().enumerate() {
        builder.push(
            Some(parent),
            IndexNode::Link {
                span: link.span,
                occurrence: occurrence as u32,
                kind: link.kind,
                text: link.text.clone(),
                destination: link.destination.clone(),
                title: link.title.clone(),
            },
        );
    }
}

fn source_key(entry: &IndexEntry) -> (u32, u8, std::cmp::Reverse<u32>, u32) {
    let span = entry.node.span();
    (
        span.byte_start,
        kind_rank(entry.node.kind()),
        std::cmp::Reverse(span.byte_end),
        entry.id.0,
    )
}

fn kind_rank(kind: IndexNodeKind) -> u8 {
    match kind {
        IndexNodeKind::Document => 0,
        IndexNodeKind::Frontmatter => 1,
        IndexNodeKind::Preamble => 2,
        IndexNodeKind::Section => 3,
        IndexNodeKind::Heading => 4,
        IndexNodeKind::HeadingMarker => 5,
        IndexNodeKind::BodyBlock => 6,
        IndexNodeKind::TaskItem => 7,
        IndexNodeKind::TableRow => 8,
        IndexNodeKind::Link => 9,
    }
}

fn render_label(node: &IndexNode, rendered: &mut String) {
    match node {
        IndexNode::Document { .. } => rendered.push_str("document"),
        IndexNode::Frontmatter { format, .. } => {
            let _ = write!(rendered, "frontmatter format={}", frontmatter_name(*format));
        }
        IndexNode::Preamble { .. } => rendered.push_str("preamble"),
        IndexNode::Section {
            heading_span,
            level,
            text,
            ..
        } => {
            let _ = write!(
                rendered,
                "section level={level} text={text:?} heading={}..{}",
                heading_span.byte_start, heading_span.byte_end
            );
        }
        IndexNode::Heading {
            parser_index,
            level,
            text,
            ..
        } => {
            let _ = write!(
                rendered,
                "heading level={level} parser-index={parser_index} text={text:?}"
            );
        }
        IndexNode::HeadingMarker {
            level, source_kind, ..
        } => {
            let _ = write!(
                rendered,
                "heading-marker level={level} source={}",
                heading_source_name(*source_kind)
            );
        }
        IndexNode::BodyBlock {
            ordinal,
            parser_index,
            kind,
            table,
            ..
        } => {
            let _ = write!(
                rendered,
                "body-block ordinal={ordinal} parser-index={parser_index} kind={kind}"
            );
            if let Some(table) = table {
                let _ = write!(
                    rendered,
                    " headers={:?} alignments={:?}",
                    table.headers, table.alignments
                );
            }
        }
        IndexNode::TaskItem {
            child_path,
            task_index,
            status,
            depth,
            symbol_byte_offset,
            summary_text,
            ..
        } => {
            let path = child_path
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(".");
            let _ = write!(
                rendered,
                "task path={path} task-index={task_index} status={status} depth={depth} symbol-byte={symbol_byte_offset} summary={summary_text:?}"
            );
        }
        IndexNode::TableRow { ordinal, cells, .. } => {
            let _ = write!(rendered, "table-row ordinal={ordinal} cells={cells:?}");
        }
        IndexNode::Link {
            occurrence,
            kind,
            text,
            destination,
            title,
            ..
        } => {
            let _ = write!(
                rendered,
                "link occurrence={occurrence} kind={kind} text={text:?} destination={destination:?} title={title:?}"
            );
        }
    }
}

fn render_span(span: SourceSpan, rendered: &mut String) {
    let _ = write!(
        rendered,
        " lines={}-{} bytes={}..{}",
        span.line_start, span.line_end, span.byte_start, span.byte_end
    );
}

fn frontmatter_name(format: FrontmatterFormat) -> &'static str {
    match format {
        FrontmatterFormat::Yaml => "yaml",
        FrontmatterFormat::Toml => "toml",
    }
}

fn heading_source_name(kind: HeadingSourceKind) -> &'static str {
    match kind {
        HeadingSourceKind::Atx => "atx",
        HeadingSourceKind::Setext => "setext",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block;
    use crate::document::Document;
    use crate::fingerprint::TargetEtag;
    use crate::frontmatter;
    use crate::link;
    use crate::section::SectionIndex;
    use crate::table;
    use crate::task;

    #[test]
    fn source_order_uses_byte_positions_instead_of_parser_order() {
        let projection =
            ParsedDocument::parse(include_str!("../tests/fixtures/footnote_midbody.md").into())
                .unwrap();
        let index = DocumentIndex::build(projection);
        let starts = index
            .source_order()
            .map(|id| index.span(id).byte_start)
            .collect::<Vec<_>>();
        assert!(starts.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn cached_index_matches_current_semantic_reads_field_for_field() {
        let source = "---\ntitle: Demo\n---\n\n# [Work](work.md)\n\n- [ ] root\n  - ordinary\n    - [x] child\n\n| Name | State |\n| --- | --- |\n| A | open |\n";
        let document = Document::parse_for_frontmatter(source).unwrap();
        let index = document.index();

        let frontmatter_read = frontmatter::read(&document).unwrap();
        let frontmatter_node = index
            .nodes
            .iter()
            .find_map(|entry| match entry.node {
                IndexNode::Frontmatter { span, format } => Some((span, format)),
                _ => None,
            })
            .unwrap();
        assert_eq!(frontmatter_read.span, Some(frontmatter_node.0));
        assert_eq!(frontmatter_read.format, Some(frontmatter_node.1));
        let frontmatter_source = document.slice(&frontmatter_node.0).unwrap();
        assert_eq!(frontmatter_read.raw.as_deref(), Some(frontmatter_source));
        assert_eq!(
            frontmatter_read.data,
            frontmatter::parse_data(frontmatter_source, frontmatter_node.1).unwrap()
        );

        let block_reads = block::blocks(&document);
        for entry in &index.nodes {
            match &entry.node {
                IndexNode::BodyBlock {
                    parser_index,
                    kind,
                    span,
                    table: indexed_table,
                    ..
                } => {
                    let current = &block_reads[*parser_index as usize];
                    assert_eq!(current.index, *parser_index);
                    assert_eq!(current.kind, *kind);
                    assert_eq!(current.span, *span);
                    let content = document.slice(span).unwrap();
                    assert_eq!(current.etag, TargetEtag::for_bytes(content.as_bytes()));
                    if let Some(indexed_table) = indexed_table {
                        let current_table =
                            table::table(&document, *parser_index, &Default::default()).unwrap();
                        assert_eq!(current_table.span, *span);
                        assert_eq!(current_table.headers, indexed_table.headers);
                        assert_eq!(current_table.alignments, indexed_table.alignments);
                    }
                }
                IndexNode::Heading {
                    parser_index,
                    level,
                    text,
                    span,
                } => {
                    let current = &document.blocks()[*parser_index as usize];
                    let heading = current.heading.as_ref().unwrap();
                    assert_eq!(current.span, *span);
                    assert_eq!(heading.level, *level);
                    assert_eq!(heading.text, *text);
                }
                _ => {}
            }
        }

        let outline = SectionIndex::new(&document).outline();
        for entry in &index.nodes {
            if let IndexNode::Section {
                span,
                heading_span,
                level,
                text,
            } = &entry.node
            {
                let current = outline
                    .iter()
                    .find(|section| section.heading.span == *heading_span)
                    .unwrap();
                assert_eq!(current.section_span, *span);
                assert_eq!(current.heading.span, *heading_span);
                assert_eq!(current.heading.level, *level);
                assert_eq!(current.heading.text, *text);
            }
        }

        let task_reads = task::tasks(&document, &Default::default()).unwrap();
        for entry in &index.nodes {
            if let IndexNode::TaskItem {
                span,
                child_path,
                task_index,
                status,
                depth,
                summary_text,
                ..
            } = &entry.node
            {
                let block_index = owning_source_block(index, entry.id);
                let current = task_reads
                    .iter()
                    .find(|task| {
                        task.loc.block_index() == block_index && task.loc.child_path() == child_path
                    })
                    .unwrap();
                assert_eq!(current.task_index, *task_index);
                assert_eq!(current.status, *status);
                assert_eq!(current.depth, *depth);
                assert_eq!(current.span, *span);
                assert_eq!(current.summary_text, *summary_text);
                let owning_section = owning_section(index, entry.id);
                assert_eq!(
                    current.nearest_heading.as_deref(),
                    owning_section.as_ref().map(|(_, text)| text.as_str())
                );
                assert_eq!(
                    current.nearest_heading_block_index,
                    owning_section.map(|(block_index, _)| block_index)
                );
            }
        }

        let link_reads = link::links(&document);
        for entry in &index.nodes {
            if let IndexNode::Link {
                span,
                occurrence,
                kind,
                text,
                destination,
                title,
            } = &entry.node
            {
                let block_index = owning_source_block(index, entry.id);
                let current = link_reads
                    .iter()
                    .filter(|link| link.source_block_index == block_index)
                    .nth(*occurrence as usize)
                    .unwrap();
                assert_eq!(current.span, *span);
                assert_eq!(current.kind, *kind);
                assert_eq!(current.text, *text);
                assert_eq!(current.destination, *destination);
                assert_eq!(current.title, *title);
            }
        }

        for entry in &index.nodes {
            if let IndexNode::TableRow {
                span,
                ordinal,
                cells,
            } = &entry.node
            {
                let block_index = owning_source_block(index, entry.id);
                let cached = document.blocks()[block_index as usize]
                    .table
                    .as_ref()
                    .unwrap();
                assert_eq!(cached.rows[*ordinal as usize].span, *span);
                assert_eq!(cached.rows[*ordinal as usize].cells, *cells);
            }
        }
    }

    fn owning_source_block(index: &DocumentIndex, mut id: IndexNodeId) -> u32 {
        loop {
            let entry = &index.nodes[id.0 as usize];
            match entry.node {
                IndexNode::BodyBlock { parser_index, .. }
                | IndexNode::Heading { parser_index, .. } => return parser_index,
                _ => id = entry.parent.expect("semantic node has a source block"),
            }
        }
    }

    fn owning_section(index: &DocumentIndex, mut id: IndexNodeId) -> Option<(u32, String)> {
        loop {
            let entry = &index.nodes[id.0 as usize];
            if let IndexNode::Section { ref text, .. } = entry.node {
                let parser_index = index.nodes.iter().find_map(|candidate| {
                    if candidate.parent == Some(entry.id) {
                        if let IndexNode::Heading { parser_index, .. } = candidate.node {
                            return Some(parser_index);
                        }
                    }
                    None
                })?;
                return Some((parser_index, text.clone()));
            }
            id = entry.parent?;
        }
    }
}
