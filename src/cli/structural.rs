use clap::{ArgGroup, Args};
use std::path::{Path, PathBuf};

use crate::errors::{CommandError, DiagnosticCode};
use crate::output;
use mdtools::core_error::CoreError;
use mdtools::target::{TargetAddress, TargetKind, TargetQuery};
use mdtools::HeadingMatchMode;

#[derive(Args)]
#[command(
    after_help = "Examples:\n  md map README.md          # compact addresses and summaries\n  md map README.md --json   # full snapshots for patch preparation"
)]
pub struct MapArgs {
    pub file: PathBuf,
}

#[derive(Args)]
#[command(group = ArgGroup::new("address_input").args(["address", "from", "query", "section"]))]
#[command(
    after_help = "Examples:\n  md read README.md\n  md read README.md --section Install\n  md read README.md --address '{\"kind\":\"preamble\"}'\n\nWithout a selector, reads the whole document. Reads return original content.\nUse --json for the typed view and snapshot.\n--section matches visible heading text exactly, including case, and includes descendants.\n--section and --query must match exactly one target; search evidence is not a read target.\n--from accepts TargetAddress JSON only."
)]
pub struct ReadTargetArgs {
    pub file: PathBuf,
    /// Inline TargetAddress JSON.
    #[arg(long, value_name = "JSON")]
    pub address: Option<String>,
    /// Read TargetAddress JSON from a file, or '-' for stdin.
    #[arg(long, value_name = "PATH")]
    pub from: Option<PathBuf>,
    /// Inline TargetQuery JSON selecting exactly one target.
    #[arg(long, value_name = "JSON")]
    pub query: Option<String>,
    /// Read one section by its exact visible heading text.
    #[arg(long, value_name = "HEADING")]
    pub section: Option<String>,
}

#[derive(Args)]
#[command(group = ArgGroup::new("query_input").required(true).args(["query", "from", "kind"]))]
#[command(
    after_help = "Examples:\n  md query README.md --kind section\n  md query README.md --query '{\"type\":\"task\",\"status\":\"pending\",\"contains\":null}'\n\nDiscover headings before selecting an unfamiliar section.\nResults contain compact addresses and summaries. Pass an address to `md read`.\n--kind uses protocol names, such as section, task, and table_row.\nUse --json only when full snapshots or evidence metadata are needed."
)]
pub struct QueryTargetsArgs {
    pub file: PathBuf,
    /// Inline TargetQuery JSON.
    #[arg(long, value_name = "JSON")]
    pub query: Option<String>,
    /// Read TargetQuery JSON from a file, or '-' for stdin.
    #[arg(long, value_name = "PATH")]
    pub from: Option<PathBuf>,
    /// Discover targets of one kind, such as section or task.
    #[arg(long, value_name = "KIND", value_parser = parse_target_kind)]
    pub kind: Option<TargetKind>,
}

