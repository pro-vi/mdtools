use serde::Serialize;
use std::fmt;
use std::process::ExitCode;

use crate::model::SCHEMA_VERSION;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdExitCode {
    Success = 0,
    NotFound = 1,
    ParseError = 2,
    InvalidInput = 3,
    Conflict = 4,
}

impl From<MdExitCode> for ExitCode {
    fn from(code: MdExitCode) -> Self {
        ExitCode::from(code as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    IoOpenFailed,
    ParseFailed,
    FrontmatterParseFailed,
    HeadingNotFound,
    OccurrenceOutOfRange,
    BlockIndexOutOfRange,
    DuplicateHeadingMatch,
    InvalidSelector,
    InvalidUtf8OnStdin,
    InvalidKeyPath,
    FrontmatterFieldConflict,
    NoTablesInDocument,
    TableNotATable,
    ColumnNotFound,
    TableRowNotFound,
    InvalidTableRow,
    TaskItemNotFound,
    NotATaskList,
    InvalidTaskLoc,
    EtagMismatch,
    MultiFileFailure,
}

impl DiagnosticCode {
    pub fn exit_code(self) -> MdExitCode {
        match self {
            Self::IoOpenFailed => MdExitCode::NotFound,
            Self::ParseFailed | Self::FrontmatterParseFailed => MdExitCode::ParseError,
            Self::HeadingNotFound | Self::OccurrenceOutOfRange | Self::BlockIndexOutOfRange => {
                MdExitCode::NotFound
            }
            Self::InvalidSelector
            | Self::InvalidUtf8OnStdin
            | Self::InvalidKeyPath
            | Self::ColumnNotFound
            | Self::InvalidTableRow => MdExitCode::InvalidInput,
            Self::DuplicateHeadingMatch | Self::FrontmatterFieldConflict => MdExitCode::Conflict,
            Self::NoTablesInDocument | Self::TableNotATable | Self::TableRowNotFound => {
                MdExitCode::NotFound
            }
            Self::TaskItemNotFound | Self::NotATaskList => MdExitCode::NotFound,
            Self::InvalidTaskLoc => MdExitCode::InvalidInput,
            Self::EtagMismatch => MdExitCode::Conflict,
            // Aggregate multi-file failures override exit_code with the worst
            // per-file code at construction; this static mapping is the floor.
            Self::MultiFileFailure => MdExitCode::NotFound,
        }
    }

    /// Every variant, for the `md schema` diagnostic table and its
    /// exhaustiveness test.
    pub const ALL: &'static [DiagnosticCode] = &[
        Self::IoOpenFailed,
        Self::ParseFailed,
        Self::FrontmatterParseFailed,
        Self::HeadingNotFound,
        Self::OccurrenceOutOfRange,
        Self::BlockIndexOutOfRange,
        Self::DuplicateHeadingMatch,
        Self::InvalidSelector,
        Self::InvalidUtf8OnStdin,
        Self::InvalidKeyPath,
        Self::FrontmatterFieldConflict,
        Self::NoTablesInDocument,
        Self::TableNotATable,
        Self::ColumnNotFound,
        Self::TableRowNotFound,
        Self::InvalidTableRow,
        Self::TaskItemNotFound,
        Self::NotATaskList,
        Self::InvalidTaskLoc,
        Self::EtagMismatch,
        Self::MultiFileFailure,
    ];
}

/// The role a section selector was playing when it failed, so adapters can
/// recommend the right disambiguation flag (`--occurrence` vs
/// `--source-occurrence` vs `--dest-occurrence`).
pub const ROLE_TARGET: &str = "target";
pub const ROLE_SOURCE: &str = "source";
pub const ROLE_DESTINATION: &str = "destination";

const MATCHES_CAP: usize = 20;

#[derive(Clone, Debug, Serialize)]
pub struct MatchRef {
    pub block_index: u32,
    pub occurrence: u32,
    pub line: u32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ErrorContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_occurrence: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matches: Option<usize>,
    /// Capped at MATCHES_CAP entries; total_matches carries the full count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<MatchRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found_etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_files: Option<u32>,
}

impl ErrorContext {
    pub fn capped_matches(all: &[MatchRef]) -> Vec<MatchRef> {
        all.iter().take(MATCHES_CAP).cloned().collect()
    }
}

#[derive(Debug)]
pub struct CommandError {
    pub exit_code: MdExitCode,
    pub code: DiagnosticCode,
    pub message: String,
    /// Standalone remediation for the JSON envelope. Display output remains
    /// `message` alone, byte-identical to the pre-envelope CLI.
    pub hint: Option<String>,
    pub context: Option<ErrorContext>,
    /// True when the command already delivered this failure structurally in
    /// its own JSON payload (e.g. `tasks` failures[]); tells main.rs not to
    /// print a second envelope object.
    pub payload_delivered: bool,
}

impl CommandError {
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            exit_code: code.exit_code(),
            code,
            message: message.into(),
            hint: None,
            context: None,
            payload_delivered: false,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Aggregate multi-file failure: exit code is the worst per-file code.
    pub fn multi_file(worst: MdExitCode, failed: u32, message: impl Into<String>) -> Self {
        let mut e = Self::new(DiagnosticCode::MultiFileFailure, message);
        e.exit_code = worst;
        e.context = Some(ErrorContext {
            failed_files: Some(failed),
            ..ErrorContext::default()
        });
        e
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(DiagnosticCode::IoOpenFailed, message)
    }

