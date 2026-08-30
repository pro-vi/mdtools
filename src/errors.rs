use mdtools::core_error::CoreError;
use serde::Serialize;
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Io,
    Parse,
    InvalidInput,
    NotFound,
    Conflict,
    Invariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdExitCode {
    Success = 0,
    NotFound = 1,
    Parse = 2,
    InvalidInput = 3,
    Conflict = 4,
}

impl From<MdExitCode> for ExitCode {
    fn from(code: MdExitCode) -> Self {
        ExitCode::from(code as u8)
    }
}

#[derive(Debug)]
pub struct CommandError {
    pub exit_code: MdExitCode,
    pub code: DiagnosticCode,
    pub message: String,
    pub hint: Option<String>,
    pub payload_delivered: bool,
}

impl CommandError {
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        let exit_code = match code {
            DiagnosticCode::Io | DiagnosticCode::NotFound => MdExitCode::NotFound,
            DiagnosticCode::Parse => MdExitCode::Parse,
            DiagnosticCode::InvalidInput => MdExitCode::InvalidInput,
            DiagnosticCode::Conflict | DiagnosticCode::Invariant => MdExitCode::Conflict,
        };
        Self {
            exit_code,
            code,
            message: message.into(),
            hint: None,
            payload_delivered: false,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(DiagnosticCode::Io, message)
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(hint) = &self.hint {
            write!(formatter, "\nhint: {hint}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CommandError {}

impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

impl From<CoreError> for CommandError {
    fn from(error: CoreError) -> Self {
        let code = match error {
            CoreError::ParseFailed(_) | CoreError::FrontmatterParseFailed(_) => {
                DiagnosticCode::Parse
            }
            CoreError::TargetNotFound { .. }
            | CoreError::HeadingNotFound { .. }
            | CoreError::BlockIndexOutOfRange { .. }
            | CoreError::TaskNotFound { .. } => DiagnosticCode::NotFound,
            CoreError::DocumentRevisionMismatch { .. }
            | CoreError::TargetAuthorityMismatch { .. }
            | CoreError::FrontmatterFieldConflict { .. }
            | CoreError::DuplicateHeading { .. }
            | CoreError::AmbiguousTargetAddress { .. }
            | CoreError::AmbiguousTargetQuery { .. } => DiagnosticCode::Conflict,
            CoreError::PatchInvariant(_) => DiagnosticCode::Invariant,
            _ => DiagnosticCode::InvalidInput,
        };
        Self::new(code, error.to_string())
    }
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    schema_version: &'static str,
    error: DiagnosticCode,
    exit_code: u8,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'a str>,
}

pub fn error_envelope_json(error: &CommandError, _file: Option<&str>) -> Option<serde_json::Value> {
    serde_json::to_value(ErrorEnvelope {
        schema_version: mdtools::SCHEMA_VERSION,
        error: error.code,
        exit_code: error.exit_code as u8,
        message: &error.message,
        hint: error.hint.as_deref(),
    })
    .ok()
}
