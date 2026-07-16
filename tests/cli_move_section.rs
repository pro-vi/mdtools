use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn tempfile(content: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/mdtools_move_section_{}_{}.md", std::process::id(), id);
    std::fs::write(&path, content).unwrap();
    path
}

fn tempfile_bytes(bytes: &[u8]) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!(
        "/tmp/mdtools_move_section_b_{}_{}.md",
        std::process::id(),
        id
    );
    std::fs::write(&path, bytes).unwrap();
    path
}

// === Core moves (5) ===

#[test]
fn t01_sibling_keep_level_after() {
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\nbody b\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--after", "B", "--keep-level"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The moved section carries the blank-line gap that originally separated
    // it from the next sibling; B's section had no trailing gap of its own,
    // so A's gap lands at end-of-document.
    assert_eq!(stdout, "# Doc\n\n## B\nbody b\n## A\nbody a\n\n");
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t02_sibling_auto_level_promote() {
    // src=h3, dest=h2 → delta -1, descendant cascades
    let tmp = tempfile(
        "# Doc\n\n## Backend\n### API\napi body\n#### Auth\nauth body\n## Frontend\nfront\n",
    );
    let output = md()
        .args(["move-section", "API", &tmp, "--after", "Frontend"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // API was h3 → h2, Auth was h4 → h3
    assert!(stdout.contains("\n## API\n"), "got: {}", stdout);
    assert!(stdout.contains("\n### Auth\n"), "got: {}", stdout);
    assert!(stdout.contains("api body"));
    assert!(stdout.contains("auth body"));
    // API should now appear AFTER Frontend
    let api = stdout.find("## API").unwrap();
    let front = stdout.find("## Frontend").unwrap();
    assert!(api > front);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t03_sibling_auto_level_demote() {
    // src=h2, dest=h4 → delta +2
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\n### B1\n#### B1a\nb1a body\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--after", "B1a"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\n#### A\n"),
        "expected #### A; got:\n{}",
        stdout
    );
    assert!(stdout.contains("body a"));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t04_into_as_child() {
    // src=h2 (with descendants), dest=h2 → into makes new top h3 (delta +1)
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\nbody b\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--into", "B"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\n### A\n"),
        "expected ### A; got:\n{}",
        stdout
    );
    // A should appear after B's body inside B's section
    let a = stdout.find("### A").unwrap();
    let b = stdout.find("## B").unwrap();
    assert!(a > b);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t05_before_sibling_auto_level() {
    // src=h3, dest=h2 → before, delta -1
    let tmp = tempfile("# Doc\n\n## Backend\n### API\napi\n## Frontend\nfront\n");
    let output = md()
        .args(["move-section", "API", &tmp, "--before", "Frontend"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n## API\n"));
    let api = stdout.find("## API").unwrap();
    let front = stdout.find("## Frontend").unwrap();
    assert!(api < front);
    std::fs::remove_file(&tmp).unwrap();
}

// === Splice positions (4) ===

#[test]
fn t06_source_before_destination() {
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\nbody b\n\n## C\nbody c\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--after", "C", "--keep-level"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let a = stdout.find("## A").unwrap();
    let c = stdout.find("## C").unwrap();
    assert!(a > c);
    assert!(stdout.contains("body a"));
    assert!(stdout.contains("body b"));
    assert!(stdout.contains("body c"));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t07_source_after_destination() {
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\nbody b\n\n## C\nbody c\n");
    let output = md()
        .args(["move-section", "C", &tmp, "--after", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let c = stdout.find("## C").unwrap();
    let a = stdout.find("## A").unwrap();
    let b = stdout.find("## B").unwrap();
    assert!(c > a && c < b);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t08_destination_inside_source_errors() {
    let tmp = tempfile("# Doc\n\n## Outer\n### Inner\ninner body\n");
    let output = md()
        .args(["move-section", "Outer", &tmp, "--after", "Inner"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("destination is inside source"));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t09_source_inside_destination_after_errors() {
    let tmp = tempfile("# Doc\n\n## Outer\n### Inner\ninner body\nmore outer\n");
    let output = md()
        .args(["move-section", "Inner", &tmp, "--after", "Outer"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("destination contains source"));
    std::fs::remove_file(&tmp).unwrap();
}

// === Boundary edge cases (5) ===

#[test]
fn t10_last_section_in_file() {
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\nbody b\n");
    let output = md()
        .args(["move-section", "B", &tmp, "--before", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let b = stdout.find("## B").unwrap();
    let a = stdout.find("## A").unwrap();
    assert!(b < a);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t11_last_section_no_trailing_newline() {
    // Pro 5.5-pro golden case
    let tmp = tempfile("# Doc\n## A\na\n## B\nb");
    let output = md()
        .args(["move-section", "B", &tmp, "--before", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Newline must have been synthesized between "b" and "## A"
    assert!(stdout.contains("## B\nb\n## A\na"), "got:\n{}", stdout);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t12_only_top_level_section() {
    // Trying to move the only section in a doc onto something else should
    // either succeed (moving relative to a heading we manufacture) or fail
    // cleanly. With one section there's nothing to move it relative to —
    // expect an error.
    let tmp = tempfile("# Only\nbody only\n");
    let output = md()
        .args([
            "move-section",
            "Only",
            &tmp,
            "--after",
            "Only",
            "--keep-level",
        ])
        .output()
        .unwrap();
    // Same source and dest → dest is inside source (overlap == self)
    assert_eq!(output.status.code(), Some(3));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t13_crlf_preserved() {
    let tmp = tempfile_bytes(b"# Doc\r\n\r\n## A\r\nbody a\r\n\r\n## B\r\nbody b\r\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--after", "B", "--keep-level"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let bytes = output.stdout;
    // CRLF preserved on output
    assert!(
        bytes.windows(2).any(|w| w == b"\r\n"),
        "CRLF lost: {:?}",
        String::from_utf8_lossy(&bytes)
    );
    let s = String::from_utf8_lossy(&bytes);
    let a = s.find("## A").unwrap();
    let b = s.find("## B").unwrap();
    assert!(a > b);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t14_multibyte_utf8_in_heading() {
    let tmp = tempfile("# Doc\n\n## café\nbody café\n\n## bistro\nbody bistro\n");
    let output = md()
        .args([
            "move-section",
            "café",
            &tmp,
            "--after",
            "bistro",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cafe = stdout.find("## café").unwrap();
    let bistro = stdout.find("## bistro").unwrap();
    assert!(cafe > bistro);
    assert!(stdout.contains("body café"));
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t14b_contains_applies_to_source_and_destination() {
    let tmp = tempfile("# Doc\n\n## Backend API\nbody a\n\n## Frontend App\nbody b\n");
    let output = md()
        .args([
            "move-section",
            "api",
            &tmp,
            "--after",
            "front",
            "--contains",
            "--ignore-case",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let source = stdout.find("## Backend API").unwrap();
    let dest = stdout.find("## Frontend App").unwrap();
    assert!(source > dest, "got:\n{}", stdout);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t14c_contains_rejects_preamble_selector() {
    let tmp = tempfile("# Doc\n\n## A\nbody a\n\n## B\nbody b\n");
    let output = md()
        .args([
            "move-section",
            ":preamble",
            &tmp,
            "--after",
            "A",
            "--contains",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--contains"));
    assert!(stderr.contains(":preamble"));
    std::fs::remove_file(&tmp).unwrap();
}

// === Selectors (2) ===

#[test]
fn t15_occurrence_disambiguation() {
    let tmp =
        tempfile("# Doc\n\n## Notes\nfirst notes\n\n## Other\nother\n\n## Notes\nsecond notes\n");
    let output = md()
        .args([
            "move-section",
            "Notes",
            &tmp,
            "--after",
            "Other",
            "--keep-level",
            "--source-occurrence",
            "1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // First "Notes" (with "first notes" body) should now appear after Other
    let other = stdout.find("## Other").unwrap();
    let first_notes = stdout.find("first notes").unwrap();
    assert!(first_notes > other);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t16_ignore_case() {
    let tmp = tempfile("# Doc\n\n## Alpha\nbody a\n\n## Beta\nbody b\n");
    let output = md()
        .args([
            "move-section",
            "alpha",
            &tmp,
            "--after",
            "BETA",
            "--keep-level",
            "--ignore-case",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let alpha = stdout.find("## Alpha").unwrap();
    let beta = stdout.find("## Beta").unwrap();
    assert!(alpha > beta);
    std::fs::remove_file(&tmp).unwrap();
}

// === Heading representation (3) ===

#[test]
fn t17_setext_keep_level_ok() {
    // Setext h2 source moved with --keep-level: byte-exact relocation allowed
    let tmp = tempfile("# Doc\n\nA Title\n-------\nsetext body\n\n## B\nbody b\n");
    let output = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A Title\n-------"), "got:\n{}", stdout);
    let setext = stdout.find("A Title").unwrap();
    let b = stdout.find("## B").unwrap();
    assert!(setext > b);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t18_setext_auto_level_errors() {
    // Setext h2 source as a top-level sibling (not nested under an h1) so the
    // containment check passes; then auto-level into h1 dest forces delta = -1.
    let tmp = tempfile("A Title\n-------\nbody\n\n# Other\nother\n");
    let output = md()
        .args(["move-section", "A Title", &tmp, "--after", "Other"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("setext heading"), "stderr: {}", stderr);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn t19_atx_indentation_preserved_on_keep_level() {
    // 0-3 leading spaces on ATX marker — keep-level preserves bytes verbatim
    let tmp = tempfile("# Doc\n\n  ## Indented\nbody indented\n\n## Plain\nbody plain\n");
    let output = md()
        .args([
            "move-section",
            "Indented",
            &tmp,
            "--after",
            "Plain",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("  ## Indented"), "indent lost: {}", stdout);
    std::fs::remove_file(&tmp).unwrap();
}

// === Errors (2) ===

#[test]
fn t20_level_clamp_descendant_exceeds_h6() {
    // Source at h5 with h6 descendant. Demote into h6 dest → descendant lands at h7 → reject.
    let tmp = tempfile(
        "# A\n## B\n### C\n#### D\n##### E\nbody e\n###### F\nbody f\n## Target\ntarget body\n",
    );
    // Move E (h5, with F=h6 child) into Target (h2). new_top = 3, delta = -2 → F goes to h4. OK.
    // To force clamp, demote: --into a h6 destination so new_top = 7.
    // Set up explicitly:
    let tmp2 = tempfile(
        "# Doc\n\n## A\n### B\nbody b\n#### C\nbody c\n\n## D\n### D1\n#### D2\n##### D3\n###### D4\n",
    );
    let output = md()
        .args(["move-section", "B", &tmp2, "--into", "D4"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("heading level 7") || stderr.contains("max is 6"),
        "stderr: {}",
        stderr
    );
    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&tmp2).unwrap();
}

#[test]
fn t21_conflicting_destination_flags_clap_error() {
    let tmp = tempfile("# Doc\n## A\na\n## B\nb\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--after", "B", "--into", "B"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Clap renders ArgGroup conflicts; just check it didn't proceed silently
    assert!(
        stderr.contains("cannot be used")
            || stderr.contains("argument")
            || stderr.contains("conflict")
            || stderr.contains("--after")
            || stderr.contains("error"),
        "expected clap conflict error, got:\n{}",
        stderr
    );
    std::fs::remove_file(&tmp).unwrap();
}

// === Regression: review findings ===

#[test]
fn r01_eof_destination_no_trailing_newline_separator() {
    // Source before dest, dest is the final section with NO trailing newline.
    // Without a synthesized separator we'd glue moved bytes to dest's last
    // byte: "...b## A..." instead of "...b\n## A..."
    let tmp = tempfile_bytes(b"# Doc\n## A\na\n## B\nb");
    let output = md()
        .args(["move-section", "A", &tmp, "--after", "B", "--keep-level"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("b## A"),
        "moved heading glued to dest EOF: {:?}",
        stdout
    );
    assert!(stdout.contains("b\n## A"), "got:\n{}", stdout);
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r02_setext_descendant_in_atx_source_rejected_on_relevel() {
    // Source is ATX h1 "A" containing a setext h2 descendant "Child Title".
    // Dest is "Other" (h1 sibling). Moving --into Other forces delta = +1 on
    // A — and on its descendants. The ATX "A" would re-level to h2 cleanly,
    // but the setext "Child Title" can't be byte-rewritten, so without the
    // descendant check it would silently keep level 2 and end up a sibling
    // of moved-A instead of a child.
    let tmp =
        tempfile("# A\nbody a\n\nChild Title\n-----------\nchild body\n\n# Other\nother body\n");
    let output = md()
        .args(["move-section", "A", &tmp, "--into", "Other"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(3),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("setext heading") && stderr.contains("Child Title"),
        "stderr: {}",
        stderr
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r03_crlf_separator_preserves_line_endings() {
    // Source after dest, CRLF doc, no trailing newline on the moved section.
    // Synthesized separator must be CRLF too — bare LF would turn a clean
    // CRLF mutation into mixed line endings.
    let tmp = tempfile_bytes(b"# Doc\r\n## A\r\na\r\n## B\r\nb");
    let output = md()
        .args(["move-section", "B", &tmp, "--before", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = output.stdout;
    let s = String::from_utf8_lossy(&bytes);
    assert!(s.contains("## B\r\nb\r\n## A"), "got:\n{:?}", s);
    // No bare LF anywhere — every \n must be preceded by \r
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            assert!(
                i > 0 && bytes[i - 1] == b'\r',
                "bare LF at byte {} in CRLF document: {:?}",
                i,
                s
            );
        }
    }
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r04_noop_eof_move_does_not_inject_newline() {
    // Source is already immediately after destination AND is the final
    // section with no trailing newline. The "move" is a no-op, but the
    // separator-injection check incorrectly fired because insert_byte_raw
    // (== src_byte_start) was treated as a content-bearing position.
    let original = b"# Doc\n## A\na\n## B\nb";
    let tmp = tempfile_bytes(original);
    let output = md()
        .args(["move-section", "B", &tmp, "--after", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        original,
        "no-op move corrupted bytes: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r05_stdin_input_when_file_omitted() {
    // Spec promises the FILE positional is optional; omitting it reads stdin
    // and writes the spliced doc to stdout. Without this, users get a clap
    // "missing FILE" error.
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_md"))
        .args(["move-section", "A", "--after", "B", "--keep-level"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"# Doc\n\n## A\nbody a\n\n## B\nbody b\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "stdin form should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let a = s.find("## A").expect("expected ## A in output");
    let b = s.find("## B").expect("expected ## B in output");
    assert!(a > b, "A should land after B; got:\n{}", s);
}

#[test]
fn r06_in_place_without_file_errors() {
    // -i has no meaning when reading from stdin — must reject cleanly.
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_md"))
        .args(["move-section", "A", "--after", "B", "--keep-level", "-i"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"# Doc\n## A\na\n## B\nb\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(!out.status.success(), "should reject -i without FILE");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--in-place") || stderr.contains("-i") || stderr.contains("FILE"),
        "expected -i/FILE error; got:\n{}",
        stderr
    );
}

#[test]
fn r07_nochange_envelope_after_span_matches_before() {
    // Reviewer repro: NoChange must report target_span_after == target_span_before,
    // not null. Anything else lies to consumers about where the section now lives.
    let tmp = tempfile_bytes(b"# Doc\n## A\na\n## B\nb");
    let output = md()
        .args([
            "move-section",
            "B",
            &tmp,
            "--after",
            "A",
            "--keep-level",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "NoChange");
    assert_eq!(json["changed"], false);
    let inv = &json["invariant"];
    assert!(!inv["target_span_before"].is_null(), "before span missing");
    assert!(
        !inv["target_span_after"].is_null(),
        "NoChange must carry an after span equal to the before span"
    );
    assert_eq!(
        inv["target_span_before"], inv["target_span_after"],
        "NoChange after span must equal before span"
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r08_replaced_envelope_after_span_locates_moved_bytes() {
    // For a real Replaced move, target_span_after must point at where the
    // moved bytes now live in the output document.
    let tmp = tempfile_bytes(b"# Doc\n## A\na\n## B\nb");
    let output = md()
        .args([
            "move-section",
            "A",
            &tmp,
            "--after",
            "B",
            "--keep-level",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);
    let after = &json["invariant"]["target_span_after"];
    assert!(!after.is_null(), "Replaced must carry an after span");
    let content = json["content"].as_str().unwrap();
    let byte_start = after["byte_start"].as_u64().unwrap() as usize;
    let byte_end = after["byte_end"].as_u64().unwrap() as usize;
    // The bytes at the reported after-span must BE the moved section.
    let landed = &content[byte_start..byte_end];
    assert!(
        landed.starts_with("## A"),
        "after span byte_start should land on `## A`; got: {:?}",
        landed
    );
    assert!(
        landed.contains("\na"),
        "after span must include the body 'a'; got: {:?}",
        landed
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r09_after_span_matches_fresh_parse_of_output() {
    // Reviewer's recommended verification: take the move's reported
    // target_span_after, write the spliced output to a tempfile, parse it
    // with `md section <heading>`, and compare. They MUST agree — the after
    // span describes the moved section's location in the output document, so
    // it should equal what a fresh parse reports for that same section.
    let tmp = tempfile_bytes(b"# Doc\n## A\na\n## B\nb");
    let out = md()
        .args([
            "move-section",
            "B",
            &tmp,
            "--before",
            "A",
            "--keep-level",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let move_json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let after = &move_json["invariant"]["target_span_after"];
    assert!(!after.is_null(), "target_span_after must not be null");

    let content = move_json["content"].as_str().unwrap();
    let parsed_tmp = tempfile(content);
    let section_out = md()
        .args(["section", "B", &parsed_tmp, "--json"])
        .output()
        .unwrap();
    assert!(
        section_out.status.success(),
        "fresh parse failed: {}",
        String::from_utf8_lossy(&section_out.stderr)
    );
    let section_json: serde_json::Value = serde_json::from_slice(&section_out.stdout).unwrap();
    let fresh_span = &section_json["section"]["span"];

    assert_eq!(
        after["byte_start"], fresh_span["byte_start"],
        "byte_start mismatch"
    );
    assert_eq!(
        after["byte_end"], fresh_span["byte_end"],
        "byte_end mismatch"
    );
    assert_eq!(
        after["line_start"], fresh_span["line_start"],
        "line_start mismatch"
    );
    assert_eq!(
        after["line_end"], fresh_span["line_end"],
        "line_end mismatch"
    );

    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&parsed_tmp).unwrap();
}

#[test]
fn r10_setext_keep_level_after_paragraph_preserves_structure() {
    // Reviewer repro: setext source moved after a paragraph-bearing section
    // gets folded into that paragraph as setext text — the underline `------`
    // becomes the underline of `body b A Title`, destroying the heading.
    // The output must reparse with "A Title" still a top-level section.
    let tmp = tempfile("# Doc\n\nA Title\n-------\nsetext body\n\n## B\nbody b\n");
    let move_out = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        move_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&move_out.stderr)
    );
    let spliced = String::from_utf8_lossy(&move_out.stdout).to_string();

    // Reparse with `md outline --json` and confirm "A Title" is still listed.
    let out_path = tempfile(&spliced);
    let outline_out = md()
        .args(["outline", &out_path, "--json"])
        .output()
        .unwrap();
    assert!(
        outline_out.status.success(),
        "outline failed; spliced doc:\n{}\nstderr: {}",
        spliced,
        String::from_utf8_lossy(&outline_out.stderr)
    );
    let outline_json: serde_json::Value = serde_json::from_slice(&outline_out.stdout).unwrap();
    let outline_text = outline_json.to_string();
    assert!(
        outline_text.contains("\"A Title\""),
        "moved setext heading lost its identity in output:\n{}\noutline:\n{}",
        spliced,
        outline_text
    );
    // Also assert `body b` did NOT absorb "A Title" into a single h2.
    assert!(
        !outline_text.contains("body b A Title"),
        "preceding paragraph absorbed the setext heading: {}",
        outline_text
    );
    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&out_path).unwrap();
}

#[test]
fn r11_setext_noop_after_heading_stays_byte_exact() {
    // A setext heading can validly follow an ATX heading with a single
    // newline. If the source is already immediately after the destination,
    // move-section must not add a blank line just because the source is
    // setext.
    let original = b"# Doc\n## B\nA Title\n-------\nsetext body";
    let tmp = tempfile_bytes(original);
    let output = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout, original,
        "already-correct setext move should be byte-exact"
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r12_setext_after_heading_only_destination_uses_minimal_boundary() {
    // Moving a setext section after a heading-only destination does not need
    // a blank line: `## B\nA Title\n-------` reparses as separate headings.
    let tmp = tempfile("# Doc\n\nA Title\n-------\nsetext body\n\n## B\n");
    let output = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("## B\nA Title\n-------"),
        "setext after heading should not grow a blank line:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("## B\n\nA Title"),
        "unexpected blank line inserted:\n{}",
        stdout
    );
    let parsed = tempfile(&stdout);
    let section_out = md()
        .args(["section", "A Title", &parsed, "--json"])
        .output()
        .unwrap();
    assert!(
        section_out.status.success(),
        "fresh parse lost moved setext section: {}",
        String::from_utf8_lossy(&section_out.stderr)
    );
    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&parsed).unwrap();
}

#[test]
fn r13_setext_after_list_body_gets_block_boundary() {
    // After a list body, a following setext heading without a blank line is
    // not a sibling section. The mover must add enough boundary whitespace
    // for a fresh parse to recover the moved section.
    let tmp = tempfile("# Doc\n\nA Title\n-------\nsetext body\n\n## B\n- item\n");
    let output = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed = tempfile(&stdout);
    let section_out = md()
        .args(["section", "A Title", &parsed, "--json"])
        .output()
        .unwrap();
    assert!(
        section_out.status.success(),
        "fresh parse lost moved setext section:\n{}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&section_out.stderr)
    );
    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&parsed).unwrap();
}

#[test]
fn r14_crlf_setext_noop_after_heading_stays_byte_exact() {
    let original = b"# Doc\r\n## B\r\nA Title\r\n-------\r\nsetext body";
    let tmp = tempfile_bytes(original);
    let output = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout, original,
        "CRLF setext no-op should remain byte-exact"
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r15_setext_after_fenced_code_uses_minimal_boundary() {
    // A fenced code block is a hard block boundary, so a setext heading can
    // follow it without an extra blank line.
    let tmp = tempfile("# Doc\n\nA Title\n-------\nsetext body\n\n## B\n```\ncode\n```\n");
    let output = md()
        .args([
            "move-section",
            "A Title",
            &tmp,
            "--after",
            "B",
            "--keep-level",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("```\nA Title\n-------"),
        "setext after fenced code should not grow a blank line:\n{}",
        stdout
    );
    let parsed = tempfile(&stdout);
    let section_out = md()
        .args(["section", "A Title", &parsed, "--json"])
        .output()
        .unwrap();
    assert!(
        section_out.status.success(),
        "fresh parse lost moved setext section:\n{}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&section_out.stderr)
    );
    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&parsed).unwrap();
}

#[test]
fn r16_atx_before_setext_destination_preserves_moved_body() {
    // The following setext underline can reinterpret the moved ATX section's
    // trailing paragraph as part of the setext heading unless the splice adds a
    // blank-line boundary after the moved bytes.
    let tmp = tempfile("# Doc\n\nA\n-\na body\n\n## B\nb body\n\n## C\nc body");
    let output = md()
        .args([
            "move-section",
            "C",
            &tmp,
            "--before",
            "A",
            "--keep-level",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let content = json["content"].as_str().unwrap();
    assert!(
        content.contains("## C\nc body\n\nA\n-"),
        "moved section should be separated from following setext heading:\n{}",
        content
    );

    let parsed = tempfile(content);
    let section_out = md()
        .args(["section", "C", &parsed, "--json"])
        .output()
        .unwrap();
    assert!(
        section_out.status.success(),
        "fresh parse lost moved ATX body:\n{}\nstderr: {}",
        content,
        String::from_utf8_lossy(&section_out.stderr)
    );
    let section_json: serde_json::Value = serde_json::from_slice(&section_out.stdout).unwrap();
    assert_eq!(
        json["invariant"]["target_span_after"], section_json["section"]["span"],
        "move envelope should match fresh section parse"
    );
    assert!(
        section_json["content"].as_str().unwrap().contains("c body"),
        "fresh section should still own its paragraph body"
    );

    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&parsed).unwrap();
}

#[test]
fn r17_heading_only_atx_before_setext_uses_minimal_boundary() {
    // A heading-only ATX section can precede setext with a single newline; do
    // not add a blank line when CommonMark already reparses both headings.
    let tmp = tempfile("# Doc\n\nA\n-\na body\n\n## C\n");
    let output = md()
        .args(["move-section", "C", &tmp, "--before", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("## C\nA\n-"),
        "heading-only move should not grow a blank line:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("## C\n\nA\n-"),
        "unexpected blank line inserted:\n{}",
        stdout
    );
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r18_atx_list_body_before_setext_gets_block_boundary() {
    let tmp = tempfile("# Doc\n\nA\n-\na body\n\n## C\n- item");
    let output = md()
        .args(["move-section", "C", &tmp, "--before", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed = tempfile(&stdout);
    let section_out = md()
        .args(["section", "C", &parsed, "--json"])
        .output()
        .unwrap();
    assert!(
        section_out.status.success(),
        "fresh parse lost moved list section:\n{}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&section_out.stderr)
    );
    let section_json: serde_json::Value = serde_json::from_slice(&section_out.stdout).unwrap();
    assert!(
        section_json["content"].as_str().unwrap().contains("- item"),
        "fresh section should still own the moved list body"
    );
    std::fs::remove_file(&tmp).unwrap();
    std::fs::remove_file(&parsed).unwrap();
}

#[test]
fn r19_crlf_atx_before_setext_preserves_line_endings() {
    let tmp = tempfile_bytes(b"# Doc\r\n\r\nA\r\n-\r\na body\r\n\r\n## C\r\nc body");
    let output = md()
        .args(["move-section", "C", &tmp, "--before", "A", "--keep-level"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = output.stdout;
    let stdout = String::from_utf8_lossy(&bytes);
    assert!(
        stdout.contains("## C\r\nc body\r\n\r\nA\r\n-"),
        "CRLF boundary should be synthesized before setext:\n{:?}",
        stdout
    );
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            assert!(
                i > 0 && bytes[i - 1] == b'\r',
                "bare LF at byte {} in CRLF document: {:?}",
                i,
                stdout
            );
        }
    }
    std::fs::remove_file(&tmp).unwrap();
}

#[test]
fn r20_keep_level_rejects_move_that_would_absorb_following_heading() {
    // Keeping A at h2 while inserting it before B1 (h3) would make B1 a
    // descendant of A on fresh parse. That is not a relocation of exactly A's
    // original section bytes, so the command must fail instead of reporting a
    // target_span_after that excludes B1.
    let tmp = tempfile("# Doc\n\n## A\na\n### A1\na1\n\n## B\nb\n### B1\nb1\n\n## C\nc");
    let output = md()
        .args(["move-section", "A", &tmp, "--before", "B1", "--keep-level"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(3),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("absorb or lose adjacent headings"),
        "stderr: {}",
        stderr
    );
    std::fs::remove_file(&tmp).unwrap();
}

// === Output modes (1) ===

#[test]
fn t22_output_modes_stdout_inplace_json() {
    let fixture = "# Doc\n\n## A\nbody a\n\n## B\nbody b\n";

    // (a) default: stdout, file untouched
    let tmp = tempfile(fixture);
    let out = md()
        .args(["move-section", "A", &tmp, "--after", "B", "--keep-level"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("## B\nbody b\n## A\nbody a"));
    let on_disk = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(on_disk, fixture, "file should be untouched without -i");

    // (b) --in-place: file mutated, stdout empty
    let out2 = md()
        .args([
            "move-section",
            "A",
            &tmp,
            "--after",
            "B",
            "--keep-level",
            "-i",
        ])
        .output()
        .unwrap();
    assert!(out2.status.success());
    assert!(out2.stdout.is_empty(), "stdout should be empty with -i");
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert!(after.contains("## B\nbody b\n## A\nbody a"));

    // (c) --json against fresh fixture: envelope shape
    std::fs::write(&tmp, fixture).unwrap();
    let out3 = md()
        .args([
            "move-section",
            "A",
            &tmp,
            "--after",
            "B",
            "--keep-level",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out3.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out3.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out3.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(json["command"], "MoveSection");
    assert_eq!(json["disposition"], "Replaced");
    assert_eq!(json["changed"], true);
    let target = &json["target"]["SectionMove"];
    assert_eq!(target["destination_mode"], "after_sibling");
    assert_eq!(target["level_shift_applied"], 0);
    assert!(json["content"]
        .as_str()
        .unwrap()
        .contains("## B\nbody b\n## A\nbody a"));

    std::fs::remove_file(&tmp).unwrap();
}
