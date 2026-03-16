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

#[test]
fn replace_block_stdout() {
    let output = md_with_stdin(
        &["replace-block", "1", "tests/fixtures/basic.md"],
        "Replaced paragraph content.",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Introduction"));
    assert!(stdout.contains("Replaced paragraph content."));
    assert!(!stdout.contains("This is the opening paragraph."));
}

#[test]
fn replace_block_in_place() {
    let tmp = tempfile("# Hello\n\nOriginal.\n");
    let output = md_with_stdin(
        &["replace-block", "1", &tmp, "-i"],
        "New content.",
    );
    assert!(output.status.success());
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(result.contains("New content."));
    assert!(!result.contains("Original."));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn replace_block_json() {
    let output = md_with_stdin(
        &["replace-block", "1", "tests/fixtures/basic.md", "--json"],
        "New paragraph.",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["command"], "ReplaceBlock");
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);
    assert!(json["content"].as_str().unwrap().contains("New paragraph."));
}

#[test]
fn delete_block_stdout() {
    let output = md()
        .args(["delete-block", "1", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Introduction"));
    assert!(!stdout.contains("This is the opening paragraph."));
    assert!(stdout.contains("## Methods"));
}

#[test]
fn delete_block_in_place() {
    let tmp = tempfile("# Hello\n\nTo delete.\n\nKeep this.\n");
    let output = md()
        .args(["delete-block", "1", &tmp, "-i"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(!result.contains("To delete."));
    assert!(result.contains("Keep this."));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn insert_block_after() {
    let output = md_with_stdin(
        &["insert-block", "--after", "0", "tests/fixtures/basic.md"],
        "Inserted paragraph.",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should appear after "# Introduction" and before the original paragraph
    let intro_pos = stdout.find("# Introduction").unwrap();
    let inserted_pos = stdout.find("Inserted paragraph.").unwrap();
    let original_pos = stdout.find("This is the opening paragraph.").unwrap();
    assert!(intro_pos < inserted_pos);
    assert!(inserted_pos < original_pos);
}

#[test]
fn insert_block_before() {
    let output = md_with_stdin(
        &["insert-block", "--before", "0", "tests/fixtures/basic.md"],
        "Before everything.",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let before_pos = stdout.find("Before everything.").unwrap();
    let intro_pos = stdout.find("# Introduction").unwrap();
    assert!(before_pos < intro_pos);
}

#[test]
fn insert_block_at_end() {
    let output = md_with_stdin(
        &["insert-block", "--at-end", "tests/fixtures/basic.md"],
        "Appended content.",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim_end().ends_with("Appended content."));
}

#[test]
fn insert_block_at_start() {
    let output = md_with_stdin(
        &["insert-block", "--at-start", "tests/fixtures/basic.md"],
        "Prepended content.",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("Prepended content."));
}

#[test]
fn insert_block_no_location_flag() {
    let output = md_with_stdin(
        &["insert-block", "tests/fixtures/basic.md"],
        "content",
    );
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn replace_section_stdout() {
    let output = md_with_stdin(
        &["replace-section", "Discussion", "tests/fixtures/basic.md"],
        "## Discussion\n\nReplaced discussion.\n",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Replaced discussion."));
    assert!(!stdout.contains("Final thoughts on the approach."));
}

#[test]
fn replace_section_with_occurrence() {
    let output = md_with_stdin(
        &[
            "replace-section",
            "Methods",
            "tests/fixtures/duplicate_headings.md",
            "--occurrence",
            "1",
        ],
        "## Methods\n\nReplaced first.\n",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Replaced first."));
    assert!(stdout.contains("Second methods section"));
}

#[test]
fn insert_block_in_place_json() {
    let tmp = tempfile("# Title\n\nContent.\n");
    let output = md_with_stdin(
        &["insert-block", "--at-end", &tmp, "-i", "--json"],
        "Appended.",
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "InsertBlock");
    assert_eq!(json["disposition"], "Inserted");
    assert!(json["content"].is_null()); // in-place: content is null
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(result.contains("Appended."));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn insert_block_at_start_after_frontmatter() {
    let output = md_with_stdin(
        &["insert-block", "--at-start", "tests/fixtures/frontmatter.md"],
        "Inserted after frontmatter.",
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Frontmatter should still be at the top
    assert!(stdout.starts_with("---\n"));
    // Inserted content should come before the heading
    let inserted_pos = stdout.find("Inserted after frontmatter.").unwrap();
    let heading_pos = stdout.find("# Main Content").unwrap();
    assert!(inserted_pos < heading_pos);
}

#[test]
fn delete_last_block() {
    let tmp = tempfile("# Hello\n\nOnly paragraph.\n");
    let output = md()
        .args(["delete-block", "1", &tmp, "-i"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let result = std::fs::read_to_string(&tmp).unwrap();
    assert!(result.contains("# Hello"));
    assert!(!result.contains("Only paragraph."));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn delete_last_block_json() {
    let output = md()
        .args(["delete-block", "10", "tests/fixtures/basic.md", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "DeleteBlock");
    assert_eq!(json["disposition"], "Deleted");
    assert!(json["invariant"]["target_span_after"].is_null());
}

fn tempfile(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_test_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}
