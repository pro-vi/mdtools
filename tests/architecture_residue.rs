#[cfg(feature = "cli")]
use std::process::Command;

#[test]
fn quality_workflow_is_valid_yaml() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow = root.join(".github/workflows/quality.yml");
    // Published crates include Rust tests but not repository CI configuration.
    if !root.join(".git").exists() && !workflow.exists() {
        return;
    }
    let source = std::fs::read_to_string(workflow).unwrap();
    serde_yaml::from_str::<serde_yaml::Value>(&source)
        .expect("quality workflow must be valid YAML");
}

#[test]
fn public_source_has_no_previous_authority_names() {
    fn collect(directory: &std::path::Path, source: &mut String) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, source);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                source.push_str(&std::fs::read_to_string(path).unwrap());
            }
        }
    }
    let mut source = String::new();
    collect(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut source,
    );
    for removed in [
        "SectionIndex",
        "SectionTarget",
        "ResolvedSection",
        "MutationTargetRef",
        "BlockTargetRef",
        "pub struct SearchMatch",
        "EditOutcome",
        "pub mod parser",
        "pub fn projection",
        "ParsedDocument",
        "BlockInfo",
        "legacy_facts",
        "parser_index",
        "source_block_indices",
        "fn blocks(&self)",
        "mdtools.v2",
    ] {
        assert!(
            !source.contains(removed),
            "removed authority remains: {removed}"
        );
    }
}

#[test]
fn removed_command_modules_and_manual_inventory_are_absent() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for removed in [
        "src/commands",
        "src/multifile.rs",
        "src/block_edit.rs",
        "bench/md_inventory_v1.json",
    ] {
        assert!(
            !root.join(removed).exists(),
            "removed path remains: {removed}"
        );
    }
    // The recovered evaluation policy reads each pinned producer's schema.
    // Its filename is reusable; the retired manual inventory is not.
    let policy = root.join("bench/command_policy.py");
    if policy.exists() {
        let source = std::fs::read_to_string(policy).unwrap();
        for removed in [
            "MD_INVENTORY_PATH",
            "load_md_inventory",
            "md_inventory_v1.json",
        ] {
            assert!(
                !source.contains(removed),
                "retired manual inventory remains: {removed}"
            );
        }
    }
}

#[test]
fn transaction_consumers_do_not_reconstruct_parser_projection_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "src/fragment.rs",
        "src/section_edit.rs",
        "src/table.rs",
        "src/patch.rs",
        "src/patch/planner.rs",
    ] {
        let source = std::fs::read_to_string(root.join(relative)).unwrap();
        for forbidden in [
            "legacy_facts",
            "source_block_indices",
            "document.blocks()",
            "parser_index",
            "parser_blocks",
        ] {
            assert!(
                !source.contains(forbidden),
                "{relative} retains parser projection residue: {forbidden}"
            );
        }
    }
}

#[test]
fn source_evidence_has_no_patch_refinement_path() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in ["src/patch.rs", "src/patch/planner.rs"] {
        let source = std::fs::read_to_string(root.join(relative)).unwrap();
        assert!(
            !source.contains("SourceEvidence"),
            "{relative} gives source evidence mutation authority"
        );
    }
}

#[test]
#[cfg(feature = "cli")]
fn binary_and_schema_expose_exactly_five_commands() {
    let help = Command::new(env!("CARGO_BIN_EXE_md"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    let commands = help
        .lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            (name != "help").then_some(name)
        })
        .collect::<Vec<_>>();
    assert_eq!(commands, vec!["map", "read", "query", "patch", "schema"]);

    let schema = Command::new(env!("CARGO_BIN_EXE_md"))
        .arg("schema")
        .output()
        .unwrap();
    assert!(schema.status.success());
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
