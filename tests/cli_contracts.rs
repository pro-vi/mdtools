//! Contract-level tests: verify byte spans, mutation invariants, and spec rules.
//! These tests slice source documents at reported byte offsets and assert exact content.

use serde::Deserialize;
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

fn md_help(subcommand: &str) -> String {
    let output = md().args([subcommand, "--help"]).output().unwrap();
    assert!(output.status.success(), "{} --help failed", subcommand);
    String::from_utf8(output.stdout).unwrap()
}

#[derive(Deserialize)]
struct MdInventoryFile {
    schema_version: u32,
    commands: Vec<MdInventoryEntry>,
}

#[derive(Deserialize)]
struct MdInventoryEntry {
    name: String,
}

fn inventory_commands() -> Vec<String> {
    let inventory: MdInventoryFile =
        serde_json::from_str(&std::fs::read_to_string("bench/md_inventory_v1.json").unwrap())
            .unwrap();
    assert_eq!(inventory.schema_version, 1);
    inventory
        .commands
        .into_iter()
        .map(|entry| entry.name)
        .collect()
}

fn top_level_help_commands(help: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_commands = false;

    for line in help.lines() {
        let trimmed = line.trim();
        if trimmed == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if trimmed == "Options:" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        let command = trimmed.split_whitespace().next().unwrap();
        if command != "help" {
            commands.push(command.to_string());
        }
    }

    commands
}

#[test]
fn top_level_help_lists_collect_command() {
    let output = md().arg("--help").output().unwrap();
    assert!(output.status.success(), "md --help failed");
    let help = String::from_utf8(output.stdout).unwrap();
    let normalized_help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalized_help.contains("Commands: outline")
            && normalized_help.contains("frontmatter")
            && normalized_help.contains("collect")
            && normalized_help.contains("stats"),
        "top-level help changed unexpectedly:\n{}",
        help
    );
}

