//! Adoption-blocker fixes from the editor-bench verb-adoption signal (2026-05-28).
//!
//! Finding #5 named two failure modes that cost cells even when an agent makes the
//! right structural choice:
//!   Bug A — block-index misidentification (stale index between read and write).
//!   Bug B — `--from` temp-file ergonomics (trailing newline from `cat <<'EOF' > f`).
//!
//! Bug A fix: `md blocks`/`md block` JSON carry a per-block `etag` (content
//! fingerprint); mutation verbs accept `--expect-etag` and fail-closed (exit 4)
//! when the target block's current fingerprint differs — making the
//! read → mutate → re-query loop safe.
//!
//! Bug B fix: `replace-block` strips one trailing line-ending from the
//! replacement, matching the block-span convention (spans exclude the trailing
//! newline), so the universal `cat`/editor trailing-`\n` no longer injects a
//! spurious blank line or defeats the no-op check.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn md_with_stdin(args: &[&str], stdin_content: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_md"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin_content.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn tempfile(content: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!(
        "/tmp/mdtools_adoption_{}_{}.md",
        std::process::id(),
        id
    );
    std::fs::write(&path, content).unwrap();
    path
}

fn cleanup(path: &str) {
    std::fs::remove_file(path).ok();
}

/// Read block `idx`'s etag from `md blocks --json`.
fn block_etag(path: &str, idx: usize) -> String {
    let out = md().args(["blocks", path, "--json"]).output().unwrap();
    assert!(out.status.success(), "md blocks --json should succeed");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    json["blocks"][idx]["etag"]
        .as_str()
        .expect("each block should carry an `etag` field in JSON output")
        .to_string()
}

// ---------------------------------------------------------------------------
// Bug B — `--from`/stdin trailing-newline ergonomics on replace-block
// ---------------------------------------------------------------------------

#[test]
fn replace_block_strips_single_trailing_newline_from_content() {
    // Simulates `cat <<'EOF' > new.md ... EOF; md replace-block 1 doc.md -i --from new.md`,
    // where the temp file (like any editor/`cat` output) ends with a trailing newline.
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let out = md_with_stdin(&["replace-block", "1", &tmp, "-i"], "Rewritten para.\n");
    assert!(out.status.success());
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(
        result, "# Title\n\nRewritten para.\n\nSecond para.\n",
        "trailing newline in content must not inject a spurious blank line"
    );
    assert!(
        !result.contains("\n\n\n"),
        "no doubled blank line should appear, got: {result:?}"
    );
    cleanup(&tmp);
}

#[test]
fn replace_block_roundtrip_with_trailing_newline_is_noop() {
    // Read a block's content, write it straight back with a trailing newline
    // (what a heredoc/temp-file round-trip produces). Must register as NoChange,
    // not a spurious Replaced.
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let out = md_with_stdin(
        &["replace-block", "1", &tmp, "-i", "--json"],
        "First para.\n",
    );
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        json["disposition"], "NoChange",
        "round-trip of identical content (modulo trailing newline) must be NoChange"
    );
    assert_eq!(json["changed"], false);
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(result, "# Title\n\nFirst para.\n\nSecond para.\n");
    cleanup(&tmp);
}

#[test]
fn replace_block_indented_code_preserves_trailing_newline() {
    // Indented-code spans INCLUDE the trailing newline (parser fixup), unlike
    // every other block kind. The Bug B strip must NOT truncate it: a round-trip
    // of the block's own content must register as NoChange and leave the file
    // byte-identical. Guards against a kind-blind strip regressing indented code.
    let doc = "# Title\n\n    indented code line\n\nNext para.\n";
    let tmp = tempfile(doc);
    let read = md().args(["block", "1", &tmp, "--json"]).output().unwrap();
    assert!(read.status.success());
    let json: serde_json::Value = serde_json::from_slice(&read.stdout).unwrap();
    // Confirm we're actually exercising indented code with a newline-terminated span.
    assert_eq!(json["block"]["kind"], "IndentedCode");
    let content = json["content"].as_str().unwrap().to_string();
    assert!(content.ends_with('\n'), "indented-code slice should include trailing newline");

    let out = md_with_stdin(&["replace-block", "1", &tmp, "-i", "--json"], &content);
    assert!(out.status.success());
    let json2: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        json2["disposition"], "NoChange",
        "indented-code round-trip must be NoChange (trailing newline preserved)"
    );
    assert_eq!(
        std::fs::read_to_string(&tmp).unwrap(),
        doc,
        "file must be byte-identical after an indented-code round-trip"
    );
    cleanup(&tmp);
}

