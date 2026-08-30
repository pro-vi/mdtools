#![cfg(feature = "file")]

use mdtools::document::Document;
use mdtools::file::{self, PersistenceError};
use mdtools::patch::{Patch, PatchOp, ReplaceBlockTarget};
use mdtools::target::{TargetKind, TargetSummary};
use mdtools::{BlockKind, MutationDisposition};
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

#[cfg(unix)]
#[test]
fn concurrent_commits_from_one_base_cannot_both_succeed() {
    let directory = unique_directory("concurrent");
    let path = directory.join("doc.md");
    std::fs::write(&path, "before\n").unwrap();
    let first = file::load(&path).unwrap();
    let second = file::load(&path).unwrap();
    let first_patch = replacement_patch(first.document(), "first");
    let second_patch = replacement_patch(second.document(), "second");
    let first = first.prepare_patch(&first_patch).unwrap();
    let second = second.prepare_patch(&second_patch).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let run = |prepared: mdtools::file::PreparedFilePatch,
               barrier: std::sync::Arc<std::sync::Barrier>| {
        std::thread::spawn(move || {
            barrier.wait();
            prepared.commit()
        })
    };
    let one = run(first, barrier.clone());
    let two = run(second, barrier.clone());
    barrier.wait();
    let results = [one.join().unwrap(), two.join().unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(matches!(
        std::fs::read_to_string(&path).unwrap().as_str(),
        "first\n" | "second\n"
    ));
}

#[cfg(target_os = "macos")]
#[test]
fn atomic_commit_preserves_extended_attributes() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let directory = unique_directory("xattr");
    let path = directory.join("doc.md");
    std::fs::write(&path, "before\n").unwrap();
    let path_c = CString::new(path.as_os_str().as_bytes()).unwrap();
    let name = CString::new("user.custom").unwrap();
    let value = b"kept";
    assert_eq!(
        unsafe {
            libc::setxattr(
                path_c.as_ptr(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
                0,
            )
        },
        0
    );
    let loaded = file::load(&path).unwrap();
    let patch = replacement_patch(loaded.document(), "after");
    loaded.prepare_patch(&patch).unwrap().commit().unwrap();
    let size = unsafe {
        libc::getxattr(
            path_c.as_ptr(),
            name.as_ptr(),
            std::ptr::null_mut(),
            0,
            0,
            0,
        )
    };
    assert_eq!(size, 4);
}
