use crate::core_error::CoreError;
use crate::document::Document;
use crate::edit::{normalize_line_endings, SourceEdit};
use crate::fingerprint::TargetEtag;
use crate::model::{FrontmatterFormat, LineEndingStyle, MutationDisposition, SourceSpan};
use crate::parser::strip_frontmatter_delimiters;

#[derive(Clone, Debug)]
pub(crate) struct FrontmatterRecord {
    pub(crate) present: bool,
    pub(crate) etag: TargetEtag,
    pub(crate) format: Option<FrontmatterFormat>,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) raw: Option<String>,
    pub(crate) data: serde_json::Value,
}

#[derive(Clone, Debug)]
pub(crate) enum FrontmatterAction {
    Set(serde_json::Value),
    Delete,
}

#[derive(Clone, Debug)]
pub(crate) struct FrontmatterPathMutation {
    pub(crate) path: Vec<String>,
    pub(crate) action: FrontmatterAction,
}

pub(crate) struct FrontmatterBatchPlan {
    pub(crate) edit: Option<SourceEdit>,
    pub(crate) dispositions: Vec<MutationDisposition>,
}

pub(crate) fn read(document: &Document) -> Result<FrontmatterRecord, CoreError> {
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

pub(crate) fn parse_data(
    raw: &str,
    format: FrontmatterFormat,
) -> Result<serde_json::Value, CoreError> {
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

pub(crate) fn plan_path_batch(
    document: &Document,
    mutations: &[FrontmatterPathMutation],
) -> Result<FrontmatterBatchPlan, CoreError> {
    if mutations.is_empty() {
        return Ok(FrontmatterBatchPlan {
            edit: None,
            dispositions: Vec::new(),
        });
    }
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
    let original_data = data.clone();
    let mut dispositions = Vec::with_capacity(mutations.len());
    for mutation in mutations {
        if mutation.path.is_empty() {
            return Err(CoreError::InvalidKeyPath {
                path: String::new(),
                reason: "key path cannot be empty",
            });
        }
        let display = serde_json::to_string(&mutation.path).unwrap();
        dispositions.push(match &mutation.action {
            FrontmatterAction::Set(value) => {
                set_path(&mut data, &mutation.path, &display, value.clone())?
            }
            FrontmatterAction::Delete => delete_path(&mut data, &mutation.path, &display)?,
        });
    }
    let has_changes = dispositions
        .iter()
        .any(|disposition| *disposition != MutationDisposition::NoChange);
    if has_changes {
        if let Some(raw) = state.raw {
            let round_trip = normalize_line_endings(
                &serialize(&original_data, format)?,
                document.line_ending_style(),
            );
            let comparison_source = match format {
                FrontmatterFormat::Yaml => canonicalize_yaml_mapping_keys(raw),
                FrontmatterFormat::Toml => raw.to_string(),
            };
            if round_trip != comparison_source {
                return Err(CoreError::InvalidPatch(
                    "frontmatter formatting is not stable under field serialization".into(),
                ));
            }
        }
    }
    let edit = if has_changes {
        let block =
            normalize_line_endings(&serialize(&data, format)?, document.line_ending_style());
        Some(match state.span {
            Some(span) => SourceEdit {
                start: span.byte_start as usize,
                end: span.byte_end as usize,
                replacement: block,
            },
            None if document.source().is_empty() => SourceEdit {
                start: 0,
                end: 0,
                replacement: block,
            },
            None => SourceEdit {
                start: 0,
                end: 0,
                replacement: format!("{block}{}", insertion_line_ending(document)),
            },
        })
    } else {
        None
    };
    Ok(FrontmatterBatchPlan { edit, dispositions })
}

fn insertion_line_ending(document: &Document) -> &'static str {
    match document.line_ending_style() {
        LineEndingStyle::Crlf => "\r\n",
        LineEndingStyle::Lf | LineEndingStyle::Mixed => "\n",
    }
}

fn canonicalize_yaml_mapping_keys(raw: &str) -> String {
    raw.split_inclusive('\n')
        .map(|line| {
            let (content, ending) = line
                .strip_suffix('\n')
                .map_or((line, ""), |content| (content, "\n"));
            let indent_end = content.len() - content.trim_start_matches([' ', '\t']).len();
            let (indent, rest) = content.split_at(indent_end);
            let Some(quote) = rest.as_bytes().first().copied() else {
                return line.to_string();
            };
            if !matches!(quote, b'\'' | b'"') {
                return line.to_string();
            }
            let Some(close) = rest[1..].find(char::from(quote)).map(|index| index + 1) else {
                return line.to_string();
            };
            if rest.as_bytes().get(close + 1) != Some(&b':') {
                return line.to_string();
            }
            let key = &rest[1..close];
            if key.contains(['\\', '\'', '"']) {
                return line.to_string();
            }
            let canonical = if key.is_empty() {
                "''"
            } else if key.chars().all(|character| {
                character.is_alphanumeric() || matches!(character, '_' | '-' | '.')
            }) {
                key
            } else {
                return line.to_string();
            };
            format!("{indent}{canonical}{}{ending}", &rest[close + 1..])
        })
        .collect()
}

fn set_path(
    root: &mut serde_json::Value,
    segments: &[String],
    display: &str,
    value: serde_json::Value,
) -> Result<MutationDisposition, CoreError> {
    let mut current = root;
    for (index, segment) in segments[..segments.len() - 1].iter().enumerate() {
        let map = current
            .as_object_mut()
            .ok_or_else(|| CoreError::FrontmatterFieldConflict {
                path: display.into(),
                prefix: serde_json::to_string(&segments[..index]).unwrap(),
            })?;
        current = map.entry(segment.clone()).or_insert_with(empty_object);
        if !current.is_object() {
            return Err(CoreError::FrontmatterFieldConflict {
                path: display.into(),
                prefix: serde_json::to_string(&segments[..=index]).unwrap(),
            });
        }
    }
    let key = segments.last().unwrap();
    let map = current
        .as_object_mut()
        .ok_or_else(|| CoreError::FrontmatterFieldConflict {
            path: display.into(),
            prefix: display.into(),
        })?;
    match map.get(key) {
        Some(existing) if existing == &value => Ok(MutationDisposition::NoChange),
        Some(_) => {
            map.insert(key.clone(), value);
            Ok(MutationDisposition::Replaced)
        }
        None => {
            map.insert(key.clone(), value);
            Ok(MutationDisposition::Inserted)
        }
    }
}

fn delete_path(
    root: &mut serde_json::Value,
    segments: &[String],
    display: &str,
) -> Result<MutationDisposition, CoreError> {
    let mut current = root;
    for (index, segment) in segments[..segments.len() - 1].iter().enumerate() {
        match current.as_object_mut().and_then(|map| map.get_mut(segment)) {
            Some(value) if value.is_object() => current = value,
            Some(_) => {
                return Err(CoreError::FrontmatterFieldConflict {
                    path: display.into(),
                    prefix: serde_json::to_string(&segments[..=index]).unwrap(),
                });
            }
            None => return Ok(MutationDisposition::NoChange),
        }
    }
    let key = segments.last().unwrap();
    Ok(current
        .as_object_mut()
        .and_then(|map| map.shift_remove(key))
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
