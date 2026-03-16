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
pub enum ErrorKind {
    Io,
    Parse,
    NotFound,
    InvalidInput,
    Conflict,
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
}

impl DiagnosticCode {
    pub fn error_kind(self) -> ErrorKind {
        match self {
            Self::IoOpenFailed => ErrorKind::Io,
            Self::ParseFailed | Self::FrontmatterParseFailed => ErrorKind::Parse,
            Self::HeadingNotFound | Self::BlockIndexOutOfRange => ErrorKind::NotFound,
            Self::InvalidSelector | Self::InvalidUtf8OnStdin => ErrorKind::InvalidInput,
            Self::DuplicateHeadingMatch => ErrorKind::Conflict,
        }
    }

    pub fn exit_code(self) -> MdExitCode {
        match self.error_kind() {
            ErrorKind::Io => MdExitCode::NotFound,
            ErrorKind::Parse => MdExitCode::ParseError,
            ErrorKind::NotFound => MdExitCode::NotFound,
            ErrorKind::InvalidInput => MdExitCode::InvalidInput,
            ErrorKind::Conflict => MdExitCode::Conflict,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::IoOpenFailed => "IoOpenFailed",
            Self::ParseFailed => "ParseFailed",
            Self::FrontmatterParseFailed => "FrontmatterParseFailed",
            Self::HeadingNotFound => "HeadingNotFound",
            Self::BlockIndexOutOfRange => "BlockIndexOutOfRange",
            Self::DuplicateHeadingMatch => "DuplicateHeadingMatch",
            Self::InvalidSelector => "InvalidSelector",
            Self::InvalidUtf8OnStdin => "InvalidUtf8OnStdin",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
}

#[derive(Debug)]
pub struct CommandError {
    pub kind: ErrorKind,
    pub exit_code: MdExitCode,
    pub diagnostic: Diagnostic,
}

impl CommandError {
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            kind: code.error_kind(),
            exit_code: code.exit_code(),
            diagnostic: Diagnostic {
                code,
                message: message.into(),
            },
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

    pub fn invalid_selector(selector: &str) -> Self {
        Self::new(
            DiagnosticCode::InvalidSelector,
            format!("invalid selector: {}", selector),
        )
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic.message)
    }
}

impl std::error::Error for CommandError {}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string())
    }
}
