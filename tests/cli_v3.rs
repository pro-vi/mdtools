#![cfg(feature = "cli")]

use mdtools::document::Document;
use mdtools::patch::{Patch, PatchOp, ReplaceBlockTarget};
use mdtools::target::{TargetAddress, TargetKind, TargetQuery, TargetSummary};
use mdtools::BlockKind;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

#[cfg(unix)]
fn disconnected_stdout() -> Stdio {
    use std::net::Shutdown;
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;

    let (reader, writer) = UnixStream::pair().unwrap();
    writer.shutdown(Shutdown::Write).unwrap();
    drop(reader);
    Stdio::from(std::fs::File::from(OwnedFd::from(writer)))
}

fn unique_directory(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "mdtools-cli-v3-{tag}-{}-{nanos}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn replacement_patch(source: &str, markdown: &str) -> Patch {
    let document = Document::parse(source).unwrap();
    let block = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            snapshot.kind == TargetKind::Block
                && matches!(
                    snapshot.summary,
                    TargetSummary::Block {
                        kind: BlockKind::Paragraph,
                        ..
                    }
                )
        })
        .unwrap();
    Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: ReplaceBlockTarget::try_from(&block).unwrap(),
            markdown: markdown.into(),
        }],
    }
}

#[test]
fn map_read_and_query_emit_clean_json() {
    let directory = unique_directory("reads");
    let path = directory.join("doc.md");
    std::fs::write(&path, "lead\n\n# Work\n\n- [ ] task\n").unwrap();

    let mapped = md().args(["map", path.to_str().unwrap()]).output().unwrap();
    assert!(mapped.status.success());
    assert!(mapped.stderr.is_empty());
    let mapped: Vec<serde_json::Value> = serde_json::from_slice(&mapped.stdout).unwrap();
    assert!(mapped.iter().any(|target| target["kind"] == "task"));

    let address = serde_json::to_string(&TargetAddress::Preamble).unwrap();
    let read = md()
        .args(["read", path.to_str().unwrap(), "--address", &address])
        .output()
        .unwrap();
    assert!(read.status.success());
    assert!(read.stderr.is_empty());
    let read: serde_json::Value = serde_json::from_slice(&read.stdout).unwrap();
    assert_eq!(read["type"], "preamble");
    assert_eq!(read["markdown"], "lead");

    let query = serde_json::to_string(&TargetQuery::Kind {
        kind: TargetKind::Task,
    })
    .unwrap();
    let queried = md()
        .args(["query", path.to_str().unwrap(), "--query", &query])
        .output()
        .unwrap();
    assert!(queried.status.success());
    assert!(queried.stderr.is_empty());
    let queried: Vec<serde_json::Value> = serde_json::from_slice(&queried.stdout).unwrap();
    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0]["type"], "target");
    assert_eq!(queried[0]["target"]["kind"], "task");
}

#[test]
fn query_accepts_json_from_stdin_without_prompting() {
    let directory = unique_directory("stdin");
    let path = directory.join("doc.md");
    std::fs::write(&path, "# Work\n\nbody\n").unwrap();
    let query = serde_json::to_string(&TargetQuery::Kind {
        kind: TargetKind::Section,
    })
    .unwrap();
    let mut child = md()
        .args(["query", path.to_str().unwrap(), "--from", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(query.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let result: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["target"]["kind"], "section");
}

#[test]
fn query_search_returns_non_mutable_evidence_ranges() {
    let directory = unique_directory("search");
    let path = directory.join("doc.md");
    std::fs::write(&path, "# Work\n\nfind needle here\n").unwrap();
    let query = serde_json::to_string(&TargetQuery::Search {
        text: "needle".into(),
        match_mode: mdtools::SearchMatchMode::Literal,
        block_kinds: Vec::new(),
        include_source_gaps: false,
        max_results: 100,
    })
    .unwrap();
    let output = md()
        .args(["query", path.to_str().unwrap(), "--query", &query])
        .output()
        .unwrap();
    assert!(output.status.success());
    let result: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["type"], "evidence");
    assert_eq!(result[0]["evidence"]["preview"], "find needle here");
    assert_eq!(result[0]["evidence"]["target"]["kind"], "block");
}

#[test]
fn query_search_can_return_targetless_source_evidence() {
    let directory = unique_directory("source-evidence");
    let path = directory.join("doc.md");
    std::fs::write(&path, "body\n\n[^lost]: hidden needle\n").unwrap();
    let query = serde_json::json!({
        "type": "search",
        "text": "needle",
        "match_mode": "literal",
        "block_kinds": [],
        "include_source_gaps": true,
        "max_results": 100
    })
    .to_string();
    let output = md()
        .args(["query", path.to_str().unwrap(), "--query", &query])
        .output()
        .unwrap();
    assert!(output.status.success());
    let result: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["type"], "source_evidence");
    assert_eq!(result[0]["evidence"]["preview"], "[^lost]: hidden needle");
    assert!(result[0]["evidence"].get("target").is_none());
    assert_eq!(
        result[0]["evidence"]["revision"].as_str().unwrap().len(),
        64
    );
}

#[test]
fn query_search_budget_failure_emits_no_partial_results() {
    let directory = unique_directory("search-budget");
    let path = directory.join("doc.md");
    std::fs::write(&path, "needle needle\n").unwrap();
    let query = serde_json::json!({
        "type": "search",
        "text": "needle",
        "match_mode": "literal",
        "block_kinds": [],
        "include_source_gaps": false,
        "max_results": 1
    })
    .to_string();
    let output = md()
        .args(["query", path.to_str().unwrap(), "--query", &query])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("max_results (1)"));
}

