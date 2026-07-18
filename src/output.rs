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
/// - Cleanup is OWNERSHIP-AWARE: the unconditional `remove_file` is gone. A
///   failed exclusive create (e.g. the name already names a planted/foreign
///   entry) returns immediately without removing anything, and error cleanup
///   only unlinks when the current directory entry still names the exact inode
///   this call created — so a substituted entry is relinquished, never deleted.
pub fn write_file_atomic(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    // Mutations always operate on an existing document; resolve symlinks so
    // the rename lands on the real file.
    let target = std::fs::canonicalize(path)?;
    let tmp_path = temp_sibling_path(&target);
    atomic_replace_via(&target, &tmp_path, content)
}

/// A collision-resistant sibling temp path (timestamp + pid + counter) in the
/// target's directory, so `rename` stays on the same filesystem.
fn temp_sibling_path(target: &std::path::Path) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "md".to_string());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    dir.join(format!(
        ".{}.md-tmp.{}.{}.{}",
        file_name,
        std::process::id(),
        nanos,
        TMP_COUNTER.fetch_add(1, Ordering::SeqCst),
    ))
}

/// Identity (dev, ino) of the entry currently at `tmp_path`, if any. Used to
/// prove the entry we are about to unlink is still the one we created.
#[cfg(unix)]
fn entry_identity(tmp_path: &std::path::Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(tmp_path)
        .ok()
        .map(|m| (m.dev(), m.ino()))
}

/// Remove the staging entry ONLY when it is provably still the inode we
/// created. A substituted entry (or one already renamed/gone) is relinquished,
/// never deleted. There is no unconditional-unlink path.
#[cfg(unix)]
fn cleanup_owned_temp(tmp_path: &std::path::Path, created: Option<(u64, u64)>) {
    if let Some(created) = created {
        if entry_identity(tmp_path) == Some(created) {
            let _ = std::fs::remove_file(tmp_path);
        }
    }
}

/// Off unix we cannot compare inodes; reaching cleanup means the exclusive
/// create succeeded (so the temp is ours), and no cross-process inode
/// substitution race applies with an open handle. Best-effort remove.
#[cfg(not(unix))]
fn cleanup_owned_temp(tmp_path: &std::path::Path, created: Option<()>) {
    if created.is_some() {
        let _ = std::fs::remove_file(tmp_path);
    }
}

