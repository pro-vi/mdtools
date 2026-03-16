//! Scenario-based integration tests simulating real LLM agent workflows.
//! Each test chains multiple CLI calls the way an agent would to accomplish a task.

use std::io::Write;
use std::process::{Command, Stdio};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn md_json(args: &[&str]) -> serde_json::Value {
    let mut full_args = args.to_vec();
    if !full_args.contains(&"--json") {
        full_args.push("--json");
    }
    let output = md().args(&full_args).output().unwrap();
    assert!(
        output.status.success(),
        "command {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn md_text(args: &[&str]) -> String {
    let output = md().args(args).output().unwrap();
    assert!(
        output.status.success(),
        "command {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn md_stdin(args: &[&str], input: &str) -> std::process::Output {
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
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn md_stdin_json(args: &[&str], input: &str) -> serde_json::Value {
    let mut full_args = args.to_vec();
    if !full_args.contains(&"--json") {
        full_args.push("--json");
    }
    let output = md_stdin(&full_args, input);
    assert!(
        output.status.success(),
        "command {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn tmpfile(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let id = CTR.fetch_add(1, Ordering::SeqCst);
    let p = format!("/tmp/mdtools_scenario_{}_{}.md", std::process::id(), id);
    std::fs::write(&p, content).unwrap();
    p
}

const DOC: &str = "tests/fixtures/agent_doc.md";

// ============================================================
// SCENARIO: T4-style — find "method" in body text, replace with
// "approach", but leave headings and code blocks untouched.
// An agent would: search → filter → read each block → replace word → write back
// ============================================================

#[test]
fn scenario_selective_word_replacement() {
    let source = std::fs::read_to_string(DOC).unwrap();
    let tmp = tmpfile(&source);

    // Step 1: Agent searches for "method" in paragraphs only
    let search = md_json(&["search", "method", &tmp, "--kind", "paragraph"]);
    let matches = search["matches"].as_array().unwrap();
    assert!(
        !matches.is_empty(),
        "agent should find 'method' in paragraphs"
    );

    // Step 2: Collect unique block indices that contain matches
    let mut block_indices: Vec<u32> = matches
        .iter()
        .map(|m| m["block_index"].as_u64().unwrap() as u32)
        .collect();
    block_indices.sort();
    block_indices.dedup();

    // Step 3: For each affected block, read content, replace word, write back
    // Process in reverse order so byte offsets stay valid
    for &idx in block_indices.iter().rev() {
        let block = md_json(&["block", &idx.to_string(), &tmp]);
        let content = block["content"].as_str().unwrap().to_string();
        let replaced = content.replace("method", "approach");

        if replaced != content {
            let output = md_stdin(
                &["replace-block", &idx.to_string(), &tmp, "-i"],
                &replaced,
            );
            assert!(output.status.success());
        }
    }

    // Step 4: Verify the result
    let result = std::fs::read_to_string(&tmp).unwrap();

    // "method" should be gone from paragraphs
    let remaining = md_json(&["search", "method", &tmp, "--kind", "paragraph"]);
    assert_eq!(
        remaining["matches"].as_array().unwrap().len(),
        0,
        "all paragraph occurrences of 'method' should be replaced"
    );

    // "method" should still exist in headings (we didn't touch them)
    // (actually the headings don't contain "method" in this fixture, but code does)

    // Code blocks should be untouched
    let code_search = md_json(&["search", "method", &tmp, "--kind", "code-fence"]);
    // There's no "method" in the code block of this fixture, but verify code is intact
    let blocks = md_json(&["blocks", &tmp]);
    let code_blocks: Vec<_> = blocks["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["kind"] == "CodeFence")
        .collect();
    assert!(!code_blocks.is_empty(), "code blocks should still exist");

    // Frontmatter should be completely untouched
    let fm = md_json(&["frontmatter", &tmp]);
    assert_eq!(fm["frontmatter"]["data"]["title"], "API Documentation");
    assert_eq!(fm["frontmatter"]["data"]["version"], 3);

    // "approach" should appear where "method" was
    assert!(result.contains("Use the standard approach to authenticate"));
    assert!(result.contains("Use the refresh approach to renew"));

    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// SCENARIO: T1-style — extract and verify document structure.
// An agent would: outline → verify hierarchy → check section count
// ============================================================

#[test]
fn scenario_structural_analysis() {
    // Step 1: Get the outline
    let outline = md_json(&["outline", DOC]);
    let entries = outline["entries"].as_array().unwrap();

    // Step 2: Verify the heading hierarchy
    let levels: Vec<u8> = entries
        .iter()
        .map(|e| e["heading"]["level"].as_u64().unwrap() as u8)
        .collect();
    let texts: Vec<&str> = entries
        .iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();

    // H1 > H2 > H3 structure
    assert_eq!(levels[0], 1); // API Reference
    assert_eq!(texts[0], "API Reference");
    assert!(levels[1..].iter().all(|&l| l >= 2), "all under H1 should be H2+");

    // Step 3: Verify section spans are contiguous and non-overlapping
    for i in 0..entries.len() - 1 {
        let this_level = levels[i];
        let next_level = levels[i + 1];
        let this_section_end = entries[i]["section_span"]["byte_end"].as_u64().unwrap();
        let next_section_start = entries[i + 1]["section_span"]["byte_start"].as_u64().unwrap();

        // If next heading is same or higher level, this section ends at next section start
        if next_level <= this_level {
            assert_eq!(
                this_section_end, next_section_start,
                "section {} ({:?}) should end where section {} ({:?}) starts",
                i, texts[i], i + 1, texts[i + 1]
            );
        }
    }

    // Step 4: Cross-check with stats
    let stats = md_json(&["stats", DOC]);
    assert_eq!(
        stats["stats"]["heading_count"],
        entries.len() as u64,
        "stats heading_count should match outline entries"
    );

    // Step 5: Cross-check with blocks
    let blocks = md_json(&["blocks", DOC]);
    let heading_blocks: Vec<_> = blocks["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["kind"] == "Heading")
        .collect();
    assert_eq!(
        heading_blocks.len(),
        entries.len(),
        "outline entries should match heading blocks"
    );

    // Verify block indices match
    for (entry, block) in entries.iter().zip(heading_blocks.iter()) {
        assert_eq!(
            entry["heading"]["block_index"], block["index"],
            "outline block_index should match blocks index"
        );
    }
}

// ============================================================
// SCENARIO: T2-style — insert a deprecation notice after a
// specific endpoint section.
// An agent would: outline → find target → insert after
// ============================================================

#[test]
fn scenario_insert_after_target_section() {
    let source = std::fs::read_to_string(DOC).unwrap();
    let tmp = tmpfile(&source);

    // Step 1: Find the "DELETE /users/:id" heading block index
    let outline = md_json(&["outline", &tmp]);
    let delete_entry = outline["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["heading"]["text"].as_str().unwrap().contains("DELETE"))
        .expect("should find DELETE endpoint heading");

    let delete_block_idx = delete_entry["heading"]["block_index"].as_u64().unwrap();

    // Step 2: Read the section to find the last block in it
    // The DELETE section has the heading + one paragraph
    let blocks = md_json(&["blocks", &tmp]);
    let all_blocks = blocks["blocks"].as_array().unwrap();

    // Find blocks belonging to the DELETE section (heading + following until next same-or-higher)
    let mut section_end_idx = delete_block_idx;
    for b in all_blocks.iter() {
        let idx = b["index"].as_u64().unwrap();
        if idx <= delete_block_idx {
            continue;
        }
        if b["kind"] == "Heading" {
            break;
        }
        section_end_idx = idx;
    }

    // Step 3: Insert deprecation notice after the last block of the DELETE section
    let notice = "> **Deprecated:** This endpoint will be removed in v4. Use PATCH /users/:id/archive instead.";
    let output = md_stdin(
        &[
            "insert-block",
            "--after",
            &section_end_idx.to_string(),
            &tmp,
            "-i",
        ],
        notice,
    );
    assert!(output.status.success());

    // Step 4: Verify the notice is in the right place
    let result = std::fs::read_to_string(&tmp).unwrap();
    let notice_pos = result.find("Deprecated:").unwrap();
    let delete_pos = result.find("DELETE /users/:id").unwrap();
    let error_pos = result.find("## Error Handling").unwrap();

    assert!(
        notice_pos > delete_pos && notice_pos < error_pos,
        "deprecation notice should be between DELETE section and Error Handling"
    );

    // Step 5: Verify document structure is preserved
    let new_outline = md_json(&["outline", &tmp]);
    let new_headings: Vec<&str> = new_outline["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();
    let old_headings: Vec<&str> = outline["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();

    assert_eq!(
        new_headings, old_headings,
        "heading structure should be unchanged after insertion"
    );

    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// SCENARIO: T3-style — replace duplicate-named section using
// --occurrence to disambiguate.
// ============================================================

#[test]
fn scenario_replace_duplicate_section() {
    let source = std::fs::read_to_string("tests/fixtures/duplicate_headings.md").unwrap();
    let tmp = tmpfile(&source);

    // Step 1: Agent tries to select "Methods" → gets Conflict
    let output = md()
        .args(["section", "Methods", &tmp])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4), "should conflict on duplicate");

    // Step 2: Agent uses outline to understand the structure
    let outline = md_json(&["outline", &tmp]);
    let methods_entries: Vec<_> = outline["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["heading"]["text"] == "Methods")
        .collect();
    assert_eq!(methods_entries.len(), 2, "should see two Methods sections");

    // Step 3: Agent reads each occurrence to decide which to replace
    let first = md_text(&["section", "Methods", &tmp, "--occurrence", "1"]);
    let second = md_text(&["section", "Methods", &tmp, "--occurrence", "2"]);
    assert!(first.contains("First methods section"));
    assert!(second.contains("Second methods section"));

    // Step 4: Agent replaces the second occurrence
    let new_content = "## Methods\n\nUpdated second methods section with new approach.\n";
    let output = md_stdin(
        &["replace-section", "Methods", &tmp, "--occurrence", "2", "-i"],
        new_content,
    );
    assert!(output.status.success());

    // Step 5: Verify first occurrence is untouched
    let result_first = md_text(&["section", "Methods", &tmp, "--occurrence", "1"]);
    assert!(result_first.contains("First methods section"));

    // Step 6: Verify second occurrence was replaced
    let result_second = md_text(&["section", "Methods", &tmp, "--occurrence", "2"]);
    assert!(result_second.contains("Updated second methods section"));
    assert!(!result_second.contains("Second methods section"));

    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// SCENARIO: Multi-step document surgery — reorganize sections.
// Agent reads structure, deletes one section, replaces another,
// inserts a new one.
// ============================================================

#[test]
fn scenario_multi_step_surgery() {
    let source = "# Title\n\nIntro paragraph.\n\n## Old Section\n\nRemove this.\n\n## Keep Section\n\nKeep this content.\n\n## Update Section\n\nOld content here.\n";
    let tmp = tmpfile(source);

    // Step 1: Get initial structure
    let initial_blocks = md_json(&["blocks", &tmp]);
    let initial_count = initial_blocks["blocks"].as_array().unwrap().len();

    // Step 2: Replace "Update Section" content
    let output = md_stdin(
        &["replace-section", "Update Section", &tmp, "-i"],
        "## Update Section\n\nNew improved content.\n",
    );
    assert!(output.status.success());

    // Step 3: Verify the replacement worked before doing more surgery
    let mid_result = md_text(&["section", "Update Section", &tmp]);
    assert!(mid_result.contains("New improved content"));

    // Step 4: Delete the "Old Section" heading block
    // First find it
    let blocks = md_json(&["blocks", &tmp]);
    let old_section_block = blocks["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| {
            b["kind"] == "Heading"
                && b["preview"].as_str().unwrap().contains("Old Section")
        })
        .expect("should find Old Section heading");
    let old_idx = old_section_block["index"].as_u64().unwrap();

    // Delete the section by replacing it with empty
    let output = md_stdin(
        &["replace-section", "Old Section", &tmp, "-i"],
        "",
    );
    assert!(output.status.success());

    // Step 5: Insert a new section
    // Find "Keep Section" heading to insert before it
    let blocks = md_json(&["blocks", &tmp]);
    let keep_block = blocks["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| {
            b["kind"] == "Heading"
                && b["preview"].as_str().unwrap().contains("Keep Section")
        })
        .expect("should find Keep Section");
    let keep_idx = keep_block["index"].as_u64().unwrap();

    let output = md_stdin(
        &["insert-block", "--before", &keep_idx.to_string(), &tmp, "-i"],
        "## New Section\n\nFreshly added content.\n",
    );
    assert!(output.status.success());

    // Step 6: Verify final structure
    let final_outline = md_json(&["outline", &tmp]);
    let final_headings: Vec<&str> = final_outline["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();

    assert!(final_headings.contains(&"Title"));
    assert!(!final_headings.contains(&"Old Section"), "Old Section should be deleted");
    assert!(final_headings.contains(&"New Section"));
    assert!(final_headings.contains(&"Keep Section"));
    assert!(final_headings.contains(&"Update Section"));

    // Verify content integrity
    let final_doc = std::fs::read_to_string(&tmp).unwrap();
    assert!(final_doc.contains("Intro paragraph."));
    assert!(!final_doc.contains("Remove this."));
    assert!(final_doc.contains("Freshly added content."));
    assert!(final_doc.contains("Keep this content."));
    assert!(final_doc.contains("New improved content."));

    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// SCENARIO: Frontmatter-aware editing — read metadata, modify
// body, verify frontmatter untouched through multiple mutations.
// ============================================================

#[test]
fn scenario_frontmatter_survives_body_edits() {
    let source = std::fs::read_to_string(DOC).unwrap();
    let tmp = tmpfile(&source);

    // Step 1: Read and cache frontmatter
    let fm_before = md_json(&["frontmatter", &tmp]);
    assert_eq!(fm_before["present"], true);
    let data_before = fm_before["frontmatter"]["data"].clone();

    // Step 2: Do several body mutations
    // Replace the Changelog section
    let output = md_stdin(
        &["replace-section", "Changelog", &tmp, "-i"],
        "## Changelog\n\n- v4: Replaced method with approach\n- v3: Added DELETE method\n- v2: Added pagination\n- v1: Initial release\n",
    );
    assert!(output.status.success());

    // Insert a note at the top (after frontmatter)
    let output = md_stdin(
        &["insert-block", "--at-start", &tmp, "-i"],
        "> **Note:** This document has been automatically updated.",
    );
    assert!(output.status.success());

    // Step 3: Verify frontmatter is byte-for-byte identical
    let fm_after = md_json(&["frontmatter", &tmp]);
    assert_eq!(fm_after["present"], true);
    assert_eq!(
        fm_after["frontmatter"]["data"], data_before,
        "frontmatter data should be unchanged after body edits"
    );

    // Step 4: Verify the frontmatter raw text is preserved
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(result.starts_with("---\n"), "file should still start with frontmatter");
    assert!(result.contains("title: API Documentation"));
    assert!(result.contains("version: 3"));

    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// SCENARIO: Cross-command consistency — every block from `blocks`
// can be individually read with `block` and the content matches.
// ============================================================

#[test]
fn scenario_blocks_and_block_read_are_consistent() {
    let source = std::fs::read_to_string(DOC).unwrap();
    let blocks = md_json(&["blocks", DOC]);

    for block in blocks["blocks"].as_array().unwrap() {
        let idx = block["index"].as_u64().unwrap();
        let span = &block["span"];
        let bs = span["byte_start"].as_u64().unwrap() as usize;
        let be = span["byte_end"].as_u64().unwrap() as usize;
        let expected_content = &source[bs..be];

        // Read the individual block
        let read = md_json(&["block", &idx.to_string(), DOC]);

        assert_eq!(
            read["block"]["kind"], block["kind"],
            "block {} kind mismatch between blocks and block commands",
            idx
        );
        assert_eq!(
            read["block"]["span"], block["span"],
            "block {} span mismatch between blocks and block commands",
            idx
        );
        assert_eq!(
            read["content"].as_str().unwrap(),
            expected_content,
            "block {} content from 'block' command doesn't match source slice from 'blocks' span",
            idx
        );
    }
}

// ============================================================
// SCENARIO: Section content matches concatenated block content.
// Reading a section should produce the same bytes as reading
// the source between section_span byte boundaries.
// ============================================================

#[test]
fn scenario_section_content_matches_block_union() {
    let source = std::fs::read_to_string(DOC).unwrap();

    // Read the "Endpoints" section (has subsections)
    let section = md_json(&["section", "Endpoints", DOC]);
    let section_content = section["content"].as_str().unwrap();
    let sec_bs = section["section"]["span"]["byte_start"].as_u64().unwrap() as usize;
    let sec_be = section["section"]["span"]["byte_end"].as_u64().unwrap() as usize;
    let source_slice = &source[sec_bs..sec_be];

    assert_eq!(
        section_content, source_slice,
        "section content should equal source slice at section span"
    );

    // The section should contain its heading plus all subsections
    assert!(section_content.contains("## Endpoints"));
    assert!(section_content.contains("### GET /users"));
    assert!(section_content.contains("### POST /users"));
    assert!(section_content.contains("### DELETE /users/:id"));

    // It should NOT contain the next sibling section
    assert!(!section_content.contains("## Error Handling"));
}

// ============================================================
// SCENARIO: Roundtrip — read then replace with same content → NoChange.
// Agent reads a section, "edits" it (no actual change), writes back.
// ============================================================

#[test]
fn scenario_roundtrip_nochange() {
    let source = std::fs::read_to_string(DOC).unwrap();
    let tmp = tmpfile(&source);

    // Step 1: Read the Authentication section
    let section_content = md_text(&["section", "Authentication", &tmp]);

    // Step 2: "Replace" with the same content
    let result = md_stdin_json(
        &["replace-section", "Authentication", &tmp, "-i"],
        &section_content,
    );

    assert_eq!(result["disposition"], "NoChange");
    assert_eq!(result["changed"], false);

    // Step 3: Verify file is byte-for-byte identical
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(source, after, "file should be unchanged after roundtrip");

    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// SCENARIO: Link audit — find all links, categorize by kind,
// verify destinations are reachable structure.
// ============================================================

#[test]
fn scenario_link_audit() {
    let links = md_json(&["links", DOC]);
    let link_list = links["links"].as_array().unwrap();

    // Categorize links by kind
    let mut inline = vec![];
    let mut reference = vec![];
    let mut autolink = vec![];

    for link in link_list {
        let kind = link["kind"].as_str().unwrap();
        let dest = link["destination"].as_str().unwrap_or("");
        match kind {
            "Inline" => inline.push(dest.to_string()),
            "Reference" => reference.push(dest.to_string()),
            "Autolink" => autolink.push(dest.to_string()),
            _ => {}
        }
    }

    // Verify expected links
    assert!(inline.iter().any(|l| l.contains("tools.ietf.org")), "should find RFC inline link");
    assert!(
        reference.iter().any(|l| l.contains("internal.example.com")),
        "should find internal docs reference link"
    );

    // Every link should have a valid block index
    let blocks = md_json(&["blocks", DOC]);
    let block_count = blocks["blocks"].as_array().unwrap().len() as u64;
    for link in link_list {
        let idx = link["source_block_index"].as_u64().unwrap();
        assert!(
            idx < block_count,
            "link source_block_index {} out of range ({})",
            idx,
            block_count
        );
    }

    // Verify link count matches stats
    let stats = md_json(&["stats", DOC]);
    assert_eq!(
        stats["stats"]["link_count"].as_u64().unwrap(),
        link_list.len() as u64,
        "stats link_count should match links command output"
    );
}

// ============================================================
// SCENARIO: Search-driven refactoring — find all occurrences
// of a term across specific block types, verify coverage.
// ============================================================

#[test]
fn scenario_search_driven_audit() {
    let source = std::fs::read_to_string(DOC).unwrap();

    // Step 1: Search for "method" across ALL block types
    let all_matches = md_json(&["search", "method", DOC]);
    let all_count = all_matches["matches"].as_array().unwrap().len();

    // Step 2: Search in paragraphs only
    let para_matches = md_json(&["search", "method", DOC, "--kind", "paragraph"]);
    let para_count = para_matches["matches"].as_array().unwrap().len();

    // Step 3: Search in headings only
    let heading_matches = md_json(&["search", "method", DOC, "--kind", "heading"]);
    let heading_count = heading_matches["matches"].as_array().unwrap().len();

    // Step 4: Search in lists only
    let list_matches = md_json(&["search", "method", DOC, "--kind", "list"]);
    let list_count = list_matches["matches"].as_array().unwrap().len();

    // Step 5: Search in code only
    let code_matches = md_json(&["search", "method", DOC, "--kind", "code-fence"]);
    let code_count = code_matches["matches"].as_array().unwrap().len();

    // The sum of all kind-filtered searches should cover all matches
    // (there might be kinds we haven't searched)
    assert!(
        para_count + heading_count + list_count + code_count <= all_count,
        "filtered sum {} should not exceed total {}",
        para_count + heading_count + list_count + code_count,
        all_count
    );
    assert!(para_count > 0, "should find method in paragraphs");

    // Step 6: Every match span should actually contain the query
    for m in all_matches["matches"].as_array().unwrap() {
        let bs = m["match_span"]["byte_start"].as_u64().unwrap() as usize;
        let be = m["match_span"]["byte_end"].as_u64().unwrap() as usize;
        let sliced = &source[bs..be];
        assert_eq!(
            sliced, "method",
            "match at [{},{}): {:?} != \"method\"",
            bs, be, sliced
        );
    }
}

// ============================================================
// SCENARIO: Case-insensitive section navigation.
// Agent doesn't know exact casing, uses --ignore-case.
// ============================================================

#[test]
fn scenario_case_insensitive_navigation() {
    // Step 1: Agent tries with wrong case → fails
    let output = md()
        .args(["section", "error handling", DOC])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "exact case should fail");

    // Step 2: Agent retries with --ignore-case → succeeds
    let content = md_text(&["section", "error handling", DOC, "--ignore-case"]);
    assert!(content.contains("## Error Handling"));
    assert!(content.contains("HTTP status codes"));

    // Step 3: Verify the content is the full section
    let json = md_json(&["section", "error handling", DOC, "--ignore-case"]);
    let block_indices = json["section"]["block_indices"].as_array().unwrap();
    assert!(block_indices.len() >= 2, "section should have heading + body blocks");
}
