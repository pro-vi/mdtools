use serde::Serialize;
use std::io::{self, Read, Write};

use crate::errors::CommandError;

/// Escape user-derived text fields for tab-separated text output.
/// Tabs → space, newlines → space.
pub fn escape_text_field(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect()
}

/// Truncate a string to max_len chars, appending "..." if truncated.
pub fn truncate_preview(s: &str, max_len: usize) -> String {
    let escaped = escape_text_field(s);
    if escaped.chars().count() <= max_len {
        escaped
    } else {
        let mut result: String = escaped.chars().take(max_len).collect();
        result.push_str("...");
        result
    }
}

pub fn normalize_line_endings(content: &str, style: &crate::model::LineEndingStyle) -> String {
    use crate::model::LineEndingStyle;
    match style {
        LineEndingStyle::Lf => content.replace("\r\n", "\n"),
        LineEndingStyle::Crlf => {
            let lf = content.replace("\r\n", "\n");
            lf.replace('\n', "\r\n")
        }
        LineEndingStyle::Mixed => content.to_string(),
    }
}

/// Read content from --from path (or stdin if path is "-" or None).
pub fn read_content(from: Option<&std::path::Path>) -> Result<String, CommandError> {
    match from {
        Some(path) if path.to_str() == Some("-") => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).map_err(|_| {
                CommandError::new(
                    crate::errors::DiagnosticCode::InvalidUtf8OnStdin,
                    "invalid UTF-8 on stdin",
                )
            })?;
            Ok(buf)
        }
        Some(path) => std::fs::read_to_string(path).map_err(|e| {
            CommandError::io(format!(
                "cannot read content file '{}': {}",
                path.display(),
                e
            ))
        }),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).map_err(|_| {
                CommandError::new(
                    crate::errors::DiagnosticCode::InvalidUtf8OnStdin,
                    "invalid UTF-8 on stdin",
                )
            })?;
            Ok(buf)
        }
    }
}

/// Stable, dependency-free shared exact-byte target fingerprint helper
/// (FNV-1a, 64-bit) rendered as 16 hex chars. Used by block, section, and
/// task-item read surfaces so an agent can guard a later command invocation
/// (`--expect-etag`) against target-content drift between the earlier read and
/// mutation attempt. Content-addressed fingerprint only, not durable target
/// identity. Deterministic across runs and platforms.
pub fn content_etag(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV offset basis
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3); // FNV prime
    }
    format!("{:016x}", hash)
}

pub fn write_json<T: Serialize>(value: &T) -> Result<(), CommandError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, value).map_err(|e| CommandError::io(e.to_string()))?;
    writeln!(handle).map_err(|e| CommandError::io(e.to_string()))?;
    Ok(())
}
