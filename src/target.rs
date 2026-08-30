use std::collections::HashMap;
use std::fmt::Write as _;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core_error::CoreError;
use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::frontmatter;
use crate::index::{DocumentIndex, IndexInstanceId, IndexNode, IndexNodeId};
use crate::model::{
    BlockKind, FrontmatterFormat, HeadingMatchMode, LinkKind, SearchMatchMode, SourceSpan,
    TaskStatus,
};
use crate::revision::DocumentRevision;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeadingAddressSegment {
    pub text: String,
    /// 1-based occurrence among equal-text sibling headings.
    #[schemars(range(min = 1))]
    pub occurrence: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictHeadingAddressSegment {
    text: String,
    occurrence: u32,
}

impl<'de> Deserialize<'de> for HeadingAddressSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let segment = StrictHeadingAddressSegment::deserialize(deserializer)?;
        if segment.occurrence == 0 {
            return Err(serde::de::Error::custom(
                "heading occurrence must be 1-based",
            ));
        }
        Ok(Self {
            text: segment.text,
            occurrence: segment.occurrence,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SectionAddress {
    Preamble,
    Heading {
        #[schemars(length(min = 1))]
        path: Vec<HeadingAddressSegment>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockAddress {
    pub section: SectionAddress,
    /// 0-based body-block ordinal within the owning section.
    pub ordinal: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeadingSectionAddress {
    #[schemars(length(min = 1))]
    pub path: Vec<HeadingAddressSegment>,
}

impl<'de> Deserialize<'de> for HeadingSectionAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            path: Vec<HeadingAddressSegment>,
        }
        let wire = Wire::deserialize(deserializer)?;
        validate_heading_path(&wire.path).map_err(serde::de::Error::custom)?;
        Ok(Self { path: wire.path })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LinkParentAddress {
    Heading { section: HeadingSectionAddress },
    Block { block: BlockAddress },
}

/// Exact target identity. Source state belongs in [`TargetSnapshot`], never here.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetAddress {
    Document,
    Frontmatter,
    FrontmatterField {
        #[schemars(length(min = 1))]
        path: Vec<String>,
    },
    Preamble,
    Section {
        #[schemars(length(min = 1))]
        path: Vec<HeadingAddressSegment>,
    },
    Block {
        block: BlockAddress,
    },
    Task {
        block: BlockAddress,
        #[schemars(length(min = 1))]
        path: Vec<u32>,
    },
    TableRow {
        table: BlockAddress,
        row: u32,
    },
    Link {
        parent: LinkParentAddress,
        occurrence: u32,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum StrictSectionAddress {
    Preamble {},
    Heading { path: Vec<HeadingAddressSegment> },
}

impl<'de> Deserialize<'de> for SectionAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match StrictSectionAddress::deserialize(deserializer)? {
            StrictSectionAddress::Preamble {} => Ok(Self::Preamble),
            StrictSectionAddress::Heading { path } => {
                validate_heading_path(&path).map_err(serde::de::Error::custom)?;
                Ok(Self::Heading { path })
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum StrictTargetAddress {
    Document {},
    Frontmatter {},
    FrontmatterField {
        path: Vec<String>,
    },
    Preamble {},
    Section {
        path: Vec<HeadingAddressSegment>,
    },
    Block {
        block: BlockAddress,
    },
    Task {
        block: BlockAddress,
        path: Vec<u32>,
    },
    TableRow {
        table: BlockAddress,
        row: u32,
    },
    Link {
        parent: LinkParentAddress,
        occurrence: u32,
    },
}

impl<'de> Deserialize<'de> for TargetAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let address = match StrictTargetAddress::deserialize(deserializer)? {
            StrictTargetAddress::Document {} => Self::Document,
            StrictTargetAddress::Frontmatter {} => Self::Frontmatter,
            StrictTargetAddress::FrontmatterField { path } => Self::FrontmatterField { path },
            StrictTargetAddress::Preamble {} => Self::Preamble,
            StrictTargetAddress::Section { path } => Self::Section { path },
            StrictTargetAddress::Block { block } => Self::Block { block },
            StrictTargetAddress::Task { block, path } => Self::Task { block, path },
            StrictTargetAddress::TableRow { table, row } => Self::TableRow { table, row },
            StrictTargetAddress::Link { parent, occurrence } => Self::Link { parent, occurrence },
        };
        validate_address(&address).map_err(serde::de::Error::custom)?;
        Ok(address)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Document,
    Frontmatter,
    FrontmatterField,
    Preamble,
    Section,
    Block,
    Task,
    TableRow,
    Link,
}

/// Fuzzy discovery criteria. Mutation APIs accept [`TargetAddress`], not this type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetQuery {
    All,
    Kind {
        kind: TargetKind,
    },
    Section {
        text: String,
        match_mode: HeadingMatchMode,
    },
    Task {
        status: Option<TaskStatus>,
        contains: Option<String>,
    },
    Link {
        text: Option<String>,
        destination: Option<String>,
    },
    FrontmatterField {
        #[schemars(length(min = 1))]
        path: Vec<String>,
    },
    Search {
        text: String,
        match_mode: SearchMatchMode,
        block_kinds: Vec<BlockKind>,
    },
}

pub(crate) const NON_EMPTY_SECTION_MATCH_MODES: &[HeadingMatchMode] = &[
    HeadingMatchMode::Contains,
    HeadingMatchMode::ContainsIgnoreCase,
];
pub(crate) const NON_EMPTY_QUERY_TEXT_MIN_LENGTH: u64 = 1;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum StrictTargetQuery {
    All {},
    Kind {
        kind: TargetKind,
    },
    Section {
        text: String,
        match_mode: HeadingMatchMode,
    },
    Task {
        status: Option<TaskStatus>,
        contains: Option<String>,
    },
    Link {
        text: Option<String>,
        destination: Option<String>,
    },
    FrontmatterField {
        path: Vec<String>,
    },
    Search {
        text: String,
        match_mode: SearchMatchMode,
        block_kinds: Vec<BlockKind>,
    },
}

impl<'de> Deserialize<'de> for TargetQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let query = match StrictTargetQuery::deserialize(deserializer)? {
            StrictTargetQuery::All {} => Self::All,
            StrictTargetQuery::Kind { kind } => Self::Kind { kind },
            StrictTargetQuery::Section { text, match_mode } => Self::Section { text, match_mode },
            StrictTargetQuery::Task { status, contains } => Self::Task { status, contains },
            StrictTargetQuery::Link { text, destination } => Self::Link { text, destination },
            StrictTargetQuery::FrontmatterField { path } => Self::FrontmatterField { path },
            StrictTargetQuery::Search {
                text,
                match_mode,
                block_kinds,
            } => Self::Search {
                text,
                match_mode,
                block_kinds,
            },
        };
        validate_query(&query).map_err(serde::de::Error::custom)?;
        Ok(query)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRange {
    pub target: TargetAddress,
    pub span: SourceSpan,
    pub etag: TargetEtag,
    pub preview: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryResult {
    Target { target: TargetSnapshot },
    Evidence { evidence: EvidenceRange },
}

impl QueryResult {
    pub fn target(&self) -> Option<&TargetSnapshot> {
        match self {
            Self::Target { target } => Some(target),
            Self::Evidence { .. } => None,
        }
    }

    pub fn evidence(&self) -> Option<&EvidenceRange> {
        match self {
            Self::Evidence { evidence } => Some(evidence),
            Self::Target { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GuardAuthority {
    Document {
        revision: DocumentRevision,
    },
    Selection {
        span: SourceSpan,
        etag: TargetEtag,
    },
    Container {
        address: Box<TargetAddress>,
        span: SourceSpan,
        etag: TargetEtag,
    },
    Frontmatter {
        span: Option<SourceSpan>,
        etag: TargetEtag,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JsonValueKind {
    Missing,
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetSummary {
    Document {
        bytes: u32,
        lines: u32,
    },
    Frontmatter {
        present: bool,
        format: Option<FrontmatterFormat>,
    },
    FrontmatterField {
        path: Vec<String>,
        value: JsonValueKind,
    },
    Preamble,
    Section {
        level: u8,
        heading: String,
    },
    Block {
        kind: BlockKind,
        preview: String,
    },
    Task {
        status: TaskStatus,
        depth: u32,
        text: String,
    },
    TableRow {
        row: u32,
        cells: Vec<String>,
    },
    Link {
        kind: LinkKind,
        text: String,
        destination: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetSnapshot {
    pub address: TargetAddress,
    pub kind: TargetKind,
    pub selection_span: Option<SourceSpan>,
    pub revision: DocumentRevision,
    pub guard: GuardAuthority,
    pub summary: TargetSummary,
}

#[derive(Clone, Debug)]
pub struct ResolvedTarget {
    snapshot: TargetSnapshot,
    locator: ResolvedLocator,
    index_instance: IndexInstanceId,
    containment_span: Option<SourceSpan>,
}

#[derive(Clone, Debug)]
pub(crate) enum ResolvedLocator {
    Node(IndexNodeId),
    FrontmatterField(Vec<String>),
}

pub(crate) fn build_index_addresses(
    index: &DocumentIndex,
) -> (
    HashMap<IndexNodeId, TargetAddress>,
    HashMap<TargetAddress, IndexNodeId>,
) {
    let mut address_by_node = HashMap::new();
    let mut node_by_address = HashMap::new();
    let mut section_by_node = HashMap::<IndexNodeId, SectionAddress>::new();
    let mut block_by_node = HashMap::<IndexNodeId, BlockAddress>::new();
    let mut owning_block = HashMap::<IndexNodeId, BlockAddress>::new();
    let mut heading_section = HashMap::<IndexNodeId, SectionAddress>::new();
    let mut sibling_occurrences = HashMap::<(IndexNodeId, String), u32>::new();

    for entry in index.entries_in_source_order() {
        let address = match &entry.node {
            IndexNode::Document { .. } => Some(TargetAddress::Document),
            IndexNode::Frontmatter { .. } | IndexNode::HeadingMarker { .. } => None,
            IndexNode::Preamble { .. } => {
                section_by_node.insert(entry.id, SectionAddress::Preamble);
                Some(TargetAddress::Preamble)
            }
            IndexNode::Section { text, .. } => {
                let parent = entry.parent.expect("section parent");
                let occurrence = sibling_occurrences
                    .entry((parent, text.clone()))
                    .and_modify(|value| *value += 1)
                    .or_insert(1);
                let mut path = match section_by_node.get(&parent) {
                    Some(SectionAddress::Heading { path }) => path.clone(),
                    Some(SectionAddress::Preamble) | None => Vec::new(),
                };
                path.push(HeadingAddressSegment {
                    text: text.clone(),
                    occurrence: *occurrence,
                });
                let section = SectionAddress::Heading { path: path.clone() };
                section_by_node.insert(entry.id, section);
                Some(TargetAddress::Section { path })
            }
            IndexNode::Heading { .. } => {
                let section = section_by_node
                    .get(&entry.parent.expect("heading section"))
                    .expect("section address exists before heading")
                    .clone();
                heading_section.insert(entry.id, section);
                None
            }
            IndexNode::BodyBlock { ordinal, .. } => {
                let section = section_by_node
                    .get(&entry.parent.expect("body block owner"))
                    .expect("section address exists before body block")
                    .clone();
                let block = BlockAddress {
                    section,
                    ordinal: *ordinal,
                };
                block_by_node.insert(entry.id, block.clone());
                owning_block.insert(entry.id, block.clone());
                Some(TargetAddress::Block { block })
            }
            IndexNode::TaskItem { child_path, .. } => {
                let block = owning_block
                    .get(&entry.parent.expect("task owner"))
                    .expect("owning block exists before task")
                    .clone();
                owning_block.insert(entry.id, block.clone());
                Some(TargetAddress::Task {
                    block,
                    path: child_path.clone(),
                })
            }
            IndexNode::TableRow { ordinal, .. } => {
                let table = block_by_node
                    .get(&entry.parent.expect("table row owner"))
                    .expect("table address exists before row")
                    .clone();
                Some(TargetAddress::TableRow {
                    table,
                    row: *ordinal,
                })
            }
            IndexNode::Link { occurrence, .. } => {
                let parent = entry.parent.expect("link owner");
                let parent = if let Some(section) = heading_section.get(&parent) {
                    LinkParentAddress::Heading {
                        section: match section {
                            SectionAddress::Heading { path } => {
                                HeadingSectionAddress { path: path.clone() }
                            }
                            SectionAddress::Preamble => unreachable!("heading owner is a section"),
                        },
                    }
                } else {
                    LinkParentAddress::Block {
                        block: block_by_node
                            .get(&parent)
                            .expect("link block address exists")
                            .clone(),
                    }
                };
                Some(TargetAddress::Link {
                    parent,
                    occurrence: *occurrence,
                })
            }
        };
        if let Some(address) = address {
            assert!(
                node_by_address.insert(address.clone(), entry.id).is_none(),
                "duplicate canonical target address: {address}"
            );
            address_by_node.insert(entry.id, address);
        }
    }

    (address_by_node, node_by_address)
}

impl ResolvedTarget {
    pub fn snapshot(&self) -> &TargetSnapshot {
        &self.snapshot
    }

    pub fn address(&self) -> &TargetAddress {
        &self.snapshot.address
    }

    pub(crate) fn locator(&self) -> &ResolvedLocator {
        &self.locator
    }

    pub(crate) fn ensure_document(&self, document: &Document) -> Result<(), CoreError> {
        if self.index_instance != document.index().instance_id() {
            return Err(CoreError::DocumentIndexMismatch);
        }
        if self.snapshot.revision != *document.revision() {
            Err(CoreError::DocumentRevisionMismatch {
                expected: self.snapshot.revision.to_string(),
                actual: document.revision().to_string(),
            })
        } else {
            Ok(())
        }
    }
}

pub fn map(document: &Document) -> Result<Vec<TargetSnapshot>, CoreError> {
    Ok(collect_resolved(document)?
        .into_iter()
        .map(|resolved| resolved.snapshot)
        .collect())
}

pub fn query(document: &Document, query: &TargetQuery) -> Result<Vec<QueryResult>, CoreError> {
    validate_query(query)?;
    if let TargetQuery::Search {
        text,
        match_mode,
        block_kinds,
    } = query
    {
        return Ok(
            crate::search::evidence_ranges(document, text, *match_mode, block_kinds)
                .into_iter()
                .map(|evidence| QueryResult::Evidence { evidence })
                .collect(),
        );
    }
    Ok(query_resolved(document, query)?
        .into_iter()
        .map(|resolved| QueryResult::Target {
            target: resolved.snapshot,
        })
        .collect())
}

fn query_resolved(
    document: &Document,
    query: &TargetQuery,
) -> Result<Vec<ResolvedTarget>, CoreError> {
    Ok(collect_resolved(document)?
        .into_iter()
        .filter(|resolved| query_matches(query, &resolved.snapshot))
        .collect())
}

pub fn query_one(document: &Document, query: &TargetQuery) -> Result<ResolvedTarget, CoreError> {
    validate_query(query)?;
    if matches!(query, TargetQuery::Search { .. }) {
        return Err(CoreError::InvalidSelector(
            "search evidence cannot resolve as one mutable target".into(),
        ));
    }
    let mut matches = query_resolved(document, query)?;
    match matches.len() {
        0 => Err(CoreError::TargetNotFound {
            target: format!("query {query:?}"),
        }),
        1 => Ok(matches.remove(0)),
        count => Err(CoreError::AmbiguousTargetQuery { count }),
    }
}

pub fn resolve(document: &Document, address: &TargetAddress) -> Result<ResolvedTarget, CoreError> {
    validate_address(address)?;
    match address {
        TargetAddress::Frontmatter => {
            let document_node = document
                .index()
                .entries_in_source_order()
                .find(|entry| matches!(entry.node, IndexNode::Document { .. }))
                .expect("document index root")
                .id;
            frontmatter_targets(document, document_node)?
                .into_iter()
                .find(|target| target.address() == address)
                .ok_or_else(|| CoreError::TargetNotFound {
                    target: address.to_string(),
                })
        }
        TargetAddress::FrontmatterField { path } => {
            resolve_frontmatter_field(document, path.clone())
        }
        _ => {
            let node = document.index().node_for_address(address).ok_or_else(|| {
                CoreError::TargetNotFound {
                    target: address.to_string(),
                }
            })?;
            resolve_index_node(document, node)
        }
    }
}

pub fn locate(document: &Document, byte_offset: u32) -> Result<Vec<TargetSnapshot>, CoreError> {
    let source_len = document.source().len() as u32;
    if byte_offset >= source_len {
        return Err(CoreError::ByteOffsetOutOfRange {
            byte_offset,
            source_len,
        });
    }
    Ok(collect_resolved(document)?
        .into_iter()
        .filter(|resolved| {
            resolved.snapshot.kind != TargetKind::FrontmatterField
                && resolved_contains(document.source(), resolved, byte_offset)
        })
        .map(|resolved| resolved.snapshot)
        .collect())
}

fn collect_resolved(document: &Document) -> Result<Vec<ResolvedTarget>, CoreError> {
    let index = document.index();
    let mut targets = Vec::new();
    for entry in index.entries_in_source_order() {
        match &entry.node {
            IndexNode::Document { span } => {
                targets.push(resolved(
                    document,
                    index
                        .address_for_node(entry.id)
                        .expect("document address")
                        .clone(),
                    TargetKind::Document,
                    ResolvedState {
                        selection_span: Some(*span),
                        containment_span: Some(*span),
                        guard: GuardAuthority::Document {
                            revision: document.revision().clone(),
                        },
                    },
                    TargetSummary::Document {
                        bytes: document.source().len() as u32,
                        lines: document.line_count(),
                    },
                    ResolvedLocator::Node(entry.id),
                ));
                targets.extend(frontmatter_targets(document, entry.id)?);
            }
            IndexNode::Frontmatter { .. }
            | IndexNode::Heading { .. }
            | IndexNode::HeadingMarker { .. } => {}
            IndexNode::Preamble { span } => {
                let selection_span = preamble_selection_span(index, entry.id, *span);
                targets.push(resolved(
                    document,
                    index
                        .address_for_node(entry.id)
                        .expect("preamble address")
                        .clone(),
                    TargetKind::Preamble,
                    ResolvedState {
                        selection_span: Some(selection_span),
                        containment_span: Some(*span),
                        guard: GuardAuthority::Selection {
                            span: selection_span,
                            etag: etag(document, selection_span)?,
                        },
                    },
                    TargetSummary::Preamble,
                    ResolvedLocator::Node(entry.id),
                ));
            }
            IndexNode::Section {
                span, level, text, ..
            } => {
                targets.push(resolved_selection(
                    document,
                    index
                        .address_for_node(entry.id)
                        .expect("section address")
                        .clone(),
                    TargetKind::Section,
                    *span,
                    TargetSummary::Section {
                        level: *level,
                        heading: text.clone(),
                    },
                    entry.id,
                )?);
            }
            IndexNode::BodyBlock { span, kind, .. } => {
                targets.push(resolved_selection(
                    document,
                    index
                        .address_for_node(entry.id)
                        .expect("body block address")
                        .clone(),
                    TargetKind::Block,
                    *span,
                    TargetSummary::Block {
                        kind: *kind,
                        preview: preview(document.slice(span)?),
                    },
                    entry.id,
                )?);
            }
            IndexNode::TaskItem {
                span,
                status,
                depth,
                summary_text,
                ..
            } => {
                targets.push(resolved_selection(
                    document,
                    index
                        .address_for_node(entry.id)
                        .expect("task address")
                        .clone(),
                    TargetKind::Task,
                    *span,
                    TargetSummary::Task {
                        status: *status,
                        depth: *depth,
                        text: summary_text.clone(),
                    },
                    entry.id,
                )?);
            }
            IndexNode::TableRow {
                span,
                ordinal,
                cells,
            } => {
                let address = index
                    .address_for_node(entry.id)
                    .expect("table row address")
                    .clone();
                let TargetAddress::TableRow { table, .. } = &address else {
                    unreachable!("table row node has table row address")
                };
                let table_address = TargetAddress::Block {
                    block: table.clone(),
                };
                let table_span = index
                    .entry(entry.parent.expect("table row owner"))
                    .node
                    .span();
                targets.push(resolved(
                    document,
                    address,
                    TargetKind::TableRow,
                    ResolvedState {
                        selection_span: Some(*span),
                        containment_span: Some(*span),
                        guard: GuardAuthority::Container {
                            address: Box::new(table_address),
                            span: table_span,
                            etag: etag(document, table_span)?,
                        },
                    },
                    TargetSummary::TableRow {
                        row: *ordinal,
                        cells: cells.clone(),
                    },
                    ResolvedLocator::Node(entry.id),
                ));
            }
            IndexNode::Link {
                span,
                kind,
                text,
                destination,
                ..
            } => {
                targets.push(resolved_selection(
                    document,
                    index
                        .address_for_node(entry.id)
                        .expect("link address")
                        .clone(),
                    TargetKind::Link,
                    *span,
                    TargetSummary::Link {
                        kind: *kind,
                        text: text.clone(),
                        destination: destination.clone(),
                    },
                    entry.id,
                )?);
            }
        }
    }
    Ok(targets)
}

fn resolve_index_node(document: &Document, node: IndexNodeId) -> Result<ResolvedTarget, CoreError> {
    let index = document.index();
    let entry = index.entry(node);
    let address = index
        .address_for_node(node)
        .expect("addressed node has a canonical address")
        .clone();
    match &entry.node {
        IndexNode::Document { span } => Ok(resolved(
            document,
            address,
            TargetKind::Document,
            ResolvedState {
                selection_span: Some(*span),
                containment_span: Some(*span),
                guard: GuardAuthority::Document {
                    revision: document.revision().clone(),
                },
            },
            TargetSummary::Document {
                bytes: document.source().len() as u32,
                lines: document.line_count(),
            },
            ResolvedLocator::Node(node),
        )),
        IndexNode::Preamble { span } => {
            let selection_span = preamble_selection_span(index, node, *span);
            Ok(resolved(
                document,
                address,
                TargetKind::Preamble,
                ResolvedState {
                    selection_span: Some(selection_span),
                    containment_span: Some(*span),
                    guard: GuardAuthority::Selection {
                        span: selection_span,
                        etag: etag(document, selection_span)?,
                    },
                },
                TargetSummary::Preamble,
                ResolvedLocator::Node(node),
            ))
        }
        IndexNode::Section {
            span, level, text, ..
        } => resolved_selection(
            document,
            address,
            TargetKind::Section,
            *span,
            TargetSummary::Section {
                level: *level,
                heading: text.clone(),
            },
            node,
        ),
        IndexNode::BodyBlock { span, kind, .. } => resolved_selection(
            document,
            address,
            TargetKind::Block,
            *span,
            TargetSummary::Block {
                kind: *kind,
                preview: preview(document.slice(span)?),
            },
            node,
        ),
        IndexNode::TaskItem {
            span,
            status,
            depth,
            summary_text,
            ..
        } => resolved_selection(
            document,
            address,
            TargetKind::Task,
            *span,
            TargetSummary::Task {
                status: *status,
                depth: *depth,
                text: summary_text.clone(),
            },
            node,
        ),
        IndexNode::TableRow {
            span,
            ordinal,
            cells,
        } => {
            let table = match &address {
                TargetAddress::TableRow { table, .. } => table.clone(),
                _ => unreachable!("table row node has table row address"),
            };
            let table_span = index
                .entry(entry.parent.expect("table row owner"))
                .node
                .span();
            Ok(resolved(
                document,
                address,
                TargetKind::TableRow,
                ResolvedState {
                    selection_span: Some(*span),
                    containment_span: Some(*span),
                    guard: GuardAuthority::Container {
                        address: Box::new(TargetAddress::Block { block: table }),
                        span: table_span,
                        etag: etag(document, table_span)?,
                    },
                },
                TargetSummary::TableRow {
                    row: *ordinal,
                    cells: cells.clone(),
                },
                ResolvedLocator::Node(node),
            ))
        }
        IndexNode::Link {
            span,
            kind,
            text,
            destination,
            ..
        } => resolved_selection(
            document,
            address,
            TargetKind::Link,
            *span,
            TargetSummary::Link {
                kind: *kind,
                text: text.clone(),
                destination: destination.clone(),
            },
            node,
        ),
        IndexNode::Frontmatter { .. }
        | IndexNode::Heading { .. }
        | IndexNode::HeadingMarker { .. } => Err(CoreError::TargetNotFound {
            target: address.to_string(),
        }),
    }
}

fn resolved_selection(
    document: &Document,
    address: TargetAddress,
    kind: TargetKind,
    span: SourceSpan,
    summary: TargetSummary,
    node: IndexNodeId,
) -> Result<ResolvedTarget, CoreError> {
    Ok(resolved(
        document,
        address,
        kind,
        ResolvedState {
            selection_span: Some(span),
            containment_span: Some(span),
            guard: GuardAuthority::Selection {
                span,
                etag: etag(document, span)?,
            },
        },
        summary,
        ResolvedLocator::Node(node),
    ))
}

struct ResolvedState {
    selection_span: Option<SourceSpan>,
    containment_span: Option<SourceSpan>,
    guard: GuardAuthority,
}

fn resolved(
    document: &Document,
    address: TargetAddress,
    kind: TargetKind,
    state: ResolvedState,
    summary: TargetSummary,
    locator: ResolvedLocator,
) -> ResolvedTarget {
    ResolvedTarget {
        snapshot: TargetSnapshot {
            address,
            kind,
            selection_span: state.selection_span,
            revision: document.revision().clone(),
            guard: state.guard,
            summary,
        },
        locator,
        index_instance: document.index().instance_id(),
        containment_span: state.containment_span,
    }
}

fn frontmatter_targets(
    document: &Document,
    document_node: IndexNodeId,
) -> Result<Vec<ResolvedTarget>, CoreError> {
    let record = frontmatter::read(document)?;
    let guard = GuardAuthority::Frontmatter {
        span: record.span,
        etag: record.etag.clone(),
    };
    let mut targets = vec![resolved(
        document,
        TargetAddress::Frontmatter,
        TargetKind::Frontmatter,
        ResolvedState {
            selection_span: record.span,
            containment_span: record.span,
            guard: guard.clone(),
        },
        TargetSummary::Frontmatter {
            present: record.present,
            format: record.format,
        },
        ResolvedLocator::Node(document_node),
    )];
    let mut fields = Vec::new();
    collect_fields(&record.data, &[], &mut fields);
    for (path, value) in fields {
        targets.push(resolved(
            document,
            TargetAddress::FrontmatterField { path: path.clone() },
            TargetKind::FrontmatterField,
            ResolvedState {
                selection_span: None,
                containment_span: None,
                guard: guard.clone(),
            },
            TargetSummary::FrontmatterField {
                path: path.clone(),
                value: json_value_kind(Some(value)),
            },
            ResolvedLocator::FrontmatterField(path),
        ));
    }
    Ok(targets)
}

fn resolve_frontmatter_field(
    document: &Document,
    path: Vec<String>,
) -> Result<ResolvedTarget, CoreError> {
    validate_frontmatter_path(&path)?;
    let record = frontmatter::read(document)?;
    let value = project_frontmatter_field(&record.data, &path);
    Ok(resolved(
        document,
        TargetAddress::FrontmatterField { path: path.clone() },
        TargetKind::FrontmatterField,
        ResolvedState {
            selection_span: None,
            containment_span: None,
            guard: GuardAuthority::Frontmatter {
                span: record.span,
                etag: record.etag,
            },
        },
        TargetSummary::FrontmatterField {
            path: path.clone(),
            value: json_value_kind(value),
        },
        ResolvedLocator::FrontmatterField(path),
    ))
}

fn collect_fields<'a>(
    value: &'a serde_json::Value,
    prefix: &[String],
    output: &mut Vec<(Vec<String>, &'a serde_json::Value)>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    for (key, value) in object {
        let mut path = prefix.to_vec();
        path.push(key.clone());
        output.push((path.clone(), value));
        collect_fields(value, &path, output);
    }
}

fn query_matches(query: &TargetQuery, snapshot: &TargetSnapshot) -> bool {
    match query {
        TargetQuery::All => true,
        TargetQuery::Kind { kind } => snapshot.kind == *kind,
        TargetQuery::Section { text, match_mode } => match &snapshot.summary {
            TargetSummary::Section { heading, .. } => heading_matches(heading, text, *match_mode),
            _ => false,
        },
        TargetQuery::Task { status, contains } => match &snapshot.summary {
            TargetSummary::Task {
                status: actual,
                text,
                ..
            } => {
                status.is_none_or(|expected| expected == *actual)
                    && contains.as_ref().is_none_or(|needle| text.contains(needle))
            }
            _ => false,
        },
        TargetQuery::Link { text, destination } => match &snapshot.summary {
            TargetSummary::Link {
                text: actual_text,
                destination: actual_destination,
                ..
            } => {
                text.as_ref()
                    .is_none_or(|needle| actual_text.contains(needle))
                    && destination
                        .as_ref()
                        .is_none_or(|expected| actual_destination.as_ref() == Some(expected))
            }
            _ => false,
        },
        TargetQuery::FrontmatterField { path } => matches!(
            &snapshot.address,
            TargetAddress::FrontmatterField { path: actual } if actual == path
        ),
        TargetQuery::Search { .. } => false,
    }
}

fn validate_query(query: &TargetQuery) -> Result<(), CoreError> {
    if matches!(query, TargetQuery::FrontmatterField { path } if path.is_empty()) {
        Err(CoreError::InvalidSelector(
            "frontmatter field query path cannot be empty".into(),
        ))
    } else if matches!(query, TargetQuery::Search { text, .. } if text.is_empty()) {
        Err(CoreError::InvalidSelector(
            "search query text cannot be empty".into(),
        ))
    } else if matches!(query, TargetQuery::Section { text, match_mode }
        if text.is_empty() && NON_EMPTY_SECTION_MATCH_MODES.contains(match_mode))
    {
        Err(CoreError::InvalidSelector(
            "empty section text cannot be used with contains matching".into(),
        ))
    } else {
        Ok(())
    }
}

fn heading_matches(actual: &str, expected: &str, mode: HeadingMatchMode) -> bool {
    match mode {
        HeadingMatchMode::Exact => actual == expected,
        HeadingMatchMode::ExactIgnoreCase => actual.to_lowercase() == expected.to_lowercase(),
        HeadingMatchMode::Contains => actual.contains(expected),
        HeadingMatchMode::ContainsIgnoreCase => {
            actual.to_lowercase().contains(&expected.to_lowercase())
        }
    }
}

pub fn validate_address(address: &TargetAddress) -> Result<(), CoreError> {
    let validate_section = |section: &SectionAddress| match section {
        SectionAddress::Preamble => Ok(()),
        SectionAddress::Heading { path } => validate_heading_path(path),
    };
    match address {
        TargetAddress::FrontmatterField { path } => validate_frontmatter_path(path),
        TargetAddress::Section { path } => validate_heading_path(path),
        TargetAddress::Block { block } => validate_section(&block.section),
        TargetAddress::Task { block, path } => {
            validate_section(&block.section)?;
            if path.is_empty() {
                Err(invalid_address("task paths must not be empty"))
            } else {
                Ok(())
            }
        }
        TargetAddress::TableRow { table, .. } => validate_section(&table.section),
        TargetAddress::Link { parent, .. } => match parent {
            LinkParentAddress::Heading { section } => validate_heading_path(&section.path),
            LinkParentAddress::Block { block } => validate_section(&block.section),
        },
        TargetAddress::Document | TargetAddress::Frontmatter | TargetAddress::Preamble => Ok(()),
    }
}

fn validate_heading_path(path: &[HeadingAddressSegment]) -> Result<(), CoreError> {
    if path.is_empty() || path.iter().any(|segment| segment.occurrence == 0) {
        Err(invalid_address(
            "heading address paths must be non-empty with 1-based occurrences",
        ))
    } else {
        Ok(())
    }
}

fn validate_frontmatter_path(path: &[String]) -> Result<(), CoreError> {
    if path.is_empty() {
        Err(invalid_address(
            "frontmatter field paths must contain at least one key segment",
        ))
    } else {
        Ok(())
    }
}

fn invalid_address(reason: impl Into<String>) -> CoreError {
    CoreError::InvalidTargetAddress {
        reason: reason.into(),
    }
}

pub(crate) fn project_frontmatter_field<'a>(
    data: &'a serde_json::Value,
    path: &[String],
) -> Option<&'a serde_json::Value> {
    path.iter()
        .try_fold(data, |value, segment| value.get(segment))
}

fn etag(document: &Document, span: SourceSpan) -> Result<TargetEtag, CoreError> {
    Ok(TargetEtag::for_bytes(document.slice(&span)?.as_bytes()))
}

fn preamble_selection_span(
    index: &DocumentIndex,
    preamble: IndexNodeId,
    containment: SourceSpan,
) -> SourceSpan {
    index
        .children(preamble)
        .filter_map(|entry| match entry.node {
            IndexNode::BodyBlock { span, .. } => Some(span),
            _ => None,
        })
        .reduce(|left, right| SourceSpan {
            line_start: left.line_start.min(right.line_start),
            line_end: left.line_end.max(right.line_end),
            byte_start: left.byte_start.min(right.byte_start),
            byte_end: left.byte_end.max(right.byte_end),
        })
        .unwrap_or(SourceSpan {
            line_start: containment.line_start,
            line_end: containment.line_start,
            byte_start: containment.byte_start,
            byte_end: containment.byte_start,
        })
}

fn resolved_contains(source: &str, target: &ResolvedTarget, offset: u32) -> bool {
    let Some(span) = target.containment_span else {
        return false;
    };
    if span.byte_start <= offset && offset < span.byte_end {
        return true;
    }
    if target.snapshot.kind != TargetKind::TableRow || offset < span.byte_end {
        return false;
    }
    let tail = &source.as_bytes()[span.byte_end as usize..];
    let terminator_len = if tail.starts_with(b"\r\n") {
        2
    } else if tail.starts_with(b"\n") {
        1
    } else {
        0
    };
    offset < span.byte_end + terminator_len
}

fn json_value_kind(value: Option<&serde_json::Value>) -> JsonValueKind {
    match value {
        None => JsonValueKind::Missing,
        Some(serde_json::Value::Null) => JsonValueKind::Null,
        Some(serde_json::Value::Bool(_)) => JsonValueKind::Boolean,
        Some(serde_json::Value::Number(_)) => JsonValueKind::Number,
        Some(serde_json::Value::String(_)) => JsonValueKind::String,
        Some(serde_json::Value::Array(_)) => JsonValueKind::Array,
        Some(serde_json::Value::Object(_)) => JsonValueKind::Object,
    }
}

fn preview(content: &str) -> String {
    let flattened = content.replace(['\r', '\n', '\t'], " ");
    if flattened.chars().count() <= 80 {
        flattened
    } else {
        format!("{}...", flattened.chars().take(80).collect::<String>())
    }
}

impl std::fmt::Display for TargetAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document => formatter.write_str("document"),
            Self::Frontmatter => formatter.write_str("frontmatter"),
            Self::FrontmatterField { path } => write!(formatter, "frontmatter(path={path:?})"),
            Self::Preamble => formatter.write_str("preamble"),
            Self::Section { path } => write_heading_path(formatter, "section", path),
            Self::Block { block } => write!(formatter, "block({block})"),
            Self::Task { block, path } => write!(formatter, "task(block={block},path={path:?})"),
            Self::TableRow { table, row } => {
                write!(formatter, "table-row(table={table},row={row})")
            }
            Self::Link { parent, occurrence } => {
                write!(formatter, "link(parent={parent},occurrence={occurrence})")
            }
        }
    }
}

impl std::fmt::Display for BlockAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "section={},ordinal={}",
            self.section, self.ordinal
        )
    }
}

impl std::fmt::Display for SectionAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preamble => formatter.write_str("preamble"),
            Self::Heading { path } => write_heading_path(formatter, "heading", path),
        }
    }
}

impl std::fmt::Display for LinkParentAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Heading { section } => write_heading_path(formatter, "heading", &section.path),
            Self::Block { block } => write!(formatter, "block={block}"),
        }
    }
}

fn write_heading_path(
    formatter: &mut std::fmt::Formatter<'_>,
    label: &str,
    path: &[HeadingAddressSegment],
) -> std::fmt::Result {
    let mut rendered = String::new();
    for (index, segment) in path.iter().enumerate() {
        if index > 0 {
            rendered.push('/');
        }
        let _ = write!(
            rendered,
            "{}#{}",
            segment.text.replace('/', "\\/"),
            segment.occurrence
        );
    }
    write!(formatter, "{label}(path={rendered})")
}
