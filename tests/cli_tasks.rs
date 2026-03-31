//! Tests for `md tasks` and `md set-task` commands.

use std::process::Command;

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

fn tmpfile(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let id = CTR.fetch_add(1, Ordering::SeqCst);
    let p = format!("/tmp/mdtools_tasks_{}_{}.md", std::process::id(), id);
    std::fs::write(&p, content).unwrap();
    p
}

const PROGRESS: &str = "tests/fixtures/progress_example.md";
const NESTED: &str = "tests/fixtures/nested_tasks.md";
const CRLF_TASKS: &str = "tests/fixtures/crlf_tasks.md";

// ── JSON shape ───────────────────────────────────────────────

#[test]
fn tasks_json_envelope() {
    let json = md_json(&["tasks", PROGRESS]);
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert!(json["results"].is_array());
    assert_eq!(json["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["results"][0]["file"],
        "tests/fixtures/progress_example.md"
    );
    assert!(json["results"][0]["tasks"].is_array());
}

#[test]
fn tasks_entry_has_all_fields() {
    let json = md_json(&["tasks", PROGRESS]);
    let task = &json["results"][0]["tasks"][0];
    assert!(task["loc"].is_string());
    assert!(task["block_index"].is_u64());
    assert!(task["child_path"].is_array());
    assert!(task["task_index"].is_u64());
    assert!(task["status"].is_string());
    assert!(task["depth"].is_u64());
    assert!(task["span"].is_object());
    assert!(task["summary_text"].is_string());
    assert!(task["nearest_heading"].is_string());
    assert!(task["nearest_heading_block_index"].is_u64());
}

// ── Status filter ────────────────────────────────────────────

#[test]
fn tasks_status_pending_filter() {
    let json = md_json(&["tasks", PROGRESS, "--status", "pending"]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert!(!tasks.is_empty());
    for t in tasks {
        assert_eq!(t["status"], "pending");
    }
}

#[test]
fn tasks_status_done_filter() {
    let json = md_json(&["tasks", PROGRESS, "--status", "done"]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert!(!tasks.is_empty());
    for t in tasks {
        assert_eq!(t["status"], "done");
    }
}

// ── Task counts ──────────────────────────────────────────────

#[test]
fn tasks_phase0_counts() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    let phase0: Vec<_> = tasks
        .iter()
        .filter(|t| t["block_index"].as_u64().unwrap() == 9)
        .collect();
    assert_eq!(phase0.len(), 4);
    let done = phase0.iter().filter(|t| t["status"] == "done").count();
    let pending = phase0.iter().filter(|t| t["status"] == "pending").count();
    assert_eq!(done, 3);
    assert_eq!(pending, 1);
}

#[test]
fn tasks_total_count() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    // 7 phases, each with task items
    assert!(tasks.len() > 30);
}

// ── Field correctness ────────────────────────────────────────

#[test]
fn tasks_summary_text_no_checkbox() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    for t in tasks {
        let text = t["summary_text"].as_str().unwrap();
        assert!(!text.starts_with("[x]"), "summary_text has checkbox: {}", text);
        assert!(!text.starts_with("[ ]"), "summary_text has checkbox: {}", text);
    }
}

#[test]
fn tasks_nearest_heading_correct() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    // First task is in Phase 0
    let first = &tasks[0];
    assert_eq!(
        first["nearest_heading"],
        "Phase 0: Schema Compatibility (PROJ-100)"
    );
    assert_eq!(first["nearest_heading_block_index"], 7);
}

#[test]
fn tasks_loc_format() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    for t in tasks {
        let loc = t["loc"].as_str().unwrap();
        // Loc should be N.N format (at least two dot-separated parts)
        let parts: Vec<&str> = loc.split('.').collect();
        assert!(parts.len() >= 2, "bad loc format: {}", loc);
        for p in &parts {
            assert!(p.parse::<u32>().is_ok(), "non-numeric part in loc: {}", loc);
        }
    }
}

#[test]
fn tasks_depth_is_zero_for_flat() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    // progress_example.md has flat task lists only
    for t in tasks {
        assert_eq!(t["depth"], 0);
    }
}

