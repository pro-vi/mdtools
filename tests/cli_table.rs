use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn md_with_stdin(args: &[&str], stdin_content: &str) -> std::process::Output {
    let mut child = md()
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

fn tempfile(content: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_table_test_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}

fn has_bare_lf(content: &str) -> bool {
    content
        .as_bytes()
        .iter()
        .enumerate()
        .any(|(idx, byte)| *byte == b'\n' && (idx == 0 || content.as_bytes()[idx - 1] != b'\r'))
}

// --- List tables ---

#[test]
fn table_list_text() {
    let out = md()
        .args(["table", "tests/fixtures/table.md"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert_eq!(json["tables"][0]["etag"].as_str().unwrap().len(), 16);
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
    assert_eq!(json["etag"].as_str().unwrap().len(), 16);
}

#[test]
fn table_list_and_read_json_share_whole_table_etag() {
    let list = md()
        .args(["table", "tests/fixtures/table.md", "--json"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let list_json: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    let list_etag = list_json["tables"][0]["etag"].as_str().unwrap().to_string();

    let read = md()
        .args(["table", "tests/fixtures/table.md", "--index", "1", "--json"])
        .output()
        .unwrap();
    assert!(read.status.success());
    let read_json: serde_json::Value = serde_json::from_slice(&read.stdout).unwrap();
    assert_eq!(read_json["etag"], list_etag);

    let filtered = md()
        .args([
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
            "--select",
            "Name",
            "--where",
            "Value=100",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(filtered.status.success());
    let filtered_json: serde_json::Value = serde_json::from_slice(&filtered.stdout).unwrap();
    assert_eq!(filtered_json["etag"], list_etag);
}

// --- Column projection ---

#[test]
fn table_select_by_name() {
    let out = md()
        .args([
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
            "--select",
            "Name",
        ])
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
        .args([
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
            "--select",
            "1",
        ])
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
        .args([
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
            "--select",
            "Value,Name",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_formatted.md",
            "--index",
            "1",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        json["alignments"],
        serde_json::json!(["Left", "Center", "Right"])
    );
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "1",
            "--where",
            "Priority=high",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "1",
            "--where",
            "Priority!=high",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "3",
            "--where",
            "Name~=l",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "1",
            "--where",
            "Priority=critical",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "3",
            "--select",
            "Name",
            "--where",
            "Name~=o",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "1",
            "--where",
            "Priority=medium",
            "--json",
        ])
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
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "1",
            "--where",
            "Nonexistent=val",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("not found"));
}

#[test]
fn table_where_invalid_syntax() {
    let out = md()
        .args([
            "table",
            "tests/fixtures/table_multi.md",
            "--index",
            "1",
            "--where",
            "bad filter",
        ])
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
        .args([
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
            "--where",
            "Name=Alpha",
            "--where",
            "Value=100",
        ])
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

// --- Operator tokens inside values (regression) ---

#[test]
fn table_where_value_contains_neq() {
    // "Expr=a!=b" should match the row where Expr column equals "a!=b"
    let out = md()
        .args([
            "table",
            "tests/fixtures/table_operators.md",
            "--index",
            "1",
            "--where",
            "Expr=a!=b",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2); // header + 1 row
    assert!(lines[1].starts_with("a!=b"));
}

#[test]
fn table_where_value_contains_tilde_eq() {
    // "Expr=a~=b" should match the row where Expr column equals "a~=b"
    let out = md()
        .args([
            "table",
            "tests/fixtures/table_operators.md",
            "--index",
            "1",
            "--where",
            "Expr=a~=b",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].starts_with("a~=b"));
}

#[test]
fn table_where_contains_value_with_neq() {
    // "Expr~=!=" should find rows where Expr contains "!="
    let out = md()
        .args([
            "table",
            "tests/fixtures/table_operators.md",
            "--index",
            "1",
            "--where",
            "Expr~=!=",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2); // header + "a!=b" row
    assert!(lines[1].starts_with("a!=b"));
}

#[test]
fn table_where_neq_value_with_eq() {
    // "Result!=false" should match rows where Result is not "false"
    let out = md()
        .args([
            "table",
            "tests/fixtures/table_operators.md",
            "--index",
            "1",
            "--where",
            "Result!=false",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3); // header + "true" rows (a!=b and x>y)
}

// --- replace-table-row ---

#[test]
fn replace_table_row_stdout_preserves_non_target_bytes() {
    let out = md_with_stdin(
        &["replace-table-row", "1", "1", "tests/fixtures/table.md"],
        "| Gamma | 300 |\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout,
        "# Data\n\n| Name | Value |\n|------|-------|\n| Alpha | 100 |\n| Gamma | 300 |\n\nSummary paragraph.\n"
    );
}

#[test]
fn replace_table_row_json_reports_typed_target() {
    let tmp = tempfile(include_str!("fixtures/table.md"));
    let out = md_with_stdin(
        &["replace-table-row", "1", "0", &tmp, "-i", "--json"],
        "| Gamma | 300 |\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["command"], "ReplaceTableRow");
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["target"]["TableRow"]["kind"], "TableRow");
    assert_eq!(json["target"]["TableRow"]["table_block_index"], 1);
    assert_eq!(json["target"]["TableRow"]["row_index"], 0);
    assert_eq!(json["target"]["TableRow"]["span"]["line_start"], 5);
    assert_eq!(json["invariant"]["preserves_non_target_bytes"], true);
    assert!(json["content"].is_null());

    let updated = std::fs::read_to_string(&tmp).unwrap();
    assert!(updated.contains("| Gamma | 300 |"));
    assert!(updated.contains("Summary paragraph."));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn replace_table_row_trailing_newline_noop_round_trips() {
    let tmp = tempfile(include_str!("fixtures/table.md"));
    let out = md_with_stdin(
        &["replace-table-row", "1", "0", &tmp, "-i", "--json"],
        "| Alpha | 100 |\n",
    );
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert_eq!(
        std::fs::read_to_string(&tmp).unwrap(),
        include_str!("fixtures/table.md")
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn replace_table_row_invalid_payloads_exit_3_without_writing() {
    let cases = [
        ("", "must not be empty"),
        ("| Alpha | 100 |\n| Beta | 200 |\n", "exactly one line"),
        ("not a row\n", "exactly one GFM table data row"),
        (
            "| Only one |\n",
            "column count 1 does not match table column count 2",
        ),
    ];

    for (payload, needle) in cases {
        let tmp = tempfile(include_str!("fixtures/table.md"));
        let out = md_with_stdin(&["replace-table-row", "1", "0", &tmp, "-i"], payload);
        assert_eq!(
            out.status.code(),
            Some(3),
            "payload {:?} should exit 3",
            payload
        );
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.contains(needle), "stderr was: {}", stderr);
        assert_eq!(
            std::fs::read_to_string(&tmp).unwrap(),
            include_str!("fixtures/table.md")
        );
        std::fs::remove_file(&tmp).unwrap();
    }
}

#[test]
fn replace_table_row_out_of_range_is_not_found() {
    let tmp = tempfile(include_str!("fixtures/table.md"));
    let out = md_with_stdin(
        &["replace-table-row", "1", "9", &tmp, "-i"],
        "| Gamma | 300 |\n",
    );
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("table row 9 out of range"));
    assert_eq!(
        std::fs::read_to_string(&tmp).unwrap(),
        include_str!("fixtures/table.md")
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn replace_table_row_expect_etag_conflict_leaves_bytes_unchanged() {
    let tmp = tempfile(include_str!("fixtures/table.md"));
    let read = md()
        .args(["table", &tmp, "--index", "1", "--json"])
        .output()
        .unwrap();
    assert!(read.status.success());
    let read_json: serde_json::Value = serde_json::from_slice(&read.stdout).unwrap();
    let etag = read_json["etag"].as_str().unwrap().to_string();

    let drifted =
        include_str!("fixtures/table.md").replace("| Beta | 200 |\n", "| Beta2 | 250 |\n");
    std::fs::write(&tmp, &drifted).unwrap();

    let out = md_with_stdin(
        &[
            "replace-table-row",
            "1",
            "0",
            &tmp,
            "-i",
            "--expect-etag",
            &etag,
        ],
        "| Gamma | 300 |\n",
    );
    assert_eq!(out.status.code(), Some(4));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("etag mismatch"));
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), drifted);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn replace_table_row_preserves_crlf_line_endings() {
    let tmp = tempfile(
        "# Data\r\n\r\n| Name | Value |\r\n|------|-------|\r\n| Alpha | 100 |\r\n| Beta | 200 |\r\n\r\nSummary paragraph.\r\n",
    );
    let out = md_with_stdin(
        &["replace-table-row", "1", "1", &tmp, "-i"],
        "| Gamma | 300 |\n",
    );
    assert!(out.status.success());
    let updated = std::fs::read_to_string(&tmp).unwrap();
    assert!(updated.contains("| Gamma | 300 |\r\n"));
    assert!(!has_bare_lf(&updated));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn replace_table_row_preserves_no_final_newline() {
    let tmp =
        tempfile("# Data\n\n| Name | Value |\n|------|-------|\n| Alpha | 100 |\n| Beta | 200 |");
    let out = md_with_stdin(
        &["replace-table-row", "1", "1", &tmp, "-i"],
        "| Gamma | 300 |\n",
    );
    assert!(out.status.success());
    let updated = std::fs::read_to_string(&tmp).unwrap();
    assert!(updated.ends_with("| Gamma | 300 |"));
    assert!(!updated.ends_with('\n'));
    std::fs::remove_file(&tmp).unwrap();
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
        .args([
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
            "--select",
            "Nonexistent",
        ])
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
