mod cli;
mod errors;
mod output;

use clap::Parser;
use std::process::ExitCode;

use cli::{Cli, Command};
use errors::CommandError;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if cli.json {
                if let Some(envelope) = errors::error_envelope_json(&e, None) {
                    let _ = output::write_json(&envelope);
                }
            }
            eprintln!("{}", e);
            e.exit_code.into()
        }
    }
}

fn run(cli: &Cli) -> Result<(), CommandError> {
    match &cli.command {
        Command::Map(args) => cli::structural::run_map(args),
        Command::Read(args) => cli::structural::run_read(args),
        Command::Query(args) => cli::structural::run_query(args),
        Command::Patch(args) => cli::structural::run_patch(args, cli.json),
        Command::Schema => output::write_json(&mdtools::protocol::protocol_schema()),
    }
}
