use clap::{ArgGroup, Args};
use std::io::Write;
use std::path::PathBuf;

use crate::errors::{CommandError, DiagnosticCode};
use crate::output;

#[derive(Args)]
#[command(after_help = "Example: md map README.md")]
pub struct MapArgs {
    pub file: PathBuf,
}

#[derive(Args)]
#[command(group = ArgGroup::new("address_input").required(true).args(["address", "from"]))]
#[command(after_help = "Example: md read README.md --address '{\"kind\":\"preamble\"}'")]
pub struct ReadTargetArgs {
    pub file: PathBuf,
    /// Inline TargetAddress JSON.
    #[arg(long, value_name = "JSON")]
    pub address: Option<String>,
    /// Read TargetAddress JSON from a file, or '-' for stdin.
    #[arg(long, value_name = "PATH")]
    pub from: Option<PathBuf>,
}

#[derive(Args)]
#[command(group = ArgGroup::new("query_input").required(true).args(["query", "from"]))]
#[command(
    after_help = "Example: md query README.md --query '{\"type\":\"kind\",\"kind\":\"task\"}'"
)]
pub struct QueryTargetsArgs {
    pub file: PathBuf,
    /// Inline TargetQuery JSON.
    #[arg(long, value_name = "JSON")]
    pub query: Option<String>,
    /// Read TargetQuery JSON from a file, or '-' for stdin.
    #[arg(long, value_name = "PATH")]
    pub from: Option<PathBuf>,
}

#[derive(Args)]
#[command(group = ArgGroup::new("patch_input").required(true).args(["patch", "from"]))]
#[command(
    after_help = "Examples:\n  md patch README.md --from patch.json\n  md patch README.md --from patch.json --in-place"
)]
pub struct ApplyPatchArgs {
    pub file: PathBuf,
    /// Inline Patch JSON.
    #[arg(long, value_name = "JSON")]
    pub patch: Option<String>,
    /// Read Patch JSON from a file, or '-' for stdin.
    #[arg(long, value_name = "PATH")]
    pub from: Option<PathBuf>,
    /// Commit the verified result atomically instead of writing Markdown to stdout.
    #[arg(long, short = 'i')]
    pub in_place: bool,
}

pub fn run_map(arguments: &MapArgs) -> Result<(), CommandError> {
    let loaded = mdtools::file::load(&arguments.file).map_err(output::persistence_error)?;
    output::write_json(&loaded.document().map()?)
}

pub fn run_read(arguments: &ReadTargetArgs) -> Result<(), CommandError> {
    let address: mdtools::target::TargetAddress = decode_json(
        arguments.address.as_deref(),
        arguments.from.as_deref(),
        "TargetAddress",
    )?;
    let loaded = mdtools::file::load(&arguments.file).map_err(output::persistence_error)?;
    let resolved = loaded.document().resolve(&address)?;
    output::write_json(&resolved.read(loaded.document())?)
}

pub fn run_query(arguments: &QueryTargetsArgs) -> Result<(), CommandError> {
    let query: mdtools::target::TargetQuery = decode_json(
        arguments.query.as_deref(),
        arguments.from.as_deref(),
        "TargetQuery",
    )?;
    let loaded = mdtools::file::load(&arguments.file).map_err(output::persistence_error)?;
    output::write_json(&loaded.document().query(&query)?)
}

pub fn run_patch(arguments: &ApplyPatchArgs, json: bool) -> Result<(), CommandError> {
    let patch: mdtools::patch::Patch = decode_json(
        arguments.patch.as_deref(),
        arguments.from.as_deref(),
        "Patch",
    )?;
    let loaded = mdtools::file::load(&arguments.file).map_err(output::persistence_error)?;
    let prepared = loaded
        .prepare_patch(&patch)
        .map_err(output::persistence_error)?;
    let outcome = if arguments.in_place {
        prepared.commit().map_err(output::persistence_error)?
    } else {
        prepared.into_outcome()
    };
    if arguments.in_place {
        output::write_json(&outcome.receipts)
    } else if json {
        output::write_json(&mdtools::protocol::PatchPreview {
            source: outcome.document.source().to_string(),
            receipts: outcome.receipts,
        })
    } else {
        std::io::stdout()
            .write_all(outcome.document.source().as_bytes())
            .map_err(CommandError::from)
    }
}

fn decode_json<T: serde::de::DeserializeOwned>(
    inline: Option<&str>,
    from: Option<&std::path::Path>,
    expected: &'static str,
) -> Result<T, CommandError> {
    let source = match (inline, from) {
        (Some(value), None) => value.to_string(),
        (None, Some(path)) => output::read_content(Some(path))?,
        _ => unreachable!("clap requires exactly one JSON input"),
    };
    serde_json::from_str(&source).map_err(|error| {
        CommandError::new(
            DiagnosticCode::InvalidInput,
            format!(
                "invalid {expected} JSON: {error}; use `md schema` for the generated JSON shape"
            ),
        )
        .with_hint("use `md schema` for the generated JSON shape")
    })
}