fn parse_target_kind(value: &str) -> Result<TargetKind, serde::de::value::Error> {
    serde::Deserialize::deserialize(serde::de::value::StrDeserializer::new(value))
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

pub fn run_map(
    arguments: &MapArgs,
    json: bool,
    output: &mut output::Output,
) -> Result<(), CommandError> {
    output.protect_inputs(&arguments.file, None);
    output.format(if json { "full_json" } else { "compact_json" });
    let loaded = mdtools::file::load(&arguments.file).map_err(output::persistence_error)?;
    let targets = loaded.document().map()?;
    if json {
        output.json(&targets)
    } else {
        output.json(
            &targets
                .iter()
                .map(mdtools::protocol::TargetOverview::from)
                .collect::<Vec<_>>(),
        )
    }
}

pub fn run_read(
    arguments: &ReadTargetArgs,
    json: bool,
    output: &mut output::Output,
) -> Result<(), CommandError> {
    output.protect_inputs(&arguments.file, arguments.from.as_deref());
    output.format(if json { "full_json" } else { "content" });
    let query: Option<TargetQuery> = match &arguments.section {
        Some(text) => Some(TargetQuery::Section {
            text: text.clone(),
            match_mode: HeadingMatchMode::Exact,
        }),
        None => arguments
            .query
            .as_deref()
            .map(|query| decode_json(Some(query), None, "TargetQuery"))
            .transpose()?,
    };
    if let Some(query) = &query {
        output.query_kind(query);
    }
    let address: Option<TargetAddress> = if query.is_none() {
        Some(if arguments.address.is_none() && arguments.from.is_none() {
            TargetAddress::Document
        } else {
            decode_json(
                arguments.address.as_deref(),
                arguments.from.as_deref(),
                "TargetAddress",
            )?
        })
    } else {
        None
    };
    let strict = match &query {
        Some(query) => query_reads_frontmatter(query),
        None => matches!(
            address,
            Some(TargetAddress::Frontmatter | TargetAddress::FrontmatterField { .. })
        ),
    };
    let loaded = if strict {
        mdtools::file::load_for_frontmatter_read(&arguments.file)
    } else {
        mdtools::file::load(&arguments.file)
    }
    .map_err(output::persistence_error)?;
    let resolved = match (&query, &address) {
        (Some(query), None) => loaded
            .document()
            .query_one(query)
            .map_err(|error| read_query_error(error, query, &arguments.file))?,
        (None, Some(address)) => loaded.document().resolve(address)?,
        _ => unreachable!("read selects either an address or a query"),
    };
    let read = resolved.read(loaded.document())?;
    if json {
        output.json(&read)
    } else {
        output.read(&read)
    }
}

fn read_query_error(error: CoreError, query: &TargetQuery, file: &Path) -> CommandError {
    let ambiguous = matches!(error, CoreError::AmbiguousTargetQuery { .. });
    if let TargetQuery::Section { text, match_mode } = query {
        let matches = match &error {
            CoreError::TargetNotFound { .. } => Some(0),
            CoreError::AmbiguousTargetQuery { count } => Some(*count),
            _ => None,
        };
        if let Some(count) = matches {
            let mode = match match_mode {
                HeadingMatchMode::Exact => "exact",
                HeadingMatchMode::ExactIgnoreCase => "case-insensitive exact",
                HeadingMatchMode::Contains => "substring",
                HeadingMatchMode::ContainsIgnoreCase => "case-insensitive substring",
            };
            let mut error = CommandError::from(error);
            error.message = if count == 0 {
                format!("no section matches {text:?} ({mode} match)")
            } else {
                format!("section {text:?} matched {count} sections ({mode} match)")
            };
            let mut hint = match file.to_str() {
                Some(file) => format!(
                    "md query --kind section -- '{}'",
                    file.replace('\'', "'\\''")
                ),
                None => "non-UTF-8 path: run `md query --kind section -- FILE`; replace FILE with the original path".into(),
            };
            if ambiguous {
                hint.push_str("\nThen use `md read --address` with one returned exact address.");
            }
            return error.with_hint(hint);
        }
    }
    let error = CommandError::from(error);
    if ambiguous {
        error
            .with_hint("use `md query` to discover matches, then `md read --address` to select one")
    } else {
        error
    }
}

fn query_reads_frontmatter(query: &TargetQuery) -> bool {
    match query {
        TargetQuery::FrontmatterField { .. } => true,
        TargetQuery::Kind { kind } => {
            matches!(kind, TargetKind::Frontmatter | TargetKind::FrontmatterField)
        }
        TargetQuery::All
        | TargetQuery::Section { .. }
        | TargetQuery::Task { .. }
        | TargetQuery::Link { .. }
        | TargetQuery::Search { .. } => false,
    }
}

pub fn run_query(
    arguments: &QueryTargetsArgs,
    json: bool,
    output: &mut output::Output,
) -> Result<(), CommandError> {
    output.protect_inputs(&arguments.file, arguments.from.as_deref());
    output.format(if json { "full_json" } else { "compact_json" });
    let query = match arguments.kind {
        Some(kind) => TargetQuery::Kind { kind },
        None => decode_json(
            arguments.query.as_deref(),
            arguments.from.as_deref(),
            "TargetQuery",
        )?,
    };
    output.query_kind(&query);
    let loaded = mdtools::file::load(&arguments.file).map_err(output::persistence_error)?;
    let results = loaded.document().query(&query)?;
    if json {
        output.json(&results)
    } else {
        output.json(
            &results
                .iter()
                .map(mdtools::protocol::QueryOverview::from)
                .collect::<Vec<_>>(),
        )
    }
}

pub fn run_patch(
    arguments: &ApplyPatchArgs,
    json: bool,
    output: &mut output::Output,
) -> Result<(), CommandError> {
    output.protect_inputs(&arguments.file, arguments.from.as_deref());
    output.format(if arguments.in_place {
        "receipts"
    } else if json {
        "json_preview"
    } else {
        "content"
    });
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
        if let Err(error) = output.json(&outcome.receipts) {
            let _ = output.stderr(&format!(
                "file commit succeeded, but receipt output failed and cannot be regenerated; run `md map` to observe the committed state: {error}\n"
            ));
        }
        Ok(())
    } else if json {
        output.json(&mdtools::protocol::PatchPreview {
            source: outcome.document.source().to_string(),
            receipts: outcome.receipts,
        })
    } else {
        output.bytes(outcome.document.source().as_bytes())
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
                "invalid {expected} JSON: {}",
                bounded_diagnostic(&error.to_string())
            ),
        )
        .with_hint(mdtools::protocol::input_guidance(expected, &source))
    })
}

fn bounded_diagnostic(message: &str) -> String {
    // Bound model-facing decoder prose even when Serde quotes a large rejected value.
    const MAX_BYTES: usize = 512;
    if message.len() <= MAX_BYTES {
        return message.to_string();
    }
    let mut end = MAX_BYTES - "…".len();
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &message[..end])
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn section_recovery_preserves_non_utf8_path_uncertainty() {
        let file = PathBuf::from(std::ffi::OsString::from_vec(b"skill-\xff.md".to_vec()));
        let query = TargetQuery::Section {
            text: "Missing".into(),
            match_mode: HeadingMatchMode::Exact,
        };
        let error = read_query_error(
            CoreError::TargetNotFound {
                target: "section".into(),
            },
            &query,
            &file,
        );
        let hint = error.hint.unwrap();
        assert!(hint.contains("non-UTF-8"));
        assert!(hint.contains("replace FILE"));
        assert!(!hint.contains('\u{fffd}'));
    }
}
