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
    let out = md()
        .args(["frontmatter", &path.to_string_lossy()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    json["etag"].as_str().unwrap().to_string()
}

fn raw_frontmatter_etag(raw: Option<&str>) -> String {
    const ABSENT_DOMAIN: &[u8] = b"mdtools.frontmatter.absent";
    const PRESENT_DOMAIN: &[u8] = b"mdtools.frontmatter.present\0";

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let bytes = raw.map(str::as_bytes);
    let domain = if bytes.is_some() {
        PRESENT_DOMAIN
    } else {
        ABSENT_DOMAIN
    };

    for &b in domain {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    if let Some(bytes) = bytes {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    format!("{:016x}", hash)
}

fn assert_stale_missing_delete_conflict_preserves_bytes(
    label: &str,
    original: &str,
    drifted: &str,
) {
    let tmp = temp_file(original);
    let etag = frontmatter_etag(&tmp);
    std::fs::write(&tmp, drifted).unwrap();

    let out = md()
        .args([
            "set",
            "--delete",
            "missing",
            &tmp.to_string_lossy(),
            "-i",
            "--json",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(4),
        "{label}: EtagMismatch exit code"
    );
    assert!(out.stdout.is_empty(), "{label}: stdout must stay empty");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("frontmatter etag mismatch"),
        "{label}: EtagMismatch diagnostic"
    );
    assert_eq!(
        std::fs::read(&tmp).unwrap(),
        drifted.as_bytes(),
        "{label}: file must stay byte-identical on EtagMismatch"
    );

    std::fs::remove_file(&tmp).ok();
}

fn span_value(span: &serde_json::Value) -> Option<(u64, u64, u64, u64)> {
    if span.is_null() {
        None
    } else {
        Some((
            span["line_start"].as_u64().unwrap(),
            span["line_end"].as_u64().unwrap(),
            span["byte_start"].as_u64().unwrap(),
            span["byte_end"].as_u64().unwrap(),
        ))
    }
}

fn assert_set_frontmatter_suffix_preserved(label: &str, json: &serde_json::Value, original: &str) {
    let content = json["content"].as_str().unwrap();
    let before = span_value(&json["invariant"]["target_span_before"]);
    let after = span_value(&json["invariant"]["target_span_after"]);

    match (before, after) {
        (Some((_, _, _, before_end)), Some((_, _, _, after_end))) => assert_eq!(
            &content[after_end as usize..],
            &original[before_end as usize..],
            "{label}: bytes after the owned frontmatter span must stay identical"
        ),
        (None, Some((_, _, _, after_end))) => assert_eq!(
            &content[after_end as usize..],
            &format!("\n{original}"),
            "{label}: inserting absent frontmatter must preserve the original document bytes after the inserted state"
        ),
        (None, None) => assert_eq!(
            content, original,
            "{label}: absent-state no-op must preserve the full document"
        ),
        (Some(_), None) => panic!("{label}: unexpected present-before / absent-after span state"),
    }
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

#[test]
fn set_expect_etag_matching_guards_cover_yaml_and_toml_dispositions() {
    let yaml = temp_copy("set_basic.md");
    let yaml_etag = frontmatter_etag(&yaml);

    let replace = md()
        .args([
            "set",
            "title",
            &yaml.to_string_lossy(),
            "Guarded Replace",
            "--json",
            "--expect-etag",
            &yaml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        replace.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&replace.stderr)
    );
    let replace_json: serde_json::Value = serde_json::from_slice(&replace.stdout).unwrap();
    assert_eq!(replace_json["disposition"], "Replaced");

    let insert = md()
        .args([
            "set",
            "meta.version",
            &yaml.to_string_lossy(),
            "2",
            "--json",
            "--expect-etag",
            &yaml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        insert.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&insert.stderr)
    );
    let insert_json: serde_json::Value = serde_json::from_slice(&insert.stdout).unwrap();
    assert_eq!(insert_json["disposition"], "Inserted");

    let delete = md()
        .args([
            "set",
            "--delete",
            "author",
            &yaml.to_string_lossy(),
            "--json",
            "--expect-etag",
            &yaml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        delete.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&delete.stderr)
    );
    let delete_json: serde_json::Value = serde_json::from_slice(&delete.stdout).unwrap();
    assert_eq!(delete_json["disposition"], "Deleted");

    let noop = md()
        .args([
            "set",
            "title",
            &yaml.to_string_lossy(),
            "Original",
            "--json",
            "--expect-etag",
            &yaml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        noop.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&noop.stderr)
    );
    let noop_json: serde_json::Value = serde_json::from_slice(&noop.stdout).unwrap();
    assert_eq!(noop_json["disposition"], "NoChange");

    let toml = temp_copy("toml_frontmatter.md");
    let toml_etag = frontmatter_etag(&toml);

    let toml_replace = md()
        .args([
            "set",
            "title",
            &toml.to_string_lossy(),
            "Guarded Replace",
            "--json",
            "--expect-etag",
            &toml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        toml_replace.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&toml_replace.stderr)
    );
    let toml_replace_json: serde_json::Value =
        serde_json::from_slice(&toml_replace.stdout).unwrap();
    assert_eq!(toml_replace_json["disposition"], "Replaced");

    let toml_insert = md()
        .args([
            "set",
            "meta.version",
            &toml.to_string_lossy(),
            "2",
            "--json",
            "--expect-etag",
            &toml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        toml_insert.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&toml_insert.stderr)
    );
    let toml_insert_json: serde_json::Value = serde_json::from_slice(&toml_insert.stdout).unwrap();
    assert_eq!(toml_insert_json["disposition"], "Inserted");

    let toml_delete = md()
        .args([
            "set",
            "--delete",
            "version",
            &toml.to_string_lossy(),
            "--json",
            "--expect-etag",
            &toml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        toml_delete.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&toml_delete.stderr)
    );
    let toml_delete_json: serde_json::Value = serde_json::from_slice(&toml_delete.stdout).unwrap();
    assert_eq!(toml_delete_json["disposition"], "Deleted");

    let toml_noop = md()
        .args([
            "set",
            "title",
            &toml.to_string_lossy(),
            "TOML Doc",
            "--json",
            "--expect-etag",
            &toml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        toml_noop.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&toml_noop.stderr)
    );
    let toml_noop_json: serde_json::Value = serde_json::from_slice(&toml_noop.stdout).unwrap();
    assert_eq!(toml_noop_json["disposition"], "NoChange");

    std::fs::remove_file(&yaml).ok();
    std::fs::remove_file(&toml).ok();
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
fn set_frontmatter_whole_state_span_matrix_rows() {
    let existing_insert = temp_copy("set_basic.md");
    let existing_insert_path = existing_insert.to_string_lossy().to_string();
    let existing_insert_source = std::fs::read_to_string(&existing_insert).unwrap();
    let existing_insert_out = md()
        .args(["set", "meta.version", &existing_insert_path, "2", "--json"])
        .output()
        .unwrap();
    assert!(
        existing_insert_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&existing_insert_out.stderr)
    );
    let existing_insert_json: serde_json::Value =
        serde_json::from_slice(&existing_insert_out.stdout).unwrap();
    assert_eq!(existing_insert_json["disposition"], "Inserted");
    assert_eq!(existing_insert_json["changed"], true);
    assert!(existing_insert_json["invariant"]["target_span_before"].is_object());
    assert!(existing_insert_json["invariant"]["target_span_after"].is_object());
    assert_ne!(
        existing_insert_json["invariant"]["target_span_before"],
        existing_insert_json["invariant"]["target_span_after"]
    );
    assert_eq!(
        existing_insert_json["invariant"]["preserves_non_target_bytes"],
        true
    );
    assert_set_frontmatter_suffix_preserved(
        "existing state + insert field",
        &existing_insert_json,
        &existing_insert_source,
    );
    std::fs::remove_file(&existing_insert).ok();

    let existing_replace = temp_copy("set_basic.md");
    let existing_replace_path = existing_replace.to_string_lossy().to_string();
    let existing_replace_source = std::fs::read_to_string(&existing_replace).unwrap();
    let existing_replace_out = md()
        .args([
            "set",
            "title",
            &existing_replace_path,
            "Updated Longer Title",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        existing_replace_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&existing_replace_out.stderr)
    );
    let existing_replace_json: serde_json::Value =
        serde_json::from_slice(&existing_replace_out.stdout).unwrap();
    assert_eq!(existing_replace_json["disposition"], "Replaced");
    assert_eq!(existing_replace_json["changed"], true);
    assert!(existing_replace_json["invariant"]["target_span_before"].is_object());
    assert!(existing_replace_json["invariant"]["target_span_after"].is_object());
    assert_ne!(
        existing_replace_json["invariant"]["target_span_before"],
        existing_replace_json["invariant"]["target_span_after"]
    );
    assert_eq!(
        existing_replace_json["invariant"]["preserves_non_target_bytes"],
        true
    );
    assert_set_frontmatter_suffix_preserved(
        "existing state + replace field",
        &existing_replace_json,
        &existing_replace_source,
    );
    std::fs::remove_file(&existing_replace).ok();

    let existing_delete = temp_copy("set_basic.md");
    let existing_delete_path = existing_delete.to_string_lossy().to_string();
    let existing_delete_source = std::fs::read_to_string(&existing_delete).unwrap();
    let existing_delete_out = md()
        .args(["set", "--delete", "author", &existing_delete_path, "--json"])
        .output()
        .unwrap();
    assert!(
        existing_delete_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&existing_delete_out.stderr)
    );
    let existing_delete_json: serde_json::Value =
        serde_json::from_slice(&existing_delete_out.stdout).unwrap();
    assert_eq!(existing_delete_json["disposition"], "Deleted");
    assert_eq!(existing_delete_json["changed"], true);
    assert!(existing_delete_json["invariant"]["target_span_before"].is_object());
    assert!(existing_delete_json["invariant"]["target_span_after"].is_object());
    assert_ne!(
        existing_delete_json["invariant"]["target_span_before"],
        existing_delete_json["invariant"]["target_span_after"]
    );
    assert_eq!(
        existing_delete_json["invariant"]["preserves_non_target_bytes"],
        true
    );
    assert_set_frontmatter_suffix_preserved(
        "existing state + delete field",
        &existing_delete_json,
        &existing_delete_source,
    );
    std::fs::remove_file(&existing_delete).ok();

    let existing_no_change = temp_copy("set_basic.md");
    let existing_no_change_path = existing_no_change.to_string_lossy().to_string();
    let existing_no_change_source = std::fs::read_to_string(&existing_no_change).unwrap();
    let existing_no_change_out = md()
        .args([
            "set",
            "title",
            &existing_no_change_path,
            "Original",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        existing_no_change_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&existing_no_change_out.stderr)
    );
    let existing_no_change_json: serde_json::Value =
        serde_json::from_slice(&existing_no_change_out.stdout).unwrap();
    assert_eq!(existing_no_change_json["disposition"], "NoChange");
    assert_eq!(existing_no_change_json["changed"], false);
    assert_eq!(
        existing_no_change_json["invariant"]["target_span_before"],
        existing_no_change_json["invariant"]["target_span_after"]
    );
    assert_eq!(
        existing_no_change_json["invariant"]["preserves_non_target_bytes"],
        true
    );
    assert_set_frontmatter_suffix_preserved(
        "existing state + no-change field",
        &existing_no_change_json,
        &existing_no_change_source,
    );
    std::fs::remove_file(&existing_no_change).ok();

    let absent_insert = temp_copy("no_frontmatter.md");
    let absent_insert_path = absent_insert.to_string_lossy().to_string();
    let absent_insert_source = std::fs::read_to_string(&absent_insert).unwrap();
    let absent_insert_out = md()
        .args(["set", "title", &absent_insert_path, "Hello", "--json"])
        .output()
        .unwrap();
    assert!(
        absent_insert_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&absent_insert_out.stderr)
    );
    let absent_insert_json: serde_json::Value =
        serde_json::from_slice(&absent_insert_out.stdout).unwrap();
    assert_eq!(absent_insert_json["disposition"], "Inserted");
    assert_eq!(absent_insert_json["changed"], true);
    assert!(absent_insert_json["invariant"]["target_span_before"].is_null());
    assert!(absent_insert_json["invariant"]["target_span_after"].is_object());
    assert_eq!(
        absent_insert_json["invariant"]["preserves_non_target_bytes"],
        true
    );
    assert_set_frontmatter_suffix_preserved(
        "absent state + insert field",
        &absent_insert_json,
        &absent_insert_source,
    );
    std::fs::remove_file(&absent_insert).ok();

    let absent_delete_missing = temp_copy("no_frontmatter.md");
    let absent_delete_missing_path = absent_delete_missing.to_string_lossy().to_string();
    let absent_delete_missing_source = std::fs::read_to_string(&absent_delete_missing).unwrap();
    let absent_delete_missing_out = md()
        .args([
            "set",
            "--delete",
            "missing",
            &absent_delete_missing_path,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        absent_delete_missing_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&absent_delete_missing_out.stderr)
    );
    let absent_delete_missing_json: serde_json::Value =
        serde_json::from_slice(&absent_delete_missing_out.stdout).unwrap();
    assert_eq!(absent_delete_missing_json["disposition"], "NoChange");
    assert_eq!(absent_delete_missing_json["changed"], false);
    assert!(absent_delete_missing_json["invariant"]["target_span_before"].is_null());
    assert!(absent_delete_missing_json["invariant"]["target_span_after"].is_null());
    assert_eq!(
        absent_delete_missing_json["invariant"]["preserves_non_target_bytes"],
        true
    );
    assert_set_frontmatter_suffix_preserved(
        "absent state + delete missing field",
        &absent_delete_missing_json,
        &absent_delete_missing_source,
    );
    std::fs::remove_file(&absent_delete_missing).ok();
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
fn set_expect_etag_changed_frontmatter_replaces_exact_owned_boundary_without_extra_gap() {
    let yaml = temp_file("---\ntitle: Old\n---\n# Body\n");
    let yaml_etag = frontmatter_etag(&yaml);
    let yaml_out = md()
        .args([
            "set",
            "title",
            &yaml.to_string_lossy(),
            "New",
            "--json",
            "--expect-etag",
            &yaml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        yaml_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&yaml_out.stderr)
    );
    let yaml_json: serde_json::Value = serde_json::from_slice(&yaml_out.stdout).unwrap();
    assert_eq!(yaml_json["content"], "---\ntitle: New\n---\n# Body\n");

    let toml = temp_file("+++\ntitle = \"Old\"\n+++\n# Body\n");
    let toml_etag = frontmatter_etag(&toml);
    let toml_out = md()
        .args([
            "set",
            "title",
            &toml.to_string_lossy(),
            "New",
            "--json",
            "--expect-etag",
            &toml_etag,
        ])
        .output()
        .unwrap();
    assert!(
        toml_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&toml_out.stderr)
    );
    let toml_json: serde_json::Value = serde_json::from_slice(&toml_out.stdout).unwrap();
    assert_eq!(toml_json["content"], "+++\ntitle = \"New\"\n+++\n# Body\n");

    let crlf = temp_file("---\r\ntitle: Old\r\n---\r\n# Body\r\n");
    let crlf_etag = frontmatter_etag(&crlf);
    let crlf_out = md()
        .args([
            "set",
            "title",
            &crlf.to_string_lossy(),
            "New",
            "--json",
            "--expect-etag",
            &crlf_etag,
        ])
        .output()
        .unwrap();
    assert!(
        crlf_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&crlf_out.stderr)
    );
    let crlf_json: serde_json::Value = serde_json::from_slice(&crlf_out.stdout).unwrap();
    assert_eq!(crlf_json["content"], "---\ntitle: New\n---\n# Body\r\n");

    let yaml_with_blank_line = temp_file("---\ntitle: Old\n---\n\n# Body\n");
    let yaml_with_blank_line_etag = frontmatter_etag(&yaml_with_blank_line);
    let yaml_with_blank_line_out = md()
        .args([
            "set",
            "title",
            &yaml_with_blank_line.to_string_lossy(),
            "New",
            "--json",
            "--expect-etag",
            &yaml_with_blank_line_etag,
        ])
        .output()
        .unwrap();
    assert!(
        yaml_with_blank_line_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&yaml_with_blank_line_out.stderr)
    );
    let yaml_with_blank_line_json: serde_json::Value =
        serde_json::from_slice(&yaml_with_blank_line_out.stdout).unwrap();
    let content = yaml_with_blank_line_json["content"].as_str().unwrap();
    let serialized_frontmatter = "---\ntitle: New\n---\n";
    let suffix_after_owned_terminator = "\n# Body\n";
    assert_eq!(
        content,
        format!("{serialized_frontmatter}{suffix_after_owned_terminator}"),
        "intentional blank line after the closing delimiter terminator"
    );
    assert!(
        content.starts_with(serialized_frontmatter),
        "intentional blank line after the closing delimiter terminator"
    );
    assert_eq!(
        &content[serialized_frontmatter.len()..],
        suffix_after_owned_terminator,
        "intentional blank line after the closing delimiter terminator"
    );

    std::fs::remove_file(&yaml).ok();
    std::fs::remove_file(&toml).ok();
    std::fs::remove_file(&crlf).ok();
    std::fs::remove_file(&yaml_with_blank_line).ok();
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
    assert!(out.stdout.is_empty());
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
    assert!(out.stdout.is_empty());
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
    assert!(out.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), fresh);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_stale_delete_of_missing_key_conflict_preserves_bytes() {
    let tmp = temp_copy("set_basic.md");
    let etag = frontmatter_etag(&tmp);
    let stale = std::fs::read_to_string(&tmp).unwrap();
    let fresh = stale.replace("title: Original", "title: Drifted");
    std::fs::write(&tmp, &fresh).unwrap();

    let out = md()
        .args([
            "set",
            "--delete",
            "missing",
            &tmp.to_string_lossy(),
            "-i",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(out.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), fresh);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_expect_etag_stale_delete_of_missing_key_conflict_preserves_bytes_for_named_sources() {
    let cases = [
        (
            "YAML LF with body",
            "---\ntitle: Same\nauthor: Jane\n---\n# Body\n",
            "---\ntitle: Same\nauthor: Janet\n---\n# Body\n",
        ),
        (
            "YAML CRLF with body",
            "---\r\ntitle: Same\r\nauthor: Jane\r\n---\r\n# Body\r\n",
            "---\r\ntitle: Same\r\nauthor: Janet\r\n---\r\n# Body\r\n",
        ),
        (
            "YAML mixed frontmatter/body line endings",
            "---\r\ntitle: Same\r\nauthor: Jane\r\n---\r\n# Body\n",
            "---\r\ntitle: Same\r\nauthor: Janet\r\n---\r\n# Body\n",
        ),
        (
            "frontmatter-only with a final delimiter newline",
            "---\ntitle: Same\n---\n",
            "---\ntitle: Drifted\n---\n",
        ),
        (
            "frontmatter-only with the closing delimiter as the final byte",
            "---\ntitle: Same\n---",
            "---\ntitle: Drifted\n---",
        ),
        (
            "TOML with a final delimiter newline",
            "+++\ntitle = \"Same\"\n+++\n",
            "+++\ntitle = \"Drifted\"\n+++\n",
        ),
    ];

    for (label, original, drifted) in cases {
        assert_stale_missing_delete_conflict_preserves_bytes(label, original, drifted);
    }
}

#[test]
fn set_expect_etag_stale_semantic_validation_conflicts_before_parsing() {
    let cases = [
        (
            "malformed YAML",
            "---\ntitle: Stable\n---\n# Body\n",
            "---\ntitle: [unterminated\n---\n# Body\n",
        ),
        (
            "malformed TOML",
            "+++\ntitle = \"Stable\"\n+++\n# Body\n",
            "+++\ntitle = \"unterminated\n+++\n# Body\n",
        ),
        (
            "scalar YAML",
            "---\ntitle: Stable\n---\n# Body\n",
            "---\nhello\n---\n# Body\n",
        ),
        (
            "sequence YAML",
            "---\ntitle: Stable\n---\n# Body\n",
            "---\n- one\n- two\n---\n# Body\n",
        ),
    ];

    for (label, original, drifted) in cases {
        let tmp = temp_file(original);
        let etag = frontmatter_etag(&tmp);
        std::fs::write(&tmp, drifted).unwrap();

        let out = md()
            .args([
                "set",
                "title",
                &tmp.to_string_lossy(),
                "Guarded",
                "-i",
                "--json",
                "--expect-etag",
                &etag,
            ])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(4),
            "{label}: EtagMismatch exit code"
        );
        assert!(out.stdout.is_empty(), "{label}: stdout must stay empty");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("frontmatter etag mismatch"),
            "{label}: EtagMismatch diagnostic"
        );
        assert_eq!(
            std::fs::read(&tmp).unwrap(),
            drifted.as_bytes(),
            "{label}: file must stay byte-identical on EtagMismatch"
        );

        std::fs::remove_file(&tmp).ok();
    }
}

