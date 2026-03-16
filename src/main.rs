mod cli;
mod commands;
mod errors;
mod model;
mod output;
mod parser;

use clap::Parser;
use std::process::ExitCode;

use cli::{Cli, Command};
use errors::CommandError;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            e.exit_code.into()
        }
    }
}

fn run(cli: &Cli) -> Result<(), CommandError> {
    match &cli.command {
        Command::Outline(args) => commands::outline::run(args, cli.json),
        Command::Section(args) => commands::section::run_section(args, cli.json),
        Command::Blocks(args) => commands::blocks::run_blocks(args, cli.json),
        Command::Block(args) => commands::blocks::run_block(args, cli.json),
        Command::ReplaceSection(args) => commands::section::run_replace_section(args, cli.json),
        Command::ReplaceBlock(args) => commands::replace::run_replace_block(args, cli.json),
        Command::InsertBlock(args) => commands::replace::run_insert_block(args, cli.json),
        Command::DeleteBlock(args) => commands::replace::run_delete_block(args, cli.json),
        Command::Search(args) => commands::search::run(args, cli.json),
        Command::Links(args) => commands::links::run(args, cli.json),
        Command::Frontmatter(args) => commands::frontmatter::run(args, cli.json),
        Command::Stats(args) => commands::stats::run(args, cli.json),
    }
}
