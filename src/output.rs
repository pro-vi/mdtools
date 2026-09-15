use serde::Serialize;
use std::io::{self, Read, Write};

use crate::errors::{CommandError, DiagnosticCode};
use crate::usage::{MeasuredWriter, Usage};
use mdtools::read::TargetRead;

pub struct Output {
    usage: Option<Usage>,
}

impl Output {
    pub fn new(usage: Option<Usage>) -> Self {
        Self { usage }
    }

    pub fn protect_inputs(&mut self, file: &std::path::Path, from: Option<&std::path::Path>) {
        if let Some(usage) = &mut self.usage {
            usage.protect_path(file);
            if let Some(from) = from.filter(|path| path.to_str() != Some("-")) {
                usage.protect_path(from);
            }
        }
    }

    pub fn format(&mut self, format: &'static str) {
        if let Some(usage) = &mut self.usage {
            usage.format = format;
        }
    }

    pub fn query_kind(&mut self, query: &mdtools::target::TargetQuery) {
        use mdtools::target::TargetQuery;
        if let Some(usage) = &mut self.usage {
            usage.query_kind = Some(match query {
                TargetQuery::All => "all",
                TargetQuery::Kind { .. } => "kind",
                TargetQuery::Section { .. } => "section",
                TargetQuery::Task { .. } => "task",
                TargetQuery::Link { .. } => "link",
                TargetQuery::FrontmatterField { .. } => "frontmatter_field",
                TargetQuery::Search { .. } => "search",
            });
        }
    }

    pub fn finish(self, command: &str, exit: u8, diagnostic: Option<DiagnosticCode>) {
        if let Some(usage) = self.usage {
            usage.finish(command, exit, diagnostic);
        }
    }

    fn stdout(
        &mut self,
        write: impl FnOnce(&mut dyn Write) -> Result<(), CommandError>,
    ) -> Result<(), CommandError> {
        let stdout = io::stdout();
        let mut writer = MeasuredWriter {
            inner: stdout.lock(),
            capture: self.usage.as_mut().map(|usage| &mut usage.stdout),
        };
        write(&mut writer)?;
        writer.flush().map_err(CommandError::from)
    }

    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), CommandError> {
        self.stdout(|writer| writer.write_all(bytes).map_err(CommandError::from))
    }

    pub fn stderr(&mut self, text: &str) -> Result<(), CommandError> {
        let stderr = io::stderr();
        let mut writer = MeasuredWriter {
            inner: stderr.lock(),
            capture: self.usage.as_mut().map(|usage| &mut usage.stderr),
        };
        writer.write_all(text.as_bytes())?;
        writer.flush().map_err(CommandError::from)
    }

    pub fn json<T: Serialize>(&mut self, value: &T) -> Result<(), CommandError> {
        self.stdout(|mut writer| {
            serde_json::to_writer(&mut writer, value)
                .map_err(|error| CommandError::io(error.to_string()))?;
            writer.write_all(b"\n").map_err(CommandError::from)
        })
    }

    pub fn read(&mut self, read: &TargetRead) -> Result<(), CommandError> {
        self.format("content");
        let content = match read {
            TargetRead::Document(value) => &value.source,
            TargetRead::Preamble(value) => &value.markdown,
            TargetRead::Section(value) => &value.markdown,
            TargetRead::Block(value) => &value.markdown,
            TargetRead::Task(value) => &value.markdown,
            TargetRead::Table(value) => &value.markdown,
            TargetRead::TableRow(value) => &value.markdown,
            TargetRead::Frontmatter(value) => value.raw.as_deref().unwrap_or(""),
            TargetRead::FrontmatterField(value) => {
                self.format("field_value");
                return self.json(&mdtools::protocol::FrontmatterFieldValue {
                    present: value.value.is_some(),
                    value: value.value.as_ref(),
                });
            }
            TargetRead::Link(value) => &value.markdown,
        };
        self.bytes(content.as_bytes())
    }
}

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
