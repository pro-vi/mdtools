//! U2: `md schema --json` — the machine-readable CLI surface.

use std::process::Command;

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn schema() -> serde_json::Value {
    let out = md().args(["schema", "--json"]).output().unwrap();
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn schema_carries_versions_and_capabilities() {
    let s = schema();
    assert_eq!(s["schema_version"], "mdtools.v1");
    assert_eq!(s["binary_version"], env!("CARGO_PKG_VERSION"));
    let caps: Vec<&str> = s["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert!(caps.contains(&"error_envelope"));
    assert!(caps.contains(&"section_etag"));
}

#[test]
fn schema_matches_bench_inventory() {
    // md_inventory_v1.json is the bench-side mirror; the binary is the
    // authority. Every inventory entry must match the schema's name+kind,
    // and the schema must add nothing the inventory misses except `schema`.
    let inv_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/bench/md_inventory_v1.json"
    );
    let inv: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(inv_path).unwrap()).unwrap();
    let s = schema();

    let mut schema_kinds = std::collections::BTreeMap::new();
    for c in s["commands"].as_array().unwrap() {
        schema_kinds.insert(
            c["name"].as_str().unwrap().to_string(),
            c["kind"].as_str().unwrap().to_string(),
        );
    }

    let mut inv_names = std::collections::BTreeSet::new();
    for c in inv["commands"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let kind = c["kind"].as_str().unwrap();
        inv_names.insert(name.to_string());
        assert_eq!(
            schema_kinds.get(name).map(String::as_str),
            Some(kind),
            "inventory/schema drift for {name}"
        );
    }

    for name in schema_kinds.keys() {
        assert!(
            inv_names.contains(name),
            "command {name} in schema but missing from bench/md_inventory_v1.json"
        );
    }
}

#[test]
fn expect_etag_listed_on_exactly_the_guarded_commands() {
    let s = schema();
    let mut with_flag = std::collections::BTreeSet::new();
    let mut dual = std::collections::BTreeSet::new();
    for c in s["commands"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap().to_string();
        for f in c["flags"].as_array().unwrap() {
            match f["long"].as_str().unwrap() {
                "--expect-etag" => {
                    with_flag.insert(name.clone());
                }
                "--expect-source-etag" | "--expect-dest-etag" => {
                    dual.insert(name.clone());
                }
                _ => {}
            }
        }
    }
    let expected: std::collections::BTreeSet<String> = [
        "replace-block",
        "insert-block",
        "delete-block",
        "replace-section",
        "delete-section",
        "replace-table-row",
        "delete-table-row",
        "set",
        "set-task",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(with_flag, expected);
    assert_eq!(
        dual,
        ["move-section".to_string()].into_iter().collect()
    );
}

#[test]
fn diagnostics_table_is_total_with_valid_exit_codes() {
    let s = schema();
    let diags = s["diagnostic_codes"].as_array().unwrap();
    assert_eq!(diags.len(), 22);
    for d in diags {
        let code = d["code"].as_str().unwrap();
        let exit = d["exit_code"].as_u64().unwrap();
        assert!(
            (1..=4).contains(&exit),
            "{code}: exit {exit} outside 1..=4"
        );
    }
}

#[test]
fn tsv_mode_prints_one_flat_row_per_command() {
    let s = schema();
    let expected = s["commands"].as_array().unwrap().len();
    let out = md().args(["schema"]).output().unwrap();
    assert!(out.status.success());
    let rows = String::from_utf8(out.stdout).unwrap();
    assert_eq!(rows.lines().count(), expected);
    for line in rows.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 3, "row: {line}");
        assert!(cols[1] == "query" || cols[1] == "mutation", "row: {line}");
    }
}
