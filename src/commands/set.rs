use crate::cli::SetArgs;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use crate::parser::ParsedDocument;

pub fn run(args: &SetArgs, json: bool) -> Result<(), CommandError> {
    // Validate args
    if args.key.is_empty() || args.key.split('.').any(|s| s.is_empty()) {
        return Err(CommandError::invalid_key_path(&args.key, "key cannot be empty"));
    }
    if args.delete && args.value.is_some() {
        return Err(CommandError::new(
            DiagnosticCode::InvalidKeyPath,
            "cannot provide a value with --delete",
        ));
    }
    if !args.delete && args.value.is_none() {
        return Err(CommandError::new(
            DiagnosticCode::InvalidKeyPath,
            "value is required (use --delete to remove a key)",
        ));
    }
    if args.string && args.delete {
        return Err(CommandError::new(
            DiagnosticCode::InvalidKeyPath,
            "cannot use --string with --delete",
        ));
    }

    let source = std::fs::read_to_string(&args.file)?;

    // Determine if we have existing frontmatter
    let (mut data, fm_format, fm_byte_end, had_frontmatter) = parse_existing_frontmatter(&source)?;

    // Apply mutation
    let disposition = if args.delete {
        delete_dot_path(&mut data, &args.key)?
    } else {
        let value = parse_value(args.value.as_ref().unwrap(), args.string)?;
        set_dot_path(&mut data, &args.key, value)?
    };
    let changed = disposition != MutationDisposition::NoChange;

    // Serialize back
    let new_fm_block = serialize_frontmatter(&data, fm_format)?;

    // Build output document
    let output_doc = if had_frontmatter {
        format!("{}{}", &new_fm_block, &source[fm_byte_end..])
    } else if source.is_empty() {
        new_fm_block.clone()
    } else {
        format!("{}\n{}", &new_fm_block, &source)
    };

    // Build mutation result
    let target = MutationTargetRef::FrontmatterField(FrontmatterFieldTargetRef {
        kind: MutationTargetKind::FrontmatterField,
        key_path: args.key.clone(),
        format: fm_format,
    });

    let span_before = if had_frontmatter {
        Some(SourceSpan {
            line_start: 1,
            line_end: source[..fm_byte_end].matches('\n').count() as u32,
            byte_start: 0,
            byte_end: fm_byte_end as u32,
        })
    } else {
        None
    };

    let span_after = if changed {
        let line_count = new_fm_block.matches('\n').count() as u32;
        Some(SourceSpan {
            line_start: 1,
            line_end: line_count,
            byte_start: 0,
            byte_end: new_fm_block.len() as u32,
        })
    } else {
        span_before
    };

    let line_endings = if had_frontmatter {
        let doc = ParsedDocument::parse(source)?;
        doc.line_ending_style()
    } else {
        LineEndingStyle::Lf
    };

    emit_set_mutation(
        args.in_place,
        json,
        &args.file,
        target,
        disposition,
        changed,
        line_endings,
        span_before,
        span_after,
        &output_doc,
    )
}

fn parse_existing_frontmatter(
    source: &str,
) -> Result<(serde_json::Value, FrontmatterFormat, usize, bool), CommandError> {
    // Try to detect frontmatter
    let first_line = source.lines().next().unwrap_or("");
    let (delimiter, format) = if first_line == "---" {
        ("---", FrontmatterFormat::Yaml)
    } else if first_line == "+++" {
        ("+++", FrontmatterFormat::Toml)
    } else {
        // No frontmatter — return empty object
        return Ok((
            serde_json::Value::Object(serde_json::Map::new()),
            FrontmatterFormat::Yaml,
            0,
            false,
        ));
    };

    // Find closing delimiter
    let closing_pos = find_closing_delimiter(source, delimiter);
    let fm_byte_end = match closing_pos {
        Some(end) => end,
        None => {
            return Err(CommandError::new(
                DiagnosticCode::FrontmatterParseFailed,
                format!("unclosed frontmatter (no closing '{}')", delimiter),
            ));
        }
    };

    // Extract raw content between delimiters
    let delim_len = delimiter.len();
    let after_open = delim_len + 1; // skip delimiter + newline
    let close_start = source[..fm_byte_end]
        .rfind(delimiter)
        .unwrap_or(fm_byte_end);
    let content = &source[after_open..close_start];

    // Parse to JSON Value
    let data = match format {
        FrontmatterFormat::Yaml => {
            if content.trim().is_empty() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                serde_yaml::from_str::<serde_json::Value>(content).map_err(|e| {
                    CommandError::new(
                        DiagnosticCode::FrontmatterParseFailed,
                        format!("invalid YAML frontmatter: {}", e),
                    )
                })?
            }
        }
        FrontmatterFormat::Toml => {
            if content.trim().is_empty() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                let toml_val: toml::Value = content.parse().map_err(|e: toml::de::Error| {
                    CommandError::new(
                        DiagnosticCode::FrontmatterParseFailed,
                        format!("invalid TOML frontmatter: {}", e),
                    )
                })?;
                serde_json::to_value(&toml_val).map_err(|e| {
                    CommandError::new(
                        DiagnosticCode::FrontmatterParseFailed,
                        format!("TOML to JSON conversion failed: {}", e),
                    )
                })?
            }
        }
    };

    // Ensure it's an object
    if !data.is_object() {
        return Err(CommandError::new(
            DiagnosticCode::FrontmatterParseFailed,
            "frontmatter must be a mapping/object, not a scalar or array",
        ));
    }

    Ok((data, format, fm_byte_end, true))
}

