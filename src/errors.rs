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

/// SINGLE AUTHORITY for the diagnostic vocabulary: the enum, ALL, and
/// VARIANT_COUNT all derive from this one list, so a future variant cannot
/// exist without appearing in `md schema` and the TS parity scope.
macro_rules! diagnostic_codes {
    ($($variant:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
        #[serde(rename_all = "snake_case")]
        pub enum DiagnosticCode {
            $($variant,)*
        }

        impl DiagnosticCode {
            /// Every variant, derived from the declaration itself.
            pub const ALL: &'static [DiagnosticCode] = &[$(DiagnosticCode::$variant,)*];
            pub const VARIANT_COUNT: usize = DiagnosticCode::ALL.len();
        }
    };
}

diagnostic_codes! {
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
    EtagAmbiguous,
    MultiFileFailure,
}

impl DiagnosticCode {
    /// Total remediation: every diagnostic has a default hint (exhaustive
    /// match — the compiler forces an entry for new variants), so no
    /// envelope can ship without actionable recovery. Constructor-specific
    /// hints override this.
    pub fn default_hint(self) -> &'static str {
        match self {
            Self::IoOpenFailed => "check that the file path exists and is readable",
            Self::ParseFailed => "the file could not be parsed; ensure it is UTF-8 Markdown",
            Self::FrontmatterParseFailed => {
                "fix the frontmatter syntax between the opening and closing delimiters (--- for YAML, +++ for TOML)"
            }
            Self::HeadingNotFound => "run `md outline --json <FILE>` to list current headings",
            Self::OccurrenceOutOfRange => "pass a 1-based occurrence within the reported match count",
            Self::BlockIndexOutOfRange => "re-run `md blocks --json <FILE>` for current block indices",
            Self::DuplicateHeadingMatch => "pass a 1-based occurrence flag to pick one match",
            // Domain-NEUTRAL: invalid_selector is shared by section, table, and
            // move-section. Each construction attaches a domain-specific hint;
            // this fallback must not assume any one command's vocabulary.
            Self::InvalidSelector => {
                "the selector is not valid for this command; re-check the accepted selector forms in the command's usage"
            }
            Self::InvalidUtf8OnStdin => "pipe UTF-8 content, or deliver it via --from <PATH>",
            Self::InvalidKeyPath => "use dot-separated object keys, e.g. `md set meta.title <FILE> \"value\"`",
            Self::FrontmatterFieldConflict => {
                "inspect the current shape with `md frontmatter --json <FILE>`, then set the blocking key to an object first"
            }
            Self::NoTablesInDocument => "run `md blocks --json <FILE>` to inspect what block kinds the document has",
            Self::TableNotATable => "run `md table --json <FILE>` to list the table blocks and their indices",
            Self::ColumnNotFound => "pick a column from the available list, or re-run `md table --json <FILE>` for current headers",
            Self::TableRowNotFound => "re-run `md table --json <FILE>` for current row indices",
            Self::InvalidTableRow => "re-run `md table --json <FILE>` for the current column count and row shape",
            Self::TaskItemNotFound => "re-run `md tasks --json <FILE>` for current task locs",
            Self::NotATaskList => "re-run `md tasks --json <FILE>` for current task locs",
            Self::InvalidTaskLoc => "re-run `md tasks --json <FILE>` for current task locs",
            Self::EtagMismatch => "re-read the target with the matching md read command for a fresh etag, then retry",
            Self::EtagAmbiguous => {
                "identical duplicates share this etag; edit one first so the fingerprints diverge, or retry without --expect-etag after a re-read"
            }
            Self::MultiFileFailure => "see the per-file failures (NDJSON error rows, or the failures[] array) for each file's error",
        }
    }

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
            Self::EtagMismatch | Self::EtagAmbiguous => MdExitCode::Conflict,
            // Aggregate multi-file failures override exit_code with the worst
            // per-file code at construction; this static mapping is the floor.
            Self::MultiFileFailure => MdExitCode::NotFound,
        }
    }
}