#[test]
fn patch_stdout_is_non_mutating_and_in_place_is_guarded() {
    let directory = unique_directory("patch");
    let path = directory.join("doc.md");
    let source = "# H\n\nbefore\n";
    std::fs::write(&path, source).unwrap();
    let patch = serde_json::to_string(&replacement_patch(source, "after\n")).unwrap();

    let preview = md()
        .args(["patch", path.to_str().unwrap(), "--patch", &patch])
        .output()
        .unwrap();
    assert!(preview.status.success());
    assert_eq!(String::from_utf8(preview.stdout).unwrap(), "# H\n\nafter\n");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);

    let committed = md()
        .args([
            "patch",
            path.to_str().unwrap(),
            "--patch",
            &patch,
            "--in-place",
        ])
        .output()
        .unwrap();
    assert!(committed.status.success());
    assert!(committed.stderr.is_empty());
    let receipts: Vec<serde_json::Value> = serde_json::from_slice(&committed.stdout).unwrap();
    assert_eq!(receipts[0]["operation"], "replace_block");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# H\n\nafter\n");

    let retry = md()
        .args([
            "patch",
            path.to_str().unwrap(),
            "--patch",
            &patch,
            "--in-place",
        ])
        .output()
        .unwrap();
    assert!(!retry.status.success());
    assert!(
        String::from_utf8_lossy(&retry.stderr).contains("document revision mismatch"),
        "stderr: {}",
        String::from_utf8_lossy(&retry.stderr)
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# H\n\nafter\n");
}

#[test]
fn json_patch_preview_carries_source_and_receipts() {
    let directory = unique_directory("preview-json");
    let path = directory.join("doc.md");
    let source = "before\n";
    std::fs::write(&path, source).unwrap();
    let patch = serde_json::to_string(&replacement_patch(source, "after")).unwrap();
    let output = md()
        .args(["--json", "patch", path.to_str().unwrap(), "--patch", &patch])
        .output()
        .unwrap();
    assert!(output.status.success());
    let preview: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(preview["source"], "after\n");
    assert_eq!(preview["receipts"][0]["operation"], "replace_block");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
}

#[test]
fn structural_help_and_protocol_schema_are_discoverable() {
    for command in ["map", "read", "query", "patch"] {
        let output = md().args([command, "--help"]).output().unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("Usage:"));
        assert!(help.contains("Example"));
    }

    let schema = md().arg("schema").output().unwrap();
    assert!(schema.status.success());
    assert!(schema.stderr.is_empty());
    let schema: serde_json::Value = serde_json::from_slice(&schema.stdout).unwrap();
    assert_eq!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|command| command["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["map", "read", "query", "patch", "schema"]
    );
}

#[test]
fn invalid_protocol_json_fails_fast_with_schema_guidance() {
    let directory = unique_directory("invalid");
    let path = directory.join("doc.md");
    std::fs::write(&path, "body\n").unwrap();
    let output = md()
        .args(["query", path.to_str().unwrap(), "--query", "{}"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("invalid TargetQuery JSON"));
    assert!(error.contains("md schema"));
}

#[test]
fn json_output_to_a_closed_pipe_does_not_panic() {
    let directory = unique_directory("closed-pipe");
    let path = directory.join("doc.md");
    std::fs::write(&path, "# H\n\nbody\n").unwrap();
    let output = md()
        .args(["--json", "map", path.to_str().unwrap()])
        .stdout(disconnected_stdout())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5));
}

#[test]
fn in_place_commit_remains_successful_when_receipt_pipe_closes() {
    let directory = unique_directory("closed-receipt-pipe");
    let path = directory.join("doc.md");
    let source = "before\n";
    std::fs::write(&path, source).unwrap();
    let patch = serde_json::to_string(&replacement_patch(source, "after")).unwrap();
    let output = md()
        .args([
            "patch",
            path.to_str().unwrap(),
            "--patch",
            &patch,
            "--in-place",
        ])
        .stdout(disconnected_stdout())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "after\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("file commit succeeded"));
}

#[test]
fn json_errors_are_newline_terminated() {
    let output = md()
        .args(["--json", "map", "/definitely/missing/mdtools-document.md"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.ends_with(b"\n"));
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["error"], "io");
}

#[test]
fn malformed_frontmatter_uses_strict_read_and_mutation_policies() {
    let directory = unique_directory("frontmatter-policy");
    let path = directory.join("doc.md");
    let source = "---\na: [\n---\n\nbody\n";
    std::fs::write(&path, source).unwrap();

    let address = serde_json::to_string(&TargetAddress::Frontmatter).unwrap();
    let read = md()
        .args(["read", path.to_str().unwrap(), "--address", &address])
        .output()
        .unwrap();
    assert!(!read.status.success());

    let lenient = Document::parse(source).unwrap();
    let field = lenient
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a".into()],
        })
        .unwrap();
    let patch = Patch {
        base_revision: lenient.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: mdtools::patch::FrontmatterPatchTarget::try_from(field.snapshot()).unwrap(),
            value: serde_json::json!("new"),
        }],
    };
    let patch = serde_json::to_string(&patch).unwrap();
    let mutation = md()
        .args([
            "patch",
            path.to_str().unwrap(),
            "--patch",
            &patch,
            "--in-place",
        ])
        .output()
        .unwrap();
    assert!(!mutation.status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
}

#[test]
fn help_is_an_option_not_a_sixth_subcommand() {
    let output = md().arg("help").output().unwrap();
    assert!(!output.status.success());
    let help = String::from_utf8_lossy(&output.stderr);
    assert!(help.contains("unrecognized subcommand 'help'"));
}
