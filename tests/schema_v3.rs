use mdtools::protocol::{protocol_schema, CLI_COMMANDS};
use mdtools::target::TargetQuery;
use mdtools::HeadingMatchMode;

#[test]
fn protocol_schema_covers_every_authoritative_surface() {
    let schema = protocol_schema();
    assert_eq!(schema["schema_version"], "mdtools.v3");
    for key in [
        "target_address",
        "target_query",
        "query_result",
        "target_snapshot",
        "target_read",
        "patch",
        "patch_receipt",
        "error_envelope",
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
    assert!(schema["query_result"]
        .to_string()
        .contains("SourceEvidenceRange"));

    let envelope = &schema["error_envelope"];
    assert_eq!(
        envelope["$defs"]["ProtocolSchemaVersion"]["enum"],
        serde_json::json!(["mdtools.v3"])
    );
    assert_eq!(
        envelope["$defs"]["DiagnosticCode"]["enum"],
        serde_json::json!([
            "io",
            "parse",
            "invalid_input",
            "not_found",
            "conflict",
            "invariant"
        ])
    );
    assert_eq!(envelope["properties"]["exit_code"]["minimum"], 1);
    assert_eq!(envelope["properties"]["exit_code"]["maximum"], 5);
    assert_eq!(envelope["properties"]["hint"]["type"], "string");
    assert!(!envelope["required"]
        .as_array()
        .unwrap()
        .iter()
        .any(|field| field == "hint"));
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

    assert!(serde_json::from_value::<TargetQuery>(serde_json::json!({
        "type": "search",
        "text": "",
        "match_mode": "literal",
        "block_kinds": [],
        "include_source_gaps": false
    }))
    .is_err());
    assert!(serde_json::from_value::<TargetQuery>(serde_json::json!({
        "type": "search",
        "text": "needle",
        "match_mode": "literal",
        "block_kinds": []
    }))
    .is_err());
    assert!(serde_json::from_value::<TargetQuery>(serde_json::json!({
        "type": "section",
        "text": "",
        "match_mode": "contains"
    }))
    .is_err());
    assert!(serde_json::from_value::<TargetQuery>(serde_json::json!({
        "type": "section",
        "text": "",
        "match_mode": "exact"
    }))
    .is_ok());
    assert!(serde_json::from_value::<TargetQuery>(serde_json::json!({
        "type": "frontmatter_field",
        "path": []
    }))
    .is_err());

    let schema = protocol_schema();
    let variants = schema["target_query"]["oneOf"].as_array().unwrap();
    let search = variants
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "search")
        .unwrap();
    assert_eq!(search["properties"]["text"]["minLength"], 1);
    assert!(search["required"]
        .as_array()
        .unwrap()
        .iter()
        .any(|field| field == "include_source_gaps"));
    let section = variants
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "section")
        .unwrap();
    assert_eq!(
        section["allOf"][0]["then"]["properties"]["text"]["minLength"],
        1
    );
    let frontmatter = variants
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "frontmatter_field")
        .unwrap();
    assert_eq!(frontmatter["properties"]["path"]["minItems"], 1);
}