/// The role a section selector was playing when it failed, so adapters can
/// recommend the right disambiguation flag (`--occurrence` vs
/// `--source-occurrence` vs `--dest-occurrence`). A CLOSED enum: the role can
/// only ever be one of these three, so a typo can't silently produce
/// target-flavored advice for a source/destination selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorRole {
    Target,
    Source,
    Destination,
}

impl SelectorRole {
    /// Wire value carried in ErrorContext.role.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Target => "target",
            Self::Source => "source",
            Self::Destination => "destination",
        }
    }

    /// The occurrence flag that actually exists for this role (move-section uses
    /// --source-occurrence / --dest-occurrence; plain --occurrence does not
    /// exist there).
    fn occurrence_flag(self) -> &'static str {
        match self {
            Self::Target => "--occurrence",
            Self::Source => "--source-occurrence",
            Self::Destination => "--dest-occurrence",
        }
    }

    /// The etag-guard flag for this role (move-section uses
    /// --expect-source-etag / --expect-dest-etag; plain --expect-etag does not
    /// exist there).
    fn guard_flag(self) -> &'static str {
        match self {
            Self::Target => "--expect-etag",
            Self::Source => "--expect-source-etag",
            Self::Destination => "--expect-dest-etag",
        }
    }
}

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
        Self::not_found_heading_as(heading, SelectorRole::Target)
    }

    pub fn not_found_heading_as(heading: &str, role: SelectorRole) -> Self {
        Self::new(
            DiagnosticCode::HeadingNotFound,
            format!("heading not found: {}", heading),
        )
        .with_hint("run `md outline --json <FILE>` to list current headings")
        .with_context(ErrorContext {
            role: Some(role.as_str()),
            total_matches: Some(0),
            ..ErrorContext::default()
        })
    }

    pub fn occurrence_out_of_range(
        heading: &str,
        requested: u32,
        matches: &[MatchRef],
        role: SelectorRole,
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
            "the document has {} matching heading(s); pass a 1-based {} within range",
            matches.len(),
            role.occurrence_flag()
        ))
        .with_context(ErrorContext {
            role: Some(role.as_str()),
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
        Self::duplicate_heading_as(heading, count, &[], SelectorRole::Target)
    }

    pub fn duplicate_heading_as(
        heading: &str,
        count: usize,
        matches: &[MatchRef],
        role: SelectorRole,
    ) -> Self {
        let mut ctx = ErrorContext {
            role: Some(role.as_str()),
            total_matches: Some(count),
            ..ErrorContext::default()
        };
        if !matches.is_empty() {
            ctx.matches = Some(ErrorContext::capped_matches(matches));
        }
        Self::new(
            DiagnosticCode::DuplicateHeadingMatch,
            format!(
                "heading {:?} matches {} sections; use {} to disambiguate",
                heading,
                count,
                role.occurrence_flag()
            ),
        )
        .with_hint(format!(
            "pass a 1-based {} to pick one match",
            role.occurrence_flag()
        ))
        .with_context(ctx)
    }

    pub fn invalid_key_path(key: &str, reason: &str) -> Self {
        Self::new(
            DiagnosticCode::InvalidKeyPath,
            format!("invalid key path '{}': {}", key, reason),
        )
        .with_hint("use dot-separated object keys, e.g. `md set meta.title <FILE> \"value\"`")
    }

    pub fn no_tables() -> Self {
        Self::new(
            DiagnosticCode::NoTablesInDocument,
            "no tables found in document",
        )
        .with_hint("run `md blocks --json <FILE>` to inspect what block kinds the document has")
    }

    pub fn table_not_found(index: u32) -> Self {
        Self::new(
            DiagnosticCode::TableNotATable,
            format!("block {} is not a table", index),
        )
        .with_hint("run `md table --json <FILE>` to list the table blocks and their indices")
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
        .with_hint("pick a column from the available list in the message, or re-run `md table --json <FILE>` for current headers")
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

    pub fn table_row_insertion_out_of_range(
        block_index: u32,
        row_index: u32,
        row_count: u32,
    ) -> Self {
        Self::new(
            DiagnosticCode::TableRowNotFound,
            format!(
                "table row insertion index {} out of range for block {} \
                 (valid resulting row range: 0..={})",
                row_index, block_index, row_count
            ),
        )
    }

    pub fn invalid_table_row(message: impl Into<String>) -> Self {
        Self::new(DiagnosticCode::InvalidTableRow, message)
            .with_hint("re-run `md table --json <FILE>` for the current column count and row shape")
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
        .with_hint("re-run `md tasks --json <FILE>` for current task locs")
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

    /// The expected fingerprint matched, but the same fingerprint appears on
    /// multiple same-kind targets in the document: a content match cannot
    /// prove identity, so the guard fails closed.
    pub fn etag_ambiguous(
        noun: &str,
        expected: &str,
        count: usize,
        role: Option<SelectorRole>,
    ) -> Self {
        Self::new(
            DiagnosticCode::EtagAmbiguous,
            format!(
                "{} etag {:?} is ambiguous: {} same-content {}s share this fingerprint \
                 (a content match cannot prove identity, and the guard will keep failing \
                 while the duplicates are byte-identical)",
                noun, expected, count, noun
            ),
        )
        .with_hint(match noun {
            "task" => format!(
                "{} identical task lines share this etag and locs already pin position; re-run `md tasks --json`, confirm the intended loc, then retry without --expect-etag or edit one duplicate first",
                count
            ),
            "table" => format!(
                "{} identical whole tables share this etag; re-run `md table --json <FILE>`, confirm the intended `--index`, then retry without `--expect-etag` or edit one duplicate first so the fingerprints diverge",
                count
            ),
            _ => {
                // Name BOTH role-correct flags: the disambiguator
                // (--source-occurrence / --dest-occurrence / --occurrence) and the
                // guard flag to drop (--expect-source-etag / --expect-dest-etag /
                // --expect-etag), rather than a generic phrase.
                let occ = role.map(SelectorRole::occurrence_flag).unwrap_or("--occurrence");
                let guard = role.map(SelectorRole::guard_flag).unwrap_or("--expect-etag");
                format!(
                    "{} identical {}s share this etag; either edit one duplicate first so the fingerprints diverge, or re-read and mutate the intended target by {} WITHOUT {}",
                    count, noun, occ, guard
                )
            }
        })
        .with_context(ErrorContext {
            role: role.map(|r| r.as_str()),
            expected_etag: Some(expected.to_string()),
            total_matches: Some(count),
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
        Self::section_etag_mismatch_for("section", SelectorRole::Target, selector, expected, actual)
    }

    pub fn move_section_source_etag_mismatch(selector: &str, expected: &str, actual: &str) -> Self {
        Self::section_etag_mismatch_for(
            "source section",
            SelectorRole::Source,
            selector,
            expected,
            actual,
        )
    }

    pub fn move_section_dest_etag_mismatch(selector: &str, expected: &str, actual: &str) -> Self {
        Self::section_etag_mismatch_for(
            "destination section",
            SelectorRole::Destination,
            selector,
            expected,
            actual,
        )
    }

    fn section_etag_mismatch_for(
        noun: &str,
        role: SelectorRole,
        selector: &str,
        expected: &str,
        actual: &str,
    ) -> Self {
        let mut ctx = Self::etag_ctx(expected, actual);
        ctx.role = Some(role.as_str());
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
        .with_hint("run `md frontmatter --json <FILE>` to inspect the current shape, then set the blocking key to an object first or choose a different key path")
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
            // Remediation is TOTAL on the wire: constructor hints win, and
            // every remaining diagnostic falls back to its typed default.
            hint: Some(
                e.hint
                    .clone()
                    .unwrap_or_else(|| e.code.default_hint().to_string()),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_diagnostic_has_nonempty_default_remediation() {
        for code in DiagnosticCode::ALL {
            assert!(
                !code.default_hint().trim().is_empty(),
                "{:?} has an empty default hint",
                code
            );
        }
    }

    #[test]
    fn error_info_hint_is_always_present() {
        let bare = CommandError::new(DiagnosticCode::FrontmatterParseFailed, "boom");
        let info = ErrorInfo::from(&bare);
        assert!(info.hint.is_some(), "wire hint must be total");
    }
}
