use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn temp_copy(fixture: &str) -> std::path::PathBuf {
    let src = format!("tests/fixtures/{}", fixture);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dst = std::env::temp_dir().join(format!("mdtools_set_{}_{}.md", std::process::id(), n));
    std::fs::copy(&src, &dst).unwrap();
    dst
}

fn temp_file(content: &str) -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("mdtools_set_{}_{}.md", std::process::id(), n));
    std::fs::write(&path, content).unwrap();
    path
}

fn frontmatter_etag(path: &std::path::Path) -> String {
    let out = md().args(["frontmatter", &path.to_string_lossy()]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    json["etag"].as_str().unwrap().to_string()
}

// CLI arg order: md set <key> <file> [value]

// --- Core set operations ---

#[test]
fn set_simple_field_stdout() {
    let out = md()
        .args(["set", "title", "tests/fixtures/set_basic.md", "New Title"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("title: New Title"));
    assert!(stdout.contains("author: Jane"));
    assert!(stdout.contains("# Content"));
}

#[test]
fn set_simple_field_in_place() {
    let tmp = temp_copy("set_basic.md");
    let out = md()
        .args(["set", "title", &tmp.to_string_lossy(), "Updated", "-i"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let content = std::fs::read_to_string(&tmp).unwrap();
    assert!(content.contains("title: Updated"));
    assert!(content.contains("author: Jane"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_simple_field_json() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/set_basic.md",
            "New Title",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["command"], "SetFrontmatter");
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["target"]["FrontmatterField"]["key_path"], "title");
}

#[test]
fn set_no_change() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/set_basic.md",
            "Original",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
}

// --- Dot-path operations ---

#[test]
fn set_nested_field_creates_path() {
    let out = md()
        .args(["set", "meta.version", "tests/fixtures/set_basic.md", "2"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("meta:"));
    assert!(stdout.contains("version: 2"));
}

#[test]
fn set_deep_nested() {
    let out = md()
        .args(["set", "a.b.c", "tests/fixtures/set_basic.md", "deep"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("a:"));
    assert!(stdout.contains("b:"));
    assert!(stdout.contains("c: deep"));
}

#[test]
fn set_nested_conflict_error() {
    let out = md()
        .args(["set", "title.sub", "tests/fixtures/set_basic.md", "value"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("not an object"));
}

// --- Delete operations ---

#[test]
fn delete_existing_field() {
    let out = md()
        .args([
            "set",
            "--delete",
            "author",
            "tests/fixtures/set_basic.md",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Deleted");
    assert_eq!(json["changed"], true);
    let content = json["content"].as_str().unwrap();
    assert!(!content.contains("author:"));
    assert!(content.contains("title:"));
}

#[test]
fn delete_nonexistent_field() {
    let out = md()
        .args([
            "set",
            "--delete",
            "nosuchkey",
            "tests/fixtures/set_basic.md",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
}

// --- Frontmatter creation ---

#[test]
fn set_creates_frontmatter() {
    let out = md()
        .args(["set", "title", "tests/fixtures/no_frontmatter.md", "Hello"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("---\n"));
    assert!(stdout.contains("title: Hello"));
    assert!(stdout.contains("# Just a heading"));
}

#[test]
fn set_creates_frontmatter_json() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/no_frontmatter.md",
            "Hello",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Inserted");
    assert_eq!(json["changed"], true);
}

#[test]
fn delete_on_no_frontmatter() {
    let out = md()
        .args([
            "set",
            "--delete",
            "key",
            "tests/fixtures/no_frontmatter.md",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
}

// --- Type coercion ---

#[test]
fn set_number_value() {
    let out = md()
        .args(["set", "version", "tests/fixtures/set_basic.md", "42"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("version: 42"));
}

#[test]
fn set_bool_value() {
    let out = md()
        .args(["set", "published", "tests/fixtures/set_basic.md", "true"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("published: true"));
}

#[test]
fn set_string_flag() {
    let out = md()
        .args([
            "set",
            "version",
            "--string",
            "tests/fixtures/set_basic.md",
            "2",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("version: '2'") || stdout.contains("version: \"2\""));
}

#[test]
fn set_array_value() {
    let out = md()
        .args([
            "set",
            "categories",
            "tests/fixtures/set_basic.md",
            "[a, b, c]",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("categories:"));
}

// --- Format preservation ---

#[test]
fn set_toml_preserves_format() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/toml_frontmatter.md",
            "Updated",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("+++\n"));
    assert!(stdout.contains("title"));
}

#[test]
fn set_expect_etag_matches_yaml_state() {
    let tmp = temp_copy("set_basic.md");
    let etag = frontmatter_etag(&tmp);
    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Guarded",
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
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("title: Guarded"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_matches_toml_state() {
    let tmp = temp_copy("toml_frontmatter.md");
    let etag = frontmatter_etag(&tmp);
    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Guarded",
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
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["target"]["FrontmatterField"]["format"], "Toml");
    assert!(json["content"].as_str().unwrap().starts_with("+++\n"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_matches_absent_state() {
    let tmp = temp_copy("no_frontmatter.md");
    let etag = frontmatter_etag(&tmp);
    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Guarded",
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
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Inserted");
    assert_eq!(json["changed"], true);
    assert!(json["content"].as_str().unwrap().starts_with("---\n"));
    std::fs::remove_file(&tmp).ok();
}

// --- Body preservation ---

#[test]
fn set_preserves_body_bytes() {
    let out = md()
        .args(["set", "title", "tests/fixtures/set_basic.md", "Changed"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\n# Content\n\nBody text here.\n"));
}

#[test]
fn set_frontmatter_only_eof_no_change_is_byte_identical() {
    let content = "---\ntitle: Final Byte\n---";
    let tmp = temp_file(content);
    let etag = frontmatter_etag(&tmp);

    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Final Byte",
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
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert_eq!(json["content"].as_str().unwrap(), content);
    assert_eq!(
        json["invariant"]["target_span_before"],
        json["invariant"]["target_span_after"]
    );
    assert_eq!(std::fs::read(&tmp).unwrap(), content.as_bytes());

    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Final Byte",
            "-i",
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
    assert_eq!(std::fs::read(&tmp).unwrap(), content.as_bytes());
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_stale_update_conflict_preserves_bytes() {
    let tmp = temp_copy("set_basic.md");
    let etag = frontmatter_etag(&tmp);
    let stale = std::fs::read_to_string(&tmp).unwrap();
    let fresh = stale.replace("title: Original", "title: Intervening");
    std::fs::write(&tmp, &fresh).unwrap();

    let out = md()
        .args([
            "set",
            "author",
            &tmp.to_string_lossy(),
            "Guarded",
            "-i",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("frontmatter etag mismatch"));
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), fresh);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_stale_noop_conflict_preserves_bytes() {
    let tmp = temp_copy("set_basic.md");
    let etag = frontmatter_etag(&tmp);
    let stale = std::fs::read_to_string(&tmp).unwrap();
    let fresh = stale.replace("author: Jane", "author: Janet");
    std::fs::write(&tmp, &fresh).unwrap();

    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Original",
            "-i",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), fresh);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_stale_delete_conflict_preserves_bytes() {
    let tmp = temp_copy("set_basic.md");
    let etag = frontmatter_etag(&tmp);
    let stale = std::fs::read_to_string(&tmp).unwrap();
    let fresh = stale.replace("title: Original", "title: Drifted");
    std::fs::write(&tmp, &fresh).unwrap();

    let out = md()
        .args([
            "set",
            "--delete",
            "author",
            &tmp.to_string_lossy(),
            "-i",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), fresh);
    std::fs::remove_file(&tmp).ok();
}

// --- Error paths ---

#[test]
fn set_empty_key_error() {
    let out = md()
        .args(["set", "", "tests/fixtures/set_basic.md", "value"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap(), 3);
}

#[test]
fn set_string_with_delete_error() {
    let out = md()
        .args([
            "set",
            "--delete",
            "--string",
            "key",
            "tests/fixtures/set_basic.md",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn set_malformed_frontmatter_is_treated_as_absent() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/malformed_frontmatter.md",
            "New",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Inserted");
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("title: New"));
    assert!(content.contains(": invalid: yaml: {{{"));
}

#[test]
fn set_unclosed_frontmatter_is_treated_as_absent() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/unclosed_frontmatter.md",
            "New",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "Inserted");
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("title: New"));
    assert!(content.contains("# Main"));
}

#[test]
fn set_scalar_frontmatter_root_is_rejected() {
    let tmp = temp_file("---\nhello\n---\n\n# Main\n");
    let out = md()
        .args(["set", "title", &tmp.to_string_lossy(), "New"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("mapping/object"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_sequence_frontmatter_root_is_rejected() {
    let tmp = temp_file("---\n- one\n- two\n---\n\n# Main\n");
    let out = md()
        .args(["set", "title", &tmp.to_string_lossy(), "New"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("mapping/object"));
    std::fs::remove_file(&tmp).ok();
}

// --- Edge cases ---

#[test]
fn set_frontmatter_only_file() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/frontmatter_only.md",
            "Updated",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("---\n"));
    assert!(stdout.contains("title: Updated"));
}

#[test]
fn set_on_empty_file() {
    let tmp = temp_file("");
    let out = md()
        .args(["set", "title", &tmp.to_string_lossy(), "Hello"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("---\n"));
    assert!(stdout.contains("title: Hello"));
    assert!(stdout.ends_with("---\n"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_multiple_fields_sequentially() {
    let tmp = temp_copy("set_basic.md");
    let path = tmp.to_string_lossy().to_string();

    let out = md()
        .args(["set", "title", &path, "New Title", "-i"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = md()
        .args(["set", "status", &path, "published", "-i"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let content = std::fs::read_to_string(&tmp).unwrap();
    assert!(content.contains("title: New Title"));
    assert!(content.contains("status: published"));
    assert!(content.contains("author: Jane"));
    assert!(content.contains("# Content"));
    std::fs::remove_file(&tmp).ok();
}
