use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn tempfile(content: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_cli_read_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn outline_basic() {
    let output = md()
        .args(["outline", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Introduction\t1-1\tblock:0"));
    assert!(stdout.contains("## Methods\t5-5\tblock:2"));
    assert!(stdout.contains("### Sub-methods\t13-13\tblock:5"));
}

#[test]
fn outline_json() {
    let output = md()
        .args(["outline", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["entries"].as_array().unwrap().len(), 5);
    assert_eq!(json["entries"][0]["heading"]["text"], "Introduction");
    assert_eq!(json["entries"][0]["heading"]["level"], 1);
}

#[test]
fn blocks_basic() {
    let output = md()
        .args(["blocks", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\tHeading\t1-1\t# Introduction"));
    assert!(stdout.contains("1\tParagraph\t3-3\tThis is the opening paragraph."));
    assert!(stdout.contains("4\tList\t9-11\t"));
}

#[test]
fn block_read() {
    let output = md()
        .args(["block", "0", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "# Introduction");
}

#[test]
fn block_read_json() {
    let output = md()
        .args(["block", "0", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["block"]["kind"], "Heading");
    assert_eq!(json["content"], "# Introduction");
}

#[test]
fn block_out_of_range() {
    let output = md()
        .args(["block", "99", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("out of range"));
}

#[test]
fn section_basic() {
    let output = md()
        .args(["section", "Methods", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("## Methods"));
    assert!(stdout.contains("Method A"));
    assert!(stdout.contains("Sub-methods"));
}

#[test]
fn section_preamble() {
    let output = md()
        .args(["section", ":preamble", "tests/fixtures/preamble.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("preamble content"));
    assert!(stdout.contains("multiple paragraphs"));
    assert!(!stdout.contains("# First Heading"));
}

#[test]
fn section_duplicate_conflict() {
    let output = md()
        .args(["section", "Methods", "tests/fixtures/duplicate_headings.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--occurrence"));
}

#[test]
fn section_duplicate_occurrence() {
    let output = md()
        .args([
            "section",
            "Methods",
            "tests/fixtures/duplicate_headings.md",
            "--occurrence",
            "2",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Second methods section"));
    assert!(!stdout.contains("First methods section"));
}

#[test]
fn section_not_found() {
    let output = md()
        .args(["section", "Nonexistent", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn section_ignore_case() {
    let output = md()
        .args([
            "section",
            "methods",
            "tests/fixtures/basic.md",
            "--ignore-case",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("## Methods"));
}

#[test]
fn indented_atx_read_json_spans_include_indentation() {
    let path = tempfile("# Doc\n\n  ## A\nbody a\n\n   ## B\nbody b\n");
    let source = std::fs::read_to_string(&path).unwrap();

    let outline_output = md().args(["outline", &path, "--json"]).output().unwrap();
    assert!(outline_output.status.success());
    let outline_json: serde_json::Value = serde_json::from_slice(&outline_output.stdout).unwrap();
    let entries = outline_json["entries"].as_array().unwrap();

    let heading_start = entries[1]["heading"]["span"]["byte_start"]
        .as_u64()
        .unwrap() as usize;
    let heading_end = entries[1]["heading"]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[heading_start..heading_end], "  ## A");

    let section_output = md()
        .args(["section", "A", &path, "--json"])
        .output()
        .unwrap();
    assert!(section_output.status.success());
    let section_json: serde_json::Value = serde_json::from_slice(&section_output.stdout).unwrap();
    let section = &section_json["section"]["span"];
    let section_start = section["byte_start"].as_u64().unwrap() as usize;
    let section_end = section["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[section_start..section_end], "  ## A\nbody a\n\n");
    assert_eq!(section_json["content"], "  ## A\nbody a\n\n");

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn unicode_section_ignore_case_exact_non_ascii_read() {
    let path = tempfile("# Doc\n\n## CAFÉ\nbody\n");
    let output = md()
        .args(["section", "café", &path, "--ignore-case"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("## CAFÉ"));
    assert!(stdout.contains("body"));
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn unicode_section_ignore_case_duplicate_conflict_after_fold() {
    let path = tempfile("# Doc\n\n## CAFÉ\nfirst\n\n## Café\nsecond\n");
    let output = md()
        .args(["section", "café", &path, "--ignore-case"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--occurrence"));
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn unicode_section_ignore_case_occurrence_selects_folded_duplicate() {
    let path = tempfile("# Doc\n\n## CAFÉ\nfirst\n\n## Café\nsecond\n");
    let output = md()
        .args([
            "section",
            "café",
            &path,
            "--ignore-case",
            "--occurrence",
            "2",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("second"));
    assert!(!stdout.contains("first"));
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn unicode_section_ignore_case_preserves_no_normalization_boundary() {
    let path = tempfile("# Doc\n\n## Café\nbody\n");
    let output = md()
        .args(["section", "Cafe\u{301}", &path, "--ignore-case"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn section_defaults_to_exact_matching() {
    let output = md()
        .args(["section", "method", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn section_contains_duplicate_conflict_without_occurrence() {
    let output = md()
        .args([
            "section",
            "method",
            "tests/fixtures/basic.md",
            "--contains",
            "--ignore-case",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--occurrence"));
}

#[test]
fn section_contains_case_sensitive_success() {
    let output = md()
        .args(["section", "Method", "tests/fixtures/basic.md", "--contains"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("## Methods"));
    assert!(stdout.contains("Method A"));
    assert!(stdout.contains("### Sub-methods"));
}

#[test]
fn section_contains_case_sensitive_zero_match_exits_one() {
    let output = md()
        .args(["section", "METHOD", "tests/fixtures/basic.md", "--contains"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("heading not found: METHOD"));
}

#[test]
fn section_contains_json_roundtrips_match_mode() {
    let output = md()
        .args([
            "section",
            "Method",
            "tests/fixtures/basic.md",
            "--contains",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["section"]["selector"]["match_mode"], "Contains");
    assert_eq!(json["section"]["heading"]["text"], "Methods");
    assert!(json["content"].as_str().unwrap().starts_with("## Methods"));
}

#[test]
fn section_contains_ignores_nested_non_top_level_headings() {
    let path = tempfile(
        "# Top Methods\nbody\n\n> ## Hidden Methods\n> quoted body\n\n```md\n## Code Methods\n```\n",
    );
    let output = md()
        .args(["section", "method", &path, "--contains", "--ignore-case"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("# Top Methods"));
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn section_contains_rejects_preamble_selector() {
    let output = md()
        .args([
            "section",
            ":preamble",
            "tests/fixtures/preamble.md",
            "--contains",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--contains"));
    assert!(stderr.contains(":preamble"));
}

#[test]
fn section_contains_rejects_empty_selector() {
    let output = md()
        .args(["section", "", "tests/fixtures/table.md", "--contains"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty selector"));
    assert!(stderr.contains("--contains"));
}

#[test]
fn search_basic() {
    let output = md()
        .args(["search", "method", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Paragraph"));
}

#[test]
fn search_kind_filter() {
    let output = md()
        .args([
            "search",
            "method",
            "tests/fixtures/basic.md",
            "--kind",
            "paragraph",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should only find matches in paragraphs, not headings
    assert!(!stdout.contains("Heading"));
}

#[test]
fn search_ignore_case() {
    let output = md()
        .args([
            "search",
            "METHOD",
            "tests/fixtures/basic.md",
            "--ignore-case",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn links_basic() {
    let output = md()
        .args(["links", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://example.com"));
    assert!(stdout.contains("Inline"));
}

#[test]
fn frontmatter_present() {
    let output = md()
        .args(["frontmatter", "tests/fixtures/frontmatter.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["present"], true);
    assert_eq!(json["frontmatter"]["format"], "Yaml");
    assert_eq!(json["frontmatter"]["data"]["title"], "Test Document");
}

#[test]
fn frontmatter_absent() {
    let output = md()
        .args(["frontmatter", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["present"], false);
    assert!(json["frontmatter"].is_null());
}

#[test]
fn stats_basic() {
    let output = md()
        .args(["stats", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("headings=5"));
    assert!(stdout.contains("blocks=11"));
    assert!(stdout.contains("links=1"));
}

#[test]
fn stats_empty() {
    let output = md().args(["stats", "/dev/null"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("words=0"));
    assert!(stdout.contains("headings=0"));
    assert!(stdout.contains("blocks=0"));
    assert!(stdout.contains("lines=1"));
}

#[test]
fn missing_file() {
    let output = md()
        .args(["outline", "/tmp/nonexistent_file_abc123.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}

// --- Edge case fixtures ---

#[test]
fn setext_headings() {
    let output = md()
        .args(["outline", "tests/fixtures/setext.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Title"));
    assert!(stdout.contains("## Subtitle"));
}

#[test]
fn setext_blocks_span_two_lines() {
    let output = md()
        .args(["blocks", "tests/fixtures/setext.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Heading\t1-2"));
    assert!(stdout.contains("Heading\t4-5"));
}

#[test]
fn fenced_code_no_inner_headings() {
    let output = md()
        .args(["outline", "tests/fixtures/fenced_code.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("block:").count(), 1);
    assert!(stdout.contains("Real Heading"));
}

#[test]
fn fenced_code_no_inner_links() {
    let output = md()
        .args(["links", "tests/fixtures/fenced_code.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://example.com"));
    assert!(!stdout.contains("http://fake.com"));
}

#[test]
fn crlf_document_blocks() {
    let output = md()
        .args(["blocks", "tests/fixtures/crlf.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Heading"));
    assert!(stdout.contains("Paragraph"));
}

#[test]
fn crlf_stats() {
    let output = md()
        .args(["stats", "tests/fixtures/crlf.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["stats"]["heading_count"], 1);
    assert_eq!(json["stats"]["block_count"], 3);
}

#[test]
fn frontmatter_only_empty_blocks() {
    let output = md()
        .args(["stats", "tests/fixtures/frontmatter_only.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("blocks=0"));
    assert!(stdout.contains("headings=0"));
}

#[test]
fn frontmatter_only_empty_preamble() {
    let output = md()
        .args(["section", ":preamble", "tests/fixtures/frontmatter_only.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty());
}

#[test]
fn indented_code_block_kind() {
    let output = md()
        .args(["blocks", "tests/fixtures/indented_code.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let blocks = json["blocks"].as_array().unwrap();
    assert!(blocks.iter().any(|b| b["kind"] == "IndentedCode"));
}

#[test]
fn html_block_kind() {
    let output = md()
        .args(["blocks", "tests/fixtures/html_block.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HtmlBlock"));
}

#[test]
fn thematic_break_kind() {
    let output = md()
        .args(["blocks", "tests/fixtures/thematic_break.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ThematicBreak"));
}

#[test]
fn malformed_frontmatter_exit_code() {
    let output = md()
        .args(["frontmatter", "tests/fixtures/malformed_frontmatter.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn malformed_frontmatter_blocks_fallback() {
    let output = md()
        .args(["blocks", "tests/fixtures/malformed_frontmatter.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ThematicBreak"));
}

#[test]
fn unclosed_frontmatter_not_present() {
    let output = md()
        .args(["frontmatter", "tests/fixtures/unclosed_frontmatter.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["present"], false);
}

#[test]
fn toml_frontmatter_parsed() {
    let output = md()
        .args(["frontmatter", "tests/fixtures/toml_frontmatter.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["present"], true);
    assert_eq!(json["frontmatter"]["format"], "Toml");
    assert_eq!(json["frontmatter"]["data"]["title"], "TOML Doc");
}

#[test]
fn link_kind_detection() {
    let output = md()
        .args(["links", "tests/fixtures/reference_links.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json["links"].as_array().unwrap();
    let kinds: Vec<&str> = links.iter().map(|l| l["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"Inline"));
    assert!(kinds.contains(&"Reference"));
    assert!(kinds.contains(&"Autolink"));
}
