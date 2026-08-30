use clap::{Parser, Subcommand};

pub(crate) mod structural;
pub use structural::{ApplyPatchArgs, MapArgs, QueryTargetsArgs, ReadTargetArgs};

#[derive(Parser)]
#[command(
    name = "md",
    about = "Indexed Markdown reads and guarded patch transactions",
    version,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Emit machine-readable errors and patch previews as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = mdtools::protocol::MAP_SUMMARY)]
    Map(MapArgs),
    #[command(about = mdtools::protocol::READ_SUMMARY)]
    Read(ReadTargetArgs),
    #[command(about = mdtools::protocol::QUERY_SUMMARY)]
    Query(QueryTargetsArgs),
    #[command(about = mdtools::protocol::PATCH_SUMMARY)]
    Patch(ApplyPatchArgs),
    #[command(about = mdtools::protocol::SCHEMA_SUMMARY)]
    Schema,
}
