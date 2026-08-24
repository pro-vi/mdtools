use crate::core_error::{CoreError, EtagTarget};
use crate::document::Document;
use crate::edit::{EditOutcome, EditPreservation};
use crate::fingerprint::{TargetEtag, TargetEtagGuard};
use crate::model::{FrontmatterFormat, MutationDisposition, SourceSpan};
use crate::parser::strip_frontmatter_delimiters;

#[derive(Clone, Debug)]
pub struct FrontmatterRecord {
    pub present: bool,
    pub etag: TargetEtag,
    pub format: Option<FrontmatterFormat>,
    pub span: Option<SourceSpan>,
    pub raw: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontmatterEditTarget {
    pub key_path: String,
    pub format: FrontmatterFormat,
}

#[derive(Clone, Debug)]
pub enum FrontmatterAction {
    Set(serde_json::Value),
    Delete,
}

#[derive(Clone, Debug)]
pub struct FrontmatterEdit {
    pub key_path: FrontmatterPath,
    pub action: FrontmatterAction,
    pub expect_etag: Option<TargetEtagGuard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontmatterPath(String);

impl FrontmatterPath {
    pub fn new(path: impl Into<String>) -> Result<Self, CoreError> {
        let path = path.into();
        if path.is_empty() || path.split('.').any(str::is_empty) {
            Err(CoreError::InvalidKeyPath {
                path,
                reason: "key cannot be empty",
            })
        } else {
            Ok(Self(path))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FrontmatterPath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub fn read(document: &Document) -> Result<FrontmatterRecord, CoreError> {
    let state = document.frontmatter_state();
    let etag = state
        .etag
        .parse::<TargetEtag>()
        .expect("parser frontmatter fingerprints are valid target etags");
    match (state.raw, state.format) {
        (Some(raw), Some(format)) => Ok(FrontmatterRecord {
            present: true,
            etag,
            format: Some(format),
            span: state.span,
            raw: Some(raw.to_string()),
            data: parse_data(raw, format)?,
        }),
        _ => Ok(FrontmatterRecord {
            present: false,
            etag,
            format: None,
            span: None,
            raw: None,
            data: empty_object(),
        }),
    }
}

pub fn project_field(data: &serde_json::Value, field: &str) -> serde_json::Value {
    let mut current = data;
    for segment in field.split('.') {
        match current.get(segment) {
            Some(value) => current = value,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

pub fn parse_data(raw: &str, format: FrontmatterFormat) -> Result<serde_json::Value, CoreError> {
    let content = strip_frontmatter_delimiters(raw);
    if content.trim().is_empty() {
        return Ok(empty_object());
    }
    match format {
        FrontmatterFormat::Yaml => serde_yaml::from_str(&content).map_err(|error| {
            CoreError::FrontmatterParseFailed(format!("invalid YAML frontmatter: {error}"))
        }),
        FrontmatterFormat::Toml => {
            let value = content.parse::<toml::Value>().map_err(|error| {
                CoreError::FrontmatterParseFailed(format!("invalid TOML frontmatter: {error}"))
            })?;
            serde_json::to_value(value).map_err(|error| {
                CoreError::FrontmatterParseFailed(format!(
                    "TOML to JSON conversion failed: {error}"
                ))
            })
        }
    }
}

pub fn edit(
    document: &Document,
    request: &FrontmatterEdit,
) -> Result<EditOutcome<FrontmatterEditTarget>, CoreError> {
    // The guard owns ordering: stale bytes must conflict before malformed
    // frontmatter or shape validation is attempted.
    let state = document.frontmatter_state();
    let actual_etag = state
        .etag
        .parse::<TargetEtag>()
        .expect("parser frontmatter fingerprints are valid target etags");
    if let Some(expected) = request.expect_etag.as_ref() {
        if expected.as_str() != actual_etag.as_str() {
            return Err(CoreError::TargetEtagMismatch {
                target: EtagTarget::Frontmatter,
                expected: expected.to_string(),
                actual: actual_etag.to_string(),
            });
        }
    }

    let validated = Document::parse_for_frontmatter_mutation(document.source())?;
    let document = &validated;
    let state = document.frontmatter_state();

    let format = state.format.unwrap_or(FrontmatterFormat::Yaml);
    let mut data = state
        .raw
        .map(|raw| parse_data(raw, format))
        .transpose()?
        .unwrap_or_else(empty_object);
    if state.raw.is_some() && !data.is_object() {
        return Err(CoreError::FrontmatterParseFailed(
            "frontmatter must be a mapping/object, not a scalar or array".into(),
        ));
    }
    let disposition = match &request.action {
        FrontmatterAction::Set(value) => {
            set_dot_path(&mut data, request.key_path.as_str(), value.clone())?
        }
        FrontmatterAction::Delete => delete_dot_path(&mut data, request.key_path.as_str())?,
    };
    let changed = disposition != MutationDisposition::NoChange;
    let span_before = state.span;
    let (content, span_after) = if changed {
        let block = serialize(&data, format)?;
        let content = if let Some(span) = span_before {
            format!("{}{}", block, &document.source()[span.byte_end as usize..])
        } else if document.source().is_empty() {
            block.clone()
        } else {
            format!("{}\n{}", block, document.source())
        };
        let span = SourceSpan {
            line_start: 1,
            line_end: block.matches('\n').count() as u32,
            byte_start: 0,
            byte_end: block.len() as u32,
        };
        (content, Some(span))
    } else {
        (document.source().to_string(), span_before)
    };

    Ok(EditOutcome {
        base_revision: document.revision().clone(),
        target: FrontmatterEditTarget {
            key_path: request.key_path.to_string(),
            format,
        },
        disposition,
        guarded: request.expect_etag.is_some(),
        line_endings: document.line_ending_style(),
        preservation: EditPreservation {
            preserves_non_target_bytes: true,
            target_span_before: span_before,
            target_span_after: span_after,
        },
        content,
    })
}

fn set_dot_path(
    root: &mut serde_json::Value,
    path: &str,
    value: serde_json::Value,
) -> Result<MutationDisposition, CoreError> {
    let segments = path.split('.').collect::<Vec<_>>();
    let mut current = root;
    for (index, segment) in segments[..segments.len() - 1].iter().enumerate() {
        match current {
            serde_json::Value::Object(map) => {
                current = map
                    .entry((*segment).to_string())
                    .or_insert_with(empty_object);
                if !current.is_object() {
                    return Err(CoreError::FrontmatterFieldConflict {
                        path: path.to_string(),
                        prefix: segments[..=index].join("."),
                    });
                }
            }
            _ => {
                return Err(CoreError::FrontmatterFieldConflict {
                    path: path.to_string(),
                    prefix: segments[..index].join("."),
                });
            }
        }
    }
    let key = segments.last().expect("validated path has one segment");
    let map = current
        .as_object_mut()
        .ok_or_else(|| CoreError::FrontmatterFieldConflict {
            path: path.to_string(),
            prefix: segments[..segments.len() - 1].join("."),
        })?;
    match map.get(*key) {
        Some(existing) if existing == &value => Ok(MutationDisposition::NoChange),
        Some(_) => {
            map.insert((*key).to_string(), value);
            Ok(MutationDisposition::Replaced)
        }
        None => {
            map.insert((*key).to_string(), value);
            Ok(MutationDisposition::Inserted)
        }
    }
}

fn delete_dot_path(
    root: &mut serde_json::Value,
    path: &str,
) -> Result<MutationDisposition, CoreError> {
    let segments = path.split('.').collect::<Vec<_>>();
    let mut current = root;
    for (index, segment) in segments[..segments.len() - 1].iter().enumerate() {
        match current {
            serde_json::Value::Object(map) => match map.get_mut(*segment) {
                Some(value) if value.is_object() => current = value,
                Some(_) => {
                    return Err(CoreError::FrontmatterFieldConflict {
                        path: path.to_string(),
                        prefix: segments[..=index].join("."),
                    });
                }
                None => return Ok(MutationDisposition::NoChange),
            },
            _ => return Ok(MutationDisposition::NoChange),
        }
    }
    let key = segments.last().expect("validated path has one segment");
    Ok(current
        .as_object_mut()
        .and_then(|map| map.shift_remove(*key))
        .map_or(MutationDisposition::NoChange, |_| {
            MutationDisposition::Deleted
        }))
}

fn serialize(data: &serde_json::Value, format: FrontmatterFormat) -> Result<String, CoreError> {
    match format {
        FrontmatterFormat::Yaml => {
            let yaml = serde_yaml::to_string(data).map_err(|error| {
                CoreError::FrontmatterParseFailed(format!("failed to serialize YAML: {error}"))
            })?;
            let content = yaml.strip_prefix("---\n").unwrap_or(&yaml);
            Ok(format!("---\n{content}---\n"))
        }
        FrontmatterFormat::Toml => {
            let value = json_to_toml(data)?;
            let content = toml::to_string_pretty(&value).map_err(|error| {
                CoreError::FrontmatterParseFailed(format!("failed to serialize TOML: {error}"))
            })?;
            Ok(format!("+++\n{content}+++\n"))
        }
    }
}

fn json_to_toml(value: &serde_json::Value) -> Result<toml::Value, CoreError> {
    match value {
        serde_json::Value::Null => Err(CoreError::FrontmatterParseFailed(
            "TOML does not support null values".into(),
        )),
        serde_json::Value::Bool(value) => Ok(toml::Value::Boolean(*value)),
        serde_json::Value::Number(value) => value
            .as_i64()
            .map(toml::Value::Integer)
            .or_else(|| value.as_f64().map(toml::Value::Float))
            .or_else(|| Some(toml::Value::String(value.to_string())))
            .ok_or_else(|| CoreError::FrontmatterParseFailed("invalid JSON number".into())),
        serde_json::Value::String(value) => Ok(toml::Value::String(value.clone())),
        serde_json::Value::Array(values) => values
            .iter()
            .map(json_to_toml)
            .collect::<Result<Vec<_>, _>>()
            .map(toml::Value::Array),
        serde_json::Value::Object(values) => values
            .iter()
            .map(|(key, value)| Ok((key.clone(), json_to_toml(value)?)))
            .collect::<Result<toml::map::Map<_, _>, _>>()
            .map(toml::Value::Table),
    }
}

fn empty_object() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}