// ---------------------------------------------------------------------------
// Bug A — per-block etag + `--expect-etag` stale-index guard
// ---------------------------------------------------------------------------

#[test]
fn blocks_json_includes_stable_per_block_etag() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let e1 = block_etag(&tmp, 1);
    let e1_again = block_etag(&tmp, 1);
    assert_eq!(e1, e1_again, "etag must be deterministic across reads");
    assert!(!e1.is_empty(), "etag must be non-empty");
    let e2 = block_etag(&tmp, 2);
    assert_ne!(
        e1, e2,
        "blocks with different content must have different etags"
    );
    cleanup(&tmp);
}

#[test]
fn block_read_json_includes_etag() {
    let tmp = tempfile("# Title\n\nFirst para.\n");
    let out = md().args(["block", "1", &tmp, "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        json["block"]["etag"].as_str().is_some(),
        "`md block N --json` should carry the block etag"
    );
    cleanup(&tmp);
}

#[test]
fn replace_block_expect_etag_match_succeeds() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let etag = block_etag(&tmp, 1);
    let out = md_with_stdin(
        &["replace-block", "1", &tmp, "-i", "--expect-etag", &etag],
        "Rewritten.",
    );
    assert!(
        out.status.success(),
        "matching etag must pass; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(result.contains("Rewritten."));
    assert!(!result.contains("First para."));
    cleanup(&tmp);
}

#[test]
fn replace_block_expect_etag_mismatch_fails_closed() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let before = std::fs::read_to_string(&tmp).unwrap();
    let out = md_with_stdin(
        &[
            "replace-block",
            "1",
            &tmp,
            "-i",
            "--expect-etag",
            "deadbeefdeadbeef",
        ],
        "Should not be written.",
    );
    assert_eq!(
        out.status.code(),
        Some(4),
        "stale etag must fail-closed with Conflict exit code 4"
    );
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(before, after, "file must be untouched on etag mismatch");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("etag"),
        "diagnostic should mention etag and steer a re-query, got: {stderr}"
    );
    cleanup(&tmp);
}

#[test]
fn delete_block_expect_etag_mismatch_fails_closed() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let before = std::fs::read_to_string(&tmp).unwrap();
    let out = md()
        .args([
            "delete-block",
            "1",
            &tmp,
            "-i",
            "--expect-etag",
            "deadbeefdeadbeef",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(before, after, "file must be untouched on etag mismatch");
    cleanup(&tmp);
}

#[test]
fn delete_block_expect_etag_match_succeeds() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let etag = block_etag(&tmp, 1);
    let out = md()
        .args(["delete-block", "1", &tmp, "-i", "--expect-etag", &etag])
        .output()
        .unwrap();
    assert!(out.status.success());
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(!result.contains("First para."));
    cleanup(&tmp);
}

#[test]
fn insert_block_after_expect_etag_mismatch_fails_closed() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let before = std::fs::read_to_string(&tmp).unwrap();
    let out = md_with_stdin(
        &[
            "insert-block",
            "--after",
            "1",
            &tmp,
            "-i",
            "--expect-etag",
            "deadbeefdeadbeef",
        ],
        "Inserted.",
    );
    assert_eq!(out.status.code(), Some(4));
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(before, after, "file must be untouched on etag mismatch");
    cleanup(&tmp);
}

#[test]
fn insert_block_after_expect_etag_match_succeeds() {
    let tmp = tempfile("# Title\n\nFirst para.\n\nSecond para.\n");
    let etag = block_etag(&tmp, 1);
    let out = md_with_stdin(
        &[
            "insert-block",
            "--after",
            "1",
            &tmp,
            "-i",
            "--expect-etag",
            &etag,
        ],
        "Inserted.",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(result.contains("Inserted."));
    cleanup(&tmp);
}

#[test]
fn insert_block_at_end_with_expect_etag_is_usage_error() {
    // No block anchor => etag has nothing to verify against. Fail-closed as a
    // usage error rather than silently ignoring the guard.
    let tmp = tempfile("# Title\n\nPara.\n");
    let out = md_with_stdin(
        &["insert-block", "--at-end", &tmp, "--expect-etag", "abc"],
        "X",
    );
    assert_eq!(
        out.status.code(),
        Some(3),
        "--expect-etag without --before/--after must be an InvalidInput usage error"
    );
    cleanup(&tmp);
}
