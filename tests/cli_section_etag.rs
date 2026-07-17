//! U3: etag ambiguity fail-closed, guarded flag, atomic write with
//! permission preservation, outline section etags.

use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn temp_file(content: &str) -> String {
    let id = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_u3_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}

/// Two byte-identical `## Setup` sections (same body), plus a distinct one.
const IDENTICAL_SECTIONS: &str =
    "# Doc\n\n## Setup\n\nsame body\n\n## Setup\n\nsame body\n\n## Other\n\ndifferent\n";

fn section_etag(file: &str, occurrence: &str) -> String {
    let out = md()
        .args([
            "section", "Setup", file, "--occurrence", occurrence, "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["section"]["etag"].as_str().unwrap().to_string()
}

#[test]
fn identical_duplicate_sections_fail_closed_as_etag_ambiguous() {
    let tmp = temp_file(IDENTICAL_SECTIONS);
    let before = std::fs::read_to_string(&tmp).unwrap();
    let etag = section_etag(&tmp, "1");
    // both occurrences share the fingerprint
    assert_eq!(etag, section_etag(&tmp, "2"));

    let out = md()
        .args([
            "delete-section",
            "Setup",
            &tmp,
            "--occurrence",
            "1",
            "-i",
            "--json",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(env["error"]["code"], "etag_ambiguous");
    assert_eq!(env["error"]["context"]["total_matches"], 2);
    // file untouched
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), before);
}

#[test]
fn unique_hash_duplicate_headings_still_mutate_with_occurrence() {
    // duplicate HEADINGS but distinct bodies -> distinct etags -> guard binds
    let tmp = temp_file("# Doc\n\n## Setup\n\nbody one\n\n## Setup\n\nbody two\n");
    let etag = section_etag(&tmp, "2");
    let out = md()
        .args([
            "delete-section",
            "Setup",
            &tmp,
            "--occurrence",
            "2",
            "-i",
            "--json",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["guarded"], true);
    assert_eq!(v["changed"], true);
    assert!(!std::fs::read_to_string(&tmp).unwrap().contains("body two"));
}

#[test]
fn identical_duplicate_tasks_fail_closed_as_etag_ambiguous() {
    let tmp = temp_file("- [ ] same text\n- [ ] same text\n");
    let tasks_out = md().args(["tasks", &tmp, "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&tasks_out.stdout).unwrap();
    let t0 = &v["results"][0]["tasks"][0];
    let etag = t0["etag"].as_str().unwrap();
    let loc = t0["loc"].as_str().unwrap();

    let out = md()
        .args([
            "set-task",
            loc,
            &tmp,
            "-i",
            "--status",
            "done",
            "--json",
            "--expect-etag",
            etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(env["error"]["code"], "etag_ambiguous");
    assert!(std::fs::read_to_string(&tmp).unwrap().contains("- [ ] same text\n- [ ] same text\n"));
}

#[test]
fn guarded_flag_reflects_etag_usage() {
    let tmp = temp_file("- [ ] one\n- [x] two\n");
    // unguarded mutation -> guarded: false
    let out = md()
        .args(["set-task", "0.0", &tmp, "-i", "--status", "done", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["guarded"], false);
    assert_eq!(v["changed"], true);
}

#[test]
fn atomic_write_preserves_permission_bits_and_leaves_no_temp() {
    let tmp = temp_file("# Doc\n\n## Setup\n\nold body\n");
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).unwrap();

    let out = md()
        .args(["delete-section", "Setup", &tmp, "-i", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let mode = std::fs::metadata(&tmp).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "permission bits must survive atomic replace");

    // no stale temp files next to the target
    let dir = std::path::Path::new(&tmp).parent().unwrap();
    let stem = std::path::Path::new(&tmp).file_name().unwrap().to_string_lossy().to_string();
    let leftovers: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains(&stem) && n.contains("md-tmp"))
        .collect();
    assert!(leftovers.is_empty(), "stale temp files: {leftovers:?}");
}

#[test]
fn outline_entries_expose_section_etags_matching_section_read() {
    let tmp = temp_file("# Doc\n\n## Setup\n\nbody one\n\n## Other\n\nbody two\n");
    let out = md().args(["outline", &tmp, "--json"]).output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let entries = v["entries"].as_array().unwrap();
    assert!(entries.iter().all(|e| e["etag"].is_string()));

    let setup_entry = entries
        .iter()
        .find(|e| e["heading"]["text"] == "Setup")
        .unwrap();
    let section = md()
        .args(["section", "Setup", &tmp, "--json"])
        .output()
        .unwrap();
    let sv: serde_json::Value = serde_json::from_slice(&section.stdout).unwrap();
    assert_eq!(setup_entry["etag"], sv["section"]["etag"]);
}
