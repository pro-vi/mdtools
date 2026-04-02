use std::process::Command;

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

// --- List tables ---

#[test]
fn table_list_text() {
    let out = md()
        .args(["table", "tests/fixtures/table.md"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Name, Value"));
    assert!(stdout.contains("2 rows"));
    assert!(stdout.contains("2 cols"));
}

#[test]
fn table_list_json() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["tables"][0]["headers"][0], "Name");
    assert_eq!(json["tables"][0]["row_count"], 2);
    assert_eq!(json["tables"][0]["column_count"], 2);
}

// --- Read table ---

#[test]
fn table_read_tsv() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "Name\tValue");
    assert_eq!(lines[1], "Alpha\t100");
    assert_eq!(lines[2], "Beta\t200");
}

#[test]
fn table_read_json() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["headers"], serde_json::json!(["Name", "Value"]));
    assert_eq!(json["rows"][0], serde_json::json!(["Alpha", "100"]));
    assert_eq!(json["rows"][1], serde_json::json!(["Beta", "200"]));
    assert_eq!(json["block_index"], 1);
}

// --- Column projection ---

#[test]
fn table_select_by_name() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1", "--select", "Name"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "Name");
    assert_eq!(lines[1], "Alpha");
    assert!(!lines[1].contains('\t'));
}

#[test]
fn table_select_by_index() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1", "--select", "1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "Value");
    assert_eq!(lines[1], "100");
}

#[test]
fn table_select_reorder() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1", "--select", "Value,Name"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "Value\tName");
    assert_eq!(lines[1], "100\tAlpha");
}

// --- Inline formatting stripped ---

#[test]
fn table_strips_formatting() {
    let out = md()
        .args(["table", "tests/fixtures/table_formatted.md", "--index", "1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    // Bold stripped
    assert!(lines[1].starts_with("Bold\t"));
    // Code span stripped
    assert!(lines[1].contains("done"));
    // Link stripped to text
    assert!(lines[1].contains("link"));
    assert!(!lines[1].contains("http"));
}

#[test]
fn table_formatted_alignments() {
    let out = md()
        .args(["table", "tests/fixtures/table_formatted.md", "--index", "1", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["alignments"], serde_json::json!(["Left", "Center", "Right"]));
}

// --- Multiple tables ---

#[test]
fn table_list_multiple() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Task, Priority"));
    assert!(lines[1].contains("Name, Email"));
}

#[test]
fn table_select_specific_from_multi() {
    // Select second table (Contacts) by its block index
    // First find the block index
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let second_idx = json["tables"][1]["block_index"].as_u64().unwrap();

    let out = md()
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            &second_idx.to_string(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("Name\tEmail"));
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Carol"));
}

// --- Row filtering (--where) ---

#[test]
fn table_where_eq() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "1", "--where", "Priority=high"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2); // header + 1 row
    assert_eq!(lines[0], "Task\tPriority");
    assert_eq!(lines[1], "Deploy\thigh");
}

#[test]
fn table_where_neq() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "1", "--where", "Priority!=high"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2); // header + 1 row
    assert!(lines[1].starts_with("Review"));
}

#[test]
fn table_where_contains() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "3", "--where", "Name~=l"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3); // header + Alice + Carol
    assert!(lines[1].starts_with("Alice"));
    assert!(lines[2].starts_with("Carol"));
}

#[test]
fn table_where_no_matches() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "1", "--where", "Priority=critical"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1); // header only
}

#[test]
fn table_where_with_select() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "3",
               "--select", "Name", "--where", "Name~=o"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "Name");
    assert_eq!(lines.len(), 3); // header + Bob + Carol
}

#[test]
fn table_where_json() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "1",
               "--where", "Priority=medium", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], "Review");
}

#[test]
fn table_where_invalid_column() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "1",
               "--where", "Nonexistent=val"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("not found"));
}

#[test]
fn table_where_invalid_syntax() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--index", "1",
               "--where", "bad filter"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("invalid filter"));
}

#[test]
fn table_where_multiple_filters() {
    // Multiple --where flags act as AND
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1",
               "--where", "Name=Alpha", "--where", "Value=100"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2); // header + 1 matching row
    assert!(lines[1].starts_with("Alpha"));
}

#[test]
fn table_where_single_table_no_index() {
    // Single-table doc: --where works without --index
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--where", "Name=Alpha"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].starts_with("Alpha"));
}

// --- Error cases ---

#[test]
fn table_no_tables_error() {
    let out = md()
        .args(["table", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap(), 1);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("no tables"));
}

#[test]
fn table_not_a_table_error() {
    // Block 0 in table.md is a heading, not a table
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "0"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap(), 1);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("not a table"));
}

#[test]
fn table_index_out_of_range() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "99"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap(), 1);
}

#[test]
fn table_column_not_found() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1", "--select", "Nonexistent"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap(), 3);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Name, Value"));
}

#[test]
fn table_select_without_index_multi_error() {
    let out = md()
        .args(["table", "tests/fixtures/table_multi.md", "--select", "Name"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("--index"));
}

// --- Single table implicit select ---

#[test]
fn table_select_without_index_single_table() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--select", "Name"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().next().unwrap(), "Name");
}

// --- TSV format compliance ---

#[test]
fn table_tsv_consistent_columns() {
    let out = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let tab_counts: Vec<usize> = stdout.lines().map(|l| l.matches('\t').count()).collect();
    // All lines should have the same number of tabs
    assert!(tab_counts.windows(2).all(|w| w[0] == w[1]));
}
