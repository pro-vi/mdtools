use crate::cli::SetArgs;
use crate::commands::edit;
use crate::errors::{CommandError, DiagnosticCode};
use crate::model::*;
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;
use mdtools::frontmatter::{self, FrontmatterAction, FrontmatterEdit};

pub fn run(args: &SetArgs, json: bool) -> Result<(), CommandError> {
    validate_args(args)?;
    let source = std::fs::read_to_string(&args.file)?;
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
        key_path: args.key.clone(),
        action,
        expect_etag: args
            .expect_etag
            .as_deref()
            .map(str::parse::<TargetEtag>)
            .transpose()?,
    };
    let outcome = frontmatter::edit(&document, &request)?;
    edit::emit(
        args.in_place,
        json,
        &args.file,
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
