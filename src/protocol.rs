use schemars::{generate::SchemaSettings, schema_for, JsonSchema};
use serde::Serialize;

use crate::patch::{Patch, PatchReceipt};
use crate::read::TargetRead;
use crate::target::{QueryResult, TargetAddress, TargetQuery, TargetSnapshot};

pub const MAP_SUMMARY: &str = "Map every canonical Markdown target in source order";
pub const READ_SUMMARY: &str = "Read one exact target through its typed Markdown view";
pub const QUERY_SUMMARY: &str = "Discover targets with a typed fuzzy query";
pub const PATCH_SUMMARY: &str = "Apply one guarded patch to a Markdown document";
pub const SCHEMA_SUMMARY: &str = "Print the generated mdtools protocol schema";

#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub struct CliCommandMetadata {
    pub name: &'static str,
    pub summary: &'static str,
    pub input: &'static str,
    pub output: &'static str,
    pub mutating: bool,
}

pub const CLI_COMMANDS: &[CliCommandMetadata] = &[
    CliCommandMetadata {
        name: "map",
        summary: MAP_SUMMARY,
        input: "file",
        output: "TargetSnapshot[]",
        mutating: false,
    },
    CliCommandMetadata {
        name: "read",
        summary: READ_SUMMARY,
        input: "TargetAddress",
        output: "TargetRead",
        mutating: false,
    },
    CliCommandMetadata {
        name: "query",
        summary: QUERY_SUMMARY,
        input: "TargetQuery",
        output: "QueryResult[]",
        mutating: false,
    },
    CliCommandMetadata {
        name: "patch",
        summary: PATCH_SUMMARY,
        input: "Patch",
        output: "PatchReceipt[], PatchPreview, or Markdown",
        mutating: true,
    },
    CliCommandMetadata {
        name: "schema",
        summary: SCHEMA_SUMMARY,
        input: "none",
        output: "JSON Schema",
        mutating: false,
    },
];

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PatchPreview {
    pub source: String,
    pub receipts: Vec<PatchReceipt>,
}

#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub enum ProtocolSchemaVersion {
    #[serde(rename = "mdtools.v2")]
    V2,
}

#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Io,
    Parse,
    InvalidInput,
    NotFound,
    Conflict,
    Invariant,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ErrorEnvelope<'a> {
    pub schema_version: ProtocolSchemaVersion,
    pub error: DiagnosticCode,
    #[schemars(range(min = 1, max = 4))]
    pub exit_code: u8,
    pub message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub hint: Option<&'a str>,
}

pub fn patch_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(Patch)).expect("generated patch schema serializes")
}

pub fn patch_receipt_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(PatchReceipt))
        .expect("generated patch receipt schema serializes")
}

pub fn protocol_schema() -> serde_json::Value {
    serde_json::json!({
        "schema_version": crate::model::SCHEMA_VERSION,
        "commands": CLI_COMMANDS,
        "target_address": serde_json::to_value(schema_for!(TargetAddress)).expect("target address schema serializes"),
        "target_query": target_query_schema(),
        "query_result": serde_json::to_value(schema_for!(QueryResult)).expect("query result schema serializes"),
        "target_snapshot": serde_json::to_value(schema_for!(TargetSnapshot)).expect("target snapshot schema serializes"),
        "target_read": serde_json::to_value(schema_for!(TargetRead)).expect("target read schema serializes"),
        "patch": patch_schema(),
        "patch_receipt": patch_receipt_schema(),
        "patch_preview": serde_json::to_value(schema_for!(PatchPreview)).expect("patch preview schema serializes"),
        "error_envelope": output_schema::<ErrorEnvelope<'static>>(),
    })
}

fn target_query_schema() -> serde_json::Value {
    let mut schema =
        serde_json::to_value(schema_for!(TargetQuery)).expect("target query schema serializes");
    for variant in schema["oneOf"]
        .as_array_mut()
        .expect("target query schema has variants")
    {
        match variant["properties"]["type"]["const"].as_str() {
            Some("search") => {
                variant["properties"]["text"]["minLength"] = serde_json::json!(1);
            }
            Some("section") => {
                variant["allOf"] = serde_json::json!([{
                    "if": {
                        "properties": {
                            "match_mode": {
                                "enum": ["contains", "contains_ignore_case"]
                            }
                        }
                    },
                    "then": {
                        "properties": {
                            "text": { "minLength": 1 }
                        }
                    }
                }]);
            }
            _ => {}
        }
    }
    schema
}

fn output_schema<T: JsonSchema>() -> serde_json::Value {
    let generator = SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator();
    serde_json::to_value(generator.into_root_schema_for::<T>())
        .expect("generated output schema serializes")
}
