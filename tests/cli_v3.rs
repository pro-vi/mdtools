#![cfg(feature = "cli")]

use mdtools::document::Document;
use mdtools::patch::{Patch, PatchOp, ReplaceBlockTarget};
use mdtools::target::{TargetAddress, TargetKind, TargetQuery, TargetSummary};
use mdtools::BlockKind;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn md() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_md"));
    command.env_remove("MD_USAGE_LOG");
    command
}

#[test]
fn opted_in_usage_counts_actual_streams_without_recording_payload_or_paths() {
    let directory = unique_directory("usage");
    let path = directory.join("private-fixture.md");
    let log = directory.join("usage.jsonl");
    let source = "hello\n文字 👋\n<|endoftext|>\n";
    std::fs::write(&path, source).unwrap();
    let output = md()
        .env("MD_USAGE_LOG", &log)
        .env("MD_USAGE_CALLER", "codex")
        .env("MD_USAGE_SESSION", "00000000-0000-4000-8000-000000000001")
        .args([
            "read",
            path.to_str().unwrap(),
            "--address",
            r#"{"kind":"document"}"#,
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, source.as_bytes());
    assert!(output.stderr.is_empty());
    let logged = std::fs::read_to_string(&log).unwrap();
    let record: serde_json::Value = serde_json::from_str(&logged).unwrap();
    assert_eq!(record["stdout"]["bytes"], source.len());
    let tokenizer = tiktoken_rs::o200k_base().unwrap();
    assert_eq!(
        record["stdout"]["tokens"],
        tokenizer
            .count(source, &std::collections::HashSet::new())
            .unwrap()
    );
    assert_eq!(record["stderr"]["bytes"], 0);
    assert_eq!(record["stderr"]["tokens"], 0);
    assert_eq!(record["surface"], "cli_streams");
    assert_eq!(record["caller"], "codex");
    assert_eq!(record["provenance"], "explicit_environment_claims_only");
    assert!(!logged.contains("private-fixture"));
    assert!(!logged.contains("hello"));
    assert!(!logged.contains("文字"));
    assert!(!logged.contains(directory.to_str().unwrap()));
}

#[test]
fn usage_failure_preserves_command_result_and_early_exits_are_not_logged() {
    let directory = unique_directory("usage-failure");
    let path = directory.join("doc.md");
    std::fs::write(&path, "body\n").unwrap();
    let args = [
        "read",
        path.to_str().unwrap(),
        "--address",
        r#"{"kind":"document"}"#,
    ];
    let baseline = md().args(args).output().unwrap();
    let output = md()
        .env("MD_USAGE_LOG", &directory)
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status, baseline.status);
    assert_eq!(output.stdout, baseline.stdout);
    assert_eq!(output.stderr, baseline.stderr);
    let log = directory.join("usage.jsonl");
    let help = md()
        .env("MD_USAGE_LOG", &log)
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(!log.exists());
}

