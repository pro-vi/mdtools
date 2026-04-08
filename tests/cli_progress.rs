//! Tests for progress_example.md — a complex real-world progress tracking document
//! with phases, task lists, code blocks, thematic breaks, and inline formatting.

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

fn tmpfile(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let id = CTR.fetch_add(1, Ordering::SeqCst);
    let p = format!("/tmp/mdtools_progress_{}_{}.md", std::process::id(), id);
    std::fs::write(&p, content).unwrap();
    p
}

const FIXTURE: &str = "tests/fixtures/progress_example.md";

// ── Outline ──────────────────────────────────────────────────

#[test]
fn progress_outline_heading_count() {
    let json = md_json(&["outline", FIXTURE]);
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 10);
}

#[test]
fn progress_outline_hierarchy() {
    let json = md_json(&["outline", FIXTURE]);
    let entries = json["entries"].as_array().unwrap();
    // First entry is h1
    assert_eq!(entries[0]["heading"]["level"], 1);
    assert_eq!(entries[0]["heading"]["text"], "Project Migration Progress");
    // All remaining are h2
    for entry in &entries[1..] {
        assert_eq!(entry["heading"]["level"], 2);
    }
}

#[test]
fn progress_outline_phase_names() {
    let json = md_json(&["outline", FIXTURE]);
    let entries = json["entries"].as_array().unwrap();
    let phase_names: Vec<&str> = entries
        .iter()
        .filter_map(|e| e["heading"]["text"].as_str())
        .collect();
    assert!(phase_names.contains(&"Composite Oracle (every iteration)"));
    assert!(phase_names.contains(&"Phase 0: Schema Compatibility (PROJ-100)"));
    assert!(phase_names.contains(&"Phase 1: DB Driver Abstraction (PROJ-101)"));
    assert!(phase_names.contains(&"Phase 2: Service Layer Extraction (PROJ-102)"));
    assert!(phase_names.contains(&"Phase 3: Provider Abstraction (PROJ-103)"));
    assert!(phase_names.contains(&"Phase 4: Auth Abstraction (PROJ-104)"));
    assert!(phase_names.contains(&"Phase 5: Desktop Shell (PROJ-105)"));
    assert!(phase_names.contains(&"Phase 6: Local Model Support (PROJ-106)"));
    assert!(phase_names.contains(&"Checkpoint Notes Template"));
}

#[test]
fn progress_outline_text() {
    let stdout = md_text(&["outline", FIXTURE]);
    // Text mode uses tab-separated format
    assert!(stdout.contains("# Project Migration Progress\t1-1\tblock:0"));
    assert!(stdout.contains("## Phase 0: Schema Compatibility (PROJ-100)\t19-19\tblock:7"));
}

// ── Blocks ───────────────────────────────────────────────────

#[test]
fn progress_blocks_count() {
    let json = md_json(&["blocks", FIXTURE]);
    let blocks = json["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 44);
}

#[test]
fn progress_blocks_kind_distribution() {
    let json = md_json(&["blocks", FIXTURE]);
    let blocks = json["blocks"].as_array().unwrap();
    let mut counts = std::collections::HashMap::new();
    for b in blocks {
        let kind = b["kind"].as_str().unwrap();
        *counts.entry(kind.to_string()).or_insert(0u32) += 1;
    }
    assert_eq!(counts["Heading"], 10);
    assert_eq!(counts["ThematicBreak"], 9);
    assert_eq!(counts["List"], 7); // one per phase (0-6)
    assert_eq!(counts["CodeFence"], 2); // oracle bash + checkpoint template
    assert_eq!(counts["Paragraph"], 16); // metadata paragraphs
}

#[test]
fn progress_blocks_indices_sequential() {
    let json = md_json(&["blocks", FIXTURE]);
    let blocks = json["blocks"].as_array().unwrap();
    for (i, b) in blocks.iter().enumerate() {
        assert_eq!(b["index"].as_u64().unwrap(), i as u64);
    }
}

// ── Block read ───────────────────────────────────────────────

#[test]
fn progress_block_read_heading() {
    let stdout = md_text(&["block", "0", FIXTURE]);
    assert_eq!(stdout.trim(), "# Project Migration Progress");
}

#[test]
fn progress_block_read_code_fence() {
    let stdout = md_text(&["block", "4", FIXTURE]);
    assert!(stdout.contains("```bash"));
    assert!(stdout.contains("check-all && test && build"));
}

#[test]
fn progress_block_read_task_list() {
    // Block 9 is the Phase 0 task list
    let json = md_json(&["block", "9", FIXTURE]);
    assert_eq!(json["block"]["kind"], "List");
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("[x] 0.1 App-side ID generation"));
    assert!(content.contains("[ ] 0.4 Remove collation overrides"));
}

