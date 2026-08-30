use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core_error::CoreError;
use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::fragment::SectionFragment;
use crate::index::IndexNode;
use crate::model::{BlockKind, MutationDisposition, SourceSpan};
use crate::revision::DocumentRevision;
use crate::target::{
    BlockAddress, GuardAuthority, ResolvedLocator, TargetAddress, TargetKind, TargetSnapshot,
};

mod planner;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub base_revision: DocumentRevision,
    #[schemars(length(min = 1))]
    pub operations: Vec<PatchOp>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum PatchOp {
    ReplaceBlock {
        target: ReplaceBlockTarget,
        markdown: String,
    },
    DeleteBlock {
        target: ReplaceBlockTarget,
    },
    InsertBlock {
        target: BlockInsertionTarget,
        markdown: String,
    },
    MoveBlock {
        source: ReplaceBlockTarget,
        destination: ReplaceBlockTarget,
        position: RelativePosition,
    },
    ReplaceSection {
        target: HeadingPatchTarget,
        fragment: SectionFragment,
    },
    InsertSection {
        target: SectionInsertionTarget,
        fragment: SectionFragment,
    },
    ReplacePreamble {
        target: PreamblePatchTarget,
        #[schemars(length(min = 1))]
        #[serde(deserialize_with = "deserialize_non_empty_string")]
        markdown: String,
    },
    DeleteSection {
        target: SectionPatchTarget,
    },
    MoveSection {
        source: HeadingPatchTarget,
        destination: HeadingPatchTarget,
        position: SectionMovePosition,
        keep_level: bool,
    },
    SetTaskStatus {
        target: TaskPatchTarget,
        status: crate::model::TaskStatus,
    },
    SetFrontmatter {
        target: FrontmatterPatchTarget,
        value: serde_json::Value,
    },
    DeleteFrontmatter {
        target: FrontmatterPatchTarget,
    },
    ReplaceTableRow {
        target: TableRowPatchTarget,
        markdown: String,
    },
    InsertTableRow {
        target: TablePatchTarget,
        row: u32,
        markdown: String,
    },
    DeleteTableRow {
        target: TableRowPatchTarget,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaceBlockTarget {
    pub address: BlockAddress,
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectionGuard {
    pub span: SourceSpan,
    pub etag: TargetEtag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelativePosition {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocumentEdge {
    Start,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BlockInsertionTarget {
    Before {
        anchor: ReplaceBlockTarget,
    },
    After {
        anchor: ReplaceBlockTarget,
    },
    DocumentEdge {
        edge: DocumentEdge,
        revision: DocumentRevision,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BlockInsertionEvidence {
    Before {
        anchor: BlockIdentity,
    },
    After {
        anchor: BlockIdentity,
    },
    DocumentEdge {
        edge: DocumentEdge,
        revision: DocumentRevision,
    },
}

impl From<&BlockInsertionTarget> for BlockInsertionEvidence {
    fn from(target: &BlockInsertionTarget) -> Self {
        match target {
            BlockInsertionTarget::Before { anchor } => Self::Before {
                anchor: BlockIdentity {
                    address: anchor.address.clone(),
                    revision: anchor.revision.clone(),
                },
            },
            BlockInsertionTarget::After { anchor } => Self::After {
                anchor: BlockIdentity {
                    address: anchor.address.clone(),
                    revision: anchor.revision.clone(),
                },
            },
            BlockInsertionTarget::DocumentEdge { edge, revision } => Self::DocumentEdge {
                edge: *edge,
                revision: revision.clone(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SectionMovePosition {
    BeforeSibling,
    AfterSibling,
    IntoAsChild,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SectionPatchTarget {
    pub address: crate::target::SectionAddress,
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeadingPatchTarget {
    #[schemars(length(min = 1))]
    #[serde(deserialize_with = "deserialize_non_empty")]
    pub path: Vec<crate::target::HeadingAddressSegment>,
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SectionInsertionTarget {
    pub parent: HeadingPatchTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreamblePatchTarget {
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
}

impl HeadingPatchTarget {
    fn address(&self) -> crate::target::SectionAddress {
        crate::target::SectionAddress::Heading {
            path: self.path.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskPatchTarget {
    pub block: BlockAddress,
    #[schemars(length(min = 1))]
    pub path: Vec<u32>,
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerGuard {
    pub address: BlockAddress,
    pub span: SourceSpan,
    pub etag: TargetEtag,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TableRowPatchTarget {
    pub table: BlockAddress,
    pub row: u32,
    pub revision: DocumentRevision,
    pub guard: ContainerGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TablePatchTarget {
    pub table: BlockAddress,
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrontmatterGuard {
    pub span: Option<SourceSpan>,
    pub etag: TargetEtag,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrontmatterPatchTarget {
    #[schemars(length(min = 1))]
    pub path: Vec<String>,
    pub revision: DocumentRevision,
    pub guard: FrontmatterGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockIdentity {
    pub address: BlockAddress,
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SectionIdentity {
    pub address: crate::target::SectionAddress,
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeadingSectionIdentity {
    #[schemars(length(min = 1))]
    #[serde(deserialize_with = "deserialize_non_empty")]
    pub path: Vec<crate::target::HeadingAddressSegment>,
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreambleIdentity {
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskIdentity {
    pub block: BlockAddress,
    #[schemars(length(min = 1))]
    pub path: Vec<u32>,
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrontmatterFieldIdentity {
    #[schemars(length(min = 1))]
    pub path: Vec<String>,
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TableRowIdentity {
    pub table: BlockAddress,
    pub row: u32,
    pub revision: DocumentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum PatchReceipt {
    ReplaceBlock {
        outcome: ReplaceBlockOutcome,
    },
    DeleteBlock {
        before: BlockIdentity,
        result_revision: DocumentRevision,
    },
    InsertBlock {
        outcome: InsertBlockOutcome,
    },
    MoveBlock {
        destination_before: BlockIdentity,
        outcome: MoveBlockOutcome,
    },
    ReplaceSection {
        outcome: ReplaceSectionOutcome,
    },
    InsertSection {
        outcome: InsertSectionOutcome,
    },
    ReplacePreamble {
        outcome: ReplacePreambleOutcome,
    },
    DeleteSection {
        outcome: DeleteSectionOutcome,
    },
    MoveSection {
        destination_before: SectionIdentity,
        outcome: MoveSectionOutcome,
    },
    SetTaskStatus {
        outcome: SetTaskOutcome,
    },
    SetFrontmatter {
        outcome: SetFrontmatterOutcome,
    },
    DeleteFrontmatter {
        outcome: DeleteFrontmatterOutcome,
    },
    ReplaceTableRow {
        outcome: ReplaceTableRowOutcome,
    },
    InsertTableRow {
        table_before: BlockIdentity,
        after: TableRowIdentity,
    },
    DeleteTableRow {
        before: TableRowIdentity,
        result_revision: DocumentRevision,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReplaceBlockOutcome {
    NoChange {
        before: ReplaceBlockState,
        after: ReplaceBlockState,
    },
    Replaced {
        before: ReplaceBlockState,
        after: ReplaceBlockState,
    },
    Deleted {
        before: ReplaceBlockState,
        result_revision: DocumentRevision,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaceBlockState {
    pub address: BlockAddress,
    pub revision: DocumentRevision,
    pub guard: SelectionGuard,
    pub block_kind: BlockKind,
    pub preview: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum InsertBlockOutcome {
    NoChange {
        target: BlockInsertionEvidence,
        result_revision: DocumentRevision,
    },
    Inserted {
        target: BlockInsertionEvidence,
        after: BlockIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum MoveBlockOutcome {
    NoChange {
        before: BlockIdentity,
        after: BlockIdentity,
    },
    Replaced {
        before: BlockIdentity,
        after: BlockIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReplaceSectionOutcome {
    NoChange {
        before: HeadingSectionIdentity,
        after: HeadingSectionIdentity,
    },
    Replaced {
        before: HeadingSectionIdentity,
        after: HeadingSectionIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum InsertSectionOutcome {
    Inserted {
        parent_before: HeadingSectionIdentity,
        after: HeadingSectionIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReplacePreambleOutcome {
    NoChange {
        before: PreambleIdentity,
        after: PreambleIdentity,
    },
    Replaced {
        before: PreambleIdentity,
        after: PreambleIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeleteSectionOutcome {
    NoChange {
        before: SectionIdentity,
        after: SectionIdentity,
    },
    Deleted {
        before: SectionIdentity,
        result_revision: DocumentRevision,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum MoveSectionOutcome {
    NoChange {
        before: SectionIdentity,
        after: SectionIdentity,
    },
    Replaced {
        before: SectionIdentity,
        after: SectionIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum SetTaskOutcome {
    NoChange {
        before: TaskIdentity,
        after: TaskIdentity,
    },
    Replaced {
        before: TaskIdentity,
        after: TaskIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum SetFrontmatterOutcome {
    NoChange {
        before: FrontmatterFieldIdentity,
        after: FrontmatterFieldIdentity,
    },
    Inserted {
        before: FrontmatterFieldIdentity,
        after: FrontmatterFieldIdentity,
    },
    Replaced {
        before: FrontmatterFieldIdentity,
        after: FrontmatterFieldIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeleteFrontmatterOutcome {
    NoChange {
        before: FrontmatterFieldIdentity,
        after: FrontmatterFieldIdentity,
    },
    Deleted {
        before: FrontmatterFieldIdentity,
        after: FrontmatterFieldIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReplaceTableRowOutcome {
    NoChange {
        before: TableRowIdentity,
        after: TableRowIdentity,
    },
    Replaced {
        before: TableRowIdentity,
        after: TableRowIdentity,
    },
}

pub struct PatchOutcome {
    pub document: Document,
    pub receipts: Vec<PatchReceipt>,
}

fn deserialize_non_empty<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    let values = Vec::<T>::deserialize(deserializer)?;
    if values.is_empty() {
        Err(serde::de::Error::custom("path must not be empty"))
    } else {
        Ok(values)
    }
}

fn deserialize_non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        Err(serde::de::Error::custom("markdown must not be empty"))
    } else {
        Ok(value)
    }
}

impl TryFrom<&TargetSnapshot> for ReplaceBlockTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Block { block } = &snapshot.address else {
            return Err(CoreError::InvalidPatch(
                "replace_block requires a block address".into(),
            ));
        };
        if snapshot.kind != TargetKind::Block {
            return Err(CoreError::InvalidPatch(
                "replace_block requires block target evidence".into(),
            ));
        }
        let GuardAuthority::Selection { span, etag } = &snapshot.guard else {
            return Err(CoreError::InvalidPatch(
                "replace_block requires selection guard authority".into(),
            ));
        };
        if snapshot.selection_span != Some(*span) {
            return Err(CoreError::InvalidPatch(
                "block selection and guard spans must agree".into(),
            ));
        }
        Ok(Self {
            address: block.clone(),
            revision: snapshot.revision.clone(),
            guard: SelectionGuard {
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for SectionPatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let address = match &snapshot.address {
            TargetAddress::Preamble => crate::target::SectionAddress::Preamble,
            TargetAddress::Section { path } => {
                crate::target::SectionAddress::Heading { path: path.clone() }
            }
            _ => {
                return Err(CoreError::InvalidPatch(
                    "section operation requires section evidence".into(),
                ));
            }
        };
        let GuardAuthority::Selection { span, etag } = &snapshot.guard else {
            return Err(CoreError::InvalidPatch(
                "section operation requires selection authority".into(),
            ));
        };
        Ok(Self {
            address,
            revision: snapshot.revision.clone(),
            guard: SelectionGuard {
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for HeadingPatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Section { path } = &snapshot.address else {
            return Err(CoreError::InvalidPatch(
                "section move requires heading-section evidence".into(),
            ));
        };
        if snapshot.kind != TargetKind::Section {
            return Err(CoreError::InvalidPatch(
                "section move requires a section target".into(),
            ));
        }
        let GuardAuthority::Selection { span, etag } = &snapshot.guard else {
            return Err(CoreError::InvalidPatch(
                "section move requires selection authority".into(),
            ));
        };
        Ok(Self {
            path: path.clone(),
            revision: snapshot.revision.clone(),
            guard: SelectionGuard {
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for SectionInsertionTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            parent: HeadingPatchTarget::try_from(snapshot)?,
        })
    }
}

impl TryFrom<&TargetSnapshot> for PreamblePatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        if snapshot.kind != TargetKind::Preamble || snapshot.address != TargetAddress::Preamble {
            return Err(CoreError::InvalidPatch(
                "replace_preamble requires preamble evidence".into(),
            ));
        }
        let GuardAuthority::Selection { span, etag } = &snapshot.guard else {
            return Err(CoreError::InvalidPatch(
                "replace_preamble requires selection authority".into(),
            ));
        };
        Ok(Self {
            revision: snapshot.revision.clone(),
            guard: SelectionGuard {
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for TaskPatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Task { block, path } = &snapshot.address else {
            return Err(CoreError::InvalidPatch(
                "task operation requires task evidence".into(),
            ));
        };
        let GuardAuthority::Selection { span, etag } = &snapshot.guard else {
            return Err(CoreError::InvalidPatch(
                "task operation requires selection authority".into(),
            ));
        };
        Ok(Self {
            block: block.clone(),
            path: path.clone(),
            revision: snapshot.revision.clone(),
            guard: SelectionGuard {
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for TableRowPatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::TableRow { table, row } = &snapshot.address else {
            return Err(CoreError::InvalidPatch(
                "table row operation requires row evidence".into(),
            ));
        };
        let GuardAuthority::Container {
            address,
            span,
            etag,
        } = &snapshot.guard
        else {
            return Err(CoreError::InvalidPatch(
                "table row operation requires container authority".into(),
            ));
        };
        let TargetAddress::Block { block: guard_table } = address.as_ref() else {
            return Err(CoreError::InvalidPatch(
                "table row guard must name its table block".into(),
            ));
        };
        if guard_table != table {
            return Err(CoreError::InvalidPatch(
                "table row address and guard container must agree".into(),
            ));
        }
        Ok(Self {
            table: table.clone(),
            row: *row,
            revision: snapshot.revision.clone(),
            guard: ContainerGuard {
                address: guard_table.clone(),
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for TablePatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Block { block } = &snapshot.address else {
            return Err(CoreError::InvalidPatch(
                "table operation requires block evidence".into(),
            ));
        };
        if !matches!(
            snapshot.summary,
            crate::target::TargetSummary::Block {
                kind: BlockKind::Table,
                ..
            }
        ) {
            return Err(CoreError::InvalidPatch(
                "table operation requires a table block".into(),
            ));
        }
        let replace = ReplaceBlockTarget::try_from(snapshot)?;
        Ok(Self {
            table: block.clone(),
            revision: replace.revision,
            guard: replace.guard,
        })
    }
}

impl TryFrom<&TargetSnapshot> for FrontmatterPatchTarget {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::FrontmatterField { path } = &snapshot.address else {
            return Err(CoreError::InvalidPatch(
                "frontmatter operation requires field evidence".into(),
            ));
        };
        let GuardAuthority::Frontmatter { span, etag } = &snapshot.guard else {
            return Err(CoreError::InvalidPatch(
                "frontmatter operation requires frontmatter authority".into(),
            ));
        };
        Ok(Self {
            path: path.clone(),
            revision: snapshot.revision.clone(),
            guard: FrontmatterGuard {
                span: *span,
                etag: etag.clone(),
            },
        })
    }
}

impl TryFrom<&TargetSnapshot> for ReplaceBlockState {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Block { block } = &snapshot.address else {
            return Err(CoreError::PatchInvariant(
                "block receipt state requires a block address".into(),
            ));
        };
        let GuardAuthority::Selection { span, etag } = &snapshot.guard else {
            return Err(CoreError::PatchInvariant(
                "block receipt state requires selection authority".into(),
            ));
        };
        let crate::target::TargetSummary::Block { kind, preview } = &snapshot.summary else {
            return Err(CoreError::PatchInvariant(
                "block receipt state requires block summary".into(),
            ));
        };
        if snapshot.kind != TargetKind::Block || snapshot.selection_span != Some(*span) {
            return Err(CoreError::PatchInvariant(
                "block receipt state is internally inconsistent".into(),
            ));
        }
        Ok(Self {
            address: block.clone(),
            revision: snapshot.revision.clone(),
            guard: SelectionGuard {
                span: *span,
                etag: etag.clone(),
            },
            block_kind: *kind,
            preview: preview.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for BlockIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Block { block } = &snapshot.address else {
            return Err(CoreError::PatchInvariant(
                "block identity requires a block address".into(),
            ));
        };
        if snapshot.kind != TargetKind::Block {
            return Err(CoreError::PatchInvariant(
                "block identity requires a block target".into(),
            ));
        }
        Ok(Self {
            address: block.clone(),
            revision: snapshot.revision.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for SectionIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let address = match &snapshot.address {
            TargetAddress::Preamble => crate::target::SectionAddress::Preamble,
            TargetAddress::Section { path } => {
                crate::target::SectionAddress::Heading { path: path.clone() }
            }
            _ => {
                return Err(CoreError::PatchInvariant(
                    "section identity requires a section address".into(),
                ));
            }
        };
        if !matches!(snapshot.kind, TargetKind::Preamble | TargetKind::Section) {
            return Err(CoreError::PatchInvariant(
                "section identity requires a section target".into(),
            ));
        }
        Ok(Self {
            address,
            revision: snapshot.revision.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for HeadingSectionIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Section { path } = &snapshot.address else {
            return Err(CoreError::PatchInvariant(
                "heading section identity requires a heading address".into(),
            ));
        };
        if snapshot.kind != TargetKind::Section {
            return Err(CoreError::PatchInvariant(
                "heading section identity requires a section target".into(),
            ));
        }
        Ok(Self {
            path: path.clone(),
            revision: snapshot.revision.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for PreambleIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        if snapshot.kind != TargetKind::Preamble || snapshot.address != TargetAddress::Preamble {
            return Err(CoreError::PatchInvariant(
                "preamble identity requires a preamble target".into(),
            ));
        }
        Ok(Self {
            revision: snapshot.revision.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for TaskIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::Task { block, path } = &snapshot.address else {
            return Err(CoreError::PatchInvariant(
                "task identity requires a task address".into(),
            ));
        };
        if snapshot.kind != TargetKind::Task {
            return Err(CoreError::PatchInvariant(
                "task identity requires a task target".into(),
            ));
        }
        Ok(Self {
            block: block.clone(),
            path: path.clone(),
            revision: snapshot.revision.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for FrontmatterFieldIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::FrontmatterField { path } = &snapshot.address else {
            return Err(CoreError::PatchInvariant(
                "frontmatter identity requires a field address".into(),
            ));
        };
        if snapshot.kind != TargetKind::FrontmatterField {
            return Err(CoreError::PatchInvariant(
                "frontmatter identity requires a field target".into(),
            ));
        }
        Ok(Self {
            path: path.clone(),
            revision: snapshot.revision.clone(),
        })
    }
}

impl TryFrom<&TargetSnapshot> for TableRowIdentity {
    type Error = CoreError;

    fn try_from(snapshot: &TargetSnapshot) -> Result<Self, Self::Error> {
        let TargetAddress::TableRow { table, row } = &snapshot.address else {
            return Err(CoreError::PatchInvariant(
                "table-row identity requires a row address".into(),
            ));
        };
        if snapshot.kind != TargetKind::TableRow {
            return Err(CoreError::PatchInvariant(
                "table-row identity requires a row target".into(),
            ));
        }
        Ok(Self {
            table: table.clone(),
            row: *row,
            revision: snapshot.revision.clone(),
        })
    }
}

impl PatchReceipt {
    pub fn disposition(&self) -> MutationDisposition {
        match self {
            Self::ReplaceBlock { outcome } => outcome.disposition(),
            Self::DeleteBlock { .. } | Self::DeleteTableRow { .. } => MutationDisposition::Deleted,
            Self::DeleteSection { outcome } => outcome.disposition(),
            Self::DeleteFrontmatter { outcome } => outcome.disposition(),
            Self::InsertTableRow { .. } => MutationDisposition::Inserted,
            Self::InsertBlock { outcome } => outcome.disposition(),
            Self::MoveBlock { outcome, .. } => outcome.disposition(),
            Self::ReplaceSection { outcome } => outcome.disposition(),
            Self::InsertSection { outcome } => outcome.disposition(),
            Self::ReplacePreamble { outcome } => outcome.disposition(),
            Self::MoveSection { outcome, .. } => outcome.disposition(),
            Self::SetTaskStatus { outcome } => outcome.disposition(),
            Self::SetFrontmatter { outcome } => outcome.disposition(),
            Self::ReplaceTableRow { outcome } => outcome.disposition(),
        }
    }

    pub fn replace_block_before(&self) -> Option<&ReplaceBlockState> {
        match self {
            Self::ReplaceBlock { outcome, .. } => Some(outcome.before()),
            _ => None,
        }
    }

    pub fn replace_block_after(&self) -> Option<&ReplaceBlockState> {
        match self {
            Self::ReplaceBlock { outcome, .. } => outcome.after(),
            _ => None,
        }
    }
}

impl ReplaceBlockOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
            Self::Deleted { .. } => MutationDisposition::Deleted,
        }
    }

    fn before(&self) -> &ReplaceBlockState {
        match self {
            Self::NoChange { before, .. } => before,
            Self::Replaced { before, .. } | Self::Deleted { before, .. } => before,
        }
    }

    fn after(&self) -> Option<&ReplaceBlockState> {
        match self {
            Self::NoChange { after, .. } => Some(after),
            Self::Replaced { after, .. } => Some(after),
            Self::Deleted { .. } => None,
        }
    }
}

impl InsertBlockOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Inserted { .. } => MutationDisposition::Inserted,
        }
    }
}

impl MoveBlockOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl ReplaceSectionOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl InsertSectionOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::Inserted { .. } => MutationDisposition::Inserted,
        }
    }
}

impl ReplacePreambleOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl DeleteSectionOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Deleted { .. } => MutationDisposition::Deleted,
        }
    }
}

impl MoveSectionOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl SetTaskOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl SetFrontmatterOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Inserted { .. } => MutationDisposition::Inserted,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl DeleteFrontmatterOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Deleted { .. } => MutationDisposition::Deleted,
        }
    }
}

impl ReplaceTableRowOutcome {
    fn disposition(&self) -> MutationDisposition {
        match self {
            Self::NoChange { .. } => MutationDisposition::NoChange,
            Self::Replaced { .. } => MutationDisposition::Replaced,
        }
    }
}

impl Patch {
    pub fn apply(&self, document: &Document) -> Result<PatchOutcome, CoreError> {
        planner::apply(self, document)
    }
}

fn preflight_operation_evidence(document: &Document, operation: &PatchOp) -> Result<(), CoreError> {
    match operation {
        PatchOp::ReplaceBlock { target, .. } | PatchOp::DeleteBlock { target } => {
            verify_block_target(document, target)
        }
        PatchOp::InsertBlock { target, .. } => match target {
            BlockInsertionTarget::Before { anchor } | BlockInsertionTarget::After { anchor } => {
                verify_block_target(document, anchor)
            }
            BlockInsertionTarget::DocumentEdge { revision, .. } => {
                verify_revision(document, revision)
            }
        },
        PatchOp::MoveBlock {
            source,
            destination,
            ..
        } => {
            verify_block_target(document, source)?;
            verify_block_target(document, destination)
        }
        PatchOp::ReplaceSection { target, .. } => verify_heading_target(document, target),
        PatchOp::InsertSection { target, .. } => verify_heading_target(document, &target.parent),
        PatchOp::ReplacePreamble { target, .. } => verify_preamble_target(document, target),
        PatchOp::DeleteSection { target } => verify_section_target(document, target),
        PatchOp::MoveSection {
            source,
            destination,
            ..
        } => {
            verify_heading_target(document, source)?;
            verify_heading_target(document, destination)
        }
        PatchOp::SetTaskStatus { target, .. } => verify_task_target(document, target),
        PatchOp::SetFrontmatter { target, .. } | PatchOp::DeleteFrontmatter { target } => {
            verify_frontmatter_target(document, target)
        }
        PatchOp::ReplaceTableRow { target, .. } | PatchOp::DeleteTableRow { target } => {
            verify_table_row_target(document, target)
        }
        PatchOp::InsertTableRow { target, .. } => verify_table_target(document, target),
    }
}

fn frontmatter_identity(
    document: &Document,
    path: &[String],
) -> Result<FrontmatterFieldIdentity, CoreError> {
    let resolved = document.resolve(&TargetAddress::FrontmatterField {
        path: path.to_vec(),
    })?;
    FrontmatterFieldIdentity::try_from(resolved.snapshot())
}

fn resolve_section_snapshot(
    document: &Document,
    address: &crate::target::SectionAddress,
) -> Result<TargetSnapshot, CoreError> {
    let address = match address {
        crate::target::SectionAddress::Preamble => TargetAddress::Preamble,
        crate::target::SectionAddress::Heading { path } => {
            TargetAddress::Section { path: path.clone() }
        }
    };
    Ok(document.resolve(&address)?.snapshot().clone())
}

fn insertion_base_anchor(document: &Document, target: &BlockInsertionTarget) -> usize {
    match target {
        BlockInsertionTarget::Before { anchor } => anchor.guard.span.byte_start as usize,
        BlockInsertionTarget::After { anchor } => anchor.guard.span.byte_end as usize,
        BlockInsertionTarget::DocumentEdge {
            edge: DocumentEdge::Start,
            ..
        } => document_start_anchor(document),
        BlockInsertionTarget::DocumentEdge {
            edge: DocumentEdge::End,
            ..
        } => document.source().len(),
    }
}

fn document_start_anchor(document: &Document) -> usize {
    document
        .index()
        .first_source_block_span()
        .map(|span| span.byte_start as usize)
        .or_else(|| {
            document
                .frontmatter()
                .map(|frontmatter| frontmatter.span.byte_end as usize)
        })
        .unwrap_or(0)
}

fn task_edit_site(
    document: &Document,
    target: &TaskPatchTarget,
) -> Result<(usize, crate::model::TaskStatus), CoreError> {
    let address = TargetAddress::Task {
        block: target.block.clone(),
        path: target.path.clone(),
    };
    let node =
        document
            .index()
            .node_for_address(&address)
            .ok_or_else(|| CoreError::TargetNotFound {
                target: address.to_string(),
            })?;
    let IndexNode::TaskItem {
        symbol_byte_offset,
        status,
        ..
    } = document.index().entry(node).node
    else {
        return Err(CoreError::InvalidPatch(
            "task address resolved to a non-task node".into(),
        ));
    };
    let symbol = symbol_byte_offset as usize;
    let source = document.source().as_bytes();
    if symbol == 0
        || symbol + 1 >= source.len()
        || source[symbol - 1] != b'['
        || source[symbol + 1] != b']'
    {
        return Err(CoreError::PatchInvariant(
            "indexed task symbol is not bounded by checkbox brackets".into(),
        ));
    }
    Ok((symbol, status))
}

fn verify_revision(document: &Document, revision: &DocumentRevision) -> Result<(), CoreError> {
    if revision == document.revision() {
        Ok(())
    } else {
        Err(CoreError::DocumentRevisionMismatch {
            expected: revision.to_string(),
            actual: document.revision().to_string(),
        })
    }
}

fn verify_block_target(document: &Document, target: &ReplaceBlockTarget) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let resolved = document.resolve(&TargetAddress::Block {
        block: target.address.clone(),
    })?;
    verify_selection_guard(
        &TargetAddress::Block {
            block: target.address.clone(),
        },
        &target.guard,
        &resolved.snapshot().guard,
    )
}

fn verify_section_target(
    document: &Document,
    target: &SectionPatchTarget,
) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let address = match &target.address {
        crate::target::SectionAddress::Preamble => TargetAddress::Preamble,
        crate::target::SectionAddress::Heading { path } => {
            TargetAddress::Section { path: path.clone() }
        }
    };
    let resolved = document.resolve(&address)?;
    verify_selection_guard(&address, &target.guard, &resolved.snapshot().guard)
}

fn verify_preamble_target(
    document: &Document,
    target: &PreamblePatchTarget,
) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let resolved = document.resolve(&TargetAddress::Preamble)?;
    verify_selection_guard(
        &TargetAddress::Preamble,
        &target.guard,
        &resolved.snapshot().guard,
    )
}

fn verify_heading_target(
    document: &Document,
    target: &HeadingPatchTarget,
) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let address = TargetAddress::Section {
        path: target.path.clone(),
    };
    let resolved = document.resolve(&address)?;
    verify_selection_guard(&address, &target.guard, &resolved.snapshot().guard)
}

fn verify_task_target(document: &Document, target: &TaskPatchTarget) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let address = TargetAddress::Task {
        block: target.block.clone(),
        path: target.path.clone(),
    };
    let resolved = document.resolve(&address)?;
    verify_selection_guard(&address, &target.guard, &resolved.snapshot().guard)
}

fn verify_table_target(document: &Document, target: &TablePatchTarget) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let address = TargetAddress::Block {
        block: target.table.clone(),
    };
    let resolved = document.resolve(&address)?;
    verify_selection_guard(&address, &target.guard, &resolved.snapshot().guard)
}

fn verify_table_row_target(
    document: &Document,
    target: &TableRowPatchTarget,
) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let address = TargetAddress::TableRow {
        table: target.table.clone(),
        row: target.row,
    };
    let resolved = document.resolve(&address)?;
    let GuardAuthority::Container {
        address: current_address,
        span,
        etag,
    } = &resolved.snapshot().guard
    else {
        return Err(CoreError::InvalidPatch(
            "table row resolved without container authority".into(),
        ));
    };
    let TargetAddress::Block { block } = current_address.as_ref() else {
        return Err(CoreError::InvalidPatch(
            "table row container is not a block".into(),
        ));
    };
    let current = ContainerGuard {
        address: block.clone(),
        span: *span,
        etag: etag.clone(),
    };
    if current == target.guard {
        Ok(())
    } else {
        Err(CoreError::TargetAuthorityMismatch {
            target: address.to_string(),
            expected: format!("{:?}", target.guard),
            actual: format!("{current:?}"),
        })
    }
}

fn verify_frontmatter_target(
    document: &Document,
    target: &FrontmatterPatchTarget,
) -> Result<(), CoreError> {
    verify_revision(document, &target.revision)?;
    let address = TargetAddress::FrontmatterField {
        path: target.path.clone(),
    };
    let resolved = document.resolve(&address)?;
    let GuardAuthority::Frontmatter { span, etag } = &resolved.snapshot().guard else {
        return Err(CoreError::InvalidPatch(
            "frontmatter field resolved without frontmatter authority".into(),
        ));
    };
    let current = FrontmatterGuard {
        span: *span,
        etag: etag.clone(),
    };
    if current == target.guard {
        Ok(())
    } else {
        Err(CoreError::TargetAuthorityMismatch {
            target: address.to_string(),
            expected: format!("{:?}", target.guard),
            actual: format!("{current:?}"),
        })
    }
}

fn verify_selection_guard(
    address: &TargetAddress,
    expected: &SelectionGuard,
    actual: &GuardAuthority,
) -> Result<(), CoreError> {
    let GuardAuthority::Selection { span, etag } = actual else {
        return Err(CoreError::InvalidPatch(
            "target resolved without selection authority".into(),
        ));
    };
    let current = SelectionGuard {
        span: *span,
        etag: etag.clone(),
    };
    if &current == expected {
        Ok(())
    } else {
        Err(authority_mismatch(address, expected, &current))
    }
}

fn block_index(document: &Document, address: &BlockAddress) -> Result<u32, CoreError> {
    let resolved = document.resolve(&TargetAddress::Block {
        block: address.clone(),
    })?;
    let ResolvedLocator::Node(node) = resolved.locator() else {
        return Err(CoreError::InvalidPatch(
            "block address resolved without index node".into(),
        ));
    };
    match document.index().entry(*node).node {
        IndexNode::BodyBlock { parser_index, .. } => Ok(parser_index),
        _ => Err(CoreError::InvalidPatch(
            "block address resolved to non-block node".into(),
        )),
    }
}

fn authority_mismatch(
    address: &TargetAddress,
    expected: &SelectionGuard,
    actual: &SelectionGuard,
) -> CoreError {
    CoreError::TargetAuthorityMismatch {
        target: address.to_string(),
        expected: format!("{expected:?}"),
        actual: format!("{actual:?}"),
    }
}
