use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

use crate::model::{BlockKind, TaskStatus};

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
    DeleteSection(DeleteSectionArgs),
    ReplaceBlock(ReplaceBlockArgs),
    InsertBlock(InsertBlockArgs),
    DeleteBlock(DeleteBlockArgs),
    Search(SearchArgs),
    Links(LinksArgs),
    Frontmatter(FrontmatterArgs),
    Stats(StatsArgs),
    Set(SetArgs),
    Table(TableArgs),
    Tasks(TasksArgs),
    SetTask(SetTaskArgs),
}

#[derive(Args)]
pub struct OutlineArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(long, short = 'r')]
    pub recursive: bool,
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
    /// Read content from file instead of stdin (use "-" for stdin)
    #[arg(long)]
    pub from: Option<PathBuf>,
}

#[derive(Args)]
pub struct DeleteSectionArgs {
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
    /// Read content from file instead of stdin (use "-" for stdin)
    #[arg(long)]
    pub from: Option<PathBuf>,
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
    /// Read content from file (use "-" for stdin)
    /// Read content from file instead of stdin (use "-" for stdin)
    #[arg(long)]
    pub from: Option<PathBuf>,
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
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "kind")]
    pub kinds: Vec<BlockKind>,
    #[arg(long, short = 'r')]
    pub recursive: bool,
}

#[derive(Args)]
pub struct LinksArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(long, short = 'r')]
    pub recursive: bool,
}

#[derive(Args)]
pub struct FrontmatterArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(long, short = 'r')]
    pub recursive: bool,
    /// Extract specific fields (repeatable or comma-separated)
    #[arg(long = "field", value_delimiter = ',')]
    pub fields: Vec<String>,
}

#[derive(Args)]
pub struct StatsArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(long, short = 'r')]
    pub recursive: bool,
}

#[derive(Args)]
pub struct SetArgs {
    /// Dot-path key (e.g., "title", "author.name")
    pub key: String,

    pub file: PathBuf,

    /// Value to set (omit when using --delete)
    pub value: Option<String>,

    /// Delete the key instead of setting it
    #[arg(long)]
    pub delete: bool,

    /// Force the value to be stored as a string
    #[arg(long)]
    pub string: bool,

    /// Write changes back to the file
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}

#[derive(Args)]
pub struct TableArgs {
    pub file: PathBuf,

    /// Block index of the table to extract (from `md blocks`)
    #[arg(long)]
    pub index: Option<u32>,

    /// Comma-separated column names or 0-based indices to project
    #[arg(long, value_delimiter = ',')]
    pub select: Vec<String>,

    /// Filter rows (repeatable). Format: col=val, col!=val, col~=substr
    #[arg(long = "where")]
    pub filters: Vec<String>,
}

#[derive(Args)]
pub struct TasksArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(long, short = 'r')]
    pub recursive: bool,
    #[arg(long)]
    pub status: Option<TaskStatus>,
}

#[derive(Args)]
pub struct SetTaskArgs {
    /// Task location from `md tasks` output (e.g., "9.0", "14.4.0")
    pub loc: String,
    pub file: PathBuf,
    #[arg(long)]
    pub status: TaskStatus,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
}
