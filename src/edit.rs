use crate::model::{
    LineEndingStyle, MutationDisposition, MutationTargetRef, SourcePreservationInvariant,
};
use crate::revision::DocumentRevision;

#[derive(Clone, Debug)]
pub struct EditOutcome {
    pub base_revision: DocumentRevision,
    pub target: MutationTargetRef,
    pub disposition: MutationDisposition,
    pub guarded: bool,
    pub line_endings: LineEndingStyle,
    pub invariant: SourcePreservationInvariant,
    pub content: String,
}

impl EditOutcome {
    pub fn changed(&self) -> bool {
        self.disposition != MutationDisposition::NoChange
    }
}
