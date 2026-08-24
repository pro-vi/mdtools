use crate::errors::CommandError;
use crate::model::{
    MutationCommandKind, MutationResult, MutationTargetRef, SourcePreservationInvariant,
    SCHEMA_VERSION,
};
use crate::output;
use mdtools::edit::EditOutcome;

pub fn emit<T>(
    in_place: bool,
    json: bool,
    file: &std::path::Path,
    edit_target: Option<&output::EditTarget>,
    command: MutationCommandKind,
    outcome: EditOutcome<T>,
    target: impl FnOnce(&T) -> MutationTargetRef,
) -> Result<(), CommandError> {
    let changed = outcome.changed();
    let target = target(&outcome.target);
    let result = |content| MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: file.to_string_lossy().to_string(),
        command,
        target: target.clone(),
        disposition: outcome.disposition,
        changed,
        guarded: outcome.guarded,
        line_endings: outcome.line_endings,
        invariant: SourcePreservationInvariant {
            preserves_non_target_bytes: outcome.preservation.preserves_non_target_bytes,
            target_span_before: outcome.preservation.target_span_before,
            target_span_after: outcome.preservation.target_span_after,
        },
        content,
    };
    if in_place {
        let edit_target = edit_target.expect("in-place edits require an initial file target");
        if changed {
            output::write_file_atomic_verified(
                edit_target,
                &outcome.content,
                &outcome.base_revision,
            )?;
        } else {
            output::verify_file_unchanged(edit_target, &outcome.base_revision)?;
        }
        if json {
            output::write_json(&result(None))?;
        }
    } else if json {
        output::write_json(&result(Some(outcome.content)))?;
    } else {
        print!("{}", outcome.content);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::replace::target_to_wire;
    use mdtools::block_edit;
    use mdtools::document::Document;

    #[test]
    fn in_place_nochange_rejects_intervening_document_change() {
        let path = std::env::temp_dir().join(format!(
            "mdtools-nochange-race-{}-{}.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, "body").unwrap();
        let (source, edit_target) = output::read_edit_file(&path).unwrap().into_parts();
        let document = Document::parse(source).unwrap();
        let outcome = block_edit::prepare_replace(&document, 0, None)
            .unwrap()
            .apply("body");
        assert!(!outcome.changed());

        std::fs::write(&path, "concurrent change").unwrap();
        let error = emit(
            true,
            false,
            &path,
            Some(&edit_target),
            MutationCommandKind::ReplaceBlock,
            outcome,
            target_to_wire,
        )
        .expect_err("a no-op receipt must still verify the current document");

        assert_eq!(error.code, crate::errors::DiagnosticCode::EtagMismatch);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "concurrent change");
        std::fs::remove_file(path).ok();
    }
}
