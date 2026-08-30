#![cfg(feature = "file")]

use mdtools::document::Document;
use mdtools::file::{self, PersistenceError};
use mdtools::model::{BlockKind, MutationDisposition};
use mdtools::patch::{Patch, PatchOp, ReplaceBlockTarget};
use mdtools::target::{TargetKind, TargetSummary};
use std::path::PathBuf;

fn unique_directory(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "mdtools-file-commit-{tag}-{}-{nanos}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn paragraph_target(document: &Document) -> ReplaceBlockTarget {
    let snapshot = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            snapshot.kind == TargetKind::Block
                && matches!(
                    snapshot.summary,
                    TargetSummary::Block {
                        kind: BlockKind::Paragraph,
                        ..
                    }
                )
        })
        .unwrap();
    ReplaceBlockTarget::try_from(&snapshot).unwrap()
}

fn replacement_patch(document: &Document, markdown: &str) -> Patch {
    Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: paragraph_target(document),
            markdown: markdown.into(),
        }],
    }
}

#[test]
fn verified_patch_commit_preserves_crlf_and_permissions() {
    let directory = unique_directory("happy");
    let path = directory.join("doc.md");
    std::fs::write(&path, "# H\r\n\r\nbefore\r\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    }

    let loaded = file::load(&path).unwrap();
    let patch = replacement_patch(loaded.document(), "after\n");
    let outcome = loaded.prepare_patch(&patch).unwrap().commit().unwrap();

    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# H\r\n\r\nafter\r\n"
    );
    assert_eq!(
        outcome.receipts[0].disposition(),
        MutationDisposition::Replaced
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
}

#[cfg(unix)]
#[test]
fn symlink_commit_replaces_the_referent_and_keeps_the_link() {
    let directory = unique_directory("symlink");
    let referent = directory.join("referent.md");
    let link = directory.join("doc.md");
    std::fs::write(&referent, "before\n").unwrap();
    std::os::unix::fs::symlink(&referent, &link).unwrap();

    let loaded = file::load(&link).unwrap();
    let patch = replacement_patch(loaded.document(), "after");
    loaded.prepare_patch(&patch).unwrap().commit().unwrap();

    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(std::fs::read_to_string(&referent).unwrap(), "after\n");
}

#[test]
fn intervening_content_change_refuses_commit() {
    let directory = unique_directory("intervening");
    let path = directory.join("doc.md");
    std::fs::write(&path, "before\n").unwrap();
    let loaded = file::load(&path).unwrap();
    let patch = replacement_patch(loaded.document(), "candidate");
    let prepared = loaded.prepare_patch(&patch).unwrap();

    std::fs::write(&path, "external\n").unwrap();
    assert!(matches!(
        prepared.commit(),
        Err(PersistenceError::Document(
            mdtools::core_error::CoreError::DocumentRevisionMismatch { .. }
        ))
    ));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "external\n");
}

#[cfg(unix)]
#[test]
fn same_content_inode_replacement_refuses_commit() {
    let directory = unique_directory("inode");
    let path = directory.join("doc.md");
    let displaced = directory.join("displaced.md");
    std::fs::write(&path, "before\n").unwrap();
    let loaded = file::load(&path).unwrap();
    let patch = replacement_patch(loaded.document(), "candidate");
    let prepared = loaded.prepare_patch(&patch).unwrap();

    std::fs::rename(&path, &displaced).unwrap();
    std::fs::write(&path, "before\n").unwrap();
    assert!(matches!(
        prepared.commit(),
        Err(PersistenceError::TargetChanged)
    ));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "before\n");
}

#[cfg(unix)]
#[test]
fn nochange_commit_verifies_without_replacing_the_inode() {
    use std::os::unix::fs::MetadataExt;
    let directory = unique_directory("nochange");
    let path = directory.join("doc.md");
    std::fs::write(&path, "before\n").unwrap();
    let before_inode = std::fs::metadata(&path).unwrap().ino();
    let loaded = file::load(&path).unwrap();
    let patch = replacement_patch(loaded.document(), "before\n");
    let outcome = loaded.prepare_patch(&patch).unwrap().commit().unwrap();

    assert_eq!(
        outcome.receipts[0].disposition(),
        MutationDisposition::NoChange
    );
    assert_eq!(std::fs::metadata(&path).unwrap().ino(), before_inode);
}

#[test]
fn stdout_outcome_does_not_mutate_the_loaded_file() {
    let directory = unique_directory("stdout");
    let path = directory.join("doc.md");
    std::fs::write(&path, "before\n").unwrap();
    let loaded = file::load(&path).unwrap();
    let patch = replacement_patch(loaded.document(), "candidate");
    let outcome = loaded.prepare_patch(&patch).unwrap().into_outcome();

    assert_eq!(outcome.document.source(), "candidate\n");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "before\n");
}
