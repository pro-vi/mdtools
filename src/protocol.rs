use schemars::{schema_for, JsonSchema};
use serde::Serialize;

use crate::patch::{Patch, PatchReceipt};
use crate::read::TargetRead;
use crate::target::{TargetAddress, TargetQuery, TargetSnapshot};

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
        output: "TargetSnapshot[]",
        mutating: false,
    },
    CliCommandMetadata {
        name: "patch",
        summary: PATCH_SUMMARY,
        input: "Patch",
        output: "PatchReceipt[] or Markdown",
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
        "target_query": serde_json::to_value(schema_for!(TargetQuery)).expect("target query schema serializes"),
        "target_snapshot": serde_json::to_value(schema_for!(TargetSnapshot)).expect("target snapshot schema serializes"),
        "target_read": serde_json::to_value(schema_for!(TargetRead)).expect("target read schema serializes"),
        "patch": patch_schema(),
        "patch_receipt": patch_receipt_schema(),
    })
}
