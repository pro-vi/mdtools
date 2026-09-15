use schemars::{generate::SchemaSettings, schema_for, JsonSchema};
use serde::Serialize;

use crate::patch::{Patch, PatchReceipt};
use crate::read::TargetRead;
use crate::target::{
    QueryResult, TargetAddress, TargetQuery, TargetSnapshot, TargetSummary,
    NON_EMPTY_QUERY_TEXT_MIN_LENGTH, NON_EMPTY_SECTION_MATCH_MODES,
};

pub const MAP_SUMMARY: &str = "List exact targets; --json includes full snapshots";
pub const READ_SUMMARY: &str = "Read exact target content; --json includes its typed view";
pub const QUERY_SUMMARY: &str = "Discover targets and search evidence with compact results";
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
        output: "TargetOverview[]; --json: TargetSnapshot[]",
        mutating: false,
    },
    CliCommandMetadata {
        name: "read",
        summary: READ_SUMMARY,
        input: "TargetAddress (--address/--from) or TargetQuery (--query, exactly one match)",
        output: "Markdown or FrontmatterFieldValue; --json: TargetRead",
        mutating: false,
    },
    CliCommandMetadata {
        name: "query",
        summary: QUERY_SUMMARY,
        input: "TargetQuery",
        output: "QueryOverview[]; --json: QueryResult[]",
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

/// Short examples of existing input types; never a replacement decoder or schema.
/// Called only after authoritative typed deserialization failed.
pub fn input_guidance(expected: &str, source: &str) -> String {
    match expected {
        "TargetQuery" => {
            let rejected = serde_json::from_str::<serde_json::Value>(source).ok();
            let kind = rejected
                .as_ref()
                .and_then(|value| value.get("type"))
                .and_then(serde_json::Value::as_str);
            let example = match kind {
                Some("section") => TargetQuery::Section {
                    // Static example text prevents rejected document values entering guidance.
                    text: "Heading".into(),
                    match_mode: crate::HeadingMatchMode::Exact,
                },
                Some("search") => TargetQuery::Search {
                    text: "text".into(),
                    match_mode: crate::SearchMatchMode::Literal,
                    block_kinds: Vec::new(),
                    include_source_gaps: false,
                    max_results: 100,
                },
                Some("task") => TargetQuery::Task {
                    status: Some(crate::TaskStatus::Pending),
                    contains: None,
                },
                Some("link") => TargetQuery::Link {
                    text: None,
                    destination: None,
                },
                Some("frontmatter_field") => TargetQuery::FrontmatterField {
                    path: vec!["field".into()],
                },
                Some("all") => TargetQuery::All,
                _ => TargetQuery::Kind {
                    kind: crate::target::TargetKind::Section,
                },
            };
            format!(
                "Example{}: {}; schema: md schema | jq -c .target_query",
                if kind == Some("section") {
                    " (exact section match)"
                } else {
                    ""
                },
                serde_json::to_string(&example).expect("typed query example serializes")
            )
        }
        "TargetAddress" => format!(
            "Example: {}; schema: md schema | jq -c .target_address",
            serde_json::to_string(&TargetAddress::Preamble)
                .expect("typed address example serializes")
        ),
        "Patch" => "use observed revisions and guards; schema: md schema | jq -c .patch".into(),
        _ => "use `md schema` for the generated input shape".into(),
    }
}

/// Discovery presentation of an existing target, without mutation evidence.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub struct TargetOverview<'a> {
    pub address: &'a TargetAddress,
    pub summary: &'a TargetSummary,
}

impl<'a> From<&'a TargetSnapshot> for TargetOverview<'a> {
    fn from(target: &'a TargetSnapshot) -> Self {
        Self {
            address: &target.address,
            summary: &target.summary,
        }
    }
}

/// Compact query presentation. Source evidence remains explicitly targetless.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryOverview<'a> {
    Target {
        target: TargetOverview<'a>,
    },
    Evidence {
        target: &'a TargetAddress,
        span: &'a crate::SourceSpan,
        preview: &'a str,
    },
    SourceEvidence {
        span: &'a crate::SourceSpan,
        preview: &'a str,
    },
}

impl<'a> From<&'a QueryResult> for QueryOverview<'a> {
    fn from(result: &'a QueryResult) -> Self {
        match result {
            QueryResult::Target { target } => Self::Target {
                target: target.into(),
            },
            QueryResult::Evidence { evidence } => Self::Evidence {
                target: &evidence.target,
                span: &evidence.span,
                preview: &evidence.preview,
            },
            QueryResult::SourceEvidence { evidence } => Self::SourceEvidence {
                span: &evidence.span,
                preview: &evidence.preview,
            },
        }
    }
}

/// Field content preserves the distinction between an absent field and JSON null.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub struct FrontmatterFieldValue<'a> {
    pub present: bool,
    pub value: Option<&'a serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PatchPreview {
    pub source: String,
    pub receipts: Vec<PatchReceipt>,
}

#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub enum ProtocolSchemaVersion {
    #[serde(rename = "mdtools.v3")]
    V3,
}

#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Io,
    Parse,
    InvalidInput,
    ResultLimit,
    NotFound,
    Conflict,
    Invariant,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ErrorEnvelope<'a> {
    pub schema_version: ProtocolSchemaVersion,
    pub error: DiagnosticCode,
    #[schemars(range(min = 1, max = 5))]
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
        "target_overview": output_schema::<TargetOverview<'static>>(),
        "query_overview": output_schema::<QueryOverview<'static>>(),
        "frontmatter_field_value": output_schema::<FrontmatterFieldValue<'static>>(),
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
                variant["properties"]["text"]["minLength"] =
                    serde_json::json!(NON_EMPTY_QUERY_TEXT_MIN_LENGTH);
            }
            Some("section") => {
                let modes = serde_json::to_value(NON_EMPTY_SECTION_MATCH_MODES)
                    .expect("section match modes serialize");
                variant["allOf"] = serde_json::json!([{
                    "if": {
                        "properties": {
                            "match_mode": {
                                "enum": modes
                            }
                        }
                    },
                    "then": {
                        "properties": {
                            "text": { "minLength": NON_EMPTY_QUERY_TEXT_MIN_LENGTH }
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
