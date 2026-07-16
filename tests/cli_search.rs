use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn write_temp_markdown(contents: &str) -> PathBuf {
    static TEMPFILE_COUNTER: AtomicU64 = AtomicU64::new(0);

    let unique = TEMPFILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "mdtools-cli-search-{}-{}.md",
        std::process::id(),
        unique
    ));
    std::fs::write(&path, contents).unwrap();
    path
}

fn run_search_json(query: &str, file: &std::path::Path, extra_args: &[&str]) -> serde_json::Value {
    let mut cmd = md();
    cmd.arg("search")
        .arg(query)
        .arg(file)
        .arg("--json")
        .args(extra_args);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn match_span_bytes(m: &serde_json::Value) -> (usize, usize) {
    let byte_start = m["match_span"]["byte_start"].as_u64().unwrap() as usize;
    let byte_end = m["match_span"]["byte_end"].as_u64().unwrap() as usize;
    (byte_start, byte_end)
}

#[test]
fn search_no_results() {
    let output = md()
        .args(["search", "zzzznotfound", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty());
}

#[test]
fn search_json() {
    let output = md()
        .args(["search", "method", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["query"], "method");
    assert_eq!(json["match_mode"], "Literal");
    assert!(json["matches"].as_array().unwrap().len() > 0);
}

#[test]
fn search_json_ignore_case() {
    let output = md()
        .args([
            "search",
            "METHOD",
            "tests/fixtures/basic.md",
            "--json",
            "--ignore-case",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["match_mode"], "LiteralIgnoreCase");
    assert!(json["matches"].as_array().unwrap().len() > 0);
}

#[test]
fn search_multiple_kind_filters() {
    let output = md()
        .args([
            "search",
            "method",
            "tests/fixtures/basic.md",
            "--kind",
            "heading",
            "--kind",
            "paragraph",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should find matches in both headings and paragraphs
    assert!(!stdout.is_empty());
}

#[test]
fn search_kind_filter_excludes() {
    // Search for "method" only in headings
    let heading_output = md()
        .args([
            "search",
            "method",
            "tests/fixtures/basic.md",
            "--kind",
            "heading",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(heading_output.status.success());
    let heading_json: serde_json::Value = serde_json::from_slice(&heading_output.stdout).unwrap();

    // Search for "method" only in paragraphs
    let para_output = md()
        .args([
            "search",
            "method",
            "tests/fixtures/basic.md",
            "--kind",
            "paragraph",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(para_output.status.success());
    let para_json: serde_json::Value = serde_json::from_slice(&para_output.stdout).unwrap();

    // All heading matches should have block_kind "Heading"
    for m in heading_json["matches"].as_array().unwrap() {
        assert_eq!(m["block_kind"], "Heading");
    }
    // All paragraph matches should have block_kind "Paragraph"
    for m in para_json["matches"].as_array().unwrap() {
        assert_eq!(m["block_kind"], "Paragraph");
    }
}

#[test]
fn search_match_span_is_exact() {
    let output = md()
        .args(["search", "positive", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    let m = &matches[0];
    // "positive" is 8 bytes
    let byte_start = m["match_span"]["byte_start"].as_u64().unwrap();
    let byte_end = m["match_span"]["byte_end"].as_u64().unwrap();
    assert_eq!(byte_end - byte_start, 8);
}

#[test]
fn search_empty_query() {
    let output = md()
        .args(["search", "", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["matches"].as_array().unwrap().len(), 0);
}

#[test]
fn search_ignore_case_unicode_expansion_maps_x_to_exact_source_slice() {
    let source = "alpha\n\nİX tail\n";
    let path = write_temp_markdown(source);

    let json = run_search_json("x", &path, &["--ignore-case"]);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);

    let (byte_start, byte_end) = match_span_bytes(&matches[0]);
    assert_eq!((byte_start, byte_end), (9, 10));
    assert_eq!(&source[byte_start..byte_end], "X");

    std::fs::remove_file(path).unwrap();
}

#[test]
fn search_ignore_case_unicode_expansion_maps_i_outward_to_whole_scalar() {
    let source = "alpha\n\nİX tail\n";
    let path = write_temp_markdown(source);

    let json = run_search_json("i", &path, &["--ignore-case"]);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 2);

    let first = &matches[0];
    let (byte_start, byte_end) = match_span_bytes(first);
    assert_eq!((byte_start, byte_end), (7, 9));
    assert_eq!(&source[byte_start..byte_end], "İ");

    std::fs::remove_file(path).unwrap();
}

#[test]
fn search_ignore_case_unicode_expansion_preserves_adjacent_multiscalar_slice() {
    let source = "alpha\n\nİX tail\n";
    let path = write_temp_markdown(source);

    let json = run_search_json("i\u{307}x ", &path, &["--ignore-case"]);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);

    let first = &matches[0];
    let (byte_start, byte_end) = match_span_bytes(first);
    assert_eq!((byte_start, byte_end), (7, 11));
    assert_eq!(&source[byte_start..byte_end], "İX ");

    std::fs::remove_file(path).unwrap();
}
