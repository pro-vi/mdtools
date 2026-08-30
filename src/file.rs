//! Feature-gated filesystem adapter for verified atomic patch commits.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::core_error::CoreError;
use crate::document::Document;
use crate::patch::{Patch, PatchOutcome};
use crate::revision::DocumentRevision;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Document(CoreError),
    TargetChanged,
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "filesystem operation failed: {error}"),
            Self::Document(error) => error.fmt(formatter),
            Self::TargetChanged => {
                formatter.write_str("document target changed since the edit candidate was created")
            }
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Document(error) => Some(error),
            Self::TargetChanged => None,
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<CoreError> for PersistenceError {
    fn from(error: CoreError) -> Self {
        Self::Document(error)
    }
}

/// Immutable document and filesystem identity captured by one open file.
pub struct LoadedFile {
    document: Document,
    target: FileTarget,
}

impl LoadedFile {
    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn requested_path(&self) -> &Path {
        &self.target.requested_path
    }

    pub fn prepare_patch(self, patch: &Patch) -> Result<PreparedFilePatch, PersistenceError> {
        let base_revision = self.document.revision().clone();
        let outcome = patch.apply(&self.document)?;
        Ok(PreparedFilePatch {
            target: self.target,
            base_revision,
            outcome,
        })
    }
}

/// Patch result tied to the filesystem snapshot from which it was prepared.
pub struct PreparedFilePatch {
    target: FileTarget,
    base_revision: DocumentRevision,
    outcome: PatchOutcome,
}

impl PreparedFilePatch {
    pub fn outcome(&self) -> &PatchOutcome {
        &self.outcome
    }

    pub fn into_outcome(self) -> PatchOutcome {
        self.outcome
    }

    pub fn commit(self) -> Result<PatchOutcome, PersistenceError> {
        verify_unchanged(&self.target, &self.base_revision)?;
        if self.outcome.document.revision() != &self.base_revision {
            commit_source(
                &self.target,
                self.outcome.document.source(),
                &self.base_revision,
            )?;
        }
        Ok(self.outcome)
    }
}

/// Filesystem identity and revision used by retained adapters until U7.
pub struct FileTarget {
    requested_path: PathBuf,
    target: PathBuf,
    revision: DocumentRevision,
    #[cfg(unix)]
    identity: (u64, u64),
}

pub fn load(path: &Path) -> Result<LoadedFile, PersistenceError> {
    let (document, target) = read_document(path)?;
    Ok(LoadedFile { document, target })
}

pub fn read_source(path: &Path) -> Result<(String, FileTarget), PersistenceError> {
    let target = std::fs::canonicalize(path)?;
    let mut file = std::fs::File::open(&target)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        let metadata = file.metadata()?;
        (metadata.dev(), metadata.ino())
    };
    let revision = DocumentRevision::for_source(&source);
    Ok((
        source,
        FileTarget {
            requested_path: path.to_path_buf(),
            target,
            revision,
            #[cfg(unix)]
            identity,
        },
    ))
}

pub fn read_document(path: &Path) -> Result<(Document, FileTarget), PersistenceError> {
    let (source, target) = read_source(path)?;
    Ok((Document::parse(source)?, target))
}

pub fn verify_unchanged(
    target: &FileTarget,
    expected_revision: &DocumentRevision,
) -> Result<(), PersistenceError> {
    if &target.revision != expected_revision {
        return Err(CoreError::DocumentRevisionMismatch {
            expected: expected_revision.to_string(),
            actual: target.revision.to_string(),
        }
        .into());
    }
    verify_target(target)
}

pub fn commit_source(
    target: &FileTarget,
    content: &str,
    expected_revision: &DocumentRevision,
) -> Result<(), PersistenceError> {
    verify_unchanged(target, expected_revision)?;
    let temporary = temporary_sibling_path(&target.target);
    atomic_replace(&target.target, &temporary, content, Some(target))
}

fn verify_target(target: &FileTarget) -> Result<(), PersistenceError> {
    let current_target = std::fs::canonicalize(&target.requested_path)?;
    if current_target != target.target {
        return Err(PersistenceError::TargetChanged);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::symlink_metadata(&target.target)?;
        if (metadata.dev(), metadata.ino()) != target.identity {
            return Err(PersistenceError::TargetChanged);
        }
    }
    let current = std::fs::read_to_string(&target.target)?;
    crate::revision::verify_source_revision(&current, &target.revision)?;
    Ok(())
}

fn temporary_sibling_path(target: &Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let directory = target
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "md".to_string());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos())
        .unwrap_or(0);
    directory.join(format!(
        ".{file_name}.md-tmp.{}.{}.{}",
        std::process::id(),
        nanos,
        COUNTER.fetch_add(1, Ordering::SeqCst),
    ))
}

#[cfg(unix)]
fn entry_identity(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(path)
        .ok()
        .map(|metadata| (metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn cleanup_owned_temporary(path: &Path, created: Option<(u64, u64)>) {
    if created.is_some() && entry_identity(path) == created {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(not(unix))]
fn cleanup_owned_temporary(_path: &Path, _created: Option<()>) {}

fn atomic_replace(
    target: &Path,
    temporary: &Path,
    content: &str,
    guard: Option<&FileTarget>,
) -> Result<(), PersistenceError> {
    if let Some(guard) = guard {
        verify_target(guard)?;
    }
    let original_permissions = std::fs::metadata(target)?.permissions();
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut temporary_file = options.open(temporary)?;
    #[cfg(unix)]
    let created = {
        use std::os::unix::fs::MetadataExt;
        temporary_file
            .metadata()
            .ok()
            .map(|metadata| (metadata.dev(), metadata.ino()))
    };
    #[cfg(not(unix))]
    let created = Some(());

    let staged = (|| -> Result<(), PersistenceError> {
        temporary_file.set_permissions(original_permissions)?;
        temporary_file.write_all(content.as_bytes())?;
        temporary_file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let handle = temporary_file.metadata()?;
            let entry = std::fs::symlink_metadata(temporary)?;
            if handle.dev() != entry.dev() || handle.ino() != entry.ino() {
                return Err(PersistenceError::TargetChanged);
            }
        }
        if let Some(guard) = guard {
            verify_target(guard)?;
        }
        Ok(())
    })();
    drop(temporary_file);

    let result =
        staged.and_then(|()| std::fs::rename(temporary, target).map_err(PersistenceError::from));
    if result.is_err() {
        cleanup_owned_temporary(temporary, created);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_directory(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "mdtools-file-{tag}-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn exclusive_create_collision_preserves_the_foreign_entry() {
        let directory = unique_directory("collision");
        let target = directory.join("doc.md");
        let foreign = directory.join("foreign");
        std::fs::write(&target, "old\n").unwrap();
        std::fs::write(&foreign, "foreign\n").unwrap();
        assert!(atomic_replace(&target, &foreign, "new\n", None).is_err());
        assert_eq!(std::fs::read_to_string(&foreign).unwrap(), "foreign\n");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "old\n");
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_removes_only_the_created_inode() {
        let directory = unique_directory("cleanup");
        let temporary = directory.join("temporary");
        std::fs::write(&temporary, "ours\n").unwrap();
        let created = entry_identity(&temporary);
        cleanup_owned_temporary(&temporary, Some((u64::MAX, u64::MAX)));
        assert!(temporary.exists());
        cleanup_owned_temporary(&temporary, created);
        assert!(!temporary.exists());
    }
}