fn atomic_replace_via(
    target: &std::path::Path,
    tmp_path: &std::path::Path,
    content: &str,
) -> std::io::Result<()> {
    use std::io::Write;

    let original_perms = std::fs::metadata(target)?.permissions();

    // create_new refuses existing files AND symlinks (dangling or not); on
    // unix the file is BORN 0600 (no umask window an observer could exploit by
    // opening the fd before a later chmod). If this fails — most importantly
    // because the name already exists (a planted/foreign entry) — we created
    // NOTHING and return immediately, without touching the pre-existing entry.
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut tmp = opts.open(tmp_path)?;

    // From here we OWN tmp_path. Capture the created inode so cleanup can only
    // ever remove the exact entry we made — never a later substitution.
    #[cfg(unix)]
    let created: Option<(u64, u64)> = {
        use std::os::unix::fs::MetadataExt;
        tmp.metadata().ok().map(|m| (m.dev(), m.ino()))
    };
    #[cfg(not(unix))]
    let created: Option<()> = Some(());

    let staged = (|| -> std::io::Result<()> {
        // Apply the original's bits through the HANDLE (fchmod), never the
        // pathname, before writing a single byte.
        tmp.set_permissions(original_perms.clone())?;
        tmp.write_all(content.as_bytes())?;
        tmp.sync_all()?;
        // Bind the rename to the inode we actually wrote: if the directory
        // entry was substituted underneath us, refuse rather than install an
        // attacker's file at the target. (The residual check→rename gap is
        // unavoidable with std's pathname rename; this narrows it to
        // nanoseconds and makes silent substitution detectable.)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let handle_meta = tmp.metadata()?;
            let entry_meta = std::fs::symlink_metadata(tmp_path)?;
            if handle_meta.dev() != entry_meta.dev() || handle_meta.ino() != entry_meta.ino() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "temp file directory entry was substituted during write; aborting mutation",
                ));
            }
        }
        Ok(())
    })();

    // Release the handle before the rename (and before any cleanup).
    drop(tmp);

    let result = staged.and_then(|()| std::fs::rename(tmp_path, target));
    if result.is_err() {
        cleanup_owned_temp(tmp_path, created);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_dir(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdtools-out-{}-{}-{}-{}",
            tag,
            std::process::id(),
            nanos,
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst),
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn atomic_replace_writes_and_removes_its_own_temp() {
        let dir = unique_dir("happy");
        let target = dir.join("doc.md");
        std::fs::write(&target, "old\n").unwrap();
        let tmp = temp_sibling_path(&target);

        atomic_replace_via(&target, &tmp, "new body\n").unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new body\n");
        assert!(
            !tmp.exists(),
            "the staging temp must be gone after a successful rename"
        );
    }

    #[test]
    fn create_collision_leaves_the_foreign_entry_untouched() {
        let dir = unique_dir("collision");
        let target = dir.join("doc.md");
        std::fs::write(&target, "original target\n").unwrap();

        // A foreign entry already sits at the temp path we will try to create.
        let planted = dir.join(".doc.md.md-tmp.planted");
        std::fs::write(&planted, "PLANTED — not ours\n").unwrap();

        let result = atomic_replace_via(&target, &planted, "attacker-provided\n");
        assert!(
            result.is_err(),
            "exclusive create must fail on an existing name"
        );
        // The foreign entry is neither overwritten nor deleted.
        assert_eq!(
            std::fs::read_to_string(&planted).unwrap(),
            "PLANTED — not ours\n"
        );
        // The target is untouched.
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "original target\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_only_removes_the_inode_it_created() {
        let dir = unique_dir("ownership");
        let owned = dir.join("owned-temp");
        std::fs::write(&owned, "ours\n").unwrap();
        let created = entry_identity(&owned);
        assert!(created.is_some());

        // Wrong identity (a foreign entry): cleanup must relinquish, not delete.
        cleanup_owned_temp(&owned, Some((999_999, 999_999)));
        assert!(
            owned.exists(),
            "an entry we do not own must never be removed"
        );

        // Matching identity: cleanup removes our own temp.
        cleanup_owned_temp(&owned, created);
        assert!(
            !owned.exists(),
            "our own created temp is removed on cleanup"
        );
    }

    #[cfg(unix)]
    #[test]
    fn substituted_entry_is_relinquished_not_deleted() {
        use std::os::unix::fs::MetadataExt;
        let dir = unique_dir("substitution");
        let tmp = dir.join("staging");
        std::fs::write(&tmp, "the temp we created\n").unwrap();
        let created = entry_identity(&tmp);

        // Simulate a substitution: replace the entry with a different inode.
        std::fs::remove_file(&tmp).unwrap();
        std::fs::write(&tmp, "attacker file at same path\n").unwrap();
        let substituted_ino = std::fs::symlink_metadata(&tmp).unwrap().ino();
        assert_ne!(created.map(|(_, ino)| ino), Some(substituted_ino));

        // Cleanup keyed on OUR created identity must not touch the foreign inode.
        cleanup_owned_temp(&tmp, created);
        assert!(
            tmp.exists(),
            "a substituted entry must be relinquished, never deleted"
        );
        assert_eq!(
            std::fs::read_to_string(&tmp).unwrap(),
            "attacker file at same path\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn atomic_replace_preserves_target_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = unique_dir("perms");
        let target = dir.join("doc.md");
        std::fs::write(&target, "old\n").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();

        let tmp = temp_sibling_path(&target);
        atomic_replace_via(&target, &tmp, "new\n").unwrap();

        let mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o640,
            "the replaced file keeps the original's permission bits"
        );
    }
}