    pub fn not_found_heading(heading: &str) -> Self {
        Self::not_found_heading_as(heading, ROLE_TARGET)
    }

    pub fn not_found_heading_as(heading: &str, role: &'static str) -> Self {
        Self::new(
            DiagnosticCode::HeadingNotFound,
            format!("heading not found: {}", heading),
        )
        .with_hint("run `md outline --json <FILE>` to list current headings")
        .with_context(ErrorContext {
            role: Some(role),
            total_matches: Some(0),
            ..ErrorContext::default()
        })
    }

    pub fn occurrence_out_of_range(
        heading: &str,
        requested: u32,
        matches: &[MatchRef],
        role: &'static str,
    ) -> Self {
        Self::new(
            DiagnosticCode::OccurrenceOutOfRange,
            format!(
                "heading not found: {} (occurrence {} of {})",
                heading,
                requested,
                matches.len()
            ),
        )
        .with_hint(format!(
            "the document has {} matching heading(s); pass a 1-based occurrence within range",
            matches.len()
        ))
        .with_context(ErrorContext {
            role: Some(role),
            requested_occurrence: Some(requested),
            total_matches: Some(matches.len()),
            matches: Some(ErrorContext::capped_matches(matches)),
            ..ErrorContext::default()
        })
    }

    pub fn block_out_of_range(index: u32, count: u32) -> Self {
        Self::new(
            DiagnosticCode::BlockIndexOutOfRange,
            format!(
                "block index {} out of range (document has {} blocks)",
                index, count
            ),
        )
        .with_hint("re-run `md blocks --json <FILE>` for current block indices")
    }

    pub fn duplicate_heading(heading: &str, count: usize) -> Self {
        Self::duplicate_heading_as(heading, count, &[], ROLE_TARGET)
    }

    pub fn duplicate_heading_as(
        heading: &str,
        count: usize,
        matches: &[MatchRef],
        role: &'static str,
    ) -> Self {
        let mut ctx = ErrorContext {
            role: Some(role),
            total_matches: Some(count),
            ..ErrorContext::default()
        };
        if !matches.is_empty() {
            ctx.matches = Some(ErrorContext::capped_matches(matches));
        }
        Self::new(
            DiagnosticCode::DuplicateHeadingMatch,
            format!(
                "heading {:?} matches {} sections; use --occurrence to disambiguate",
                heading, count
            ),
        )
        .with_hint("pass a 1-based --occurrence to pick one match")
        .with_context(ctx)
    }