#[test]
fn tasks_task_index_scoped_to_parent() {
    let json = md_json(&["tasks", PROGRESS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    // Phase 0 tasks should have task_index 0,1,2,3
    let phase0: Vec<_> = tasks
        .iter()
        .filter(|t| t["block_index"].as_u64().unwrap() == 9)
        .collect();
    for (i, t) in phase0.iter().enumerate() {
        assert_eq!(t["task_index"].as_u64().unwrap(), i as u64);
    }
}

// ── Text output ──────────────────────────────────────────────

#[test]
fn tasks_text_output_format() {
    let stdout = md_text(&["tasks", PROGRESS]);
    let first_line = stdout.lines().next().unwrap();
    // Tab-separated: LOC STATUS DEPTH LINES HEADING TEXT
    let parts: Vec<&str> = first_line.split('\t').collect();
    assert_eq!(parts.len(), 6, "expected 6 tab-separated fields, got: {:?}", parts);
    assert_eq!(parts[0], "9.0"); // loc
    assert_eq!(parts[1], "done"); // status
    assert_eq!(parts[2], "0"); // depth
}

// ── Nested tasks ─────────────────────────────────────────────

#[test]
fn nested_tasks_depth() {
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    // Find grandchild task
    let grandchild: Vec<_> = tasks.iter().filter(|t| t["depth"].as_u64().unwrap() == 2).collect();
    assert_eq!(grandchild.len(), 1);
    assert_eq!(grandchild[0]["summary_text"], "Grandchild task");
    assert_eq!(grandchild[0]["child_path"], serde_json::json!([1, 0, 0]));
    assert_eq!(grandchild[0]["loc"], "2.1.0.0");
}

#[test]
fn nested_tasks_parent_child_paths() {
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    // "1.1 Parent task" at [0], its children at [0,0] and [0,1]
    let parent = tasks.iter().find(|t| t["loc"] == "2.0").unwrap();
    assert_eq!(parent["depth"], 0);
    assert_eq!(parent["summary_text"], "1.1 Parent task");

    let child0 = tasks.iter().find(|t| t["loc"] == "2.0.0").unwrap();
    assert_eq!(child0["depth"], 1);
    assert_eq!(child0["summary_text"], "Research options");

    let child1 = tasks.iter().find(|t| t["loc"] == "2.0.1").unwrap();
    assert_eq!(child1["depth"], 1);
    assert_eq!(child1["summary_text"], "Write implementation");
}

#[test]
fn nested_tasks_mixed_list() {
    // Mixed list has non-task items interspersed
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    let mixed: Vec<_> = tasks
        .iter()
        .filter(|t| {
            t["nearest_heading"]
                .as_str()
                .map_or(false, |h| h == "Mixed List")
        })
        .collect();
    assert_eq!(mixed.len(), 2);
    // child_path indices skip non-task items
    assert_eq!(mixed[0]["loc"], "6.1");
    assert_eq!(mixed[0]["summary_text"], "Task in mixed list");
    assert_eq!(mixed[1]["loc"], "6.3");
    assert_eq!(mixed[1]["summary_text"], "Uppercase checked task");
}

#[test]
fn nested_tasks_uppercase_x() {
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    let upper = tasks
        .iter()
        .find(|t| t["summary_text"] == "Uppercase checked task")
        .unwrap();
    assert_eq!(upper["status"], "done");
}

#[test]
fn nested_tasks_multibyte() {
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    let mb = tasks
        .iter()
        .find(|t| {
            t["summary_text"]
                .as_str()
                .unwrap()
                .contains("müłtîbÿté")
        })
        .unwrap();
    assert_eq!(mb["status"], "pending");
}

#[test]
fn nested_tasks_frontmatter_present() {
    // Fixture has YAML frontmatter — tasks should still work
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert!(!tasks.is_empty());
}

#[test]
fn nested_tasks_ordered_list() {
    let json = md_json(&["tasks", NESTED]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    let ordered: Vec<_> = tasks
        .iter()
        .filter(|t| {
            t["nearest_heading"]
                .as_str()
                .map_or(false, |h| h == "Phase 2")
        })
        .collect();
    assert_eq!(ordered.len(), 3); // parent + child + second ordered
    assert_eq!(ordered[0]["summary_text"], "Ordered parent");
    assert_eq!(ordered[1]["summary_text"], "Ordered child");
    assert_eq!(ordered[2]["summary_text"], "Second ordered");
}

// ── No-task list ─────────────────────────────────────────────

#[test]
fn tasks_no_tasks_in_file() {
    let json = md_json(&["tasks", "tests/fixtures/basic.md"]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert!(tasks.is_empty());
}

// ── Mutation: set-task ───────────────────────────────────────

#[test]
fn set_task_pending_to_done() {
    let content = std::fs::read_to_string(PROGRESS).unwrap();
    let path = tmpfile(&content);
    let output = md()
        .args(["set-task", "9.3", &path, "-i", "--json", "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);

    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("[x] 0.4 Remove collation overrides"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn set_task_done_to_pending() {
    let content = std::fs::read_to_string(PROGRESS).unwrap();
    let path = tmpfile(&content);
    // 9.0 is "0.1 App-side ID generation" which is [x]
    let output = md()
        .args(["set-task", "9.0", &path, "-i", "--json", "--status", "pending"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");

    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("[ ] 0.1 App-side ID generation"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn set_task_idempotent() {
    let content = std::fs::read_to_string(PROGRESS).unwrap();
    let path = tmpfile(&content);
    // 9.0 is already done
    let output = md()
        .args(["set-task", "9.0", &path, "--json", "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    std::fs::remove_file(&path).ok();
}

#[test]
fn set_task_json_envelope() {
    let content = std::fs::read_to_string(PROGRESS).unwrap();
    let path = tmpfile(&content);
    let output = md()
        .args(["set-task", "9.3", &path, "--json", "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["command"], "SetTask");
    assert_eq!(json["target"]["TaskItem"]["loc"], "9.3");
    assert_eq!(json["target"]["TaskItem"]["block_index"], 9);
    std::fs::remove_file(&path).ok();
}

#[test]
fn set_task_stdout_without_inplace() {
    let output = md()
        .args(["set-task", "9.3", PROGRESS, "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output the full modified document
    assert!(stdout.contains("[x] 0.4 Remove collation overrides"));
    assert!(stdout.contains("# Project Migration Progress"));
}

#[test]
fn set_task_malformed_loc() {
    let output = md()
        .args(["set-task", "bad", PROGRESS, "--status", "done"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3)); // InvalidInput
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid task loc"));
}

#[test]
fn set_task_block_out_of_range() {
    let output = md()
        .args(["set-task", "999.0", PROGRESS, "--status", "done"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1)); // NotFound
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("out of range"));
}

#[test]
fn set_task_path_not_found() {
    let output = md()
        .args(["set-task", "9.99", PROGRESS, "--status", "done"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1)); // NotFound
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("task item not found"));
}

#[test]
fn set_task_not_a_task_list() {
    // Block 0 is a Heading, not a list
    let output = md()
        .args(["set-task", "0.0", PROGRESS, "--status", "done"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1)); // NotFound
}

// ── Roundtrip ────────────────────────────────────────────────

#[test]
fn set_task_roundtrip() {
    let content = std::fs::read_to_string(PROGRESS).unwrap();
    let path = tmpfile(&content);

    // Mark 9.3 as done
    let output = md()
        .args(["set-task", "9.3", &path, "-i", "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Re-read tasks and verify
    let json = md_json(&["tasks", &path, "--json"]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    let task93 = tasks.iter().find(|t| t["loc"] == "9.3").unwrap();
    assert_eq!(task93["status"], "done");

    std::fs::remove_file(&path).ok();
}

// ── Blockquote / callout tasks ────────────────────────────────

#[test]
fn tasks_inside_blockquote() {
    let content = "> [!TODO]\n> - [ ] Quoted task\n> - [x] Done quoted\n\n- [ ] Normal task\n";
    let path = tmpfile(content);
    let json = md_json(&["tasks", &path]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3);

    // Blockquote tasks are block 0
    let bq_tasks: Vec<_> = tasks.iter().filter(|t| t["block_index"] == 0).collect();
    assert_eq!(bq_tasks.len(), 2);
    assert_eq!(bq_tasks[0]["summary_text"], "Quoted task");
    assert_eq!(bq_tasks[0]["status"], "pending");
    assert_eq!(bq_tasks[1]["summary_text"], "Done quoted");
    assert_eq!(bq_tasks[1]["status"], "done");

    // Normal task is block 1
    assert_eq!(tasks[2]["block_index"], 1);
    std::fs::remove_file(&path).ok();
}

#[test]
fn tasks_sibling_lists_in_blockquote() {
    // Two separate lists inside one blockquote must get distinct locs
    let content = "> - [ ] first\n>\n> Some text\n>\n> - [ ] second\n";
    let path = tmpfile(content);
    let json = md_json(&["tasks", &path]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    // Must have different locs
    let loc0 = tasks[0]["loc"].as_str().unwrap();
    let loc1 = tasks[1]["loc"].as_str().unwrap();
    assert_ne!(loc0, loc1, "sibling lists in blockquote must have distinct locs");
    assert_eq!(tasks[0]["summary_text"], "first");
    assert_eq!(tasks[1]["summary_text"], "second");
    std::fs::remove_file(&path).ok();
}

#[test]
fn set_task_inside_blockquote() {
    let content = "> - [ ] Quoted pending\n> - [x] Quoted done\n";
    let path = tmpfile(content);
    let output = md()
        .args(["set-task", "0.0.0", &path, "-i", "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("[x] Quoted pending"));
    std::fs::remove_file(&path).ok();
}

// ── Multi-file error resilience ──────────────────────────────

#[test]
fn tasks_multifile_json_partial_results() {
    // Create one good file and one bad file
    let good = tmpfile("- [ ] Task A\n- [x] Task B\n");
    let bad = format!("/tmp/mdtools_tasks_nonexistent_{}.md", std::process::id());

    let output = md()
        .args(["tasks", &good, &bad, "--json"])
        .output()
        .unwrap();
    // Should fail (bad file) but still produce JSON with good file results
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let results = json["results"].as_array().unwrap();
        // Good file's results should be present
        assert_eq!(results.len(), 1);
        assert!(!results[0]["tasks"].as_array().unwrap().is_empty());
    }
    std::fs::remove_file(&good).ok();
}

// ── CRLF ─────────────────────────────────────────────────────

#[test]
fn tasks_crlf_reads_correctly() {
    let json = md_json(&["tasks", CRLF_TASKS]);
    let tasks = json["results"][0]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0]["status"], "done");
    assert_eq!(tasks[0]["summary_text"], "Done task");
    assert_eq!(tasks[1]["status"], "pending");
    assert_eq!(tasks[2]["depth"], 1); // nested sub-task
}

#[test]
fn set_task_crlf_mutation() {
    let content = std::fs::read(CRLF_TASKS).unwrap();
    let path = tmpfile(&String::from_utf8_lossy(&content));
    // Overwrite with exact CRLF bytes
    std::fs::write(&path, &content).unwrap();

    let output = md()
        .args(["set-task", "1.1", &path, "-i", "--status", "done"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let updated = std::fs::read(&path).unwrap();
    // Should have [x] now, and CRLF should be preserved
    assert!(updated.windows(3).any(|w| w == b"[x]"));
    // Verify CRLF preserved (look for \r\n)
    assert!(updated.windows(2).any(|w| w == b"\r\n"));

    std::fs::remove_file(&path).ok();
}

// ── Nested mutation ──────────────────────────────────────────

#[test]
fn set_task_nested_grandchild() {
    let content = std::fs::read_to_string(NESTED).unwrap();
    let path = tmpfile(&content);

    // Mark the grandchild (2.1.0.0) as done
    let output = md()
        .args(["set-task", "2.1.0.0", &path, "-i", "--json", "--status", "done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");

    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("[x] Grandchild task"));

    std::fs::remove_file(&path).ok();
}
