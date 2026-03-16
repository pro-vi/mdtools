use crate::cli::{BlockArgs, BlocksArgs};
use crate::errors::CommandError;
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run_blocks(args: &BlocksArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;
    let result = build_blocks_result(&doc, &args.file.to_string_lossy());

    if json {
        output::write_json(&result)?;
    } else {
        for block in &result.blocks {
            let preview = output::escape_text_field(&block.preview);
            println!(
                "{}\t{}\t{}-{}\t{}",
                block.index, block.kind, block.span.line_start, block.span.line_end, preview
            );
        }
    }
    Ok(())
}

pub fn run_block(args: &BlockArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let block = doc
        .blocks
        .get(args.index as usize)
        .ok_or_else(|| CommandError::block_out_of_range(args.index, doc.blocks.len() as u32))?;

    let content = doc.slice(&block.span).to_string();
    let preview = make_preview(&content);

    let entry = BlockEntry {
        index: block.index,
        kind: block.kind,
        span: block.span,
        preview,
    };

    if json {
        let result = BlockReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            block: entry,
            content,
        };
        output::write_json(&result)?;
    } else {
        print!("{}", content);
    }
    Ok(())
}

fn build_blocks_result(doc: &ParsedDocument, file: &str) -> BlocksResult {
    let blocks = doc
        .blocks
        .iter()
        .map(|b| {
            let content = doc.slice(&b.span);
            BlockEntry {
                index: b.index,
                kind: b.kind,
                span: b.span,
                preview: make_preview(content),
            }
        })
        .collect();

    BlocksResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string(),
        blocks,
    }
}

fn make_preview(content: &str) -> String {
    output::truncate_preview(content, 80)
}
