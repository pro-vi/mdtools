use crate::cli::SetArgs;
use crate::commands::edit;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use crate::output;
use mdtools::document::Document;
use mdtools::frontmatter::{self, FrontmatterAction, FrontmatterEdit, FrontmatterPath};

pub fn run(args: &SetArgs, json: bool) -> Result<(), CommandError> {
    validate_args(args)?;
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse_for_frontmatter_mutation(source)?;
    let action = if args.delete {
        FrontmatterAction::Delete
    } else {
        FrontmatterAction::Set(parse_value(
            args.value.as_deref().expect("validated set value"),
            args.string,
        ))
    };
    let request = FrontmatterEdit {
        key_path: FrontmatterPath::new(args.key.clone())?,
        action,
        expect_etag: args
            .expect_etag
            .as_deref()
            .map(mdtools::fingerprint::cli_compat::target_etag),
    };
    let outcome = frontmatter::edit(&document, &request)?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::SetFrontmatter,
        outcome,
        |target| {
            MutationTargetRef::FrontmatterField(FrontmatterFieldTargetRef {
                kind: MutationTargetKind::FrontmatterField,
                key_path: target.key_path.clone(),
                format: target.format,
            })
        },
    )
}

fn validate_args(args: &SetArgs) -> Result<(), CommandError> {
    if args.key.is_empty() || args.key.split('.').any(str::is_empty) {
        return Err(CommandError::invalid_key_path(
            &args.key,
            "key cannot be empty",
        ));
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
    Ok(())
}

fn parse_value(raw: &str, force_string: bool) -> serde_json::Value {
    if force_string {
        serde_json::Value::String(raw.to_string())
    } else {
        serde_yaml::from_str(raw).unwrap_or_else(|_| serde_json::Value::String(raw.to_string()))
    }
}
