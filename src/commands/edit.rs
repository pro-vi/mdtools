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
        if changed {
            let current = std::fs::read_to_string(file)?;
            mdtools::revision::verify_source_revision(&current, &outcome.base_revision)?;
            output::write_file_atomic(file, &outcome.content)?;
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
