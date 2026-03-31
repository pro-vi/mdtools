use std::fmt;
use std::process::ExitCode;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    IoOpenFailed,
    ParseFailed,
    FrontmatterParseFailed,
    HeadingNotFound,
    BlockIndexOutOfRange,
    DuplicateHeadingMatch,
    InvalidSelector,
    InvalidUtf8OnStdin,
    InvalidKeyPath,
    FrontmatterFieldConflict,
    NoTablesInDocument,
    TableNotATable,
    ColumnNotFound,
    TaskItemNotFound,
    NotATaskList,
    InvalidTaskLoc,
}

impl DiagnosticCode {
    pub fn exit_code(self) -> MdExitCode {
        match self {
            Self::IoOpenFailed => MdExitCode::NotFound,
            Self::ParseFailed | Self::FrontmatterParseFailed => MdExitCode::ParseError,
            Self::HeadingNotFound | Self::BlockIndexOutOfRange => MdExitCode::NotFound,
            Self::InvalidSelector | Self::InvalidUtf8OnStdin | Self::InvalidKeyPath
            | Self::ColumnNotFound => MdExitCode::InvalidInput,
            Self::DuplicateHeadingMatch | Self::FrontmatterFieldConflict => MdExitCode::Conflict,
            Self::NoTablesInDocument | Self::TableNotATable => MdExitCode::NotFound,
            Self::TaskItemNotFound | Self::NotATaskList => MdExitCode::NotFound,
            Self::InvalidTaskLoc => MdExitCode::InvalidInput,
        }
    }
}

#[derive(Debug)]
pub struct CommandError {
    pub exit_code: MdExitCode,
    pub message: String,
}

impl CommandError {
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            exit_code: code.exit_code(),
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(DiagnosticCode::IoOpenFailed, message)
    }

    pub fn not_found_heading(heading: &str) -> Self {
        Self::new(
            DiagnosticCode::HeadingNotFound,
            format!("heading not found: {}", heading),
        )
    }

    pub fn block_out_of_range(index: u32, count: u32) -> Self {
        Self::new(
            DiagnosticCode::BlockIndexOutOfRange,
            format!("block index {} out of range (document has {} blocks)", index, count),
        )
    }

    pub fn duplicate_heading(heading: &str, count: usize) -> Self {
        Self::new(
            DiagnosticCode::DuplicateHeadingMatch,
            format!("heading {:?} matches {} sections; use --occurrence to disambiguate", heading, count),
        )
    }

    pub fn invalid_key_path(key: &str, reason: &str) -> Self {
        Self::new(
            DiagnosticCode::InvalidKeyPath,
            format!("invalid key path '{}': {}", key, reason),
        )
    }

    pub fn no_tables() -> Self {
        Self::new(DiagnosticCode::NoTablesInDocument, "no tables found in document")
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
            format!("column {:?} not found; available columns: {}", name, headers.join(", ")),
        )
    }

    pub fn task_item_not_found(loc: &str) -> Self {
        Self::new(
            DiagnosticCode::TaskItemNotFound,
            format!("task item not found: {}", loc),
        )
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
