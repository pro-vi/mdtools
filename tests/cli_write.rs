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

fn section_etag(path: &str, selector: &str, extra_args: &[&str]) -> String {
    let mut args = vec!["section", selector, path];
    args.extend_from_slice(extra_args);
    args.push("--json");
    let output = md().args(&args).output().unwrap();
    assert!(
        output.status.success(),
        "command {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json["section"]["etag"]
        .as_str()
        .expect("section JSON should expose an etag")
        .to_string()
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
    let output = md_with_stdin(&["replace-block", "1", &tmp, "-i"], "New content.");
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
    let output = md_with_stdin(&["insert-block", "tests/fixtures/basic.md"], "content");
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
fn replace_section_contains_with_occurrence() {
    let output = md_with_stdin(
        &[
            "replace-section",
            "method",
            "tests/fixtures/basic.md",
            "--contains",
            "--ignore-case",
            "--occurrence",
            "2",
        ],
        "### Sub-methods\n\nReplaced nested section.\n",
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("## Methods"));
    assert!(stdout.contains("Replaced nested section."));
    assert!(!stdout.contains("Some additional detail."));
}

#[test]
fn replace_section_contains_expect_etag_roundtrips_occurrence() {
    let content = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let path = tempfile(&content);
    let etag = section_etag(
        &path,
        "method",
        &["--contains", "--ignore-case", "--occurrence", "2"],
    );
    let output = md_with_stdin(
        &[
            "replace-section",
            "method",
            &path,
            "--contains",
            "--ignore-case",
            "--occurrence",
            "2",
            "-i",
            "--expect-etag",
            &etag,
        ],
        "### Sub-methods\n\nContains-selected replacement.\n",
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("Contains-selected replacement."));
    assert!(updated.contains("## Methods"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn delete_section_contains_deletes_decorated_heading_and_preserves_neighbors() {
    let path = tempfile(
        "# API Reference\n\n## Setup\n\nKeep setup instructions.\n\n## `DELETE /users/:id`\n\nRemove this endpoint.\n\n### Edge Cases\n\nNested details to remove.\n\n## Logging\n\nKeep logging instructions.\n",
    );
    let output = md()
        .args(["delete-section", "DELETE /users", &path, "--contains", "-i"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("## Setup"));
    assert!(updated.contains("Keep setup instructions."));
    assert!(updated.contains("## Logging"));
    assert!(updated.contains("Keep logging instructions."));
    assert!(!updated.contains("## `DELETE /users/:id`"));
    assert!(!updated.contains("Remove this endpoint."));
    assert!(!updated.contains("### Edge Cases"));
    assert!(!updated.contains("Nested details to remove."));
    std::fs::remove_file(&path).ok();
}

#[test]
fn delete_section_contains_rejects_preamble_selector() {
    let output = md()
        .args([
            "delete-section",
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
fn replace_section_contains_rejects_empty_selector_without_mutating_file() {
    let original = std::fs::read_to_string("tests/fixtures/table.md").unwrap();
    let path = tempfile(&original);
    let before = std::fs::read_to_string(&path).unwrap();
    let output = md_with_stdin(
        &["replace-section", "", &path, "--contains", "-i"],
        "## Replacement\n\nshould not apply\n",
    );
    assert_eq!(output.status.code(), Some(3));
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        before, after,
        "file must stay byte-identical on invalid empty --contains selector"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty selector"));
    assert!(stderr.contains("--contains"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_expect_etag_match_succeeds() {
    let content = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let path = tempfile(&content);
    let etag = section_etag(&path, "Discussion", &[]);
    let output = md_with_stdin(
        &[
            "replace-section",
            "Discussion",
            &path,
            "-i",
            "--expect-etag",
            &etag,
        ],
        "## Discussion\n\nUpdated discussion.\n",
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("Updated discussion."));
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_expect_etag_mismatch_fails_closed() {
    let content = std::fs::read_to_string("tests/fixtures/basic.md").unwrap();
    let path = tempfile(&content);
    let etag = section_etag(&path, "Discussion", &[]);
    let stale_source = std::fs::read_to_string(&path).unwrap();
    let fresh_source = stale_source.replace(
        "Final thoughts on the approach.",
        "Final thoughts after an intervening edit.",
    );
    std::fs::write(&path, &fresh_source).unwrap();
    let before = std::fs::read_to_string(&path).unwrap();
    let output = md_with_stdin(
        &[
            "replace-section",
            "Discussion",
            &path,
            "-i",
            "--expect-etag",
            &etag,
        ],
        "## Discussion\n\nShould not apply.\n",
    );
    assert_eq!(output.status.code(), Some(4));
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        before, after,
        "file must stay byte-identical on etag mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("etag"),
        "stderr should mention etag: {stderr}"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_expect_etag_roundtrips_duplicate_occurrence() {
    let content = std::fs::read_to_string("tests/fixtures/duplicate_headings.md").unwrap();
    let path = tempfile(&content);
    let etag = section_etag(&path, "Methods", &["--occurrence", "2"]);
    let output = md_with_stdin(
        &[
            "replace-section",
            "Methods",
            &path,
            "--occurrence",
            "2",
            "-i",
            "--expect-etag",
            &etag,
        ],
        "## Methods\n\nReplaced second.\n",
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("Replaced second."));
    assert!(updated.contains("First methods section"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn unicode_section_ignore_case_contains_replace_preserves_neighboring_bytes() {
    let path = tempfile(
        "# Doc\n\n## Setup\nkeep setup\n\n## API CAFÉ rollout\nold body\n\n### Nested\nold nested\n\n## Logging\nkeep logging\n",
    );
    let output = md_with_stdin(
        &[
            "replace-section",
            "café",
            &path,
            "--contains",
            "--ignore-case",
            "-i",
        ],
        "## API CAFÉ rollout\nnew body\n\n### Nested\nnew nested\n",
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        updated,
        "# Doc\n\n## Setup\nkeep setup\n\n## API CAFÉ rollout\nnew body\n\n### Nested\nnew nested\n\n## Logging\nkeep logging\n"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn unicode_section_ignore_case_duplicate_conflict_after_fold() {
    let path = tempfile("# Doc\n\n## CAFÉ\nfirst\n\n## Café\nsecond\n");
    let output = md_with_stdin(
        &["replace-section", "café", &path, "--ignore-case", "-i"],
        "## CAFÉ\nreplaced\n",
    );
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--occurrence"));
    let updated = std::fs::read_to_string(&path).unwrap();
    assert_eq!(updated, "# Doc\n\n## CAFÉ\nfirst\n\n## Café\nsecond\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn unicode_section_ignore_case_occurrence_selects_folded_duplicate() {
    let path = tempfile("# Doc\n\n## CAFÉ\nfirst\n\n## Café\nsecond\n");
    let output = md_with_stdin(
        &[
            "replace-section",
            "café",
            &path,
            "--ignore-case",
            "--occurrence",
            "2",
            "-i",
        ],
        "## Café\nsecond replaced\n",
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("## CAFÉ\nfirst"));
    assert!(updated.contains("## Café\nsecond replaced\n"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_lf_stdin_preserves_non_final_floor() {
    let path = tempfile("# Doc\n\n## One\nold\n\n## Two\nkeep\n");
    let output = md_with_stdin(&["replace-section", "One", &path], "## One\nnew\n");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\n\n## One\nnew\n\n## Two\nkeep\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_crlf_from_preserves_non_final_floor() {
    let path = tempfile("# Doc\r\n\r\n## One\r\nold\r\n\r\n## Two\r\nkeep\r\n");
    let replacement_path = tempfile("## One\nnew\n");
    let output = md()
        .args(["replace-section", "One", &path, "--from", &replacement_path])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout,
        "# Doc\r\n\r\n## One\r\nnew\r\n\r\n## Two\r\nkeep\r\n"
    );
    std::fs::remove_file(&path).ok();
    std::fs::remove_file(&replacement_path).ok();
}

#[test]
fn replace_section_boundary_stdin_and_from_match() {
    let source = "# Doc\n\n## One\nold\n\n## Two\nkeep\n";
    let stdin_path = tempfile(source);
    let from_path = tempfile(source);
    let replacement_path = tempfile("## One\nnew\n");
    let stdin_output = md_with_stdin(&["replace-section", "One", &stdin_path], "## One\nnew\n");
    let from_output = md()
        .args([
            "replace-section",
            "One",
            &from_path,
            "--from",
            &replacement_path,
        ])
        .output()
        .unwrap();
    assert!(stdin_output.status.success());
    assert!(
        from_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&from_output.stderr)
    );
    assert_eq!(stdin_output.stdout, from_output.stdout);
    std::fs::remove_file(&stdin_path).ok();
    std::fs::remove_file(&from_path).ok();
    std::fs::remove_file(&replacement_path).ok();
}

#[test]
fn replace_section_boundary_suffix_non_doubling() {
    let path = tempfile("# Doc\n\n## One\nold\n\n## Two\nkeep\n");
    let output = md_with_stdin(&["replace-section", "One", &path], "## One\nnew\n\n");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\n\n## One\nnew\n\n## Two\nkeep\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_preserves_extra_trailing_blank_lines() {
    let path = tempfile("# Doc\n\n## One\nold\n\n## Two\nkeep\n");
    let output = md_with_stdin(&["replace-section", "One", &path], "## One\nnew\n\n\n");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\n\n## One\nnew\n\n\n## Two\nkeep\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_preserves_adjacent_heading() {
    let path = tempfile("# Doc\n\n## One\nold\n## Two\nkeep\n");
    let output = md_with_stdin(&["replace-section", "One", &path], "## One\nnew");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\n\n## One\nnew\n## Two\nkeep\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_final_section_keeps_replacement_eof() {
    let path = tempfile("# Doc\n\n## One\nold\n\n## Two\nkeep\n");
    let output = md_with_stdin(&["replace-section", "Two", &path], "## Two\nnew");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\n\n## One\nold\n\n## Two\nnew");
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_empty_deletion_does_not_inject_suffix() {
    let path = tempfile("# Doc\n\n## One\nold\n\n## Two\nkeep\n");
    let output = md_with_stdin(&["replace-section", "One", &path], "");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\n\n## Two\nkeep\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn replace_section_boundary_mixed_reuses_existing_suffix_bytes() {
    let path = tempfile("# Doc\r\n\r\n## One\r\nold\r\n\n## Two\r\nkeep\r\n");
    let output = md_with_stdin(&["replace-section", "One", &path], "## One\r\nnew");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "# Doc\r\n\r\n## One\r\nnew\r\n\n## Two\r\nkeep\r\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn delete_section_expect_etag_match_succeeds_for_preamble() {
    let content = std::fs::read_to_string("tests/fixtures/preamble.md").unwrap();
    let path = tempfile(&content);
    let etag = section_etag(&path, ":preamble", &[]);
    let output = md()
        .args([
            "delete-section",
            ":preamble",
            &path,
            "-i",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("# First Heading"));
    assert!(!updated.contains("This is the preamble content before any headings."));
    std::fs::remove_file(&path).ok();
}

#[test]
fn delete_section_expect_etag_mismatch_fails_closed() {
    let content = std::fs::read_to_string("tests/fixtures/preamble.md").unwrap();
    let path = tempfile(&content);
    let etag = section_etag(&path, ":preamble", &[]);
    let stale_source = std::fs::read_to_string(&path).unwrap();
    let fresh_source = stale_source.replace(
        "This is the preamble content before any headings.",
        "This preamble changed after the original read.",
    );
    std::fs::write(&path, &fresh_source).unwrap();
    let before = std::fs::read_to_string(&path).unwrap();
    let output = md()
        .args([
            "delete-section",
            ":preamble",
            &path,
            "-i",
            "--expect-etag",
            &etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        before, after,
        "file must stay byte-identical on etag mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("etag"),
        "stderr should mention etag: {stderr}"
    );
    std::fs::remove_file(&path).ok();
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
        &[
            "insert-block",
            "--at-start",
            "tests/fixtures/frontmatter.md",
        ],
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
fn insert_block_json_reports_exact_payload_span() {
    let payload = "Inserted paragraph.";
    let output = md_with_stdin(
        &[
            "insert-block",
            "--after",
            "0",
            "tests/fixtures/basic.md",
            "--json",
        ],
        payload,
    );
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Inserted");
    assert_eq!(json["changed"], true);
    assert!(json["invariant"]["target_span_before"].is_null());
    let after = &json["invariant"]["target_span_after"];
    let content = json["content"].as_str().unwrap();
    let bs = after["byte_start"].as_u64().unwrap() as usize;
    let be = after["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&content[bs..be], payload);
    assert_eq!(after["line_start"], 2);
    assert_eq!(after["line_end"], 2);
    assert!(content[..bs].ends_with("# Introduction\n"));
    assert!(content[be..].starts_with("\n\n"));
}

#[test]
fn insert_block_empty_payload_is_exact_noop_across_output_modes() {
    let source = "# Title\n\nBody.";
    let tmp = tempfile(source);

    let text_stdout = md_with_stdin(&["insert-block", "--at-end", &tmp], "");
    assert!(text_stdout.status.success());
    assert_eq!(String::from_utf8(text_stdout.stdout).unwrap(), source);

    let json_stdout = md_with_stdin(&["insert-block", "--at-end", &tmp, "--json"], "");
    assert!(json_stdout.status.success());
    let json_stdout: serde_json::Value = serde_json::from_slice(&json_stdout.stdout).unwrap();
    assert_eq!(json_stdout["disposition"], "NoChange");
    assert_eq!(json_stdout["changed"], false);
    assert!(json_stdout["invariant"]["target_span_before"].is_null());
    assert!(json_stdout["invariant"]["target_span_after"].is_null());
    assert_eq!(json_stdout["content"].as_str().unwrap(), source);

    let in_place = md_with_stdin(&["insert-block", "--at-end", &tmp, "-i"], "");
    assert!(in_place.status.success());
    assert!(in_place.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), source);

    let in_place_json = md_with_stdin(&["insert-block", "--at-end", &tmp, "-i", "--json"], "");
    assert!(in_place_json.status.success());
    let in_place_json: serde_json::Value = serde_json::from_slice(&in_place_json.stdout).unwrap();
    assert_eq!(in_place_json["disposition"], "NoChange");
    assert_eq!(in_place_json["changed"], false);
    assert!(in_place_json["invariant"]["target_span_before"].is_null());
    assert!(in_place_json["invariant"]["target_span_after"].is_null());
    assert!(in_place_json["content"].is_null());
    assert_eq!(std::fs::read_to_string(&tmp).unwrap(), source);

    std::fs::remove_file(&tmp).unwrap();
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
