use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::model::{BlockKind, TaskStatus};

const SECTION_EXPECT_ETAG_HELP: &str =
    "Fail-closed if the target section's current etag (from `md section --json`)\n\
     differs — guards against a stale selector after intervening edits.";
const TASK_EXPECT_ETAG_HELP: &str =
    "Fail-closed if the task item's current etag (from `md tasks --json`)\n\
     differs — guards against a stale loc after intervening edits.";
const TABLE_EXPECT_ETAG_HELP: &str =
    "Fail-closed if the table's current etag (from `md table --json`)\n\
     differs — guards against mutating a stale table after intervening edits.";
const SECTION_CONTAINS_HELP: &str =
    "Match parsed plaintext top-level heading text by literal substring; \
     exact matching remains the default";

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
    Collect(CollectArgs),
    Stats(StatsArgs),
    Set(SetArgs),
    Table(TableArgs),
    ReplaceTableRow(ReplaceTableRowArgs),
    DeleteTableRow(DeleteTableRowArgs),
    Tasks(TasksArgs),
    SetTask(SetTaskArgs),
    MoveSection(MoveSectionArgs),
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
    #[arg(long = "contains", help = SECTION_CONTAINS_HELP)]
    pub contains: bool,
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
    #[arg(long = "contains", help = SECTION_CONTAINS_HELP)]
    pub contains: bool,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "occurrence")]
    pub occurrence: Option<u32>,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
    /// Read content from file instead of stdin (use "-" for stdin)
    #[arg(long)]
    pub from: Option<PathBuf>,
    #[command(flatten)]
    pub etag_guard: SectionEtagGuardArgs,
}

#[derive(Args)]
pub struct DeleteSectionArgs {
    #[arg(value_name = "SELECTOR")]
    pub selector: String,
    pub file: PathBuf,
    #[arg(long = "contains", help = SECTION_CONTAINS_HELP)]
    pub contains: bool,
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,
    #[arg(long = "occurrence")]
    pub occurrence: Option<u32>,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
    #[command(flatten)]
    pub etag_guard: SectionEtagGuardArgs,
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
    /// Fail-closed if the target block's current etag (from `md blocks --json`)
    /// differs — guards against a stale index after intervening edits.
    #[arg(long = "expect-etag", value_name = "ETAG")]
    pub expect_etag: Option<String>,
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
    /// Read content from file instead of stdin (use "-" for stdin)
    #[arg(long)]
    pub from: Option<PathBuf>,
    /// Fail-closed if the anchor block's current etag (from `md blocks --json`)
    /// differs. Requires --before or --after (no anchor for --at-start/--at-end).
    #[arg(long = "expect-etag", value_name = "ETAG")]
    pub expect_etag: Option<String>,
}

#[derive(Args)]
pub struct DeleteBlockArgs {
    pub index: u32,
    pub file: PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
    /// Fail-closed if the target block's current etag (from `md blocks --json`)
    /// differs — guards against a stale index after intervening edits.
    #[arg(long = "expect-etag", value_name = "ETAG")]
    pub expect_etag: Option<String>,
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
pub struct CollectArgs {
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
pub struct ReplaceTableRowArgs {
    pub table_block_index: u32,
    pub row_index: u32,
    pub file: PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
    /// Read content from file instead of stdin (use "-" for stdin)
    #[arg(long)]
    pub from: Option<PathBuf>,
    #[command(flatten)]
    pub etag_guard: TableEtagGuardArgs,
}

#[derive(Args)]
pub struct DeleteTableRowArgs {
    pub table_block_index: u32,
    pub row_index: u32,
    pub file: PathBuf,
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
    #[command(flatten)]
    pub etag_guard: TableEtagGuardArgs,
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
#[command(group = clap::ArgGroup::new("dest").required(true).args(["after", "before", "into"]))]
#[command(group = clap::ArgGroup::new("level").required(false).args(["auto_level", "keep_level"]))]
pub struct MoveSectionArgs {
    /// Source section heading text (matched against `md section <text>`)
    #[arg(value_name = "SOURCE_HEADING")]
    pub source: String,

    /// Markdown file to mutate. Omit to read from stdin (writes spliced doc to stdout).
    pub file: Option<PathBuf>,

    /// Insert as next sibling of the destination heading
    #[arg(long, value_name = "DEST_HEADING")]
    pub after: Option<String>,

    /// Insert as previous sibling of the destination heading
    #[arg(long, value_name = "DEST_HEADING")]
    pub before: Option<String>,

    /// Insert as last child of the destination heading
    #[arg(long, value_name = "DEST_HEADING")]
    pub into: Option<String>,

    /// Adjust source + descendant heading levels to match destination hierarchy (default)
    #[arg(long = "auto-level")]
    pub auto_level: bool,

    /// Preserve source heading levels exactly (byte-exact relocation)
    #[arg(long = "keep-level")]
    pub keep_level: bool,

    /// Case-insensitive matching for both source and destination selectors
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,

    /// Literal substring matching for both source and destination selectors
    #[arg(long = "contains")]
    pub contains: bool,

    /// 1-indexed occurrence to disambiguate same-text source headings
    #[arg(long = "source-occurrence")]
    pub source_occurrence: Option<u32>,

    /// 1-indexed occurrence to disambiguate same-text destination headings
    #[arg(long = "dest-occurrence")]
    pub dest_occurrence: Option<u32>,

    /// Write changes back to the file
    #[arg(long = "in-place", short = 'i')]
    pub in_place: bool,
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
    #[command(flatten)]
    pub etag_guard: TaskEtagGuardArgs,
}

#[derive(Args)]
pub struct SectionEtagGuardArgs {
    #[arg(
        long = "expect-etag",
        value_name = "ETAG",
        long_help = SECTION_EXPECT_ETAG_HELP
    )]
    pub expect_etag: Option<String>,
}

#[derive(Args)]
pub struct TaskEtagGuardArgs {
    #[arg(
        long = "expect-etag",
        value_name = "ETAG",
        long_help = TASK_EXPECT_ETAG_HELP
    )]
    pub expect_etag: Option<String>,
}

#[derive(Args)]
pub struct TableEtagGuardArgs {
    #[arg(
        long = "expect-etag",
        value_name = "ETAG",
        long_help = TABLE_EXPECT_ETAG_HELP
    )]
    pub expect_etag: Option<String>,
}