    pub fn invalid_key_path(key: &str, reason: &str) -> Self {
        Self::new(
            DiagnosticCode::InvalidKeyPath,
            format!("invalid key path '{}': {}", key, reason),
        )
    }

    pub fn no_tables() -> Self {
        Self::new(
            DiagnosticCode::NoTablesInDocument,
            "no tables found in document",
        )
    }

    pub fn table_not_found(index: u32) -> Self {
        Self::new(
            DiagnosticCode::TableNotATable,
            format!("block {} is not a table", index),
        )
    }

    pub fn column_not_found(name: &str, headers: &[String]) -> Self {
        Self::new(
            DiagnosticCode::ColumnNotFound,
            format!(
                "column {:?} not found; available columns: {}",
                name,
                headers.join(", ")
            ),
        )
    }

    pub fn table_row_not_found(block_index: u32, row_index: u32, row_count: u32) -> Self {
        Self::new(
            DiagnosticCode::TableRowNotFound,
            format!(
                "table row {} out of range for block {} (table has {} data rows)",
                row_index, block_index, row_count
            ),
        )
        .with_hint("re-run `md table --json <FILE>` for current row indices")
    }

    pub fn invalid_table_row(message: impl Into<String>) -> Self {
        Self::new(DiagnosticCode::InvalidTableRow, message)
    }

    pub fn task_item_not_found(loc: &str) -> Self {
        Self::new(
            DiagnosticCode::TaskItemNotFound,
            format!("task item not found: {}", loc),
        )
        .with_hint("re-run `md tasks --json <FILE>` for current task locs")
        .with_context(ErrorContext {
            loc: Some(loc.to_string()),
            ..ErrorContext::default()
        })
    }

    pub fn not_a_task_list(block_index: u32) -> Self {
        Self::new(
            DiagnosticCode::NotATaskList,
            format!("block {} has no task items", block_index),
        )
    }

    pub fn invalid_task_loc(loc: &str) -> Self {
        Self::new(
            DiagnosticCode::InvalidTaskLoc,
            format!("invalid task loc: {:?} (expected N.N[.N...] format)", loc),
        )
        .with_hint("re-run `md tasks --json <FILE>` for current task locs")
        .with_context(ErrorContext {
            loc: Some(loc.to_string()),
            ..ErrorContext::default()
        })
    }

    fn etag_ctx(expected: &str, actual: &str) -> ErrorContext {
        ErrorContext {
            expected_etag: Some(expected.to_string()),
            found_etag: Some(actual.to_string()),
            ..ErrorContext::default()
        }
    }

    pub fn etag_mismatch(index: u32, expected: &str, actual: &str) -> Self {
        Self::new(
            DiagnosticCode::EtagMismatch,
            format!(
                "block {} etag mismatch: expected {:?}, found {:?} \
                 (block content changed since you read it; re-run `md blocks --json` \
                 for current indices and etags, then retry)",
                index, expected, actual
            ),
        )
        .with_hint("re-run `md blocks --json` for current indices and etags, then retry")
        .with_context(Self::etag_ctx(expected, actual))
    }

    pub fn section_etag_mismatch(selector: &str, expected: &str, actual: &str) -> Self {
        Self::section_etag_mismatch_for("section", ROLE_TARGET, selector, expected, actual)
    }

    pub fn move_section_source_etag_mismatch(selector: &str, expected: &str, actual: &str) -> Self {
        Self::section_etag_mismatch_for("source section", ROLE_SOURCE, selector, expected, actual)
    }

    pub fn move_section_dest_etag_mismatch(selector: &str, expected: &str, actual: &str) -> Self {
        Self::section_etag_mismatch_for(
            "destination section",
            ROLE_DESTINATION,
            selector,
            expected,
            actual,
        )
    }

