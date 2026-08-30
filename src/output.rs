use serde::Serialize;
use std::io::{self, Read, Write};

use crate::errors::CommandError;
use mdtools::document::Document;
use mdtools::revision::DocumentRevision;

pub fn escape_text_field(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect()
}

pub fn read_content(from: Option<&std::path::Path>) -> Result<String, CommandError> {
    match from {
        Some(path) if path.to_str() == Some("-") => read_stdin(),
        Some(path) => std::fs::read_to_string(path).map_err(|error| {
            CommandError::io(format!(
                "cannot read content file '{}': {error}",
                path.display()
            ))
        }),
        None => read_stdin(),
    }
}

fn read_stdin() -> Result<String, CommandError> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).map_err(|_| {
        CommandError::new(
            crate::errors::DiagnosticCode::InvalidUtf8OnStdin,
            "invalid UTF-8 on stdin",
        )
    })?;
    Ok(buffer)
}

pub(crate) struct EditFile {
    source: String,
    target: EditTarget,
}

impl EditFile {
    pub fn into_parts(self) -> (String, EditTarget) {
        (self.source, self.target)
    }
}

pub(crate) type EditTarget = mdtools::file::FileTarget;

pub(crate) fn read_edit_file(path: &std::path::Path) -> Result<EditFile, CommandError> {
    let (source, target) = mdtools::file::read_source(path).map_err(persistence_error)?;
    Ok(EditFile { source, target })
}

pub(crate) fn read_edit_document(
    path: &std::path::Path,
) -> Result<(Document, EditTarget), CommandError> {
    mdtools::file::read_document(path).map_err(persistence_error)
}

pub(crate) fn verify_file_unchanged(
    target: &EditTarget,
    expected_revision: &DocumentRevision,
) -> Result<(), CommandError> {
    mdtools::file::verify_unchanged(target, expected_revision).map_err(persistence_error)
}

pub(crate) fn write_file_atomic_verified(
    target: &EditTarget,
    content: &str,
    expected_revision: &DocumentRevision,
) -> Result<(), CommandError> {
    mdtools::file::commit_source(target, content, expected_revision).map_err(persistence_error)
}

fn persistence_error(error: mdtools::file::PersistenceError) -> CommandError {
    match error {
        mdtools::file::PersistenceError::Io(error) => error.into(),
        mdtools::file::PersistenceError::Document(error) => error.into(),
        mdtools::file::PersistenceError::TargetChanged => CommandError::new(
            crate::errors::DiagnosticCode::EtagMismatch,
            "document target changed since the edit candidate was created",
        )
        .with_hint("re-read the document path and rebuild the edit candidate before retrying"),
    }
}

pub fn write_json<T: Serialize>(value: &T) -> Result<(), CommandError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, value)
        .map_err(|error| CommandError::io(error.to_string()))?;
    writeln!(handle).map_err(|error| CommandError::io(error.to_string()))?;
    Ok(())
}
