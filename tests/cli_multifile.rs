use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

// --- Collect multi-file ---

#[test]
fn collect_directory_recursive_tsv() {
    let out = md()
        .args([
            "collect",
            "--field",
            "title,owner.name,status,count",
            "tests/fixtures/collect_vault",
            "-r",
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
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0], "path\ttitle\towner.name\tstatus\tcount");
    assert_eq!(lines[1], "tests/fixtures/collect_vault/plain.md\t\t\t\t");
    assert_eq!(
        lines[2],
        "tests/fixtures/collect_vault/root_toml.md\tRoot TOML\tLin\tdraft\t7"
    );
    assert_eq!(
        lines[3],
        "tests/fixtures/collect_vault/root_yaml.md\tRoot YAML\tAda\tpublished\t3"
    );
    assert_eq!(
        lines[4],
        "tests/fixtures/collect_vault/sub/nested.md\tNested Doc\tKai\tfalse\t"
    );
}

#[test]
fn collect_json_preserves_types_and_missing_rows() {
    let out = md()
        .args([
            "collect",
            "--field",
            "title,owner.name,status,count,tags,meta",
            "tests/fixtures/collect_vault",
            "-r",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["schema_version"], "mdtools.v1");
    assert_eq!(
        json["headers"],
        serde_json::json!([
            "path",
            "title",
            "owner.name",
            "status",
            "count",
            "tags",
            "meta"
        ])
    );

    let rows = json["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 4);

    let plain = rows
        .iter()
        .find(|row| row[0] == "tests/fixtures/collect_vault/plain.md")
        .unwrap();
    assert!(plain[1].is_null());
    assert!(plain[5].is_null());

    let yaml = rows
        .iter()
        .find(|row| row[0] == "tests/fixtures/collect_vault/root_yaml.md")
        .unwrap();
    assert_eq!(yaml[1], "Root YAML");
    assert_eq!(yaml[3], "published");
    assert_eq!(yaml[4], 3);
    assert_eq!(yaml[5], serde_json::json!(["alpha", "beta"]));

    let toml = rows
        .iter()
        .find(|row| row[0] == "tests/fixtures/collect_vault/root_toml.md")
        .unwrap();
    assert_eq!(toml[2], "Lin");
    assert_eq!(toml[4], 7);
    assert_eq!(toml[6], serde_json::json!({"reviewed": true}));

    let nested = rows
        .iter()
        .find(|row| row[0] == "tests/fixtures/collect_vault/sub/nested.md")
        .unwrap();
    assert_eq!(nested[3], false);
    assert_eq!(nested[6], serde_json::json!({"score": 9.5}));
}

#[test]
fn collect_continues_on_partial_failure() {
    let vault_dir = make_temp_collect_vault();
    let bad_path = vault_dir.join("broken.md");
    std::fs::write(
        &bad_path,
        "---\ntitle: valid\nnested: [broken\n---\n# Broken\n",
    )
    .unwrap();

    let out = md()
        .args([
            "collect",
            "--field",
            "title",
            vault_dir.to_str().unwrap(),
            "-r",
        ])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("path\ttitle\n"));
    assert!(stdout.contains("plain.md\t"));
    assert!(stdout.contains("root_yaml.md\tRoot YAML"));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("broken.md"));
    assert!(stderr.contains("1 file(s) failed"));

    std::fs::remove_dir_all(vault_dir).unwrap();
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

fn make_temp_collect_vault() -> std::path::PathBuf {
    let unique = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("mdtools-collect-{}-{}", std::process::id(), unique));
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    copy_fixture(
        "tests/fixtures/collect_vault/plain.md",
        &dir.join("plain.md"),
    );
    copy_fixture(
        "tests/fixtures/collect_vault/root_toml.md",
        &dir.join("root_toml.md"),
    );
    copy_fixture(
        "tests/fixtures/collect_vault/root_yaml.md",
        &dir.join("root_yaml.md"),
    );
    copy_fixture(
        "tests/fixtures/collect_vault/sub/nested.md",
        &dir.join("sub/nested.md"),
    );
    dir
}

fn copy_fixture(from: &str, to: &std::path::Path) {
    std::fs::write(to, std::fs::read(from).unwrap()).unwrap();
}
