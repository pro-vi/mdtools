use crate::core_error::CoreError;
use crate::document::Document;
use crate::index::IndexNode;
use crate::model::{HeadingRef, SectionEntry};
use crate::revision::DocumentRevision;
use crate::target::{SectionAddress, TargetAddress};

#[derive(Clone, Debug)]
pub(crate) struct SectionPlanTarget {
    entry: SectionEntry,
    revision: DocumentRevision,
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

    pub(crate) fn into_entry(self) -> SectionEntry {
        self.entry
    }
}

impl std::ops::Deref for SectionPlanTarget {
    type Target = SectionEntry;

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
        IndexNode::Preamble { span } => SectionEntry {
            heading: None,
            block_indices: document.index().section_block_indices(None),
            span: *span,
        },
        IndexNode::Section { span, level, .. } => {
            let parser_index = document
                .index()
                .children(node)
                .find_map(|child| match child.node {
                    IndexNode::Heading { parser_index, .. } => Some(parser_index),
                    _ => None,
                })
                .ok_or_else(|| {
                    CoreError::PatchInvariant("section has no indexed heading block".into())
                })?;
            SectionEntry {
                heading: Some(HeadingRef { level: *level }),
                block_indices: document.index().section_block_indices(Some(parser_index)),
                span: *span,
            }
        }
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
