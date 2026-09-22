mod cli;
mod errors;
mod output;
mod usage;

use clap::{CommandFactory, FromArgMatches};
use std::process::ExitCode;

use cli::{Cli, Command};
use errors::CommandError;

fn main() -> ExitCode {
    // Failed argument parsing has no authoritative input-file set to protect
    // from a colliding log sink. Clap prints and exits without measurement.
    let matches = Cli::command()
        .try_get_matches()
        .unwrap_or_else(|error| error.exit());
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    let usage = usage::Usage::from_env();
    let command = matches.subcommand_name().expect("clap requires a command");
    let mut output = output::Output::new(usage);
    let (exit, diagnostic) = match run(&cli, &mut output) {
        Ok(()) => (errors::MdExitCode::Success, None),
        Err(e) => {
            output.format(if cli.json { "error_json" } else { "error_text" });
            if cli.json {
                if let Some(envelope) = errors::error_envelope_json(&e, None) {
                    let _ = output.json(&envelope);
                }
            }
            let _ = output.stderr(&format!("{e}\n"));
            (e.exit_code, Some(e.code))
        }
    };
    output.finish(command, exit as u8, diagnostic);
    exit.into()
}

fn run(cli: &Cli, output: &mut output::Output) -> Result<(), CommandError> {
    match &cli.command {
        Command::Map(args) => cli::structural::run_map(args, cli.json, output),
        Command::Read(args) => cli::structural::run_read(args, cli.json, output),
        Command::Query(args) => cli::structural::run_query(args, cli.json, output),
        Command::Patch(args) => cli::structural::run_patch(args, cli.json, output),
        Command::Schema => {
            output.format("schema_json");
            output.json(&mdtools::protocol::protocol_schema())
        }
    }
}
