use crate::core_error::CoreError;
use crate::document::Document;
use crate::index::{IndexNode, IndexNodeId};
use crate::model::SourceSpan;
use crate::revision::DocumentRevision;
use crate::target::{SectionAddress, TargetAddress};

#[derive(Clone, Debug)]
pub(crate) struct SectionPlanTarget {
    entry: SectionPlanEntry,
    revision: DocumentRevision,
}

#[derive(Clone, Debug)]
pub(crate) struct SectionPlanEntry {
    pub(crate) heading_level: Option<u8>,
    pub(crate) block_nodes: Vec<IndexNodeId>,
    pub(crate) span: SourceSpan,
}

impl SectionPlanTarget {
    pub(crate) fn ensure_document(&self, document: &Document) -> Result<(), CoreError> {
        if self.revision == *document.revision() {
            Ok(())
        } else {
            Err(CoreError::DocumentRevisionMismatch {
                expected: self.revision.to_string(),
                actual: document.revision().to_string(),
            })
        }
    }

    pub(crate) fn into_entry(self) -> SectionPlanEntry {
        self.entry
    }
}

impl std::ops::Deref for SectionPlanTarget {
    type Target = SectionPlanEntry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

pub(crate) fn resolve_address(
    document: &Document,
    address: &SectionAddress,
) -> Result<SectionPlanTarget, CoreError> {
    let target_address = match address {
        SectionAddress::Preamble => TargetAddress::Preamble,
        SectionAddress::Heading { path } => TargetAddress::Section { path: path.clone() },
    };
    let node = document
        .index()
        .node_for_address(&target_address)
        .ok_or_else(|| CoreError::TargetNotFound {
            target: target_address.to_string(),
        })?;
    let entry = match &document.index().entry(node).node {
        IndexNode::Preamble { span } => SectionPlanEntry {
            heading_level: None,
            block_nodes: document.index().section_source_blocks(node),
            span: *span,
        },
        IndexNode::Section { span, level, .. } => SectionPlanEntry {
            heading_level: Some(*level),
            block_nodes: document.index().section_source_blocks(node),
            span: *span,
        },
        _ => {
            return Err(CoreError::PatchInvariant(
                "section address resolved to a non-section index node".into(),
            ));
        }
    };
    Ok(SectionPlanTarget {
        entry,
        revision: document.revision().clone(),
    })
}