#[test]
fn top_level_help_matches_shared_inventory() {
    let output = md().arg("--help").output().unwrap();
    assert!(output.status.success(), "md --help failed");
    let help = String::from_utf8(output.stdout).unwrap();
    assert_eq!(top_level_help_commands(&help), inventory_commands());
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
            block["index"],
            kind,
            bs,
            be
        );

        // Heading blocks must start with # or be setext (contain ==/--  on second line)
        if kind == "Heading" {
            assert!(
                sliced.starts_with('#') || sliced.contains("\n=") || sliced.contains("\n-"),
                "Heading block at [{},{}): {:?} doesn't look like a heading",
                bs,
                be,
                sliced
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
            text,
            bs,
            be,
            sliced
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
fn indented_atx_block_heading_section_spans_include_indentation() {
    let source = "# Doc\n\n  ## A\nbody a\n\n   ## B\nbody b\n";
    let path = tempfile_str(source);

    let blocks_output = md().args(["blocks", &path, "--json"]).output().unwrap();
    assert!(blocks_output.status.success());
    let blocks_json: serde_json::Value = serde_json::from_slice(&blocks_output.stdout).unwrap();
    let blocks = blocks_json["blocks"].as_array().unwrap();

    let a_block_start = blocks[1]["span"]["byte_start"].as_u64().unwrap() as usize;
    let a_block_end = blocks[1]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[a_block_start..a_block_end], "  ## A");

    let b_block_start = blocks[3]["span"]["byte_start"].as_u64().unwrap() as usize;
    let b_block_end = blocks[3]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[b_block_start..b_block_end], "   ## B");

    let outline_output = md().args(["outline", &path, "--json"]).output().unwrap();
    assert!(outline_output.status.success());
    let outline_json: serde_json::Value = serde_json::from_slice(&outline_output.stdout).unwrap();
    let entries = outline_json["entries"].as_array().unwrap();

    let a_heading_start = entries[1]["heading"]["span"]["byte_start"]
        .as_u64()
        .unwrap() as usize;
    let a_heading_end = entries[1]["heading"]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[a_heading_start..a_heading_end], "  ## A");

    let a_section_start = entries[1]["section_span"]["byte_start"].as_u64().unwrap() as usize;
    let a_section_end = entries[1]["section_span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &source[a_section_start..a_section_end],
        "  ## A\nbody a\n\n"
    );

    let b_section_start = entries[2]["section_span"]["byte_start"].as_u64().unwrap() as usize;
    let b_section_end = entries[2]["section_span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[b_section_start..b_section_end], "   ## B\nbody b\n");

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn indented_atx_etag_changes_when_legal_indentation_changes() {
    let two_space = tempfile_str("# Doc\n\n  ## A\nbody a\n");
    let three_space = tempfile_str("# Doc\n\n   ## A\nbody a\n");

    let etag_for = |path: &str| -> String {
        let output = md()
            .args(["section", "A", path, "--json"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        json["section"]["etag"].as_str().unwrap().to_string()
    };

    assert_ne!(etag_for(&two_space), etag_for(&three_space));

    std::fs::remove_file(&two_space).unwrap();
    std::fs::remove_file(&three_space).unwrap();
}

#[test]
fn indented_atx_prior_section_ends_before_following_heading_indentation() {
    let path = tempfile_str("# Doc\n\n## A\nbody a\n\n  ## B\nbody b\n");
    let output = md()
        .args(["section", "A", &path, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["content"], "## A\nbody a\n\n");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn indented_atx_marker_span_stays_on_hash_run() {
    let path = tempfile_str("# Doc\n\n  ### Source\nbody s\n\n## Dest\nbody d\n");
    let output = md()
        .args(["move-section", "Source", &path, "--after", "Dest"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let moved_heading = stdout.lines().find(|line| line.contains("Source")).unwrap();
    assert_eq!(moved_heading, "  ## Source");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn indented_atx_crlf_projection_is_byte_accurate() {
    let source = "# Doc\r\n\r\n  ## A\r\nbody a\r\n\r\n   ## B\r\nbody b\r\n";
    let path = tempfile_bytes(source.as_bytes());

    let blocks_output = md().args(["blocks", &path, "--json"]).output().unwrap();
    assert!(blocks_output.status.success());
    let blocks_json: serde_json::Value = serde_json::from_slice(&blocks_output.stdout).unwrap();
    let blocks = blocks_json["blocks"].as_array().unwrap();
    let a_block_start = blocks[1]["span"]["byte_start"].as_u64().unwrap() as usize;
    let a_block_end = blocks[1]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[a_block_start..a_block_end], "  ## A");

    let section_output = md()
        .args(["section", "A", &path, "--json"])
        .output()
        .unwrap();
    assert!(section_output.status.success());
    let section_json: serde_json::Value = serde_json::from_slice(&section_output.stdout).unwrap();
    assert_eq!(section_json["content"], "  ## A\r\nbody a\r\n\r\n");

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn indented_setext_block_heading_section_spans_include_indentation() {
    let source = "# Doc\n\n  A Title\n  --------\nbody a\n\n   B Title\n   --------\nbody b\n";
    let path = tempfile_str(source);

    let blocks_output = md().args(["blocks", &path, "--json"]).output().unwrap();
    assert!(blocks_output.status.success());
    let blocks_json: serde_json::Value = serde_json::from_slice(&blocks_output.stdout).unwrap();
    let blocks = blocks_json["blocks"].as_array().unwrap();

    let a_block_start = blocks[1]["span"]["byte_start"].as_u64().unwrap() as usize;
    let a_block_end = blocks[1]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[a_block_start..a_block_end], "  A Title\n  --------");

    let b_block_start = blocks[3]["span"]["byte_start"].as_u64().unwrap() as usize;
    let b_block_end = blocks[3]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &source[b_block_start..b_block_end],
        "   B Title\n   --------"
    );

    let outline_output = md().args(["outline", &path, "--json"]).output().unwrap();
    assert!(outline_output.status.success());
    let outline_json: serde_json::Value = serde_json::from_slice(&outline_output.stdout).unwrap();
    let entries = outline_json["entries"].as_array().unwrap();

    let a_heading_start = entries[1]["heading"]["span"]["byte_start"]
        .as_u64()
        .unwrap() as usize;
    let a_heading_end = entries[1]["heading"]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &source[a_heading_start..a_heading_end],
        "  A Title\n  --------"
    );

    let a_section_start = entries[1]["section_span"]["byte_start"].as_u64().unwrap() as usize;
    let a_section_end = entries[1]["section_span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &source[a_section_start..a_section_end],
        "  A Title\n  --------\nbody a\n\n"
    );

    let b_section_start = entries[2]["section_span"]["byte_start"].as_u64().unwrap() as usize;
    let b_section_end = entries[2]["section_span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &source[b_section_start..b_section_end],
        "   B Title\n   --------\nbody b\n"
    );

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn indented_setext_etag_changes_when_legal_indentation_changes() {
    let two_space = tempfile_str("# Doc\n\n  A Title\n  --------\nbody a\n");
    let three_space = tempfile_str("# Doc\n\n   A Title\n   --------\nbody a\n");

    let etag_for = |path: &str| -> String {
        let output = md()
            .args(["section", "A Title", path, "--json"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        json["section"]["etag"].as_str().unwrap().to_string()
    };

    assert_ne!(etag_for(&two_space), etag_for(&three_space));

    std::fs::remove_file(&two_space).unwrap();
    std::fs::remove_file(&three_space).unwrap();
}

#[test]
fn indented_setext_prior_section_ends_before_following_heading_indentation() {
    let path = tempfile_str("# Doc\n\n## A\nbody a\n\n  B Title\n  --------\nbody b\n");
    let output = md()
        .args(["section", "A", &path, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["content"], "## A\nbody a\n\n");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn indented_atx_setext_and_indented_code_projection_is_unchanged() {
    let setext_source = std::fs::read_to_string("tests/fixtures/setext.md").unwrap();
    let setext_output = md()
        .args(["blocks", "tests/fixtures/setext.md", "--json"])
        .output()
        .unwrap();
    assert!(setext_output.status.success());
    let setext_json: serde_json::Value = serde_json::from_slice(&setext_output.stdout).unwrap();
    let setext_blocks = setext_json["blocks"].as_array().unwrap();
    let title_start = setext_blocks[0]["span"]["byte_start"].as_u64().unwrap() as usize;
    let title_end = setext_blocks[0]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&setext_source[title_start..title_end], "Title\n=====");

    let subtitle_start = setext_blocks[1]["span"]["byte_start"].as_u64().unwrap() as usize;
    let subtitle_end = setext_blocks[1]["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &setext_source[subtitle_start..subtitle_end],
        "Subtitle\n--------"
    );

    let indented_source = std::fs::read_to_string("tests/fixtures/indented_code.md").unwrap();
    let indented_output = md()
        .args(["blocks", "tests/fixtures/indented_code.md", "--json"])
        .output()
        .unwrap();
    assert!(indented_output.status.success());
    let indented_json: serde_json::Value = serde_json::from_slice(&indented_output.stdout).unwrap();
    let indented_blocks = indented_json["blocks"].as_array().unwrap();
    let code_block = indented_blocks
        .iter()
        .find(|block| block["kind"] == "IndentedCode")
        .unwrap();
    let code_start = code_block["span"]["byte_start"].as_u64().unwrap() as usize;
    let code_end = code_block["span"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(
        &indented_source[code_start..code_end],
        "    fn main() {\n        println!(\"hello\");\n    }\n"
    );
}

#[test]
fn block_read_content_matches_span_slice() {
    let source = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();

    for idx in 0..5 {
        let output = md()
            .args([
                "block",
                &idx.to_string(),
                "tests/fixtures/basic.md",
                "--json",
            ])
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
            bs,
            be,
            sliced,
            dest
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
            block["index"],
            bs
        );
        assert!(
            source.is_char_boundary(be),
            "block {} byte_end {} is not a char boundary",
            block["index"],
            be
        );

        let sliced = &source[bs..be];
        assert!(
            !sliced.is_empty(),
            "block {} has empty span",
            block["index"]
        );
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
    assert_ne!(
        before_end, after_end,
        "replaced spans should differ in byte_end"
    );
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
        json["content"].as_str().unwrap(),
        source,
        "NoChange should preserve the full document content"
    );
    assert_eq!(
        json["invariant"]["target_span_before"], json["invariant"]["target_span_after"],
        "NoChange spans must be identical"
    );
}

#[test]
fn mutation_insert_block_target_span_after_slices_exact_payload_across_locations() {
    let after_output = md_with_stdin(
        &[
            "insert-block",
            "--after",
            "0",
            "tests/fixtures/basic.md",
            "--json",
        ],
        "Inserted paragraph.",
    );
    assert!(after_output.status.success());
    let after_json: serde_json::Value = serde_json::from_slice(&after_output.stdout).unwrap();
    let after_content = after_json["content"].as_str().unwrap();
    let after_span = &after_json["invariant"]["target_span_after"];
    let after_bs = after_span["byte_start"].as_u64().unwrap() as usize;
    let after_be = after_span["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(after_json["disposition"], "Inserted");
    assert!(after_json["invariant"]["target_span_before"].is_null());
    assert_eq!(&after_content[after_bs..after_be], "Inserted paragraph.");
    assert_eq!(after_span["line_start"], 2);
    assert_eq!(after_span["line_end"], 2);

    let before_output = md_with_stdin(
        &[
            "insert-block",
            "--before",
            "0",
            "tests/fixtures/basic.md",
            "--json",
        ],
        "Lead one\nLead two",
    );
    assert!(before_output.status.success());
    let before_json: serde_json::Value = serde_json::from_slice(&before_output.stdout).unwrap();
    let before_content = before_json["content"].as_str().unwrap();
    let before_span = &before_json["invariant"]["target_span_after"];
    let before_bs = before_span["byte_start"].as_u64().unwrap() as usize;
    let before_be = before_span["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&before_content[before_bs..before_be], "Lead one\nLead two");
    assert_eq!(before_span["line_start"], 1);
    assert_eq!(before_span["line_end"], 2);
    assert!(before_content[before_be..].starts_with("\n\n# Introduction"));

    let start_output = md_with_stdin(
        &[
            "insert-block",
            "--at-start",
            "tests/fixtures/frontmatter.md",
            "--json",
        ],
        "Inserted A\nInserted B",
    );
    assert!(start_output.status.success());
    let start_json: serde_json::Value = serde_json::from_slice(&start_output.stdout).unwrap();
    let start_content = start_json["content"].as_str().unwrap();
    let start_span = &start_json["invariant"]["target_span_after"];
    let start_bs = start_span["byte_start"].as_u64().unwrap() as usize;
    let start_be = start_span["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&start_content[start_bs..start_be], "Inserted A\nInserted B");
    assert_eq!(start_span["line_start"], 9);
    assert_eq!(start_span["line_end"], 10);
    assert!(start_content[start_be..].starts_with("\n\n# Main Content"));

    let crlf_path = tempfile_str("# T\r\n\r\nBody.");
    let end_output = md_with_stdin(
        &["insert-block", "--at-end", &crlf_path, "--json"],
        "Tail 1\nTail 2",
    );
    assert!(end_output.status.success());
    let end_json: serde_json::Value = serde_json::from_slice(&end_output.stdout).unwrap();
    let end_content = end_json["content"].as_str().unwrap();
    let end_span = &end_json["invariant"]["target_span_after"];
    let end_bs = end_span["byte_start"].as_u64().unwrap() as usize;
    let end_be = end_span["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&end_content[end_bs..end_be], "Tail 1\r\nTail 2");
    assert_eq!(end_span["line_start"], 4);
    assert_eq!(end_span["line_end"], 5);
    assert!(end_content[..end_bs].ends_with("Body.\r\n"));
    std::fs::remove_file(&crlf_path).ok();
}

#[test]
fn mutation_insert_block_empty_payload_uses_no_owned_target_exception() {
    let path = tempfile_str("# Title\n\nBody.");
    let source = std::fs::read_to_string(&path).unwrap();
    let output = md_with_stdin(&["insert-block", "--at-end", &path, "--json"], "");
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert!(json["invariant"]["target_span_before"].is_null());
    assert!(json["invariant"]["target_span_after"].is_null());
    assert_eq!(json["content"].as_str().unwrap(), source);
    std::fs::remove_file(&path).ok();
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
    let bs = json["target"]["Block"]["span"]["byte_start"]
        .as_u64()
        .unwrap() as usize;
    let be = json["target"]["Block"]["span"]["byte_end"]
        .as_u64()
        .unwrap() as usize;

    // Source bytes before the target span must be identical
    assert_eq!(&content[..bs], &source[..bs], "bytes before target changed");
    // Source bytes after the target span must be identical
    assert_eq!(
        &content[content.len() - (source.len() - be)..],
        &source[be..],
        "bytes after target changed"
    );
}

#[test]
fn set_frontmatter_span_nullability_uses_whole_state_ownership_exception() {
    let existing_insert_path =
        tempfile_str(&std::fs::read_to_string("tests/fixtures/set_basic.md").unwrap());
    let existing_insert_source = std::fs::read_to_string(&existing_insert_path).unwrap();
    let existing_insert = md()
        .args(["set", "meta.version", &existing_insert_path, "2", "--json"])
        .output()
        .unwrap();
    assert!(existing_insert.status.success());
    let existing_insert_json: serde_json::Value =
        serde_json::from_slice(&existing_insert.stdout).unwrap();
    assert_eq!(existing_insert_json["command"], "SetFrontmatter");
    assert_eq!(existing_insert_json["disposition"], "Inserted");
    assert!(existing_insert_json["invariant"]["target_span_before"].is_object());
    assert!(existing_insert_json["invariant"]["target_span_after"].is_object());
    let existing_before_end = existing_insert_json["invariant"]["target_span_before"]["byte_end"]
        .as_u64()
        .unwrap() as usize;
    let existing_after_end = existing_insert_json["invariant"]["target_span_after"]["byte_end"]
        .as_u64()
        .unwrap() as usize;
    let existing_insert_content = existing_insert_json["content"].as_str().unwrap();
    assert_eq!(
        &existing_insert_content[existing_after_end..],
        &existing_insert_source[existing_before_end..],
        "existing-state insert must preserve bytes after the owned frontmatter span"
    );
    std::fs::remove_file(&existing_insert_path).ok();

    let existing_delete_path =
        tempfile_str(&std::fs::read_to_string("tests/fixtures/set_basic.md").unwrap());
    let existing_delete_source = std::fs::read_to_string(&existing_delete_path).unwrap();
    let existing_delete = md()
        .args(["set", "--delete", "author", &existing_delete_path, "--json"])
        .output()
        .unwrap();
    assert!(existing_delete.status.success());
    let existing_delete_json: serde_json::Value =
        serde_json::from_slice(&existing_delete.stdout).unwrap();
    assert_eq!(existing_delete_json["command"], "SetFrontmatter");
    assert_eq!(existing_delete_json["disposition"], "Deleted");
    assert!(existing_delete_json["invariant"]["target_span_before"].is_object());
    assert!(existing_delete_json["invariant"]["target_span_after"].is_object());
    let deleted_before_end = existing_delete_json["invariant"]["target_span_before"]["byte_end"]
        .as_u64()
        .unwrap() as usize;
    let deleted_after_end = existing_delete_json["invariant"]["target_span_after"]["byte_end"]
        .as_u64()
        .unwrap() as usize;
    let existing_delete_content = existing_delete_json["content"].as_str().unwrap();
    assert_eq!(
        &existing_delete_content[deleted_after_end..],
        &existing_delete_source[deleted_before_end..],
        "existing-state delete must preserve bytes after the owned frontmatter span"
    );
    std::fs::remove_file(&existing_delete_path).ok();

    let absent_delete_path =
        tempfile_str(&std::fs::read_to_string("tests/fixtures/no_frontmatter.md").unwrap());
    let absent_delete_source = std::fs::read_to_string(&absent_delete_path).unwrap();
    let absent_delete = md()
        .args(["set", "--delete", "missing", &absent_delete_path, "--json"])
        .output()
        .unwrap();
    assert!(absent_delete.status.success());
    let absent_delete_json: serde_json::Value =
        serde_json::from_slice(&absent_delete.stdout).unwrap();
    assert_eq!(absent_delete_json["command"], "SetFrontmatter");
    assert_eq!(absent_delete_json["disposition"], "NoChange");
    assert!(absent_delete_json["invariant"]["target_span_before"].is_null());
    assert!(absent_delete_json["invariant"]["target_span_after"].is_null());
    assert_eq!(
        absent_delete_json["content"].as_str().unwrap(),
        absent_delete_source
    );
    std::fs::remove_file(&absent_delete_path).ok();
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
        &[
            "replace-section",
            "Discussion",
            "tests/fixtures/basic.md",
            "--json",
        ],
        "",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Deleted");
}

#[test]
fn replace_section_boundary_target_span_after_covers_zero_one_and_multiple_trailing_lf_cases() {
    let path = tempfile_str("# Doc\n\n## One\nold\n\n## Two\nkeep\n");
    let cases = [
        (
            "zero trailing newlines",
            "## One\nnew",
            "## One\nnew\n\n",
            5,
        ),
        (
            "one trailing newline",
            "## One\nnew\n",
            "## One\nnew\n\n",
            5,
        ),
        (
            "multiple trailing newlines",
            "## One\nnew\n\n\n",
            "## One\nnew\n\n\n",
            6,
        ),
    ];

    for (label, replacement, expected_slice, expected_line_end) in cases {
        assert_replace_section_target_span_after(
            label,
            &path,
            replacement,
            expected_slice,
            expected_line_end,
        );
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn replace_section_boundary_target_span_after_stops_at_boundary_floor_in_crlf_docs() {
    let path = tempfile_bytes(b"# Doc\r\n\r\n## One\r\nold\r\n\r\n## Two\r\nkeep\r\n");
    let cases = [
        (
            "zero trailing newlines",
            "## One\nnew",
            "## One\r\nnew\r\n\r\n",
            5,
        ),
        (
            "one trailing newline",
            "## One\nnew\n",
            "## One\r\nnew\r\n\r\n",
            5,
        ),
        (
            "multiple trailing newlines",
            "## One\nnew\n\n\n",
            "## One\r\nnew\r\n\r\n\r\n",
            6,
        ),
    ];

    for (label, replacement, expected_slice, expected_line_end) in cases {
        assert_replace_section_target_span_after(
            label,
            &path,
            replacement,
            expected_slice,
            expected_line_end,
        );
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn replace_section_boundary_nochange_fields_reflect_effective_inserted_bytes() {
    let source = "# Doc\n\n## One\nold\n\n## Two\nkeep\n";
    let path = tempfile_str(source);
    let output = md_with_stdin(
        &["replace-section", "One", &path, "--json"],
        "## One\nold\n",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert_eq!(json["content"].as_str().unwrap(), source);
    assert_eq!(
        json["invariant"]["target_span_before"],
        json["invariant"]["target_span_after"]
    );

    std::fs::remove_file(&path).ok();
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
            "search",
            "method",
            "tests/fixtures/search_in_code.md",
            "--kind",
            "code-fence",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matches = json["matches"].as_array().unwrap();
    assert!(
        !matches.is_empty(),
        "should find 'method' inside code fence"
    );
    for m in matches {
        assert_eq!(m["block_kind"], "CodeFence");
    }
}

#[test]
fn search_inside_indented_code() {
    let output = md()
        .args([
            "search",
            "method",
            "tests/fixtures/search_in_code.md",
            "--kind",
            "indented-code",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matches = json["matches"].as_array().unwrap();
    assert!(
        !matches.is_empty(),
        "should find 'method' inside indented code"
    );
    for m in matches {
        assert_eq!(m["block_kind"], "IndentedCode");
    }
}

#[test]
fn search_excludes_code_by_default_when_filtered() {
    // Search only paragraphs — should not find code block matches
    let output = md()
        .args([
            "search",
            "method",
            "tests/fixtures/search_in_code.md",
            "--kind",
            "paragraph",
            "--json",
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
fn stats_heading_word_count_uses_parser_plaintext_for_lf_inputs() {
    let cases = [
        ("# Alpha Beta\n", 2_u64),
        ("  ## Alpha Beta\n", 2_u64),
        ("## Alpha Beta ##\n", 2_u64),
        ("Alpha Beta\n----------\n", 2_u64),
        ("  Alpha Beta\n  ----------\n", 2_u64),
        (
            "# Alpha *Beta* [Gamma](https://example.com) `delta`\n",
            4_u64,
        ),
    ];

    for (source, expected) in cases {
        let path = tempfile_str(source);
        let output = md().args(["stats", &path, "--json"]).output().unwrap();
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            json["stats"]["word_count"], expected,
            "heading plaintext stats mismatch for source {:?}",
            source
        );
        std::fs::remove_file(&path).unwrap();
    }
}

#[test]
fn stats_heading_word_count_uses_parser_plaintext_for_crlf_inputs() {
    let cases: [(&[u8], u64); 2] = [
        (b"  ## Alpha Beta\r\n", 2_u64),
        (
            b"Alpha *Beta* [Gamma](https://example.com) `delta`\r\n----------\r\n",
            4_u64,
        ),
    ];

    for (source, expected) in cases {
        let path = tempfile_bytes(source);
        let output = md().args(["stats", &path, "--json"]).output().unwrap();
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            json["stats"]["word_count"],
            expected,
            "heading plaintext stats mismatch for source {:?}",
            String::from_utf8_lossy(source)
        );
        std::fs::remove_file(&path).unwrap();
    }
}

#[test]
fn stats_line_count_no_trailing_newline() {
    // A file with no trailing newline: last line still counts
    let tmp = tempfile_str("line1\nline2\nline3");
    let output = md().args(["stats", &tmp, "--json"]).output().unwrap();
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
    assert!(
        kinds.contains(&"FootnoteDefinition"),
        "expected FootnoteDefinition block"
    );
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

    let table = json["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["kind"] == "Table")
        .unwrap();
    let bs = table["span"]["byte_start"].as_u64().unwrap() as usize;
    let be = table["span"]["byte_end"].as_u64().unwrap() as usize;
    let sliced = &source[bs..be];
    assert!(
        sliced.contains("| Name"),
        "table span should contain header row"
    );
    assert!(
        sliced.contains("| Alpha"),
        "table span should contain data rows"
    );
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
        assert_eq!(
            fields.len(),
            4,
            "blocks text line should have 4 tab-separated fields: {:?}",
            line
        );
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
        assert_eq!(
            fields.len(),
            3,
            "outline text line should have 3 tab-separated fields: {:?}",
            line
        );
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
        assert_eq!(
            fields.len(),
            4,
            "search text line should have 4 tab-separated fields: {:?}",
            line
        );
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
        assert_eq!(
            fields.len(),
            4,
            "links text line should have 4 tab-separated fields: {:?}",
            line
        );
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
    let keys: Vec<&str> = stdout
        .lines()
        .map(|l| l.split('=').next().unwrap())
        .collect();
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
        vec!["collect", "tests/fixtures/collect_vault", "-r", "--json"],
        vec!["stats", "tests/fixtures/basic.md", "--json"],
    ];

    for args in &commands {
        let output = md().args(args).output().unwrap();
        assert!(output.status.success(), "command {:?} failed", args);
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            json["schema_version"], "mdtools.v1",
            "schema_version missing for {:?}",
            args
        );
    }
}

#[test]
fn set_help_mentions_frontmatter_expect_etag() {
    let help = md_help("set");
    let normalized = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains("--expect-etag"));
    assert!(normalized.contains("whole-frontmatter etag"));
    assert!(normalized.contains("md frontmatter --json"));
}

#[test]
fn frontmatter_state_etag_tracks_exact_owned_bytes() {
    let base = tempfile_str("---\ntitle: Same\n---");
    let with_lf = tempfile_str("---\ntitle: Same\n---\n");
    let reordered = tempfile_str("---\n# comment\ntitle: Same\n---\n# Body\n");
    let crlf = tempfile_bytes(b"---\r\ntitle: Same\r\n---\r\n# Body\r\n");
    let toml = tempfile_str("+++\ntitle = \"Same\"\n+++\n");
    let empty_present = tempfile_str("---\n\n---\n# Body\n");
    let absent = tempfile_str("# Body\n");

    let base_etag = frontmatter_etag(&base);
    assert_ne!(base_etag, frontmatter_etag(&with_lf));
    assert_ne!(base_etag, frontmatter_etag(&reordered));
    assert_ne!(base_etag, frontmatter_etag(&crlf));
    assert_ne!(base_etag, frontmatter_etag(&toml));
    assert_ne!(frontmatter_etag(&empty_present), frontmatter_etag(&absent));

    std::fs::remove_file(&base).ok();
    std::fs::remove_file(&with_lf).ok();
    std::fs::remove_file(&reordered).ok();
    std::fs::remove_file(&crlf).ok();
    std::fs::remove_file(&toml).ok();
    std::fs::remove_file(&empty_present).ok();
    std::fs::remove_file(&absent).ok();
}

#[test]
fn frontmatter_present_state_raw_span_and_etag_match_owned_boundary_bytes() {
    let eof = tempfile_str("---\ntitle: Same\n---");
    let lf = tempfile_str("---\ntitle: Same\n---\n# Body\n");
    let crlf = tempfile_bytes(b"---\r\ntitle: Same\r\n---\r\n# Body\r\n");

    let read = |path: &str| -> serde_json::Value {
        let output = md().args(["frontmatter", path]).output().unwrap();
        assert!(
            output.status.success(),
            "frontmatter read failed for {}: {}",
            path,
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    };

    let eof_json = read(&eof);
    assert_eq!(eof_json["frontmatter"]["raw"], "---\ntitle: Same\n---");
    assert_eq!(eof_json["frontmatter"]["span"]["byte_start"], 0);
    assert_eq!(eof_json["frontmatter"]["span"]["byte_end"], 19);

    let lf_json = read(&lf);
    assert_eq!(lf_json["frontmatter"]["raw"], "---\ntitle: Same\n---\n");
    assert_eq!(lf_json["frontmatter"]["span"]["byte_start"], 0);
    assert_eq!(lf_json["frontmatter"]["span"]["byte_end"], 20);
    assert_eq!(lf_json["frontmatter"]["data"]["title"], "Same");

    let crlf_json = read(&crlf);
    assert_eq!(
        crlf_json["frontmatter"]["raw"],
        "---\r\ntitle: Same\r\n---\r\n"
    );
    assert_eq!(crlf_json["frontmatter"]["span"]["byte_start"], 0);
    assert_eq!(crlf_json["frontmatter"]["span"]["byte_end"], 23);
    assert_eq!(crlf_json["frontmatter"]["data"]["title"], "Same");

    assert_ne!(eof_json["etag"], lf_json["etag"]);
    assert_ne!(lf_json["etag"], crlf_json["etag"]);

    std::fs::remove_file(&eof).ok();
    std::fs::remove_file(&lf).ok();
    std::fs::remove_file(&crlf).ok();
}

#[test]
fn frontmatter_state_etag_detects_comment_drift() {
    let base = tempfile_str("---\ntitle: Same\n---\n");
    let with_comment = tempfile_str("---\n# retained comment\ntitle: Same\n---\n");
    assert_ne!(
        frontmatter_etag(&base),
        frontmatter_etag(&with_comment),
        "comment drift"
    );
    std::fs::remove_file(&base).ok();
    std::fs::remove_file(&with_comment).ok();
}

#[test]
fn frontmatter_state_etag_detects_key_order_drift() {
    let base = tempfile_str("---\ntitle: Same\nauthor: Jane\n---\n");
    let reordered = tempfile_str("---\nauthor: Jane\ntitle: Same\n---\n");
    assert_ne!(
        frontmatter_etag(&base),
        frontmatter_etag(&reordered),
        "key order drift"
    );
    std::fs::remove_file(&base).ok();
    std::fs::remove_file(&reordered).ok();
}

#[test]
fn frontmatter_state_etag_detects_whitespace_drift() {
    let base = tempfile_str("---\ntitle: Same\n---\n");
    let extra_blank_line = tempfile_str("---\ntitle: Same\n\n---\n");
    assert_ne!(
        frontmatter_etag(&base),
        frontmatter_etag(&extra_blank_line),
        "whitespace drift"
    );
    std::fs::remove_file(&base).ok();
    std::fs::remove_file(&extra_blank_line).ok();
}

#[test]
fn frontmatter_state_etag_detects_lf_versus_crlf_drift() {
    let lf = tempfile_str("---\ntitle: Same\n---\n# Body\n");
    let crlf = tempfile_bytes(b"---\r\ntitle: Same\r\n---\r\n# Body\r\n");
    assert_ne!(
        frontmatter_etag(&lf),
        frontmatter_etag(&crlf),
        "LF versus CRLF"
    );
    std::fs::remove_file(&lf).ok();
    std::fs::remove_file(&crlf).ok();
}

#[test]
fn frontmatter_state_etag_detects_yaml_versus_toml_delimiter_family_drift() {
    let yaml = tempfile_str("---\ntitle: Same\n---\n");
    let toml = tempfile_str("+++\ntitle = \"Same\"\n+++\n");
    assert_ne!(
        frontmatter_etag(&yaml),
        frontmatter_etag(&toml),
        "YAML versus TOML delimiter family"
    );
    std::fs::remove_file(&yaml).ok();
    std::fs::remove_file(&toml).ok();
}

#[test]
fn frontmatter_state_etag_detects_closing_delimiter_at_eof_versus_followed_by_lf() {
    let eof = tempfile_str("---\ntitle: Same\n---");
    let with_lf = tempfile_str("---\ntitle: Same\n---\n");
    assert_ne!(
        frontmatter_etag(&eof),
        frontmatter_etag(&with_lf),
        "closing delimiter at EOF versus followed by LF"
    );
    std::fs::remove_file(&eof).ok();
    std::fs::remove_file(&with_lf).ok();
}

#[test]
fn frontmatter_state_etag_distinguishes_absent_versus_present_empty_state() {
    let empty_present = tempfile_str("---\n\n---\n# Body\n");
    let absent = tempfile_str("# Body\n");
    assert_ne!(
        frontmatter_etag(&empty_present),
        frontmatter_etag(&absent),
        "absent versus present empty state"
    );
    std::fs::remove_file(&empty_present).ok();
    std::fs::remove_file(&absent).ok();
}

#[test]
fn frontmatter_state_etag_ignores_non_frontmatter_body_bytes() {
    let present_first = tempfile_str("---\ntitle: Same\n---\n# Body\nalpha\n");
    let present_second = tempfile_str("---\ntitle: Same\n---\n# Different\nbeta\n");
    let absent_first = tempfile_str("# Body\nalpha\n");
    let absent_second = tempfile_str("# Different\nbeta\n");

    let present_etag = frontmatter_etag(&present_first);
    assert_eq!(
        present_etag,
        frontmatter_etag(&present_second),
        "present frontmatter etag should ignore body bytes"
    );

    let absent_etag = frontmatter_etag(&absent_first);
    assert_eq!(
        absent_etag,
        frontmatter_etag(&absent_second),
        "absent frontmatter etag should ignore body bytes"
    );
    assert_ne!(
        present_etag, absent_etag,
        "present and absent frontmatter states stay domain-separated"
    );

    std::fs::remove_file(&present_first).ok();
    std::fs::remove_file(&present_second).ok();
    std::fs::remove_file(&absent_first).ok();
    std::fs::remove_file(&absent_second).ok();
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
    assert_eq!(
        lines.len(),
        1,
        "stderr should be exactly one line: {:?}",
        stderr
    );
}

#[test]
fn error_exits_are_correct() {
    // NotFound = 1
    let o = md()
        .args(["section", "Nonexistent", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));

    // NotFound = 1 (block out of range)
    let o = md()
        .args(["block", "99", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));

    // NotFound = 1 (missing file)
    let o = md()
        .args(["blocks", "/tmp/nonexistent_mdtools_xyz.md"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));

    // ParseError = 2 (malformed frontmatter)
    let o = md()
        .args(["frontmatter", "tests/fixtures/malformed_frontmatter.md"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));

    // InvalidInput = 3 (no insert location)
    let o = md_with_stdin(&["insert-block", "tests/fixtures/basic.md"], "x");
    assert_eq!(o.status.code(), Some(3));

    // Conflict = 4 (duplicate heading)
    let o = md()
        .args(["section", "Methods", "tests/fixtures/duplicate_headings.md"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(4));
}

#[test]
fn stale_frontmatter_section_and_task_expect_etag_conflicts_exit_four() {
    let frontmatter_path =
        tempfile_str(&std::fs::read_to_string("tests/fixtures/frontmatter.md").unwrap());
    let frontmatter_etag = frontmatter_etag(&frontmatter_path);
    let stale_frontmatter = std::fs::read_to_string(&frontmatter_path).unwrap();
    let fresh_frontmatter =
        stale_frontmatter.replace("title: Test Document", "title: Intervening Document");
    std::fs::write(&frontmatter_path, fresh_frontmatter.clone()).unwrap();
    let o = md()
        .args([
            "set",
            "date",
            &frontmatter_path,
            "2024-02-01",
            "-i",
            "--expect-etag",
            &frontmatter_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&o.stderr).contains("frontmatter etag mismatch"));
    assert_eq!(
        std::fs::read_to_string(&frontmatter_path).unwrap(),
        fresh_frontmatter
    );
    std::fs::remove_file(&frontmatter_path).ok();

    let section_path = tempfile_str(&std::fs::read_to_string("tests/fixtures/basic.md").unwrap());
    let section_read = md()
        .args(["section", "Discussion", &section_path, "--json"])
        .output()
        .unwrap();
    assert!(section_read.status.success());
    let section_json: serde_json::Value = serde_json::from_slice(&section_read.stdout).unwrap();
    let section_etag = section_json["section"]["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let stale_section = std::fs::read_to_string(&section_path).unwrap();
    let fresh_section = stale_section.replace(
        "Final thoughts on the approach.",
        "Final thoughts after an intervening edit.",
    );
    std::fs::write(&section_path, fresh_section).unwrap();
    let o = md_with_stdin(
        &[
            "replace-section",
            "Discussion",
            &section_path,
            "-i",
            "--expect-etag",
            &section_etag,
        ],
        "## Discussion\n\nShould not apply.\n",
    );
    assert_eq!(o.status.code(), Some(4));
    std::fs::remove_file(&section_path).ok();

    let task_path =
        tempfile_str(&std::fs::read_to_string("tests/fixtures/progress_example.md").unwrap());
    let tasks_read = md().args(["tasks", &task_path, "--json"]).output().unwrap();
    assert!(tasks_read.status.success());
    let tasks_json: serde_json::Value = serde_json::from_slice(&tasks_read.stdout).unwrap();
    let task_etag = tasks_json["results"][0]["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["loc"] == "9.3")
        .unwrap()["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let stale_task = std::fs::read_to_string(&task_path).unwrap();
    let fresh_task = stale_task.replace(
        "[ ] 0.4 Remove collation overrides",
        "[x] 0.4 Remove collation overrides",
    );
    std::fs::write(&task_path, fresh_task).unwrap();
    let o = md()
        .args([
            "set-task",
            "9.3",
            &task_path,
            "-i",
            "--status",
            "done",
            "--expect-etag",
            &task_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(4));
    std::fs::remove_file(&task_path).ok();
}

// ============================================================
// HELPERS
// ============================================================

fn tempfile_str(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_contract_str_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}

fn tempfile_bytes(content: &[u8]) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!(
        "/tmp/mdtools_contract_bytes_{}_{}.md",
        std::process::id(),
        id
    );
    std::fs::write(&path, content).unwrap();
    path
}

fn frontmatter_etag(path: &str) -> String {
    let output = md().args(["frontmatter", path]).output().unwrap();
    assert!(
        output.status.success(),
        "frontmatter read failed for {}",
        path
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json["etag"].as_str().unwrap().to_string()
}

fn assert_replace_section_target_span_after(
    label: &str,
    path: &str,
    replacement: &str,
    expected_slice: &str,
    expected_line_end: u64,
) {
    let output = md_with_stdin(&["replace-section", "One", path, "--json"], replacement);
    assert!(output.status.success(), "{label}: replace-section failed");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced", "{label}: disposition");
    assert_eq!(json["changed"], true, "{label}: changed");

    let content = json["content"].as_str().unwrap();
    let after = &json["invariant"]["target_span_after"];
    let bs = after["byte_start"].as_u64().unwrap() as usize;
    let be = after["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&content[bs..be], expected_slice, "{label}: target slice");
    assert_eq!(after["line_start"], 3, "{label}: line_start");
    assert_eq!(after["line_end"], expected_line_end, "{label}: line_end");
}

// Regression guard: tempfile_str and tempfile_bytes own independent counters
// starting at 0. Before namespacing by prefix, both helpers produced the same
// path at id=0 and collided when called concurrently within the same test
// binary, intermittently failing tests like stats_line_count_no_trailing_newline
// (see iteration-6 notes). Assert that their first-issued paths differ.
#[test]
fn tempfile_helpers_produce_disjoint_paths() {
    let a = tempfile_str("a");
    let b = tempfile_bytes(b"b");
    assert_ne!(
        a, b,
        "tempfile_str and tempfile_bytes must not collide on disk"
    );
    std::fs::remove_file(&a).ok();
    std::fs::remove_file(&b).ok();
}

#[test]
fn contains_help_text_is_shared_without_changing_move_section_wording() {
    let shared_help =
        "Match parsed plaintext top-level heading text by literal substring; exact matching remains the default";
    for subcommand in ["section", "replace-section", "delete-section"] {
        let help = md_help(subcommand);
        let normalized_help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized_help.contains("--contains") && normalized_help.contains(shared_help),
            "{} --help missing shared --contains text:\n{}",
            subcommand,
            help
        );
    }

    let move_help = md_help("move-section");
    let normalized_move_help = move_help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalized_move_help.contains(
            "--contains Literal substring matching for both source and destination selectors"
        ),
        "move-section --help changed unexpectedly:\n{}",
        move_help
    );
}

#[test]
fn collect_help_mentions_recursive_and_field_projection() {
    let help = md_help("collect");
    let normalized_help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalized_help.contains("Usage: md collect [OPTIONS] <FILES>...")
            && normalized_help.contains("--recursive")
            && normalized_help.contains("--field <FIELDS>")
            && normalized_help.contains("Extract specific fields (repeatable or comma-separated)"),
        "collect --help changed unexpectedly:\n{}",
        help
    );
}
