use crate::cli::LinksArgs;
use crate::errors::CommandError;
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &LinksArgs, json: bool) -> Result<(), CommandError> {
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    let links: Vec<LinkEntry> = doc
        .blocks
        .iter()
        .flat_map(|block| {
            block.links.iter().map(move |link| LinkEntry {
                kind: link.kind,
                text: link.text.clone(),
                destination: link.destination.clone(),
                title: link.title.clone(),
                source_block_index: block.index,
                span: link.span,
            })
        })
        .collect();

    if json {
        let result = LinksResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            links,
        };
        output::write_json(&result)?;
    } else {
        for link in &links {
            let dest = link.destination.as_deref().unwrap_or("");
            let dest = output::escape_text_field(dest);
            println!(
                "{}\t{}\tblock:{}\t{}-{}",
                link.kind, dest, link.source_block_index, link.span.line_start, link.span.line_end
            );
        }
    }
    Ok(())
}

impl std::fmt::Display for LinkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inline => write!(f, "Inline"),
            Self::Reference => write!(f, "Reference"),
            Self::Autolink => write!(f, "Autolink"),
            Self::Wiki => write!(f, "Wiki"),
        }
    }
}
