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
use crate::parser::{HeadingSourceKind, ParsedFacts, TableFact};
use crate::source::DocumentSource;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceRegionKind {
    Structural,
    Boundary,
    ParserUnrepresented,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceOwner {
    Preamble(IndexNodeId),
    Node(IndexNodeId),
}

impl SourceOwner {
    fn node(self) -> IndexNodeId {
        match self {
            Self::Preamble(node) | Self::Node(node) => node,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceRegion {
    pub(crate) span: SourceSpan,
    pub(crate) kind: SourceRegionKind,
    pub(crate) owner: SourceOwner,
}

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

/// One immutable index owning exact source state and consumed parser facts.
pub struct DocumentIndex {
    instance_id: IndexInstanceId,
    source: DocumentSource,
    legacy_facts: ParsedFacts,
    // U2 establishes the retained ledger before U4 moves preservation consumers onto it.
    #[allow(dead_code)]
    source_regions: Vec<SourceRegion>,
    nodes: Vec<IndexEntry>,
    source_order: Vec<IndexNodeId>,
    children_by_parent: HashMap<IndexNodeId, Vec<IndexNodeId>>,
    address_by_node: HashMap<IndexNodeId, TargetAddress>,
    node_by_address: HashMap<TargetAddress, IndexNodeId>,
    root: IndexNodeId,
}

impl DocumentIndex {
    pub(crate) fn build(
        source: DocumentSource,
        legacy_facts: ParsedFacts,
    ) -> Result<Self, crate::core_error::CoreError> {
        let mut builder = IndexBuilder::default();
        let mut structural_regions = Vec::<(SourceSpan, IndexNodeId)>::new();
        let document_span = SourceSpan {
            line_start: 1,
            line_end: source.line_count(),
            byte_start: 0,
            byte_end: source.len() as u32,
        };
        let root = builder.push(
            None,
            IndexNode::Document {
                span: document_span,
            },
        );

        if let Some(frontmatter) = &legacy_facts.frontmatter {
            let frontmatter_node = builder.push(
                Some(root),
                IndexNode::Frontmatter {
                    span: frontmatter.span,
                    format: frontmatter.format,
                },
            );
            structural_regions.push((frontmatter.span, frontmatter_node));
        }

        let mut block_order = (0..legacy_facts.blocks.len()).collect::<Vec<_>>();
        block_order.sort_by_key(|position| {
            let block = &legacy_facts.blocks[*position];
            (block.span.byte_start, block.span.byte_end, block.index)
        });

        let mut sections = Vec::<SectionSpec>::new();
        let mut section_stack = Vec::<usize>::new();
        let mut owner_by_block = vec![None; legacy_facts.blocks.len()];
        let mut section_by_heading_block = HashMap::<u32, usize>::new();

        for position in &block_order {
            let block = &legacy_facts.blocks[*position];
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
                byte_end: source.len() as u32,
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
            .unwrap_or(source.len() as u32);
        let preamble_start = legacy_facts
            .frontmatter
            .as_ref()
            .map(|frontmatter| frontmatter.span.byte_end)
            .unwrap_or(0);
        let preamble_span = source.span_for_byte_range(preamble_start, first_heading_start);
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
            let span = section_span(&source, section.heading_span, section.byte_end);
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
            structural_regions.push((section.heading_span, heading_node));
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
            let block = &legacy_facts.blocks[position];
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
            structural_regions.push((block.span, body));
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

        let source_regions = build_source_regions(&source, preamble, structural_regions)?;
        let mut index = Self {
            instance_id: IndexInstanceId(NEXT_INDEX_INSTANCE_ID.fetch_add(1, Ordering::Relaxed)),
            source,
            legacy_facts,
            source_regions,
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
        Ok(index)
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

    pub(crate) fn source(&self) -> &DocumentSource {
        &self.source
    }

    pub(crate) fn legacy_facts(&self) -> &ParsedFacts {
        &self.legacy_facts
    }

    #[cfg(test)]
    fn source_regions(&self) -> &[SourceRegion] {
        &self.source_regions
    }

    #[cfg(test)]
    fn render_source_coverage(&self) -> String {
        let mut rendered = String::new();
        for region in &self.source_regions {
            let _ = writeln!(
                rendered,
                "{} bytes={}..{} lines={}-{} owner={} content={:?}",
                source_region_kind_name(region.kind),
                region.span.byte_start,
                region.span.byte_end,
                region.span.line_start,
                region.span.line_end,
                self.source_owner_name(region.owner),
                self.source.slice_unchecked(&region.span),
            );
        }
        rendered
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

    pub(crate) fn address_for_parser_block(&self, parser_index: u32) -> Option<&TargetAddress> {
        self.nodes.iter().find_map(|entry| match entry.node {
            IndexNode::Heading {
                parser_index: index,
                ..
            } if index == parser_index => entry
                .parent
                .and_then(|section| self.address_for_node(section)),
            IndexNode::BodyBlock {
                parser_index: index,
                ..
            } if index == parser_index => self.address_for_node(entry.id),
            _ => None,
        })
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

    #[cfg(test)]
    fn source_owner_name(&self, owner: SourceOwner) -> String {
        match owner {
            SourceOwner::Preamble(_) => "preamble".into(),
            SourceOwner::Node(node) => match &self.entry(node).node {
                IndexNode::Frontmatter { .. } => "frontmatter".into(),
                IndexNode::Heading { parser_index, .. } => {
                    format!("heading[{parser_index}]")
                }
                IndexNode::BodyBlock { parser_index, .. } => {
                    format!("body-block[{parser_index}]")
                }
                other => format!("{:?}", other.kind()),
            },
        }
    }
}

fn build_source_regions(
    source: &DocumentSource,
    preamble: IndexNodeId,
    mut structural: Vec<(SourceSpan, IndexNodeId)>,
) -> Result<Vec<SourceRegion>, crate::core_error::CoreError> {
    structural.sort_by_key(|(span, node)| (span.byte_start, span.byte_end, node.0));
    let mut regions = Vec::with_capacity(structural.len().saturating_mul(2) + 1);
    let mut cursor = 0u32;
    let mut preceding_owner = SourceOwner::Preamble(preamble);

    for (span, node) in structural {
        validate_structural_region(source, span, cursor)?;
        if cursor < span.byte_start {
            regions.push(classify_complement(
                source,
                cursor,
                span.byte_start,
                preceding_owner,
            ));
        }
        regions.push(SourceRegion {
            span: source.span_for_byte_range(span.byte_start, span.byte_end),
            kind: SourceRegionKind::Structural,
            owner: SourceOwner::Node(node),
        });
        cursor = span.byte_end;
        preceding_owner = SourceOwner::Node(node);
    }

    if cursor < source.len() as u32 {
        regions.push(classify_complement(
            source,
            cursor,
            source.len() as u32,
            preceding_owner,
        ));
    }

    validate_complete_coverage(source, &regions)?;
    Ok(regions)
}

fn validate_structural_region(
    source: &DocumentSource,
    span: SourceSpan,
    cursor: u32,
) -> Result<(), crate::core_error::CoreError> {
    if span.byte_start >= span.byte_end {
        return Err(crate::core_error::CoreError::InvalidSourceCoverage {
            reason: format!(
                "structural region {}..{} is empty or reversed",
                span.byte_start, span.byte_end
            ),
        });
    }
    if span.byte_end as usize > source.len() {
        return Err(crate::core_error::CoreError::InvalidSourceCoverage {
            reason: format!(
                "structural region {}..{} exceeds {} source bytes",
                span.byte_start,
                span.byte_end,
                source.len()
            ),
        });
    }
    if span.byte_start < cursor {
        return Err(crate::core_error::CoreError::InvalidSourceCoverage {
            reason: format!(
                "structural region {}..{} overlaps coverage ending at {cursor}",
                span.byte_start, span.byte_end
            ),
        });
    }
    if !source.is_char_boundary(span.byte_start as usize)
        || !source.is_char_boundary(span.byte_end as usize)
    {
        return Err(crate::core_error::CoreError::InvalidSourceCoverage {
            reason: format!(
                "structural region {}..{} is not UTF-8 aligned",
                span.byte_start, span.byte_end
            ),
        });
    }
    Ok(())
}

fn classify_complement(
    source: &DocumentSource,
    byte_start: u32,
    byte_end: u32,
    owner: SourceOwner,
) -> SourceRegion {
    let span = source.span_for_byte_range(byte_start, byte_end);
    let text = source.slice_unchecked(&span);
    let kind = if text.chars().all(char::is_whitespace) {
        SourceRegionKind::Boundary
    } else {
        SourceRegionKind::ParserUnrepresented
    };
    SourceRegion { span, kind, owner }
}

fn validate_complete_coverage(
    source: &DocumentSource,
    regions: &[SourceRegion],
) -> Result<(), crate::core_error::CoreError> {
    if source.len() == 0 {
        if regions.is_empty() {
            return Ok(());
        }
        return Err(crate::core_error::CoreError::InvalidSourceCoverage {
            reason: "empty source has non-empty coverage".into(),
        });
    }

    let mut cursor = 0u32;
    for region in regions {
        if region.span.byte_start != cursor || region.span.byte_end <= region.span.byte_start {
            return Err(crate::core_error::CoreError::InvalidSourceCoverage {
                reason: format!(
                    "region {}..{} does not continue coverage at {cursor}",
                    region.span.byte_start, region.span.byte_end
                ),
            });
        }
        let _ = region.owner.node();
        source.try_slice(&region.span)?;
        cursor = region.span.byte_end;
    }
    if cursor as usize != source.len() {
        return Err(crate::core_error::CoreError::InvalidSourceCoverage {
            reason: format!("coverage ends at {cursor}, source ends at {}", source.len()),
        });
    }
    Ok(())
}

#[cfg(test)]
fn source_region_kind_name(kind: SourceRegionKind) -> &'static str {
    match kind {
        SourceRegionKind::Structural => "structural",
        SourceRegionKind::Boundary => "boundary",
        SourceRegionKind::ParserUnrepresented => "parser-unrepresented",
    }
}

fn section_span(source: &DocumentSource, heading_span: SourceSpan, byte_end: u32) -> SourceSpan {
    let line_end = if byte_end as usize >= source.len() {
        source.line_count()
    } else {
        let line_at_end = source.byte_to_line(byte_end);
        if byte_end > 0 && source.text().as_bytes().get(byte_end as usize - 1) == Some(&b'\n') {
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

fn indexed_table(table: &TableFact) -> IndexedTable {
    IndexedTable {
        headers: table.headers.clone(),
        alignments: table.alignments.clone(),
    }
}

fn add_tasks(builder: &mut IndexBuilder, body: IndexNodeId, tasks: &[crate::parser::TaskItemFact]) {
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

fn add_links(builder: &mut IndexBuilder, parent: IndexNodeId, links: &[crate::parser::LinkFact]) {
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
mod source_region_tests {
    use super::*;
    use crate::core_error::CoreError;
    use crate::document::Document;
    use crate::source::ParsePolicy;

    fn assert_partition(source: &str) -> Document {
        let document = Document::parse(source).unwrap();
        let regions = document.index().source_regions();
        let reconstructed = regions
            .iter()
            .map(|region| document.slice_unchecked(&region.span))
            .collect::<String>();
        assert_eq!(reconstructed, source);

        let mut cursor = 0u32;
        for region in regions {
            assert_eq!(region.span.byte_start, cursor);
            assert!(region.span.byte_end > region.span.byte_start);
            assert!(source.is_char_boundary(region.span.byte_start as usize));
            assert!(source.is_char_boundary(region.span.byte_end as usize));
            cursor = region.span.byte_end;
        }
        assert_eq!(cursor as usize, source.len());

        let mut expected_structural = document
            .index()
            .legacy_facts
            .blocks
            .iter()
            .map(|block| (block.span.byte_start, block.span.byte_end))
            .collect::<Vec<_>>();
        if let Some(frontmatter) = &document.index().legacy_facts.frontmatter {
            expected_structural.push((frontmatter.span.byte_start, frontmatter.span.byte_end));
        }
        expected_structural.sort_unstable();
        let actual_structural = regions
            .iter()
            .filter(|region| region.kind == SourceRegionKind::Structural)
            .map(|region| (region.span.byte_start, region.span.byte_end))
            .collect::<Vec<_>>();
        assert_eq!(actual_structural, expected_structural);
        assert!(regions.len() <= expected_structural.len().saturating_mul(2) + 1);
        document
    }

    #[test]
    fn source_regions_partition_fixture_shapes_exactly() {
        for source in [
            "",
            " \n\t",
            "# Title\n\nbody",
            "Title\r\n=====\r\n\r\n世界\r\n",
            "---\nk: v\n---\n\n# Title\n\nbody\n",
            "body [known]\n\n[known]: https://example.com\n",
            "body[^kept]\n\n[^kept]: note\n",
            "body\n\n[^lost]: note\n",
            "    indented code\n\n<div>html</div>\n",
            "one\r\ntwo\nthree\r\n",
            "| Name | State |\r\n| --- | --- |\r\n| 世界 | open |\r\n",
        ] {
            assert_partition(source);
        }
    }

    #[test]
    fn source_regions_distinguish_boundaries_from_parser_omissions() {
        let whitespace = assert_partition(" \n\t");
        assert_eq!(
            whitespace.index().source_regions()[0].kind,
            SourceRegionKind::Boundary
        );
        assert!(matches!(
            whitespace.index().source_regions()[0].owner,
            SourceOwner::Preamble(_)
        ));

        let unreferenced = assert_partition("body\n\n[^lost]: note\n");
        let omitted = unreferenced
            .index()
            .source_regions()
            .iter()
            .find(|region| region.kind == SourceRegionKind::ParserUnrepresented)
            .unwrap();
        assert!(unreferenced
            .slice_unchecked(&omitted.span)
            .contains("[^lost]: note"));
        assert!(matches!(
            omitted.owner,
            SourceOwner::Node(node)
                if matches!(unreferenced.index().entry(node).node, IndexNode::BodyBlock { .. })
        ));

        let reference = assert_partition("body [known]\n\n[known]: https://example.com\n\nnext\n");
        let definition = reference
            .index()
            .source_regions()
            .iter()
            .find(|region| {
                region.kind == SourceRegionKind::ParserUnrepresented
                    && reference.slice_unchecked(&region.span).contains("[known]:")
            })
            .unwrap();
        assert!(matches!(
            definition.owner,
            SourceOwner::Node(node)
                if matches!(reference.index().entry(node).node, IndexNode::BodyBlock { .. })
        ));

        let referenced = assert_partition("body[^kept]\n\n[^kept]: note\n");
        assert!(referenced.index().source_regions().iter().any(|region| {
            region.kind == SourceRegionKind::Structural
                && referenced
                    .slice_unchecked(&region.span)
                    .contains("[^kept]: note")
        }));
    }

    #[test]
    fn invalid_structural_regions_fail_index_construction() {
        let ascii = DocumentSource::new("abc".into(), ParsePolicy::Lenient).unwrap();
        let preamble = IndexNodeId(0);
        let node = IndexNodeId(1);
        for regions in [
            vec![(span(2, 1), node)],
            vec![(span(0, 2), node), (span(1, 3), IndexNodeId(2))],
            vec![(span(0, 4), node)],
        ] {
            assert!(matches!(
                build_source_regions(&ascii, preamble, regions),
                Err(CoreError::InvalidSourceCoverage { .. })
            ));
        }

        let utf8 = DocumentSource::new("éx".into(), ParsePolicy::Lenient).unwrap();
        assert!(matches!(
            build_source_regions(&utf8, preamble, vec![(span(1, 2), node)]),
            Err(CoreError::InvalidSourceCoverage { .. })
        ));
    }

    #[test]
    fn source_region_count_stays_linear_in_top_level_facts() {
        let source = (0..1_000)
            .map(|index| format!("paragraph {index}\n\n[^unused-{index}]: note\n\n"))
            .collect::<String>();
        let document = assert_partition(&source);
        let structural = document.index().legacy_facts.blocks.len()
            + usize::from(document.index().legacy_facts.frontmatter.is_some());
        assert!(document.index().source_regions().len() <= structural * 2 + 1);
    }

    #[test]
    fn representative_source_coverage_is_reviewable() {
        for (name, source) in [
            ("basic", "# Root\n\nbody\n"),
            (
                "frontmatter-reference",
                "---\ntitle: Notes\n---\n\n# Root\n\nbody [known]\n\n[known]: https://example.com\n\nnext\n",
            ),
            (
                "mid-body-unreferenced-footnote",
                "# Root\n\nbefore\n\n[^lost]: omitted\n\nafter\n",
            ),
            (
                "referenced-footnote",
                "body[^kept]\n\n[^kept]: note\n",
            ),
            (
                "table-crlf-utf8",
                "| A | B |\r\n| --- | --- |\r\n| 世界 | open |\r\n",
            ),
        ] {
            let document = assert_partition(source);
            println!("[{name}]\n{}", document.index().render_source_coverage());
        }
    }

    fn span(byte_start: u32, byte_end: u32) -> SourceSpan {
        SourceSpan {
            line_start: 1,
            line_end: 1,
            byte_start,
            byte_end,
        }
    }
}