fn find_closing_delimiter(source: &str, delimiter: &str) -> Option<usize> {
    // Skip the first line (opening delimiter)
    let after_first_line = source.find('\n')? + 1;
    let mut pos = after_first_line;
    for line in source[after_first_line..].lines() {
        let line_end = pos + line.len();
        if line == delimiter {
            // Include trailing newline if present
            let end = if source.as_bytes().get(line_end) == Some(&b'\n') {
                line_end + 1
            } else {
                line_end
            };
            return Some(end);
        }
        // Advance past the line + its newline
        pos = if source.as_bytes().get(line_end) == Some(&b'\n') {
            line_end + 1
        } else {
            line_end
        };
    }
    None
}

fn parse_value(raw: &str, force_string: bool) -> Result<serde_json::Value, CommandError> {
    if force_string {
        return Ok(serde_json::Value::String(raw.to_string()));
    }
    // Try YAML scalar parsing
    match serde_yaml::from_str::<serde_json::Value>(raw) {
        Ok(val) => Ok(val),
        Err(_) => Ok(serde_json::Value::String(raw.to_string())),
    }
}

fn set_dot_path(
    root: &mut serde_json::Value,
    path: &str,
    value: serde_json::Value,
) -> Result<MutationDisposition, CommandError> {
    let segments: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, segment) in segments[..segments.len() - 1].iter().enumerate() {
        match current {
            serde_json::Value::Object(map) => {
                current = map
                    .entry(segment.to_string())
                    .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                if !current.is_object() {
                    let prefix = segments[..=i].join(".");
                    return Err(CommandError::frontmatter_field_conflict(path, &prefix));
                }
            }
            _ => {
                let prefix = segments[..i].join(".");
                return Err(CommandError::frontmatter_field_conflict(path, &prefix));
            }
        }
    }

    let last_key = segments.last().unwrap();
    match current.as_object_mut() {
        Some(map) => {
            if let Some(existing) = map.get(*last_key) {
                if *existing == value {
                    Ok(MutationDisposition::NoChange)
                } else {
                    map.insert(last_key.to_string(), value);
                    Ok(MutationDisposition::Replaced)
                }
            } else {
                map.insert(last_key.to_string(), value);
                Ok(MutationDisposition::Inserted)
            }
        }
        None => {
            let prefix = segments[..segments.len() - 1].join(".");
            Err(CommandError::frontmatter_field_conflict(path, &prefix))
        }
    }
}

fn delete_dot_path(
    root: &mut serde_json::Value,
    path: &str,
) -> Result<MutationDisposition, CommandError> {
    let segments: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, segment) in segments[..segments.len() - 1].iter().enumerate() {
        match current {
            serde_json::Value::Object(map) => match map.get_mut(*segment) {
                Some(val) => {
                    if !val.is_object() {
                        let prefix = segments[..=i].join(".");
                        return Err(CommandError::frontmatter_field_conflict(path, &prefix));
                    }
                    current = val;
                }
                None => return Ok(MutationDisposition::NoChange),
            },
            _ => return Ok(MutationDisposition::NoChange),
        }
    }

    let last_key = segments.last().unwrap();
    match current.as_object_mut() {
        Some(map) => {
            if map.shift_remove(*last_key).is_some() {
                Ok(MutationDisposition::Deleted)
            } else {
                Ok(MutationDisposition::NoChange)
            }
        }
        None => Ok(MutationDisposition::NoChange),
    }
}

fn serialize_frontmatter(
    data: &serde_json::Value,
    format: FrontmatterFormat,
) -> Result<String, CommandError> {
    match format {
        FrontmatterFormat::Yaml => {
            let yaml = serde_yaml::to_string(data).map_err(|e| {
                CommandError::new(
                    DiagnosticCode::FrontmatterParseFailed,
                    format!("failed to serialize YAML: {}", e),
                )
            })?;
            // serde_yaml may or may not produce leading "---\n"
            let content = yaml.strip_prefix("---\n").unwrap_or(&yaml);
            Ok(format!("---\n{}---\n", content))
        }
        FrontmatterFormat::Toml => {
            let toml_val = json_to_toml(data)?;
            let toml_str = toml::to_string_pretty(&toml_val).map_err(|e| {
                CommandError::new(
                    DiagnosticCode::FrontmatterParseFailed,
                    format!("failed to serialize TOML: {}", e),
                )
            })?;
            Ok(format!("+++\n{}+++\n", toml_str))
        }
    }
}

fn json_to_toml(value: &serde_json::Value) -> Result<toml::Value, CommandError> {
    match value {
        serde_json::Value::Null => Err(CommandError::new(
            DiagnosticCode::FrontmatterParseFailed,
            "TOML does not support null values",
        )),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Ok(toml::Value::String(n.to_string()))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let items: Result<Vec<_>, _> = arr.iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(items?))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, v) in map {
                table.insert(k.clone(), json_to_toml(v)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

fn emit_set_mutation(
    in_place: bool,
    json: bool,
    file: &std::path::Path,
    target: MutationTargetRef,
    disposition: MutationDisposition,
    changed: bool,
    line_endings: LineEndingStyle,
    span_before: Option<SourceSpan>,
    span_after: Option<SourceSpan>,
    output_doc: &str,
) -> Result<(), CommandError> {
    let build_result = |content: Option<String>| MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string_lossy().to_string(),
        command: MutationCommandKind::SetFrontmatter,
        target: target.clone(),
        disposition,
        changed,
        line_endings,
        invariant: SourcePreservationInvariant {
            preserves_non_target_bytes: true,
            target_span_before: span_before,
            target_span_after: span_after,
        },
        content,
    };

    if in_place {
        if changed {
            std::fs::write(file, output_doc)?;
        }
        if json {
            output::write_json(&build_result(None))?;
        }
    } else if json {
        output::write_json(&build_result(Some(output_doc.to_string())))?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}
