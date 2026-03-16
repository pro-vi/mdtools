//! Contract-level tests: verify byte spans, mutation invariants, and spec rules.
//! These tests slice source documents at reported byte offsets and assert exact content.

use std::io::Write;
use std::process::{Command, Stdio};

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

// ============================================================
// BYTE-SPAN ACCURACY: slice source at reported spans, verify content
// ============================================================

#[test]
fn block_spans_match_source_content() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let output = md()
        .args(["blocks", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    for block in json["blocks"].as_array().unwrap() {
        let bs = block["span"]["byte_start"].as_u64().unwrap() as usize;
        let be = block["span"]["byte_end"].as_u64().unwrap() as usize;
        let kind = block["kind"].as_str().unwrap();
        let sliced = &source[bs..be];

        // Every sliced span must be non-empty (except ThematicBreak which is "---")
        assert!(
            !sliced.is_empty(),
            "block {} ({}) has empty span [{}, {})",
            block["index"], kind, bs, be
        );

        // Heading blocks must start with # or be setext (contain ==/--  on second line)
        if kind == "Heading" {
            assert!(
                sliced.starts_with('#') || sliced.contains("\n=") || sliced.contains("\n-"),
                "Heading block at [{},{}): {:?} doesn't look like a heading",
                bs, be, sliced
            );
        }
    }
}

#[test]
fn outline_heading_spans_match_source() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let output = md()
        .args(["outline", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    for entry in json["entries"].as_array().unwrap() {
        let heading = &entry["heading"];
        let text = heading["text"].as_str().unwrap();
        let bs = heading["span"]["byte_start"].as_u64().unwrap() as usize;
        let be = heading["span"]["byte_end"].as_u64().unwrap() as usize;
        let sliced = &source[bs..be];

        // The heading text must appear in the sliced span
        assert!(
            sliced.contains(text),
            "heading {:?} not found in span [{},{}): {:?}",
            text, bs, be, sliced
        );
    }
}

#[test]
fn section_span_matches_source() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let output = md()
        .args(["section", "Methods", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let content = json["content"].as_str().unwrap();
    let bs = json["section"]["span"]["byte_start"].as_u64().unwrap() as usize;
    let be = json["section"]["span"]["byte_end"].as_u64().unwrap() as usize;
    let sliced = &source[bs..be];

    assert_eq!(
        content, sliced,
        "section content doesn't match source slice"
    );
}

#[test]
fn block_read_content_matches_span_slice() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();

    for idx in 0..5 {
        let output = md()
            .args(["block", &idx.to_string(), "tests/fixtures/basic.md", "--json"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let content = json["content"].as_str().unwrap();
        let bs = json["block"]["span"]["byte_start"].as_u64().unwrap() as usize;
        let be = json["block"]["span"]["byte_end"].as_u64().unwrap() as usize;
        let sliced = &source[bs..be];

        assert_eq!(
            content, sliced,
            "block {} content != source[{}..{}]",
            idx, bs, be
        );
    }
}

#[test]
fn search_match_span_slices_to_query() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let output = md()
        .args(["search", "positive", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    for m in json["matches"].as_array().unwrap() {
        let bs = m["match_span"]["byte_start"].as_u64().unwrap() as usize;
        let be = m["match_span"]["byte_end"].as_u64().unwrap() as usize;
        let sliced = &source[bs..be];
        assert_eq!(
            sliced, "positive",
            "search match span [{},{}): {:?} != \"positive\"",
            bs, be, sliced
        );
    }
}

#[test]
fn link_spans_match_source() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let output = md()
        .args(["links", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    for link in json["links"].as_array().unwrap() {
        let bs = link["span"]["byte_start"].as_u64().unwrap() as usize;
        let be = link["span"]["byte_end"].as_u64().unwrap() as usize;
        let sliced = &source[bs..be];
        let dest = link["destination"].as_str().unwrap_or("");

        // The link span must contain the destination URL
        assert!(
            sliced.contains(dest) || dest.is_empty(),
            "link span [{},{}): {:?} doesn't contain destination {:?}",
            bs, be, sliced, dest
        );
    }
}

// ============================================================
// UTF-8 MULTI-BYTE: verify spans handle non-ASCII correctly
// ============================================================

#[test]
fn utf8_block_spans_are_byte_accurate() {
    let source = std::fs::read_to_string("tests/fixtures/utf8.md").unwrap();
    let output = md()
        .args(["blocks", "tests/fixtures/utf8.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    for block in json["blocks"].as_array().unwrap() {
        let bs = block["span"]["byte_start"].as_u64().unwrap() as usize;
        let be = block["span"]["byte_end"].as_u64().unwrap() as usize;

        // Verify byte offsets are valid UTF-8 boundaries
        assert!(
            source.is_char_boundary(bs),
            "block {} byte_start {} is not a char boundary",
            block["index"], bs
        );
        assert!(
            source.is_char_boundary(be),
            "block {} byte_end {} is not a char boundary",
            block["index"], be
        );

        let sliced = &source[bs..be];
        assert!(!sliced.is_empty(), "block {} has empty span", block["index"]);
    }
}

#[test]
fn utf8_search_spans_are_byte_accurate() {
    let source = std::fs::read_to_string("tests/fixtures/utf8.md").unwrap();
    let output = md()
        .args(["search", "résumé", "tests/fixtures/utf8.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty(), "should find 'résumé' in utf8 doc");

    for m in matches {
        let bs = m["match_span"]["byte_start"].as_u64().unwrap() as usize;
        let be = m["match_span"]["byte_end"].as_u64().unwrap() as usize;
        assert!(source.is_char_boundary(bs));
        assert!(source.is_char_boundary(be));
        let sliced = &source[bs..be];
        assert_eq!(sliced, "résumé", "UTF-8 search span mismatch: {:?}", sliced);
    }
}

#[test]
fn utf8_heading_text_is_correct() {
    let output = md()
        .args(["outline", "tests/fixtures/utf8.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let texts: Vec<&str> = json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();
    assert!(texts.contains(&"Héllo Wörld"));
    assert!(texts.contains(&"Ünïcödé"));
}

// ============================================================
// MUTATION INVARIANTS: span nullability rules
// ============================================================

#[test]
fn mutation_replaced_both_spans_present() {
    let output = md_with_stdin(
        &["replace-block", "1", "tests/fixtures/basic.md", "--json"],
        "Replaced paragraph.",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");
    assert!(json["invariant"]["target_span_before"].is_object());
    assert!(json["invariant"]["target_span_after"].is_object());

    // Spans must differ (content changed)
    let before_end = json["invariant"]["target_span_before"]["byte_end"].as_u64();
    let after_end = json["invariant"]["target_span_after"]["byte_end"].as_u64();
    assert_ne!(before_end, after_end, "replaced spans should differ in byte_end");
}

#[test]
fn mutation_deleted_span_after_is_null() {
    let output = md()
        .args(["delete-block", "1", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Deleted");
    assert!(json["invariant"]["target_span_before"].is_object());
    assert!(json["invariant"]["target_span_after"].is_null());
}

#[test]
fn mutation_nochange_spans_identical() {
    // Replace block with its own content → NoChange
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    // Get block 1 content
    let block_output = md()
        .args(["block", "1", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    let original_content = String::from_utf8_lossy(&block_output.stdout).to_string();

    let output = md_with_stdin(
        &["replace-block", "1", "tests/fixtures/basic.md", "--json"],
        &original_content,
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert_eq!(
        json["invariant"]["target_span_before"],
        json["invariant"]["target_span_after"],
        "NoChange spans must be identical"
    );
}

#[test]
fn mutation_preserves_non_target_bytes() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let output = md_with_stdin(
        &["replace-block", "1", "tests/fixtures/basic.md", "--json"],
        "REPLACED.",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["invariant"]["preserves_non_target_bytes"], true);

    // Actually verify: content before block 1 and after block 1 must be unchanged
    let content = json["content"].as_str().unwrap();
    let bs = json["target"]["Block"]["span"]["byte_start"].as_u64().unwrap() as usize;
    let be = json["target"]["Block"]["span"]["byte_end"].as_u64().unwrap() as usize;

    // Source bytes before the target span must be identical
    assert_eq!(&content[..bs], &source[..bs], "bytes before target changed");
    // Source bytes after the target span must be identical
    assert_eq!(&content[content.len() - (source.len() - be)..], &source[be..], "bytes after target changed");
}

// ============================================================
// EMPTY STDIN: replace with empty → Deleted disposition
// ============================================================

#[test]
fn replace_block_empty_stdin_yields_deleted() {
    let output = md_with_stdin(
        &["replace-block", "1", "tests/fixtures/basic.md", "--json"],
        "",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Deleted");
    assert_eq!(json["changed"], true);
}

#[test]
fn replace_section_empty_stdin_yields_deleted() {
    let output = md_with_stdin(
        &["replace-section", "Discussion", "tests/fixtures/basic.md", "--json"],
        "",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Deleted");
}

// ============================================================
// CRLF MUTATIONS: line ending normalization
// ============================================================

#[test]
fn crlf_mutation_normalizes_line_endings() {
    let tmp = tempfile_bytes(b"# Hello\r\n\r\nOriginal.\r\n");
    let output = md_with_stdin(
        &["replace-block", "1", &tmp, "--json"],
        "Line one\nLine two\n",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let content = json["content"].as_str().unwrap();

    // Inserted content should be normalized to CRLF (document is uniformly CRLF)
    assert!(
        content.contains("Line one\r\nLine two\r\n"),
        "CRLF normalization failed: content = {:?}",
        content
    );
    assert_eq!(json["line_endings"], "Crlf");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn crlf_stats_line_endings_detected() {
    let output = md()
        .args(["blocks", "tests/fixtures/crlf.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    // Verify blocks parse correctly even with CRLF
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["blocks"].as_array().unwrap().len() >= 2);
}

// ============================================================
// SEARCH IN CODE BLOCKS
// ============================================================

#[test]
fn search_inside_code_fence() {
    let output = md()
        .args([
            "search", "method", "tests/fixtures/search_in_code.md",
            "--kind", "code-fence", "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty(), "should find 'method' inside code fence");
    for m in matches {
        assert_eq!(m["block_kind"], "CodeFence");
    }
}

#[test]
fn search_inside_indented_code() {
    let output = md()
        .args([
            "search", "method", "tests/fixtures/search_in_code.md",
            "--kind", "indented-code", "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty(), "should find 'method' inside indented code");
    for m in matches {
        assert_eq!(m["block_kind"], "IndentedCode");
    }
}

#[test]
fn search_excludes_code_by_default_when_filtered() {
    // Search only paragraphs — should not find code block matches
    let output = md()
        .args([
            "search", "method", "tests/fixtures/search_in_code.md",
            "--kind", "paragraph", "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for m in json["matches"].as_array().unwrap() {
        assert_eq!(m["block_kind"], "Paragraph");
    }
}

// ============================================================
// STATS CONTRACT VERIFICATION
// ============================================================

#[test]
fn stats_section_count_includes_preamble() {
    let output = md()
        .args(["stats", "tests/fixtures/preamble.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // preamble.md has: preamble (2 paragraphs) + 1 heading = 2 sections
    assert_eq!(
        json["stats"]["section_count"], 2,
        "section_count should include non-empty preamble"
    );
}

#[test]
fn stats_section_count_excludes_empty_preamble() {
    let output = md()
        .args(["stats", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // basic.md starts with # Introduction — no preamble
    // 5 headings = 5 sections (no preamble)
    assert_eq!(json["stats"]["section_count"], 5);
}

#[test]
fn stats_word_count_excludes_code() {
    let output = md()
        .args(["stats", "tests/fixtures/search_in_code.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wc = json["stats"]["word_count"].as_u64().unwrap();
    // "Title" (1) + "The method is important." (4) + "Another paragraph mentioning method." (4) = 9
    // Code blocks should NOT contribute to word count
    assert_eq!(wc, 9, "word_count should exclude code block contents");
}

#[test]
fn stats_line_count_no_trailing_newline() {
    // A file with no trailing newline: last line still counts
    let tmp = tempfile_str("line1\nline2\nline3");
    let output = md()
        .args(["stats", &tmp, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["stats"]["line_count"], 3);
    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// ADDITIONAL BLOCK KIND COVERAGE
// ============================================================

#[test]
fn footnote_definition_block() {
    let output = md()
        .args(["blocks", "tests/fixtures/footnote.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let kinds: Vec<&str> = json["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"FootnoteDefinition"), "expected FootnoteDefinition block");
}

#[test]
fn table_block() {
    let output = md()
        .args(["blocks", "tests/fixtures/table.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let kinds: Vec<&str> = json["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"Table"), "expected Table block");
}

#[test]
fn table_block_span_is_byte_accurate() {
    let source = std::fs::read_to_string("tests/fixtures/table.md").unwrap();
    let output = md()
        .args(["blocks", "tests/fixtures/table.md", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let table = json["blocks"].as_array().unwrap()
        .iter()
        .find(|b| b["kind"] == "Table")
        .unwrap();
    let bs = table["span"]["byte_start"].as_u64().unwrap() as usize;
    let be = table["span"]["byte_end"].as_u64().unwrap() as usize;
    let sliced = &source[bs..be];
    assert!(sliced.contains("| Name"), "table span should contain header row");
    assert!(sliced.contains("| Alpha"), "table span should contain data rows");
}

// ============================================================
// TEXT OUTPUT FORMAT COMPLIANCE
// ============================================================

#[test]
fn blocks_text_format_is_tab_separated() {
    let output = md()
        .args(["blocks", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "blocks text line should have 4 tab-separated fields: {:?}", line);
    }
}

#[test]
fn outline_text_format_is_tab_separated() {
    let output = md()
        .args(["outline", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 3, "outline text line should have 3 tab-separated fields: {:?}", line);
    }
}

#[test]
fn search_text_format_is_tab_separated() {
    let output = md()
        .args(["search", "method", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "search text line should have 4 tab-separated fields: {:?}", line);
    }
}

#[test]
fn links_text_format_is_tab_separated() {
    let output = md()
        .args(["links", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "links text line should have 4 tab-separated fields: {:?}", line);
    }
}

#[test]
fn stats_text_format_key_equals_value() {
    let output = md()
        .args(["stats", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        assert!(
            line.contains('='),
            "stats line should be key=value: {:?}",
            line
        );
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        assert_eq!(parts.len(), 2);
        assert!(
            parts[1].parse::<u32>().is_ok(),
            "stats value should be numeric: {:?}",
            line
        );
    }
    // Verify all required keys present
    let keys: Vec<&str> = stdout.lines().map(|l| l.split('=').next().unwrap()).collect();
    for required in &["words", "headings", "blocks", "links", "sections", "lines"] {
        assert!(keys.contains(required), "missing stats key: {}", required);
    }
}

// ============================================================
// JSON SCHEMA VERSION
// ============================================================

#[test]
fn all_json_commands_have_schema_version() {
    let commands: Vec<Vec<&str>> = vec![
        vec!["outline", "tests/fixtures/basic.md", "--json"],
        vec!["blocks", "tests/fixtures/basic.md", "--json"],
        vec!["block", "0", "tests/fixtures/basic.md", "--json"],
        vec!["section", "Methods", "tests/fixtures/basic.md", "--json"],
        vec!["search", "method", "tests/fixtures/basic.md", "--json"],
        vec!["links", "tests/fixtures/basic.md", "--json"],
        vec!["frontmatter", "tests/fixtures/basic.md"],
        vec!["stats", "tests/fixtures/basic.md", "--json"],
    ];

    for args in &commands {
        let output = md().args(args).output().unwrap();
        assert!(
            output.status.success(),
            "command {:?} failed",
            args
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            json["schema_version"], "mdtools.v1",
            "schema_version missing for {:?}",
            args
        );
    }
}

// ============================================================
// ERROR PATH COVERAGE
// ============================================================

#[test]
fn error_stderr_is_single_line() {
    let output = md()
        .args(["block", "99", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "stderr should be exactly one line: {:?}", stderr);
}

#[test]
fn error_exits_are_correct() {
    // NotFound = 1
    let o = md().args(["section", "Nonexistent", "tests/fixtures/basic.md"]).output().unwrap();
    assert_eq!(o.status.code(), Some(1));

    // NotFound = 1 (block out of range)
    let o = md().args(["block", "99", "tests/fixtures/basic.md"]).output().unwrap();
    assert_eq!(o.status.code(), Some(1));

    // NotFound = 1 (missing file)
    let o = md().args(["blocks", "/tmp/nonexistent_mdtools_xyz.md"]).output().unwrap();
    assert_eq!(o.status.code(), Some(1));

    // ParseError = 2 (malformed frontmatter)
    let o = md().args(["frontmatter", "tests/fixtures/malformed_frontmatter.md"]).output().unwrap();
    assert_eq!(o.status.code(), Some(2));

    // InvalidInput = 3 (no insert location)
    let o = md_with_stdin(&["insert-block", "tests/fixtures/basic.md"], "x");
    assert_eq!(o.status.code(), Some(3));

    // Conflict = 4 (duplicate heading)
    let o = md().args(["section", "Methods", "tests/fixtures/duplicate_headings.md"]).output().unwrap();
    assert_eq!(o.status.code(), Some(4));
}

// ============================================================
// HELPERS
// ============================================================

fn tempfile_str(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_contract_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}

fn tempfile_bytes(content: &[u8]) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_contract_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}