#[test]
fn progress_block_read_thematic_break() {
    let stdout = md_text(&["block", "2", FIXTURE]);
    assert_eq!(stdout.trim(), "---");
}

// ── Section ──────────────────────────────────────────────────

#[test]
fn progress_section_phase0() {
    let json = md_json(&[
        "section",
        "Phase 0: Schema Compatibility (PROJ-100)",
        FIXTURE,
    ]);
    assert_eq!(json["section"]["heading"]["level"], 2);
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("**Goal:** Make schema dual-compatible"));
    assert!(content.contains("[x] 0.1 App-side ID generation"));
    assert!(content.contains("[ ] 0.4 Remove collation overrides"));
    assert!(content.contains("**Checkpoint:**"));
}

#[test]
fn progress_section_phase0_does_not_bleed() {
    let json = md_json(&[
        "section",
        "Phase 0: Schema Compatibility (PROJ-100)",
        FIXTURE,
    ]);
    let content = json["content"].as_str().unwrap();
    // Should not contain Phase 1 content
    assert!(!content.contains("Phase 1"));
    assert!(!content.contains("DB Driver Abstraction"));
}

#[test]
fn progress_section_last_phase() {
    let json = md_json(&[
        "section",
        "Phase 6: Local Model Support (PROJ-106)",
        FIXTURE,
    ]);
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("Local inference provider"));
    assert!(content.contains("6.5 End-to-end local flow"));
    // Should not contain Checkpoint Notes Template
    assert!(!content.contains("Checkpoint Notes Template"));
}

#[test]
fn progress_section_checkpoint_template() {
    let json = md_json(&["section", "Checkpoint Notes Template", FIXTURE]);
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("**Phase N Checkpoint:**"));
    assert!(content.contains("**Oracle Result:**"));
}

#[test]
fn progress_section_ignore_case() {
    let json = md_json(&[
        "section",
        "phase 0: schema compatibility (proj-100)",
        FIXTURE,
        "--ignore-case",
    ]);
    assert_eq!(
        json["section"]["heading"]["text"],
        "Phase 0: Schema Compatibility (PROJ-100)"
    );
}

// ── Search ───────────────────────────────────────────────────

#[test]
fn progress_search_oracle_base() {
    let json = md_json(&["search", "ORACLE_BASE", FIXTURE]);
    let matches = json["matches"].as_array().unwrap();
    // ORACLE_BASE appears in multiple blocks across the document
    assert!(matches.len() >= 3);
}

#[test]
fn progress_search_proj_tickets() {
    let json = md_json(&["search", "PROJ-", FIXTURE]);
    let matches = json["matches"].as_array().unwrap();
    // PROJ-100 through PROJ-106 = 7 tickets in headings
    assert!(matches.len() >= 7);
}

#[test]
fn progress_search_in_headings_only() {
    let json = md_json(&["search", "Phase", FIXTURE, "--kind", "heading"]);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 7); // Phase 0 through Phase 6
    for m in matches {
        assert_eq!(m["block_kind"], "Heading");
    }
}

#[test]
fn progress_search_in_lists_only() {
    let json = md_json(&["search", "E2E test page", FIXTURE, "--kind", "list"]);
    let matches = json["matches"].as_array().unwrap();
    // Each phase 1-6 has an E2E test page task in its list
    assert_eq!(matches.len(), 6);
}

#[test]
fn progress_search_ignore_case() {
    let json = md_json(&["search", "oracle_base", FIXTURE, "--ignore-case"]);
    let matches = json["matches"].as_array().unwrap();
    assert!(matches.len() >= 3);
}

#[test]
fn progress_search_code_fence_content() {
    let json = md_json(&["search", "check-all", FIXTURE, "--kind", "code-fence"]);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["block_kind"], "CodeFence");
}

// ── Stats ────────────────────────────────────────────────────

#[test]
fn progress_stats() {
    let json = md_json(&["stats", FIXTURE]);
    let stats = &json["stats"];
    assert_eq!(stats["heading_count"], 10);
    assert_eq!(stats["block_count"], 44);
    assert_eq!(stats["section_count"], 10);
    assert_eq!(stats["line_count"], 287);
    assert_eq!(stats["link_count"], 0);
    assert!(stats["word_count"].as_u64().unwrap() > 900);
}

// ── Frontmatter ──────────────────────────────────────────────

#[test]
fn progress_no_frontmatter() {
    let json = md_json(&["frontmatter", FIXTURE]);
    assert_eq!(json["present"], false);
    assert!(json["frontmatter"].is_null());
}

// ── Links ────────────────────────────────────────────────────

#[test]
fn progress_no_links() {
    let json = md_json(&["links", FIXTURE]);
    let links = json["links"].as_array().unwrap();
    assert_eq!(links.len(), 0);
}

