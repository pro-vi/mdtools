use mdtools::protocol::{protocol_schema, CLI_COMMANDS};
use mdtools::target::TargetQuery;
use mdtools::HeadingMatchMode;

#[test]
fn protocol_schema_covers_every_authoritative_surface() {
    let schema = protocol_schema();
    assert_eq!(schema["schema_version"], "mdtools.v2");
    for key in [
        "target_address",
        "target_query",
        "query_result",
        "target_snapshot",
        "target_read",
        "patch",
        "patch_receipt",
    ] {
        assert_eq!(
            schema[key]["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
    }
    assert!(schema["target_read"]["$defs"]["SectionFragment"].is_object());
    assert!(schema["target_snapshot"]["$defs"]["GuardAuthority"].is_object());
    assert!(schema["target_query"].to_string().contains("search"));
    assert!(schema["query_result"].to_string().contains("EvidenceRange"));
}

#[test]
fn five_command_metadata_is_unique_and_protocol_named() {
    assert_eq!(
        CLI_COMMANDS
            .iter()
            .map(|command| command.name)
            .collect::<Vec<_>>(),
        vec!["map", "read", "query", "patch", "schema"]
    );
    let mut names = CLI_COMMANDS
        .iter()
        .map(|command| command.name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), CLI_COMMANDS.len());
}

#[test]
fn patch_and_receipt_variants_appear_exactly_once() {
    let schema = protocol_schema();
    let patch = schema["patch"]["$defs"]["PatchOp"]["oneOf"]
        .as_array()
        .unwrap();
    let receipt = schema["patch_receipt"]["oneOf"].as_array().unwrap();
    let operations = [
        "replace_block",
        "delete_block",
        "insert_block",
        "move_block",
        "replace_section",
        "insert_section",
        "replace_preamble",
        "delete_section",
        "move_section",
        "set_task_status",
        "set_frontmatter",
        "delete_frontmatter",
        "replace_table_row",
        "insert_table_row",
        "delete_table_row",
    ];
    for operation in operations {
        assert_eq!(
            patch
                .iter()
                .filter(|variant| variant["properties"]["op"]["const"] == operation)
                .count(),
            1,
            "patch operation {operation}"
        );
        assert_eq!(
            receipt
                .iter()
                .filter(|variant| variant["properties"]["operation"]["const"] == operation)
                .count(),
            1,
            "receipt operation {operation}"
        );
    }
    assert_eq!(patch.len(), operations.len());
    assert_eq!(receipt.len(), operations.len());
}

#[test]
fn target_query_wire_round_trips_and_rejects_unknown_fields() {
    let query = TargetQuery::Section {
        text: "Work".into(),
        match_mode: HeadingMatchMode::ContainsIgnoreCase,
    };
    let wire = serde_json::to_value(&query).unwrap();
    assert_eq!(serde_json::from_value::<TargetQuery>(wire).unwrap(), query);
    assert!(serde_json::from_value::<TargetQuery>(serde_json::json!({
        "type": "all",
        "unknown": true
    }))
    .is_err());
}
