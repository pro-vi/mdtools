use crate::core_error::CoreError;
use crate::document::Document;
use crate::fingerprint::TargetEtag;
use crate::model::{BlockKind, SourceSpan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockRecord {
    pub index: u32,
    pub kind: BlockKind,
    pub span: SourceSpan,
    pub etag: TargetEtag,
    pub preview: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockRead {
    pub block: BlockRecord,
    pub content: String,
}

pub fn blocks(document: &Document) -> Vec<BlockRecord> {
    document
        .blocks()
        .iter()
        .map(|block| {
            let content = document.slice(&block.span);
            BlockRecord {
                index: block.index,
                kind: block.kind,
                span: block.span,
                etag: TargetEtag::for_bytes(content.as_bytes()),
                preview: preview(content),
            }
        })
        .collect()
}

pub fn block(document: &Document, index: u32) -> Result<BlockRead, CoreError> {
    let info = document
        .blocks()
        .get(index as usize)
        .ok_or(CoreError::BlockIndexOutOfRange {
            index,
            block_count: document.blocks().len() as u32,
        })?;
    let content = document.slice(&info.span).to_string();
    Ok(BlockRead {
        block: BlockRecord {
            index: info.index,
            kind: info.kind,
            span: info.span,
            etag: TargetEtag::for_bytes(content.as_bytes()),
            preview: preview(&content),
        },
        content,
    })
}

fn preview(content: &str) -> String {
    let escaped = content
        .chars()
        .map(|character| match character {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect::<String>();
    if escaped.chars().count() <= 80 {
        escaped
    } else {
        let truncated = escaped.chars().take(80).collect::<String>();
        format!("{truncated}...")
    }
}