// ── Mutations ────────────────────────────────────────────────

#[test]
fn progress_replace_section_roundtrip() {
    // Copy fixture, replace Phase 0, verify other phases untouched
    let content = std::fs::read_to_string(FIXTURE).unwrap();
    let path = tmpfile(&content);

    let replacement = "## Phase 0: Schema Compatibility (PROJ-100)\n\n**Status:** Complete\n";
    let output = md_stdin(
        &[
            "replace-section",
            "Phase 0: Schema Compatibility (PROJ-100)",
            &path,
            "-i",
            "--json",
        ],
        replacement,
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");

    // Verify replacement took effect
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("**Status:** Complete"));
    // Other phases still intact
    assert!(updated.contains("Phase 1: DB Driver Abstraction"));
    assert!(updated.contains("Phase 6: Local Model Support"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn progress_delete_thematic_break() {
    let content = std::fs::read_to_string(FIXTURE).unwrap();
    let path = tmpfile(&content);

    // Delete first thematic break (block 2)
    let output = md()
        .args(["delete-block", "2", &path, "-i", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Deleted");
    assert_eq!(json["target"]["Block"]["block_index"], 2);

    let updated = std::fs::read_to_string(&path).unwrap();
    // Document should have one fewer thematic break at the start
    // But all headings and content preserved
    assert!(updated.contains("# Project Migration Progress"));
    assert!(updated.contains("## Composite Oracle"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn progress_insert_block_after_heading() {
    let content = std::fs::read_to_string(FIXTURE).unwrap();
    let path = tmpfile(&content);

    // Insert a status banner after the h1 heading (block 0)
    let banner = "> **Last updated:** 2026-03-30\n";
    let output = md_stdin(
        &["insert-block", "--after", "0", &path, "-i", "--json"],
        banner,
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Inserted");

    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("**Last updated:** 2026-03-30"));
    // The banner should appear before the Status/Spec/Fidelity paragraph
    let banner_pos = updated.find("Last updated").unwrap();
    let status_pos = updated.find("**Status:** In Progress").unwrap();
    assert!(banner_pos < status_pos);

    std::fs::remove_file(&path).ok();
}

#[test]
fn progress_replace_code_block() {
    let content = std::fs::read_to_string(FIXTURE).unwrap();
    let path = tmpfile(&content);

    // Replace the oracle code block (block 4)
    let new_code = "```bash\ncheck-all && test && build && lint\n```\n";
    let output = md_stdin(&["replace-block", "4", &path, "-i", "--json"], new_code);
    assert!(output.status.success());

    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("lint"));
    assert!(!updated.contains("check-all && test && build\n```"));

    std::fs::remove_file(&path).ok();
}

// ── Scenario: count completed tasks per phase ────────────────

#[test]
fn progress_scenario_count_completed_tasks() {
    // An agent would: outline -> find phase sections -> read each list -> count [x]
    let outline = md_json(&["outline", FIXTURE]);
    let entries = outline["entries"].as_array().unwrap();

    let phase_headings: Vec<&str> = entries
        .iter()
        .filter_map(|e| {
            let text = e["heading"]["text"].as_str()?;
            if text.starts_with("Phase ") {
                Some(text)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(phase_headings.len(), 7);

    // Check Phase 0: 3 completed, 1 incomplete
    let phase0 = md_json(&[
        "section",
        "Phase 0: Schema Compatibility (PROJ-100)",
        FIXTURE,
    ]);
    let content = phase0["content"].as_str().unwrap();
    let completed = content.matches("[x]").count();
    let incomplete = content.matches("[ ]").count();
    assert_eq!(completed, 3);
    assert_eq!(incomplete, 1);
}

// ── Scenario: find all incomplete tasks ──────────────────────

#[test]
fn progress_scenario_search_incomplete_tasks() {
    let json = md_json(&["search", "[ ]", FIXTURE, "--kind", "list"]);
    let matches = json["matches"].as_array().unwrap();
    // Search returns one match per occurrence of "[ ]" across all list blocks
    // Phase 0: 1, Phase 1: 3, Phase 2: 4, Phase 3: 6, Phase 4: 4, Phase 5: 8, Phase 6: 5 = 31
    assert_eq!(matches.len(), 31);
}

// ── Scenario: extract oracle command for a phase ─────────────

#[test]
fn progress_scenario_extract_oracle() {
    // Phase 1 has a custom oracle: ORACLE_BASE + `DRIVER=b test-cmd run src/lib/db/`
    let phase1 = md_json(&[
        "section",
        "Phase 1: DB Driver Abstraction (PROJ-101)",
        FIXTURE,
    ]);
    let content = phase1["content"].as_str().unwrap();
    assert!(content.contains("DRIVER=b test-cmd run src/lib/db/"));
}
