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
/// table, and task-item read surfaces so an agent can guard a later command invocation
/// (`--expect-etag`) against target-content drift between the earlier read and
/// mutation attempt. Content-addressed fingerprint only, not durable target
/// identity. Deterministic across runs and platforms.
/// Atomically replace `path` with `content`.
///
/// Safety properties (review-hardened):
/// - The target is canonicalized first, so mutating through a symlinked
///   document rewrites the REFERENT (editor semantics) instead of replacing
///   the symlink itself.
/// - The temp file is created with `create_new` (O_CREAT|O_EXCL), which
///   refuses to follow a pre-planted symlink or reuse an existing file, and
///   its name carries a timestamp + pid + counter so it is not predictable
///   from the pid alone.
/// - Permission bits are copied from the original BEFORE any content is
///   written, so restrictive documents are never readable via a
///   default-mode temp window.
/// - A killed process can leave a stale temp file but never a truncated or
///   partially-written target. Waiver: the parent directory is not fsynced
///   after the rename, so a machine crash in that instant can revert to the
///   OLD contents on some filesystems — old-or-new, never corrupt — which
///   is proportionate for a document CLI.
pub fn write_file_atomic(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Mutations always operate on an existing document; resolve symlinks so
    // the rename lands on the real file.
    let target = std::fs::canonicalize(path)?;
    let original_perms = std::fs::metadata(&target)?.permissions();

    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "md".to_string());

    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let tmp_path = dir.join(format!(
        ".{}.md-tmp.{}.{}.{}",
        file_name,
        std::process::id(),
        nanos,
        TMP_COUNTER.fetch_add(1, Ordering::SeqCst),
    ));

    let result = (|| {
        // create_new refuses existing files AND symlinks (dangling or not);
        // on unix the file is BORN 0600 (no umask window an observer could
        // exploit by opening the fd before a later chmod).
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut tmp = opts.open(&tmp_path)?;
        // Apply the original's bits through the HANDLE (fchmod), never the
        // pathname, before writing a single byte.
        tmp.set_permissions(original_perms.clone())?;
        tmp.write_all(content.as_bytes())?;
        tmp.sync_all()?;
        // Bind the rename to the inode we actually wrote: if the directory
        // entry was substituted underneath us, refuse rather than install
        // an attacker's file at the target. (The residual check→rename gap
        // is unavoidable with std's pathname rename; this narrows it to
        // nanoseconds and makes silent substitution detectable.)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let handle_meta = tmp.metadata()?;
            let entry_meta = std::fs::symlink_metadata(&tmp_path)?;
            if handle_meta.dev() != entry_meta.dev() || handle_meta.ino() != entry_meta.ino() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "temp file directory entry was substituted during write; aborting mutation",
                ));
            }
        }
        drop(tmp);
        std::fs::rename(&tmp_path, &target)
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }
    result
}

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
