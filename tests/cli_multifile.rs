use std::process::Command;

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

// --- Frontmatter multi-file ---

#[test]
fn frontmatter_directory() {
    let out = md()
        .args(["frontmatter", "--field", "title", "tests/fixtures/vault/"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Should have alpha and beta (non-recursive)
    assert!(stdout.contains("alpha.md\tAlpha Doc"));
    assert!(stdout.contains("beta.md\tBeta Doc"));
    // Should NOT have sub/ files
    assert!(!stdout.contains("gamma"));
}

#[test]
fn frontmatter_directory_recursive() {
    let out = md()
        .args([
            "frontmatter",
            "--field",
            "status,title",
            "tests/fixtures/vault/",
            "-r",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(stdout.contains("alpha.md\tpublished\tAlpha Doc"));
    assert!(stdout.contains("gamma.md\tdraft\tGamma Doc"));
}

#[test]
fn frontmatter_field_json() {
    let out = md()
        .args([
            "frontmatter",
            "--field",
            "status",
            "tests/fixtures/vault/",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // JSONL — each line is a separate JSON object
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    let json1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(json1["schema_version"], "mdtools.v1");
    assert!(json1["fields"]["status"].is_string());
}

#[test]
fn frontmatter_explicit_multi_file() {
    let out = md()
        .args([
            "frontmatter",
            "--field",
            "title",
            "tests/fixtures/vault/alpha.md",
            "tests/fixtures/vault/beta.md",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alpha Doc"));
    assert!(stdout.contains("Beta Doc"));
}

// --- Search multi-file ---

#[test]
fn search_directory_recursive() {
    let out = md()
        .args(["search", "content", "tests/fixtures/vault/", "-r"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Should find matches in all 4 files
    assert!(stdout.contains("alpha.md:"));
    assert!(stdout.contains("gamma.md:"));
}

#[test]
fn search_multi_json() {
    let out = md()
        .args(["search", "content", "tests/fixtures/vault/", "-r", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // JSONL — one SearchResult per file per line
    let jsons: Vec<serde_json::Value> = stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(jsons.len(), 4);
    assert!(jsons.iter().all(|j| j["schema_version"] == "mdtools.v1"));
}

// --- Outline multi-file ---

#[test]
fn outline_directory_recursive() {
    let out = md()
        .args(["outline", "tests/fixtures/vault/", "-r"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("alpha.md:\t# Alpha"));
    assert!(stdout.contains("delta.md:\t## Sub Section"));
}

// --- Stats multi-file ---

#[test]
fn stats_directory() {
    let out = md()
        .args(["stats", "tests/fixtures/vault/"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("alpha.md:\twords="));
    assert!(stdout.contains("beta.md:\twords="));
}

// --- Links multi-file ---

#[test]
fn links_directory_recursive() {
    let out = md()
        .args(["links", "tests/fixtures/vault/", "-r"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Beta has a link
    assert!(stdout.contains("beta.md:"));
    assert!(stdout.contains("example.com"));
}

// --- Backward compatibility ---

#[test]
fn outline_single_file_no_prefix() {
    let out = md()
        .args(["outline", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Single file — no filename prefix
    assert!(!stdout.contains("basic.md:"));
    assert!(stdout.contains("# "));
}

#[test]
fn search_single_file_no_prefix() {
    let out = md()
        .args(["search", "Content", "tests/fixtures/basic.md"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("basic.md:"));
}

#[test]
fn frontmatter_single_file_json() {
    let out = md()
        .args(["frontmatter", "tests/fixtures/frontmatter.md"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["present"], true);
}

// --- Error handling ---

#[test]
fn multi_file_continues_on_error() {
    // Pass a nonexistent file alongside a real one
    let out = md()
        .args([
            "frontmatter",
            "--field",
            "title",
            "tests/fixtures/vault/alpha.md",
            "tests/fixtures/nonexistent.md",
        ])
        .output()
        .unwrap();
    // Should fail (some files failed)
    assert!(!out.status.success());
    // But should still produce output for the successful file
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alpha Doc"));
    // And stderr should mention the failed file
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("nonexistent"));
}
