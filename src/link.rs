use crate::document::Document;
use crate::model::{LinkKind, SourceSpan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkRecord {
    pub kind: LinkKind,
    pub text: String,
    pub destination: Option<String>,
    pub title: Option<String>,
    pub source_block_index: u32,
    pub span: SourceSpan,
}

pub fn links(document: &Document) -> Vec<LinkRecord> {
    document
        .blocks()
        .iter()
        .flat_map(|block| {
            block.links.iter().map(move |link| LinkRecord {
                kind: link.kind,
                text: link.text.clone(),
                destination: link.destination.clone(),
                title: link.title.clone(),
                source_block_index: block.index,
                span: link.span,
            })
        })
        .collect()
}
