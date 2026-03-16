use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

use crate::model::BlockKind;

#[derive(Parser)]
#[command(name = "md", about = "Markdown-aware CLI for agent operations")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Outline(OutlineArgs),
    Section(SectionArgs),
    Blocks(BlocksArgs),
    Block(BlockArgs),
    ReplaceSection(ReplaceSectionArgs),
    ReplaceBlock(ReplaceBlockArgs),
    InsertBlock(InsertBlockArgs),
    DeleteBlock(DeleteBlockArgs),
    Search(SearchArgs),
    Links(LinksArgs),
    Frontmatter(FrontmatterArgs),
    Stats(StatsArgs),
}

#[derive(Args)]
pub struct OutlineArgs {
    pub file: PathBuf,
}

#[derive(Args)]
pub struct SectionArgs {
    #[arg(value_name = "SELECTOR")]
    pub selector: String,
    pub file: PathBuf,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "occurrence")]
    pub occurrence: Option<u32>,
}

#[derive(Args)]
pub struct BlocksArgs {
    pub file: PathBuf,
}

#[derive(Args)]
pub struct BlockArgs {
    pub index: u32,
    pub file: PathBuf,
}

#[derive(Args)]
pub struct ReplaceSectionArgs {
    #[arg(value_name = "SELECTOR")]
    pub selector: String,
    pub file: PathBuf,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "occurrence")]
    pub occurrence: Option<u32>,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

#[derive(Args)]
pub struct ReplaceBlockArgs {
    pub index: u32,
    pub file: PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

#[derive(Args)]
pub struct InsertBlockArgs {
    #[arg(long = "before", value_name = "INDEX")]
    pub before: Option<u32>,
    #[arg(long = "after", value_name = "INDEX")]
    pub after: Option<u32>,
    #[arg(long = "at-start")]
    pub at_start: bool,
    #[arg(long = "at-end")]
    pub at_end: bool,
    pub file: PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

#[derive(Args)]
pub struct DeleteBlockArgs {
    pub index: u32,
    pub file: PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

#[derive(Args)]
pub struct SearchArgs {
    pub query: String,
    pub file: PathBuf,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "kind")]
    pub kinds: Vec<BlockKind>,
}

#[derive(Args)]
pub struct LinksArgs {
    pub file: PathBuf,
}

#[derive(Args)]
pub struct FrontmatterArgs {
    pub file: PathBuf,
}

#[derive(Args)]
pub struct StatsArgs {
    pub file: PathBuf,
}