    fn section_etag_mismatch_for(
        noun: &str,
        role: &'static str,
        selector: &str,
        expected: &str,
        actual: &str,
    ) -> Self {
        let mut ctx = Self::etag_ctx(expected, actual);
        ctx.role = Some(role);
        Self::new(
            DiagnosticCode::EtagMismatch,
            format!(
                "{} {} etag mismatch: expected {:?}, found {:?} \
                 (section content changed since you read it; re-run `md section <SELECTOR> <FILE> --json` \
                 for the current section span and etag, then retry)",
                noun, selector, expected, actual
            ),
        )
        .with_hint(
            "re-run `md section <SELECTOR> <FILE> --json` for the current section span and etag, then retry",
        )
        .with_context(ctx)
    }

    pub fn task_etag_mismatch(loc: &str, expected: &str, actual: &str) -> Self {
        let mut ctx = Self::etag_ctx(expected, actual);
        ctx.loc = Some(loc.to_string());
        Self::new(
            DiagnosticCode::EtagMismatch,
            format!(
                "task {} etag mismatch: expected {:?}, found {:?} \
                 (task content changed since you read it; re-run `md tasks <FILE> --json` \
                 for current locs and etags, then retry)",
                loc, expected, actual
            ),
        )
        .with_hint("re-run `md tasks <FILE> --json` for current locs and etags, then retry")
        .with_context(ctx)
    }

    pub fn table_etag_mismatch(index: u32, expected: &str, actual: &str) -> Self {
        Self::new(
            DiagnosticCode::EtagMismatch,
            format!(
                "table {} etag mismatch: expected {:?}, found {:?} \
                 (table content changed since you read it; re-run `md table <FILE> --json` \
                 or `md table --index {} <FILE> --json` for current indices and etags, then retry)",
                index, expected, actual, index
            ),
        )
        .with_hint("re-run `md table --json <FILE>` for current indices and etags, then retry")
        .with_context(Self::etag_ctx(expected, actual))
    }

    pub fn frontmatter_etag_mismatch(expected: &str, actual: &str) -> Self {
        Self::new(
            DiagnosticCode::EtagMismatch,
            format!(
                "frontmatter etag mismatch: expected {:?}, found {:?} \
                 (frontmatter state changed since you read it; re-run `md frontmatter <FILE> --json` \
                 for the current frontmatter etag, then retry)",
                expected, actual
            ),
        )
        .with_hint("re-run `md frontmatter <FILE> --json` for the current frontmatter etag, then retry")
        .with_context(Self::etag_ctx(expected, actual))
    }

    pub fn frontmatter_field_conflict(key: &str, blocker: &str) -> Self {
        Self::new(
            DiagnosticCode::FrontmatterFieldConflict,
            format!("cannot set '{}': '{}' is not an object", key, blocker),
        )
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandError {}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string())
    }
}

/// Owned, serializable projection of a CommandError for JSON envelopes and
/// per-file failure rows.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorInfo {
    pub code: DiagnosticCode,
    pub exit_code: u8,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ErrorContext>,
}

impl From<&CommandError> for ErrorInfo {
    fn from(e: &CommandError) -> Self {
        Self {
            code: e.code,
            exit_code: e.exit_code as u8,
            message: e.message.clone(),
            hint: e.hint.clone(),
            context: e.context.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
    schema_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<&'a str>,
    error: ErrorInfo,
}

/// One-line JSON envelope for stdout under --json. Best-effort: on
/// serialization failure returns None and the caller falls back to exit code
/// + stderr alone.
pub fn error_envelope_json(e: &CommandError, file: Option<&str>) -> Option<String> {
    serde_json::to_string(&ErrorEnvelope {
        schema_version: SCHEMA_VERSION,
        file,
        error: ErrorInfo::from(e),
    })
    .ok()
}