#[test]
fn set_expect_etag_absent_guard_turns_stale_after_intervening_creation() {
    let tmp = temp_copy("no_frontmatter.md");
    let absent_etag = frontmatter_etag(&tmp);

    let first = md()
        .args(["set", "title", &tmp.to_string_lossy(), "Created", "-i"])
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let after_creation = std::fs::read_to_string(&tmp).unwrap();
    let out = md()
        .args([
            "set",
            "title",
            &tmp.to_string_lossy(),
            "Guarded",
            "-i",
            "--expect-etag",
            &absent_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(out.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), after_creation);
    std::fs::remove_file(&tmp).ok();
}

// --- Error paths ---

#[test]
fn set_expect_etag_current_scalar_and_sequence_retain_mapping_errors() {
    let cases = [
        ("scalar YAML", "---\nhello\n---\n# Main\n"),
        ("sequence YAML", "---\n- one\n- two\n---\n# Main\n"),
    ];

    for (label, content) in cases {
        let tmp = temp_file(content);
        let etag = frontmatter_etag(&tmp);
        let out = md()
            .args([
                "set",
                "title",
                &tmp.to_string_lossy(),
                "New",
                "--expect-etag",
                &etag,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{label}: parse error exit code");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("mapping/object"),
            "{label}: mapping-root validation"
        );
        std::fs::remove_file(&tmp).ok();
    }
}

#[test]
fn set_expect_etag_current_malformed_yaml_and_toml_retain_parse_errors() {
    let cases = [
        (
            "malformed YAML",
            "---\ntitle: [unterminated\n---\n",
            "---\ntitle: [unterminated\n---\n# Main\n",
            "invalid YAML frontmatter",
        ),
        (
            "malformed TOML",
            "+++\ntitle = \"unterminated\n+++\n",
            "+++\ntitle = \"unterminated\n+++\n# Main\n",
            "invalid TOML frontmatter",
        ),
    ];

    for (label, raw_frontmatter, content, expected_message) in cases {
        let tmp = temp_file(content);
        let etag = raw_frontmatter_etag(Some(raw_frontmatter));
        let out = md()
            .args([
                "set",
                "title",
                &tmp.to_string_lossy(),
                "New",
                "--expect-etag",
                &etag,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{label}: parse error exit code");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(expected_message),
            "{label}: semantic parse error"
        );
        std::fs::remove_file(&tmp).ok();
    }
}

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
fn set_empty_frontmatter_block_allows_insertion() {
    let tmp = temp_file("---\n\n---\n# Main\n");
    let out = md()
        .args(["set", "title", &tmp.to_string_lossy(), "New", "--json"])
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
    assert!(content.starts_with("---\n"));
    assert!(content.contains("title: New"));
    assert!(content.contains("# Main"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn set_malformed_frontmatter_errors() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/malformed_frontmatter.md",
            "New",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("invalid YAML frontmatter"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn set_unclosed_frontmatter_errors() {
    let out = md()
        .args([
            "set",
            "title",
            "tests/fixtures/unclosed_frontmatter.md",
            "New",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("unclosed frontmatter"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
