//! `md schema` — machine-readable dump of the CLI surface.
//!
//! One derivable authority for command names, flags, per-command kind
//! (query|mutation), the diagnostic/exit-code table, and capabilities, so
//! consumers (pi-mdtools, bench/command_policy.py, bench/md_inventory_v1.json)
//! test against the binary instead of restating it. Commands and flags are
//! walked from clap at runtime — never hand-listed here.

use clap::CommandFactory;
use serde::Serialize;

use crate::cli::{Cli, SchemaArgs};
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::SCHEMA_VERSION;
use crate::output;

/// Feature handshake tokens for adapters. Append-only.
pub const CAPABILITIES: &[&str] = &[
    "error_envelope",
    "occurrence_context",
    "section_etag",
    "task_etag",
    "table_etag",
    "frontmatter_etag",
    "move_section_dual_etag",
];

/// query|mutation kind per command. The exhaustiveness test below fails when
/// a new clap subcommand has no entry here.
fn kind_of(name: &str) -> Option<&'static str> {
    Some(match name {
        "outline" | "section" | "blocks" | "block" | "search" | "links" | "frontmatter"
        | "collect" | "stats" | "table" | "tasks" | "schema" => "query",
        "replace-section" | "delete-section" | "replace-block" | "insert-block"
        | "delete-block" | "set" | "replace-table-row" | "delete-table-row" | "set-task"
        | "move-section" => "mutation",
        _ => return None,
    })
}

#[derive(Serialize)]
struct FlagInfo {
    long: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    short: Option<char>,
    takes_value: bool,
    repeatable: bool,
    required: bool,
}

#[derive(Serialize)]
struct ArgInfo {
    name: String,
    required: bool,
    variadic: bool,
}

#[derive(Serialize)]
struct CommandInfo {
    name: String,
    kind: &'static str,
    args: Vec<ArgInfo>,
    flags: Vec<FlagInfo>,
}

#[derive(Serialize)]
struct DiagnosticInfo {
    code: DiagnosticCode,
    exit_code: u8,
}

#[derive(Serialize)]
struct SchemaResult {
    schema_version: String,
    binary_version: &'static str,
    capabilities: Vec<&'static str>,
    global_flags: Vec<FlagInfo>,
    commands: Vec<CommandInfo>,
    diagnostic_codes: Vec<DiagnosticInfo>,
}

fn flag_info(arg: &clap::Arg) -> Option<FlagInfo> {
    let long = arg.get_long()?;
    if long == "help" || long == "version" {
        return None;
    }
    let takes_value = arg.get_action().takes_values();
    let repeatable = matches!(arg.get_action(), clap::ArgAction::Append);
    Some(FlagInfo {
        long: format!("--{}", long),
        short: arg.get_short(),
        takes_value,
        repeatable,
        required: arg.is_required_set(),
    })
}

fn build_schema() -> SchemaResult {
    let root = Cli::command();

    let global_flags = root.get_arguments().filter_map(flag_info).collect();

    let mut commands = Vec::new();
    for sub in root.get_subcommands() {
        let name = sub.get_name().to_string();
        let kind = kind_of(&name).unwrap_or("unknown");
        let mut args = Vec::new();
        let mut flags = Vec::new();
        for arg in sub.get_arguments() {
            if arg.is_positional() {
                args.push(ArgInfo {
                    name: arg.get_id().to_string(),
                    required: arg.is_required_set(),
                    variadic: matches!(arg.get_action(), clap::ArgAction::Append)
                        || arg
                            .get_num_args()
                            .map_or(false, |r| r.max_values() > 1),
                });
            } else if let Some(f) = flag_info(arg) {
                flags.push(f);
            }
        }
        commands.push(CommandInfo {
            name,
            kind,
            args,
            flags,
        });
    }

    let diagnostic_codes = DiagnosticCode::ALL
        .iter()
        .map(|c| DiagnosticInfo {
            code: *c,
            exit_code: c.exit_code() as u8,
        })
        .collect();

    SchemaResult {
        schema_version: SCHEMA_VERSION.to_string(),
        binary_version: env!("CARGO_PKG_VERSION"),
        capabilities: CAPABILITIES.to_vec(),
        global_flags,
        commands,
        diagnostic_codes,
    }
}

pub fn run(_args: &SchemaArgs, json: bool) -> Result<(), CommandError> {
    let schema = build_schema();
    if json {
        output::write_json(&schema)?;
    } else {
        for cmd in &schema.commands {
            let flags: Vec<String> = cmd.flags.iter().map(|f| f.long.clone()).collect();
            println!("{}\t{}\t{}", cmd.name, cmd.kind, flags.join(","));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_subcommand_has_a_kind() {
        let root = Cli::command();
        for sub in root.get_subcommands() {
            assert!(
                kind_of(sub.get_name()).is_some(),
                "no kind entry for subcommand {:?}; add it to kind_of",
                sub.get_name()
            );
        }
    }

    #[test]
    fn diagnostic_table_covers_every_code() {
        let schema = build_schema();
        assert_eq!(schema.diagnostic_codes.len(), DiagnosticCode::ALL.len());
    }
}
