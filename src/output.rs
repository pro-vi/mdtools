use serde::Serialize;
use std::io::{self, Read, Write};

use crate::errors::{CommandError, DiagnosticCode};

pub fn read_content(from: Option<&std::path::Path>) -> Result<String, CommandError> {
    match from {
        Some(path) if path.to_str() == Some("-") => read_stdin(),
        Some(path) => std::fs::read_to_string(path).map_err(|error| {
            CommandError::io(format!(
                "cannot read protocol file '{}': {error}",
                path.display()
            ))
        }),
        None => read_stdin(),
    }
}

fn read_stdin() -> Result<String, CommandError> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|_| CommandError::new(DiagnosticCode::InvalidInput, "invalid UTF-8 on stdin"))?;
    Ok(buffer)
}

pub(crate) fn persistence_error(error: mdtools::file::PersistenceError) -> CommandError {
    match error {
        mdtools::file::PersistenceError::Io(error) => CommandError::io(error.to_string()),
        mdtools::file::PersistenceError::Document(error) => error.into(),
        mdtools::file::PersistenceError::TargetChanged => CommandError::new(
            DiagnosticCode::Conflict,
            "document target changed since the patch was prepared",
        ),
    }
}

pub fn write_json<T: Serialize>(value: &T) -> Result<(), CommandError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, value)
        .map_err(|error| CommandError::io(error.to_string()))?;
    writeln!(handle).map_err(|error| CommandError::io(error.to_string()))?;
    handle.flush().map_err(CommandError::from)?;
    Ok(())
}
