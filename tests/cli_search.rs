use std::process::Command;

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
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
        .args([
            "search",
            "method",
            "tests/fixtures/basic.md",
            "--json",
        ])
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
    let heading_json: serde_json::Value =
        serde_json::from_slice(&heading_output.stdout).unwrap();

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
        .args([
            "search",
            "positive",
            "tests/fixtures/basic.md",
            "--json",
        ])
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
