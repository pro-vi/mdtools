//! U1: structured JSON error envelope contract.
//!
//! Under --json, every single-file error path emits exactly one envelope
//! object on stdout: {schema_version, file?, error: {code, exit_code,
//! message, hint?, context?}}. Multi-file NDJSON commands emit per-file
//! envelope rows plus one final aggregate row; `tasks` keeps its single
//! aggregate object and carries failures[] instead. Without --json, stdout
//! stays untouched on error.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn temp_file(content: &str) -> String {
    let id = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!(
        "/tmp/mdtools_envelope_{}_{}.md",
        std::process::id(),
        id
    );
    std::fs::write(&path, content).unwrap();
    path
}

const DUP_DOC: &str = "# Alpha\n\nbody a\n\n## Setup\n\none\n\n## Setup\n\ntwo\n\n## Setup\n\nthree\n";

fn parse_envelope(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).unwrap_or_else(|e| {
        panic!(
            "stdout must be one JSON envelope: {e}\nstdout: {}",
            String::from_utf8_lossy(stdout)
        )
    })
}

#[test]
fn heading_not_found_envelope_carries_code_hint_and_role() {
    let tmp = temp_file("# Only\n\nbody\n");
    let out = md()
        .args(["section", "Missing", &tmp, "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["schema_version"], "mdtools.v1");
    assert_eq!(env["error"]["code"], "heading_not_found");
    assert_eq!(env["error"]["exit_code"], 1);
    assert!(env["error"]["hint"]
        .as_str()
        .unwrap()
        .contains("md outline"));
    assert_eq!(env["error"]["context"]["role"], "target");
    assert_eq!(env["error"]["context"]["total_matches"], 0);
    // stderr text unchanged (pre-envelope contract)
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("heading not found: Missing"));
}

#[test]
fn duplicate_heading_envelope_lists_matches_with_occurrences() {
    let tmp = temp_file(DUP_DOC);
    let out = md()
        .args(["section", "Setup", &tmp, "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["error"]["code"], "duplicate_heading_match");
    assert_eq!(env["error"]["context"]["total_matches"], 3);
    let matches = env["error"]["context"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0]["occurrence"], 1);
    assert_eq!(matches[2]["occurrence"], 3);
    assert!(matches[0]["block_index"].is_number());
    assert!(matches[0]["line"].is_number());
}

#[test]
fn occurrence_out_of_range_is_distinct_from_not_found() {
    let tmp = temp_file(DUP_DOC);
    let out = md()
        .args(["section", "Setup", &tmp, "--occurrence", "9", "--json"])
        .output()
        .unwrap();
    // exit code stays 1 for back-compat; the envelope code discriminates
    assert_eq!(out.status.code(), Some(1));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["error"]["code"], "occurrence_out_of_range");
    assert_eq!(env["error"]["context"]["requested_occurrence"], 9);
    assert_eq!(env["error"]["context"]["total_matches"], 3);
    assert_eq!(env["error"]["context"]["role"], "target");
}

#[test]
fn move_section_selector_errors_carry_source_and_destination_roles() {
    let tmp = temp_file(DUP_DOC);
    // Ambiguous source
    let out = md()
        .args(["move-section", "Setup", "--into", "Alpha", &tmp, "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["error"]["code"], "duplicate_heading_match");
    assert_eq!(env["error"]["context"]["role"], "source");

    // Missing destination
    let out = md()
        .args([
            "move-section",
            "Alpha",
            "--into",
            "Nowhere",
            &tmp,
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["error"]["code"], "heading_not_found");
    assert_eq!(env["error"]["context"]["role"], "destination");
}

#[test]
fn etag_mismatch_envelope_carries_expected_and_found() {
    let tmp = temp_file("# A\n\nfirst body\n");
    let out = md()
        .args([
            "replace-block",
            "1",
            &tmp,
            "-i",
            "--json",
            "--expect-etag",
            "0000000000000000",
            "--from",
            "-",
        ])
        .env("MDTOOLS_USAGE_LOG", "")
        .output_with_stdin("replacement\n");
    assert_eq!(out.status.code(), Some(4));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["error"]["code"], "etag_mismatch");
    assert_eq!(
        env["error"]["context"]["expected_etag"],
        "0000000000000000"
    );
    assert!(env["error"]["context"]["found_etag"].is_string());
    assert!(env["error"]["hint"]
        .as_str()
        .unwrap()
        .contains("md blocks --json"));
}

#[test]
fn invalid_task_loc_envelope_carries_loc() {
    let tmp = temp_file("- [ ] one\n");
    let out = md()
        .args(["set-task", "bogus", &tmp, "-i", "--status", "done", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let env = parse_envelope(&out.stdout);
    assert_eq!(env["error"]["code"], "invalid_task_loc");
    assert_eq!(env["error"]["context"]["loc"], "bogus");
}

#[test]
fn without_json_flag_stdout_stays_untouched_on_error() {
    let tmp = temp_file("# Only\n\nbody\n");
    let out = md().args(["section", "Missing", &tmp]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        out.stdout.is_empty(),
        "stdout: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn multifile_ndjson_emits_per_file_error_rows_and_aggregate() {
    let good = temp_file("# Good\n\nbody\n");
    let dir = format!(
        "/tmp/mdtools_envelope_dir_{}_{}",
        std::process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    std::fs::create_dir_all(&dir).unwrap();
    let bad = format!("{}/bad.md", dir);
    std::fs::write(&bad, "---\ntitle: [broken\n---\n# Bad\n").unwrap();

    let out = md()
        .args(["frontmatter", &good, &bad, "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let rows: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("every NDJSON row parses"))
        .collect();
    // one success row, one per-file error row, one aggregate error row
    assert_eq!(rows.len(), 3);
    assert!(rows[0]["error"].is_null());
    assert_eq!(rows[1]["error"]["code"], "frontmatter_parse_failed");
    assert_eq!(rows[1]["file"].as_str().unwrap(), bad);
    assert_eq!(rows[2]["error"]["code"], "multi_file_failure");
    assert_eq!(rows[2]["error"]["context"]["failed_files"], 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn tasks_multifile_keeps_single_object_with_failures() {
    let good = temp_file("- [ ] alpha\n");
    // invalid UTF-8 makes the per-file read fail for tasks
    let bad = format!(
        "/tmp/mdtools_envelope_bad_{}_{}.md",
        std::process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    std::fs::write(&bad, [0xFF, 0xFE, 0x00, 0x2D]).unwrap();

    let out = md().args(["tasks", &good, &bad, "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    // exactly ONE JSON object on stdout, never NDJSON
    let obj = parse_envelope(&out.stdout);
    assert_eq!(obj["schema_version"], "mdtools.v1");
    assert_eq!(obj["results"].as_array().unwrap().len(), 1);
    let failures = obj["failures"].as_array().unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0]["file"].as_str().unwrap(), bad);
    assert_eq!(failures[0]["error"]["code"], "io_open_failed");
}

#[test]
fn diagnostic_codes_serialize_snake_case_and_cover_exit_codes() {
    use mdtools::errors::DiagnosticCode;
    for code in DiagnosticCode::ALL {
        let name = serde_json::to_value(code).unwrap();
        let name = name.as_str().unwrap();
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
            "snake_case violated: {name}"
        );
        let exit = code.exit_code() as u8;
        assert!(exit >= 1 && exit <= 4, "{name}: exit {exit}");
    }
    // count pins the schema surface; update deliberately with new variants
    assert_eq!(DiagnosticCode::ALL.len(), 21);
}

trait OutputWithStdin {
    fn output_with_stdin(&mut self, input: &str) -> std::process::Output;
}

impl OutputWithStdin for Command {
    fn output_with_stdin(&mut self, input: &str) -> std::process::Output {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = self
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
}