#[test]
fn usage_observes_error_json_and_stderr_without_changing_exit_status() {
    let directory = unique_directory("usage-error");
    let log = directory.join("usage.jsonl");
    let path = directory.join("doc.md");
    std::fs::write(&path, "body\n").unwrap();
    let output = md()
        .env("MD_USAGE_LOG", &log)
        .args(["query", path.to_str().unwrap(), "--query", "{}", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let record: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(log).unwrap()).unwrap();
    assert_eq!(record["stdout"]["bytes"], output.stdout.len());
    assert_eq!(record["stderr"]["bytes"], output.stderr.len());
    assert_eq!(record["diagnostic"], "invalid_input");
    assert_eq!(record["format"], "error_json");
    assert!(record["query_kind"].is_null());
}

#[cfg(unix)]
fn private_file(path: &std::path::Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, contents).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

#[test]
#[cfg(unix)]
fn usage_declines_document_protocol_and_stdio_aliases() {
    use std::os::unix::fs::symlink;
    let directory = unique_directory("usage-alias");
    let path = directory.join("doc.md");
    let source = "body\n";
    private_file(&path, source);
    let link = directory.join("link.md");
    symlink(&path, &link).unwrap();
    for document in [&path, &link] {
        let output = md()
            .env("MD_USAGE_LOG", &path)
            .args([
                "read",
                document.to_str().unwrap(),
                "--address",
                r#"{"kind":"document"}"#,
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, source.as_bytes());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    }
    let input = directory.join("address.json");
    let address = r#"{"kind":"document"}"#;
    private_file(&input, address);
    let output = md()
        .env("MD_USAGE_LOG", &input)
        .args([
            "read",
            path.to_str().unwrap(),
            "--from",
            input.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::fs::read_to_string(&input).unwrap(), address);
    let output = md()
        .env("MD_USAGE_LOG", &input)
        .stdin(std::fs::File::open(&input).unwrap())
        .args(["read", path.to_str().unwrap(), "--from", "-"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::fs::read_to_string(&input).unwrap(), address);

    let redirected = directory.join("stdout.txt");
    private_file(&redirected, "");
    let output = md()
        .env("MD_USAGE_LOG", &redirected)
        .stdout(std::fs::File::create(&redirected).unwrap())
        .args(["read", path.to_str().unwrap(), "--address", address])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::fs::read_to_string(&redirected).unwrap(), source);

    let error_args = ["query", path.to_str().unwrap(), "--query", "{}"];
    let baseline = md().args(error_args).output().unwrap();
    let error_file = directory.join("stderr.txt");
    private_file(&error_file, "");
    let output = md()
        .env("MD_USAGE_LOG", &error_file)
        .stderr(std::fs::File::create(&error_file).unwrap())
        .args(error_args)
        .output()
        .unwrap();
    assert_eq!(output.status, baseline.status);
    assert_eq!(std::fs::read(&error_file).unwrap(), baseline.stderr);
}

#[test]
#[cfg(unix)]
fn usage_declines_old_and_new_document_identities_across_atomic_patch() {
    for hard_link in [false, true] {
        let directory = unique_directory("usage-patch-alias");
        let path = directory.join("doc.md");
        private_file(&path, "before\n");
        let log = if hard_link {
            let link = directory.join("old-document.md");
            std::fs::hard_link(&path, &link).unwrap();
            link
        } else {
            path.clone()
        };
        let patch = serde_json::to_string(&replacement_patch("before\n", "after\n")).unwrap();
        let output = md()
            .env("MD_USAGE_LOG", &log)
            .args([
                "patch",
                path.to_str().unwrap(),
                "--patch",
                &patch,
                "--in-place",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "after\n");
        if hard_link {
            assert_eq!(std::fs::read_to_string(log).unwrap(), "before\n");
        }
    }
}

#[test]
#[cfg(unix)]
fn usage_does_not_create_missing_input_or_touch_input_on_clap_early_exit() {
    let directory = unique_directory("usage-missing-alias");
    let path = directory.join("missing.md");
    let output = md()
        .env("MD_USAGE_LOG", &path)
        .args([
            "read",
            path.to_str().unwrap(),
            "--address",
            r#"{"kind":"document"}"#,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!path.exists());
    let alias = directory.join("alias.md");
    std::os::unix::fs::symlink("missing.md", &alias).unwrap();
    let output = md()
        .env("MD_USAGE_LOG", &path)
        .args([
            "read",
            alias.to_str().unwrap(),
            "--address",
            r#"{"kind":"document"}"#,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!path.exists());
    assert!(std::fs::symlink_metadata(&alias)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(!alias.exists());
    private_file(&path, "body\n");
    for flag in ["--help", "--not-an-option"] {
        let _output = md()
            .env("MD_USAGE_LOG", &path)
            .args(["read", path.to_str().unwrap(), flag])
            .output()
            .unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "body\n");
    }
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
fn default_section_discovery_returns_only_addresses_and_summaries() {
    let directory = unique_directory("compact-discovery");
    let path = directory.join("doc.md");
    std::fs::write(&path, "# Work\n\n## Same\n\nfirst\n\n## Same\n\nsecond\n").unwrap();
    let query = r#"{"type":"kind","kind":"section"}"#;
    let output = md()
        .args(["query", path.to_str().unwrap(), "--query", query])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let results: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(results.len(), 3);
    for (index, result) in results.iter().enumerate() {
        assert_eq!(result["type"], "target");
        let target = result["target"].as_object().unwrap();
        assert_eq!(
            target.len(),
            2,
            "discovery must not include editing evidence"
        );
        assert!(target.contains_key("address"));
        assert!(target.contains_key("summary"));
        let address = serde_json::to_string(&target["address"]).unwrap();
        let read = md()
            .args(["read", path.to_str().unwrap(), "--address", &address])
            .output()
            .unwrap();
        assert!(read.status.success());
        if index == 2 {
            assert_eq!(read.stdout, b"## Same\n\nsecond\n");
        }
    }
}

#[test]
fn last_task_span_and_guard_exclude_following_paragraphs() {
    let directory = unique_directory("last-task-boundary");
    let path = directory.join("doc.md");
    for newline in ["\n", "\r\n"] {
        let mut previous_guard = None;
        for tail in ["paragraph", "changed paragraph with more text"] {
            let source = ["- [ ] a", "- [ ] b", "", tail, ""].join(newline);
            std::fs::write(&path, &source).unwrap();
            let output = md()
                .args([
                    "query",
                    path.to_str().unwrap(),
                    "--query",
                    r#"{"type":"kind","kind":"task"}"#,
                    "--json",
                ])
                .output()
                .unwrap();
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            let results: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(results.len(), 2);
            let task = &results[1]["target"];
            let span = &task["selection_span"];
            let start = span["byte_start"].as_u64().unwrap() as usize;
            let end = span["byte_end"].as_u64().unwrap() as usize;
            assert_eq!(&source[start..end], "- [ ] b");
            assert_eq!(span["line_start"], 2);
            assert_eq!(span["line_end"], 2);
            assert!(end < source.len());
            let etag = task["guard"]["etag"].as_str().unwrap().to_string();
            if let Some(previous) = &previous_guard {
                assert_eq!(
                    &etag, previous,
                    "unrelated trailing prose changed task guard"
                );
            }
            previous_guard = Some(etag);
            let address = serde_json::to_string(&task["address"]).unwrap();
            let read = md()
                .args(["read", path.to_str().unwrap(), "--address", &address])
                .output()
                .unwrap();
            assert!(read.status.success());
            assert_eq!(read.stdout, b"- [ ] b");
        }
    }
}

#[test]
fn default_section_read_preserves_original_markdown_bytes_once() {
    let directory = unique_directory("content-read");
    let path = directory.join("doc.md");
    let section = "Section\r\n-------\r\n\r\n文字 with `code`\r\n\r\n### Child\r\n\r\nbody";
    std::fs::write(&path, format!("# Root\r\n\r\n{section}")).unwrap();
    let address = r#"{"kind":"section","path":[{"text":"Root","occurrence":1},{"text":"Section","occurrence":1}]}"#;
    let read = md()
        .args(["read", path.to_str().unwrap(), "--address", address])
        .output()
        .unwrap();
    assert!(read.status.success());
    assert!(read.stderr.is_empty());
    assert_eq!(read.stdout, section.as_bytes());
}

#[test]
fn query_read_selects_one_target_and_rejects_zero_multiple_and_search_matches() {
    let directory = unique_directory("query-read");
    let path = directory.join("doc.md");
    std::fs::write(
        &path,
        "# Work\n\n## Unique\n\n文字\r\n\n## Same\n\none\n\n## Same\n\ntwo\n",
    )
    .unwrap();
    for (text, exit) in [("Unique", 0), ("Absent", 1), ("Same", 4)] {
        let query =
            serde_json::json!({"type":"section","text":text,"match_mode":"exact"}).to_string();
        let output = md()
            .args(["read", path.to_str().unwrap(), "--query", &query])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(exit));
        if exit == 0 {
            assert_eq!(output.stdout, "## Unique\n\n文字\r\n\n".as_bytes());
        } else {
            assert!(output.stdout.is_empty());
        }
        if exit == 4 {
            assert!(String::from_utf8_lossy(&output.stderr).contains("md read --address"));
        }
    }
    let query = r#"{"type":"search","text":"Unique","match_mode":"literal","block_kinds":[],"include_source_gaps":false,"max_results":10}"#;
    let output = md()
        .args(["read", path.to_str().unwrap(), "--query", query])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let query = r#"{"type":"section","text":"Unique","match_mode":"exact"}"#;
    let output = md()
        .args(["read", path.to_str().unwrap(), "--query", query, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["type"], "section");
    assert!(value["snapshot"]["guard"].is_object());
    let conflict = md()
        .args([
            "read",
            path.to_str().unwrap(),
            "--query",
            query,
            "--address",
            r#"{"kind":"document"}"#,
        ])
        .output()
        .unwrap();
    assert_eq!(conflict.status.code(), Some(2));
}

#[test]
fn query_read_preserves_frontmatter_selection_and_parse_policy() {
    let directory = unique_directory("query-frontmatter");
    let path = directory.join("doc.md");
    let field = r#"{"type":"frontmatter_field","path":["value"]}"#;
    let whole = r#"{"type":"kind","kind":"frontmatter"}"#;
    for (source, query, exit, expected) in [
        (
            "---\nvalue: null\n---\n",
            field,
            0,
            "{\"present\":true,\"value\":null}\n",
        ),
        ("---\nother: yes\n---\n", field, 1, ""),
        ("# H\n\nbody\n", whole, 0, ""),
        ("---\na: [\n---\n\n# H\n\nbody\n", whole, 2, ""),
        ("+++\na = [\n+++\n\n# H\n\nbody\n", whole, 2, ""),
        (
            "---\na: [\n---\n\n# H\n\nbody\n",
            r#"{"type":"section","text":"H","match_mode":"exact"}"#,
            0,
            "# H\n\nbody\n",
        ),
        ("---\na: 1\n", whole, 0, ""),
    ] {
        std::fs::write(&path, source).unwrap();
        let output = md()
            .args(["read", path.to_str().unwrap(), "--query", query])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{source:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected.as_bytes());
    }
}

#[test]
fn invalid_input_guidance_uses_typed_examples_and_stays_bounded() {
    let path = "/missing/file-is-not-loaded-for-invalid-input.md";
    let query = r#"{"type":"section","text":"Discussion"}"#;
    let output = md()
        .args(["query", path, "--query", query, "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(error["message"].as_str().unwrap().contains("missing field"));
    let hint = error["hint"].as_str().unwrap();
    let example = hint
        .split_once(": ")
        .unwrap()
        .1
        .split_once("; schema:")
        .unwrap()
        .0;
    let example: TargetQuery = serde_json::from_str(example).unwrap();
    assert!(matches!(example, TargetQuery::Section { .. }));
    assert!(!hint.contains("Discussion"));
    let query = serde_json::json!({"type":"文".repeat(4000)}).to_string();
    let output = md()
        .args(["query", path, "--query", &query, "--json"])
        .output()
        .unwrap();
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        error["message"].as_str().unwrap().len() + error["hint"].as_str().unwrap().len() < 1024
    );
    let duplicate = r#"{"type":"section","text":"A","text":"B","match_mode":"exact"}"#;
    let output = md()
        .args(["query", path, "--query", duplicate])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate field"));
}

#[test]
fn explicit_json_keeps_full_map_read_and_query_protocol() {
    let directory = unique_directory("reads");
    let path = directory.join("doc.md");
    std::fs::write(&path, "lead\n\n# Work\n\n- [ ] task\n").unwrap();

    let mapped = md()
        .args(["--json", "map", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(mapped.status.success());
    assert!(mapped.stderr.is_empty());
    let mapped: Vec<serde_json::Value> = serde_json::from_slice(&mapped.stdout).unwrap();
    assert!(mapped.iter().any(|target| target["kind"] == "task"));

    let address = serde_json::to_string(&TargetAddress::Preamble).unwrap();
    let read = md()
        .args([
            "--json",
            "read",
            path.to_str().unwrap(),
            "--address",
            &address,
        ])
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
        .args(["--json", "query", path.to_str().unwrap(), "--query", &query])
        .output()
        .unwrap();
    assert!(queried.status.success());
    assert!(queried.stderr.is_empty());
    let queried: Vec<serde_json::Value> = serde_json::from_slice(&queried.stdout).unwrap();
    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0]["type"], "target");
    assert_eq!(queried[0]["target"]["kind"], "task");
    assert!(queried[0]["target"]["guard"].is_object());
    assert!(read["snapshot"]["guard"].is_object());
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
        .args(["--json", "query", path.to_str().unwrap(), "--from", "-"])
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
        .args(["--json", "query", path.to_str().unwrap(), "--query", &query])
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
        .args(["--json", "query", path.to_str().unwrap(), "--query", &query])
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
fn default_reads_return_exact_source_for_every_source_backed_target_kind() {
    let directory = unique_directory("source-content");
    let path = directory.join("doc.md");
    let source = "---\ntitle: Demo\n---\n\nlead\n\n# Work\n\nRead [guide](guide.md).\n\n- [x] finished\n\n| Name | State |\n| --- | --- |\n| A | open |\n";
    std::fs::write(&path, source).unwrap();
    let document = Document::parse_for_frontmatter(source).unwrap();
    for snapshot in document.map().unwrap() {
        let Some(span) = snapshot.selection_span else {
            continue;
        };
        let address = serde_json::to_string(&snapshot.address).unwrap();
        let output = md()
            .args(["read", path.to_str().unwrap(), "--address", &address])
            .output()
            .unwrap();
        assert!(output.status.success(), "{address}");
        assert!(output.stderr.is_empty(), "{address}");
        assert_eq!(
            output.stdout,
            source.as_bytes()[span.byte_start as usize..span.byte_end as usize],
            "{address}"
        );
    }
}

#[test]
fn default_frontmatter_field_read_distinguishes_missing_from_null() {
    let directory = unique_directory("field-content");
    let path = directory.join("doc.md");
    std::fs::write(&path, "---\nvalue: null\nname: Demo\n---\n").unwrap();
    for (field, expected) in [
        ("value", serde_json::json!({"present": true, "value": null})),
        (
            "missing",
            serde_json::json!({"present": false, "value": null}),
        ),
        (
            "name",
            serde_json::json!({"present": true, "value": "Demo"}),
        ),
    ] {
        let address = serde_json::json!({"kind": "frontmatter_field", "path": [field]}).to_string();
        let output = md()
            .args(["read", path.to_str().unwrap(), "--address", &address])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value, expected);
    }
}

#[test]
fn compact_map_retains_each_exact_address_and_summary_without_guards() {
    let directory = unique_directory("compact-map");
    let path = directory.join("doc.md");
    let source = "# Work\n\n- [ ] first\n- [x] second\n";
    std::fs::write(&path, source).unwrap();
    let output = md().args(["map", path.to_str().unwrap()]).output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let results: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let targets = Document::parse(source).unwrap().map().unwrap();
    assert_eq!(results.len(), targets.len());
    for (result, target) in results.iter().zip(targets) {
        assert_eq!(
            result,
            &serde_json::json!({"address": target.address, "summary": target.summary})
        );
    }
}

#[test]
fn compact_search_keeps_previews_spans_and_targetless_evidence_distinct() {
    let directory = unique_directory("compact-search");
    let path = directory.join("doc.md");
    std::fs::write(&path, "find needle here\n\n[^lost]: hidden needle\n").unwrap();
    let query = r#"{"type":"search","text":"needle","match_mode":"literal","block_kinds":[],"include_source_gaps":true,"max_results":100}"#;
    let output = md()
        .args(["query", path.to_str().unwrap(), "--query", query])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let results: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["type"], "evidence");
    assert_eq!(results[0]["target"]["kind"], "block");
    assert_eq!(results[0]["preview"], "find needle here");
    assert_eq!(results[1]["type"], "source_evidence");
    assert_eq!(results[1]["preview"], "[^lost]: hidden needle");
    assert!(results[1].get("target").is_none());
    for result in results {
        assert!(result["span"].is_object());
        assert!(result.get("revision").is_none());
        assert!(result.get("etag").is_none());
        assert!(result.get("guard").is_none());
    }
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

    let json = md()
        .args(["--json", "query", path.to_str().unwrap(), "--query", &query])
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(3));
    let envelope: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(envelope["schema_version"], "mdtools.v3");
    assert_eq!(envelope["error"], "result_limit");
    assert_eq!(envelope["exit_code"], 3);
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
