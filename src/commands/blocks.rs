use crate::cli::{BlockArgs, BlocksArgs};
use crate::errors::CommandError;
use crate::model::*;
use crate::output;
use mdtools::block;
use mdtools::document::Document;

pub fn run_blocks(args: &BlocksArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = Document::parse(source)?;
    let result = BlocksResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: args.file.to_string_lossy().to_string(),
        blocks: block::blocks(&doc).into_iter().map(to_wire).collect(),
    };

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
    let doc = Document::parse(source)?;
    let read = block::block(&doc, args.index)?;
    let entry = to_wire(read.block);
    let content = read.content;

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

fn to_wire(record: block::BlockRecord) -> BlockEntry {
    BlockEntry {
        index: record.index,
        kind: record.kind,
        span: record.span,
        etag: record.etag.into_string(),
        preview: record.preview,
    }
}
