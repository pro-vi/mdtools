use serde::Serialize;

use crate::core_error::CoreError;
use crate::document::Document;
use crate::frontmatter;
use crate::index::IndexNode;
use crate::model::{BlockKind, ColumnAlignment, FrontmatterFormat, LinkKind, TaskStatus};
use crate::target::{ResolvedLocator, ResolvedTarget, TargetKind, TargetSnapshot};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentRead {
    pub snapshot: TargetSnapshot,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PreambleRead {
    pub snapshot: TargetSnapshot,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SectionRead {
    pub snapshot: TargetSnapshot,
    pub level: u8,
    pub heading: String,
    pub markdown: String,
    pub fragment: crate::fragment::SectionFragment,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BlockRead {
    pub snapshot: TargetSnapshot,
    pub kind: BlockKind,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TaskRead {
    pub snapshot: TargetSnapshot,
    pub status: TaskStatus,
    pub depth: u32,
    pub summary: String,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TableRead {
    pub snapshot: TargetSnapshot,
    pub markdown: String,
    pub headers: Vec<String>,
    pub alignments: Vec<ColumnAlignment>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TableRowRead {
    pub snapshot: TargetSnapshot,
    pub row: u32,
    pub cells: Vec<String>,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FrontmatterRead {
    pub snapshot: TargetSnapshot,
    pub present: bool,
    pub format: Option<FrontmatterFormat>,
    pub raw: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FrontmatterFieldRead {
    pub snapshot: TargetSnapshot,
    pub path: Vec<String>,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkRead {
    pub snapshot: TargetSnapshot,
    pub kind: LinkKind,
    pub text: String,
    pub destination: Option<String>,
    pub title: Option<String>,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetRead {
    Document(DocumentRead),
    Preamble(PreambleRead),
    Section(SectionRead),
    Block(BlockRead),
    Task(TaskRead),
    Table(TableRead),
    TableRow(TableRowRead),
    Frontmatter(FrontmatterRead),
    FrontmatterField(FrontmatterFieldRead),
    Link(LinkRead),
}

impl TargetRead {
    pub fn snapshot(&self) -> &TargetSnapshot {
        match self {
            Self::Document(value) => &value.snapshot,
            Self::Preamble(value) => &value.snapshot,
            Self::Section(value) => &value.snapshot,
            Self::Block(value) => &value.snapshot,
            Self::Task(value) => &value.snapshot,
            Self::Table(value) => &value.snapshot,
            Self::TableRow(value) => &value.snapshot,
            Self::Frontmatter(value) => &value.snapshot,
            Self::FrontmatterField(value) => &value.snapshot,
            Self::Link(value) => &value.snapshot,
        }
    }
}

pub fn read(document: &Document, target: &ResolvedTarget) -> Result<TargetRead, CoreError> {
    target.ensure_document(document)?;
    let snapshot = target.snapshot().clone();
    match (snapshot.kind, target.locator()) {
        (TargetKind::Document, ResolvedLocator::Node(_)) => {
            Ok(TargetRead::Document(DocumentRead {
                snapshot,
                source: document.source().to_string(),
            }))
        }
        (TargetKind::Frontmatter, ResolvedLocator::Node(_)) => {
            let record = frontmatter::read(document)?;
            Ok(TargetRead::Frontmatter(FrontmatterRead {
                snapshot,
                present: record.present,
                format: record.format,
                raw: record.raw,
                data: record.data,
            }))
        }
        (TargetKind::FrontmatterField, ResolvedLocator::FrontmatterField(path)) => {
            let record = frontmatter::read(document)?;
            Ok(TargetRead::FrontmatterField(FrontmatterFieldRead {
                snapshot,
                path: path.clone(),
                value: crate::target::project_frontmatter_field(&record.data, path)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            }))
        }
        (_, ResolvedLocator::Node(node)) => read_index_node(document, *node, snapshot),
        (actual, _) => Err(kind_mismatch("resolved target", actual)),
    }
}

fn read_index_node(
    document: &Document,
    node: crate::index::IndexNodeId,
    snapshot: TargetSnapshot,
) -> Result<TargetRead, CoreError> {
    let entry = document.index().entry(node);
    match &entry.node {
        IndexNode::Preamble { .. } => {
            let span = snapshot
                .selection_span
                .ok_or(CoreError::TargetKindMismatch {
                    expected: "preamble selection",
                    actual: "missing selection",
                })?;
            Ok(TargetRead::Preamble(PreambleRead {
                snapshot,
                markdown: document.slice(&span)?.to_string(),
            }))
        }
        IndexNode::Section {
            span, level, text, ..
        } => {
            let fragment = crate::fragment::SectionFragment::from_placed_section(document, *span)?;
            Ok(TargetRead::Section(SectionRead {
                snapshot,
                level: *level,
                heading: text.clone(),
                markdown: document.slice(span)?.to_string(),
                fragment,
            }))
        }
        IndexNode::BodyBlock {
            span,
            parser_index,
            kind,
            ..
        } => {
            if *kind == BlockKind::Table {
                let table = document.index().projection().blocks[*parser_index as usize]
                    .table
                    .as_ref()
                    .ok_or_else(|| {
                        CoreError::ParseFailed(format!(
                            "table block {parser_index} is missing its cached projection"
                        ))
                    })?;
                Ok(TargetRead::Table(TableRead {
                    snapshot,
                    markdown: document.slice(span)?.to_string(),
                    headers: table.headers.clone(),
                    alignments: table.alignments.clone(),
                    rows: table.rows.iter().map(|row| row.cells.clone()).collect(),
                }))
            } else {
                Ok(TargetRead::Block(BlockRead {
                    snapshot,
                    kind: *kind,
                    markdown: document.slice(span)?.to_string(),
                }))
            }
        }
        IndexNode::TaskItem {
            span,
            status,
            depth,
            summary_text,
            ..
        } => Ok(TargetRead::Task(TaskRead {
            snapshot,
            status: *status,
            depth: *depth,
            summary: summary_text.clone(),
            markdown: document.slice(span)?.to_string(),
        })),
        IndexNode::TableRow {
            span,
            ordinal,
            cells,
        } => Ok(TargetRead::TableRow(TableRowRead {
            snapshot,
            row: *ordinal,
            cells: cells.clone(),
            markdown: document.slice(span)?.to_string(),
        })),
        IndexNode::Link {
            span,
            kind,
            text,
            destination,
            title,
            ..
        } => Ok(TargetRead::Link(LinkRead {
            snapshot,
            kind: *kind,
            text: text.clone(),
            destination: destination.clone(),
            title: title.clone(),
            markdown: document.slice(span)?.to_string(),
        })),
        other => Err(CoreError::TargetKindMismatch {
            expected: "addressable read target",
            actual: index_kind_name(other),
        }),
    }
}

impl ResolvedTarget {
    pub fn read(&self, document: &Document) -> Result<TargetRead, CoreError> {
        read(document, self)
    }

    pub fn read_document(&self, document: &Document) -> Result<DocumentRead, CoreError> {
        match self.read(document)? {
            TargetRead::Document(value) => Ok(value),
            _ => Err(kind_mismatch("document", self.snapshot().kind)),
        }
    }

    pub fn read_preamble(&self, document: &Document) -> Result<PreambleRead, CoreError> {
        match self.read(document)? {
            TargetRead::Preamble(value) => Ok(value),
            _ => Err(kind_mismatch("preamble", self.snapshot().kind)),
        }
    }

    pub fn read_section(&self, document: &Document) -> Result<SectionRead, CoreError> {
        match self.read(document)? {
            TargetRead::Section(value) => Ok(value),
            _ => Err(kind_mismatch("section", self.snapshot().kind)),
        }
    }

    pub fn read_block(&self, document: &Document) -> Result<BlockRead, CoreError> {
        match self.read(document)? {
            TargetRead::Block(value) => Ok(value),
            TargetRead::Table(_) => Err(kind_mismatch("non-table block", self.snapshot().kind)),
            _ => Err(kind_mismatch("block", self.snapshot().kind)),
        }
    }

    pub fn read_task(&self, document: &Document) -> Result<TaskRead, CoreError> {
        match self.read(document)? {
            TargetRead::Task(value) => Ok(value),
            _ => Err(kind_mismatch("task", self.snapshot().kind)),
        }
    }

    pub fn read_table(&self, document: &Document) -> Result<TableRead, CoreError> {
        match self.read(document)? {
            TargetRead::Table(value) => Ok(value),
            _ => Err(kind_mismatch("table block", self.snapshot().kind)),
        }
    }

    pub fn read_table_row(&self, document: &Document) -> Result<TableRowRead, CoreError> {
        match self.read(document)? {
            TargetRead::TableRow(value) => Ok(value),
            _ => Err(kind_mismatch("table row", self.snapshot().kind)),
        }
    }

    pub fn read_frontmatter(&self, document: &Document) -> Result<FrontmatterRead, CoreError> {
        match self.read(document)? {
            TargetRead::Frontmatter(value) => Ok(value),
            _ => Err(kind_mismatch("frontmatter", self.snapshot().kind)),
        }
    }

    pub fn read_frontmatter_field(
        &self,
        document: &Document,
    ) -> Result<FrontmatterFieldRead, CoreError> {
        match self.read(document)? {
            TargetRead::FrontmatterField(value) => Ok(value),
            _ => Err(kind_mismatch("frontmatter field", self.snapshot().kind)),
        }
    }

    pub fn read_link(&self, document: &Document) -> Result<LinkRead, CoreError> {
        match self.read(document)? {
            TargetRead::Link(value) => Ok(value),
            _ => Err(kind_mismatch("link", self.snapshot().kind)),
        }
    }
}

fn kind_mismatch(expected: &'static str, actual: TargetKind) -> CoreError {
    CoreError::TargetKindMismatch {
        expected,
        actual: target_kind_name(actual),
    }
}

fn target_kind_name(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Document => "document",
        TargetKind::Frontmatter => "frontmatter",
        TargetKind::FrontmatterField => "frontmatter field",
        TargetKind::Preamble => "preamble",
        TargetKind::Section => "section",
        TargetKind::Block => "block",
        TargetKind::Task => "task",
        TargetKind::TableRow => "table row",
        TargetKind::Link => "link",
    }
}

fn index_kind_name(node: &IndexNode) -> &'static str {
    match node {
        IndexNode::Document { .. } => "document",
        IndexNode::Frontmatter { .. } => "frontmatter",
        IndexNode::Preamble { .. } => "preamble",
        IndexNode::Section { .. } => "section",
        IndexNode::Heading { .. } => "heading",
        IndexNode::HeadingMarker { .. } => "heading marker",
        IndexNode::BodyBlock { .. } => "body block",
        IndexNode::TaskItem { .. } => "task",
        IndexNode::TableRow { .. } => "table row",
        IndexNode::Link { .. } => "link",
    }
}
