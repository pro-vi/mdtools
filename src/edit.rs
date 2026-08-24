use crate::model::{LineEndingStyle, MutationDisposition, SourceSpan};
use crate::revision::DocumentRevision;

#[derive(Clone, Debug)]
pub struct EditOutcome<T> {
    pub base_revision: DocumentRevision,
    pub target: T,
    pub disposition: MutationDisposition,
    pub guarded: bool,
    pub line_endings: LineEndingStyle,
    pub preservation: EditPreservation,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditPreservation {
    pub preserves_non_target_bytes: bool,
    pub target_span_before: Option<SourceSpan>,
    pub target_span_after: Option<SourceSpan>,
}

impl<T> EditOutcome<T> {
    pub fn changed(&self) -> bool {
        self.disposition != MutationDisposition::NoChange
    }
}
