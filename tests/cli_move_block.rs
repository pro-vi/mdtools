use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn tempfile_str(content: &str) -> String {
    tempfile_bytes(content.as_bytes())
}

fn tempfile_bytes(bytes: &[u8]) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_move_block_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[cfg(unix)]
fn inode(path: &str) -> u64 {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(path).unwrap().ino()
}

fn blocks_json(path: &str) -> serde_json::Value {
    let output = md().args(["blocks", path, "--json"]).output().unwrap();
    assert!(
        output.status.success(),
        "blocks failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn block_etag(path: &str, index: u32) -> String {
    let json = blocks_json(path);
    json["blocks"][index as usize]["etag"]
        .as_str()
        .unwrap()
        .to_string()
}

fn expected_permutation(source: &[u8], blocks: &serde_json::Value, order: &[u32]) -> Vec<u8> {
    let block_entries = blocks["blocks"].as_array().unwrap();
    let prefix_end = block_entries
        .first()
        .map(|block| block["span"]["byte_start"].as_u64().unwrap() as usize)
        .unwrap_or(source.len());
    let mut gap_slots = Vec::new();
    for (index, block) in block_entries.iter().enumerate() {
        let gap_start = block["span"]["byte_end"].as_u64().unwrap() as usize;
        let gap_end = block_entries
            .get(index + 1)
            .map(|next| next["span"]["byte_start"].as_u64().unwrap() as usize)
            .unwrap_or(source.len());
        gap_slots.push((gap_start, gap_end));
    }

    let mut expected = Vec::with_capacity(source.len());
    expected.extend_from_slice(&source[..prefix_end]);
    for (slot_index, &block_index) in order.iter().enumerate() {
        let block = &block_entries[block_index as usize];
        let block_start = block["span"]["byte_start"].as_u64().unwrap() as usize;
        let block_end = block["span"]["byte_end"].as_u64().unwrap() as usize;
        expected.extend_from_slice(&source[block_start..block_end]);
        let (gap_start, gap_end) = gap_slots[slot_index];
        expected.extend_from_slice(&source[gap_start..gap_end]);
    }
    expected
}

fn move_order(block_count: usize, source_index: u32, dest_index: u32, mode: &str) -> Vec<u32> {
    let mut order: Vec<u32> = (0..block_count as u32).collect();
    let moved = order.remove(source_index as usize);
    let dest_position = order.iter().position(|&index| index == dest_index).unwrap();
    let insert_position = if mode == "before" {
        dest_position
    } else {
        dest_position + 1
    };
    order.insert(insert_position, moved);
    order
}

#[test]
fn move_block_json_and_text_surfaces_report_typed_relocation() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n\ngamma\n");
    let original = std::fs::read(&path).unwrap();
    let blocks = blocks_json(&path);
    let order = move_order(4, 3, 1, "before");
    let expected = expected_permutation(&original, &blocks, &order);

    let json_output = md()
        .args(["move-block", "3", &path, "--before", "1", "--json"])
        .output()
        .unwrap();
    assert!(
        json_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["command"], "MoveBlock");
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);
    assert_eq!(json["guarded"], false);
    let target = &json["target"]["BlockMove"];
    assert_eq!(target["destination_mode"], "before");
    assert_eq!(target["source"]["block_index"], 3);
    assert_eq!(target["destination"]["block_index"], 1);
    assert_eq!(
        json["invariant"]["target_span_before"], target["source"]["span"],
        "before span should be the original source block span"
    );
    let content = json["content"].as_str().unwrap().as_bytes();
    assert_eq!(content, expected.as_slice());

    let source_span = &target["source"]["span"];
    let source_start = source_span["byte_start"].as_u64().unwrap() as usize;
    let source_end = source_span["byte_end"].as_u64().unwrap() as usize;
    let moved_slice = &original[source_start..source_end];
    let after_span = &json["invariant"]["target_span_after"];
    let after_start = after_span["byte_start"].as_u64().unwrap() as usize;
    let after_end = after_span["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&content[after_start..after_end], moved_slice);

    let text_output = md()
        .args(["move-block", "3", &path, "--before", "1"])
        .output()
        .unwrap();
    assert!(text_output.status.success());
    assert_eq!(text_output.stdout, expected);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_after_has_literal_expected_text() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n\ngamma\n");
    let output = md()
        .args(["move-block", "3", &path, "--after", "1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "# Doc\n\nalpha\n\ngamma\n\nbeta\n"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_adjacent_noops_report_nochange_and_skip_writes() {
    for (source_index, flag, dest_index) in [("1", "--after", "0"), ("0", "--before", "1")] {
        let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n");
        let before = std::fs::read_to_string(&path).unwrap();
        let output = md()
            .args([
                "move-block",
                source_index,
                &path,
                flag,
                dest_index,
                "-i",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["command"], "MoveBlock");
        assert_eq!(json["disposition"], "NoChange");
        assert_eq!(json["changed"], false);
        assert_eq!(
            json["invariant"]["target_span_before"],
            json["invariant"]["target_span_after"]
        );
        assert!(json["content"].is_null());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
        std::fs::remove_file(&path).ok();
    }
}

#[test]
fn move_block_duplicate_permutation_stdout_json_reports_nochange() {
    let original = "# Doc\n\nsame\n\nsame\n\nsame\n\n## Tail\n";
    let path = tempfile_str(original);
    let output = md()
        .args(["move-block", "3", &path, "--before", "1", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "MoveBlock");
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert_eq!(json["guarded"], false);
    assert_eq!(json["content"], original);
    assert_eq!(
        json["invariant"]["target_span_before"],
        json["invariant"]["target_span_after"]
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_duplicate_permutation_in_place_json_skips_atomic_write() {
    let original = "# Doc\n\nsame\n\nsame\n\nsame\n\n## Tail\n";
    let path = tempfile_str(original);
    let before_bytes = std::fs::read(&path).unwrap();
    #[cfg(unix)]
    let before_inode = inode(&path);

    let output = md()
        .args(["move-block", "3", &path, "--before", "1", "-i", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "MoveBlock");
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    assert_eq!(json["guarded"], false);
    assert!(json["content"].is_null());
    assert_eq!(
        json["invariant"]["target_span_before"],
        json["invariant"]["target_span_after"]
    );
    let after_bytes = std::fs::read(&path).unwrap();
    assert_eq!(after_bytes, before_bytes);
    #[cfg(unix)]
    assert_eq!(inode(&path), before_inode);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_crlf_code_fence_has_literal_expected_bytes() {
    let path =
        tempfile_str("# Doc\r\n\r\npara\r\n\r\n```rust\r\nfn main() {}\r\n```\r\n\r\n## Tail\r\n");
    let output = md()
        .args(["move-block", "2", &path, "--before", "1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        b"# Doc\r\n\r\n```rust\r\nfn main() {}\r\n```\r\n\r\npara\r\n\r\n## Tail\r\n"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_no_final_newline_has_literal_expected_bytes() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta");
    let output = md()
        .args(["move-block", "2", &path, "--before", "1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"# Doc\n\nbeta\n\nalpha");
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_is_exact_permutation_across_required_edge_cases() {
    struct Case {
        label: &'static str,
        source: &'static str,
        source_index: u32,
        dest_index: u32,
        mode: &'static str,
    }

    let cases = [
        Case {
            label: "frontmatter_utf8_lf",
            source: "---\ntitle: caf\u{e9}\n---\n\n# One\n\nalpha ☕\n\n## Two\n\nbeta\n",
            source_index: 1,
            dest_index: 3,
            mode: "after",
        },
        Case {
            label: "mixed_list",
            source: "# Doc\r\n\r\n- one\n- two\n\r\nplain\n\n## Tail\r\n",
            source_index: 1,
            dest_index: 3,
            mode: "after",
        },
        Case {
            label: "indented_heading",
            source: "# Doc\n\nbody\n\n  ## Indented\n\ntext\n",
            source_index: 2,
            dest_index: 1,
            mode: "before",
        },
    ];

    for case in cases {
        let path = tempfile_str(case.source);
        let original = std::fs::read(&path).unwrap();
        let blocks = blocks_json(&path);
        let block_count = blocks["blocks"].as_array().unwrap().len();
        let order = move_order(block_count, case.source_index, case.dest_index, case.mode);
        let expected = expected_permutation(&original, &blocks, &order);

        let flag = if case.mode == "before" {
            "--before"
        } else {
            "--after"
        };
        let output = md()
            .args([
                "move-block",
                &case.source_index.to_string(),
                &path,
                flag,
                &case.dest_index.to_string(),
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} stderr: {}",
            case.label,
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected, "{}", case.label);
        std::fs::remove_file(&path).ok();
    }
}

#[test]
fn move_block_fails_closed_when_reparse_would_change_block_structure() {
    let original = "# Doc\n\n- item\n\n## Anchor\nTarget\n------\n";
    let path = tempfile_str(original);
    let output = md()
        .args(["move-block", "1", &path, "--after", "2"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(
        output.stdout.is_empty(),
        "stdout should stay empty on structural-closure failure"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parsed top-level block sequence"),
        "{stderr}"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_rejects_same_source_and_destination_index_without_writing() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n");
    let before = std::fs::read_to_string(&path).unwrap();
    let output = md()
        .args(["move-block", "1", &path, "--before", "1", "-i"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_source_out_of_range_fails_without_writing() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n");
    let before = std::fs::read_to_string(&path).unwrap();
    let output = md()
        .args(["move-block", "9", &path, "--before", "1", "-i"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_destination_out_of_range_fails_without_writing() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n");
    let before = std::fs::read_to_string(&path).unwrap();
    let output = md()
        .args(["move-block", "1", &path, "--before", "9", "-i"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_requires_exactly_one_destination_flag() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n");
    let before = std::fs::read_to_string(&path).unwrap();

    let missing_destination = md().args(["move-block", "1", &path]).output().unwrap();
    assert_eq!(missing_destination.status.code(), Some(2));
    assert!(missing_destination.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);

    let conflicting_destination = md()
        .args(["move-block", "1", &path, "--before", "0", "--after", "0"])
        .output()
        .unwrap();
    assert_eq!(conflicting_destination.status.code(), Some(2));
    assert!(conflicting_destination.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);

    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_matching_guards_allow_successful_in_place_write() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n\ngamma\n");
    let source_etag = block_etag(&path, 3);
    let dest_etag = block_etag(&path, 1);
    let output = md()
        .args([
            "move-block",
            "3",
            &path,
            "--before",
            "1",
            "-i",
            "--json",
            "--expect-source-etag",
            &source_etag,
            "--expect-dest-etag",
            &dest_etag,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["guarded"], true);
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# Doc\n\ngamma\n\nalpha\n\nbeta\n"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_stale_source_guard_fails_closed_without_writing() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n\ngamma\n");
    let stale_source_etag = block_etag(&path, 3);
    std::fs::write(&path, "# Doc\n\nalpha\n\nbeta\n\ngamma updated\n").unwrap();
    let dest_etag = block_etag(&path, 1);
    let before = std::fs::read_to_string(&path).unwrap();

    let output = md()
        .args([
            "move-block",
            "3",
            &path,
            "--before",
            "1",
            "-i",
            "--expect-source-etag",
            &stale_source_etag,
            "--expect-dest-etag",
            &dest_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("source block"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_stale_dest_guard_fails_closed_without_writing() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n\ngamma\n");
    let source_etag = block_etag(&path, 3);
    let stale_dest_etag = block_etag(&path, 1);
    std::fs::write(&path, "# Doc\n\nalpha updated\n\nbeta\n\ngamma\n").unwrap();
    let before = std::fs::read_to_string(&path).unwrap();

    let output = md()
        .args([
            "move-block",
            "3",
            &path,
            "--before",
            "1",
            "-i",
            "--expect-source-etag",
            &source_etag,
            "--expect-dest-etag",
            &stale_dest_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("destination block"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_checks_source_guard_before_destination_guard() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n\ngamma\n");
    let stale_source_etag = block_etag(&path, 3);
    let stale_dest_etag = block_etag(&path, 1);
    std::fs::write(&path, "# Doc\n\nalpha updated\n\nbeta\n\ngamma updated\n").unwrap();

    let output = md()
        .args([
            "move-block",
            "3",
            &path,
            "--before",
            "1",
            "--expect-source-etag",
            &stale_source_etag,
            "--expect-dest-etag",
            &stale_dest_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("source block"), "{stderr}");
    assert!(!stderr.contains("destination block"), "{stderr}");
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_runs_guards_before_adjacent_nochange_shortcut() {
    let path = tempfile_str("# Doc\n\nalpha\n\nbeta\n");
    let stale_source_etag = block_etag(&path, 1);
    let dest_etag = block_etag(&path, 0);
    std::fs::write(&path, "# Doc\n\nalpha updated\n\nbeta\n").unwrap();
    let before = std::fs::read_to_string(&path).unwrap();

    let output = md()
        .args([
            "move-block",
            "1",
            &path,
            "--after",
            "0",
            "-i",
            "--expect-source-etag",
            &stale_source_etag,
            "--expect-dest-etag",
            &dest_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("source block"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_ambiguous_source_guard_reports_source_role_in_json_envelope() {
    let path = tempfile_str("# Doc\n\nsame\n\nsame\n\n## Tail\n");
    let ambiguous_etag = block_etag(&path, 1);
    let output = md()
        .args([
            "move-block",
            "1",
            &path,
            "--before",
            "3",
            "--json",
            "--expect-source-etag",
            &ambiguous_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let env: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(env["error"]["code"], "etag_ambiguous");
    assert_eq!(env["error"]["context"]["role"], "source");
    std::fs::remove_file(&path).ok();
}

#[test]
fn move_block_ambiguous_dest_guard_reports_destination_role_in_json_envelope() {
    let path = tempfile_str("# Doc\n\nsame\n\nsame\n\n## Tail\n");
    let ambiguous_etag = block_etag(&path, 1);
    let output = md()
        .args([
            "move-block",
            "3",
            &path,
            "--before",
            "1",
            "--json",
            "--expect-dest-etag",
            &ambiguous_etag,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let env: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(env["error"]["code"], "etag_ambiguous");
    assert_eq!(env["error"]["context"]["role"], "destination");
    std::fs::remove_file(&path).ok();
}
