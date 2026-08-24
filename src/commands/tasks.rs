use std::path::Path;
use std::str::FromStr;

use crate::cli::{SetTaskArgs, TaskArgs, TasksArgs};
use crate::commands::edit;
use crate::commands::section;
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use mdtools::document::Document;
use mdtools::task::{self, SetTaskEdit, TaskLoc, TaskQuery, TaskRecord};

pub fn run_tasks(args: &TasksArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    let mut file_results = Vec::new();

    let collect_fn = |file: &Path| -> Result<TaskFileResult, CommandError> {
        let source = std::fs::read_to_string(file)?;
        let document = Document::parse(source)?;
        let under = args
            .under
            .as_deref()
            .map(|selector| section::build_selector(selector, args.occurrence, false, false))
            .transpose()?;
        let tasks = task::tasks(
            &document,
            &TaskQuery {
                status: args.status,
                contains: args.contains.clone(),
                under,
            },
        )?
        .into_iter()
        .map(task_to_wire)
        .collect();
        Ok(TaskFileResult {
            file: file.to_string_lossy().to_string(),
            tasks,
        })
    };

    if json {
        let mut error_count = 0u32;
        let mut worst_code = crate::errors::MdExitCode::Success;
        let mut failures = Vec::new();
        for path in &file_set.paths {
            match collect_fn(path) {
                Ok(result) => file_results.push(result),
                Err(error) if multi => {
                    multifile::report_file_error(path, &error, false);
                    if (error.exit_code as u8) > (worst_code as u8) {
                        worst_code = error.exit_code;
                    }
                    failures.push(FileFailure {
                        file: path.display().to_string(),
                        error: crate::errors::ErrorInfo::from(&error),
                    });
                    error_count += 1;
                }
                Err(error) => return Err(error),
            }
        }
        output::write_json(&TasksResult {
            schema_version: SCHEMA_VERSION.to_string(),
            results: file_results,
            failures,
        })?;
        if error_count > 0 {
            let mut error = CommandError::multi_file(
                worst_code,
                error_count,
                format!("{} file(s) failed", error_count),
            );
            error.payload_delivered = true;
            Err(error)
        } else {
            Ok(())
        }
    } else {
        multifile::for_each_file(&file_set, false, |file| {
            let result = collect_fn(file)?;
            for task in &result.tasks {
                let heading =
                    output::escape_text_field(task.nearest_heading.as_deref().unwrap_or(""));
                let text = output::escape_text_field(&task.summary_text);
                if multi {
                    println!(
                        "{}:\t{}\t{}\t{}\t{}-{}\t{}\t{}",
                        result.file,
                        task.loc,
                        task.status,
                        task.depth,
                        task.span.line_start,
                        task.span.line_end,
                        heading,
                        text,
                    );
                } else {
                    println!(
                        "{}\t{}\t{}\t{}-{}\t{}\t{}",
                        task.loc,
                        task.status,
                        task.depth,
                        task.span.line_start,
                        task.span.line_end,
                        heading,
                        text,
                    );
                }
            }
            Ok(())
        })
    }
}

pub fn run_task(args: &TaskArgs, json: bool) -> Result<(), CommandError> {
    let loc = TaskLoc::from_str(&args.loc)?;
    let source = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(source)?;
    let result = task::task(&document, &loc)?;

    if json {
        output::write_json(&TaskReadResult {
            schema_version: SCHEMA_VERSION.to_string(),
            file: args.file.to_string_lossy().to_string(),
            task: task_to_wire(result.task),
            content: result.content,
        })?;
    } else {
        print!("{}", result.content);
    }
    Ok(())
}

fn task_to_wire(record: TaskRecord) -> TaskEntry {
    TaskEntry {
        loc: record.loc.to_string(),
        block_index: record.loc.block_index(),
        child_path: record.loc.child_path().to_vec(),
        task_index: record.task_index,
        status: record.status,
        depth: record.depth,
        nearest_heading: record.nearest_heading,
        nearest_heading_block_index: record.nearest_heading_block_index,
        span: record.span,
        etag: record.etag.into_string(),
        summary_text: record.summary_text,
    }
}

pub fn run_set_task(args: &SetTaskArgs, json: bool) -> Result<(), CommandError> {
    let loc = TaskLoc::from_str(&args.loc)?;
    let (source, edit_target) = output::read_edit_file(&args.file)?.into_parts();
    let document = Document::parse(source)?;
    let outcome = task::set_task(
        &document,
        &SetTaskEdit {
            loc,
            status: args.status,
            expect_etag: args
                .etag_guard
                .expect_etag
                .as_deref()
                .map(mdtools::fingerprint::TargetEtagGuard::new),
        },
    )?;

    edit::emit(
        args.in_place,
        json,
        &args.file,
        Some(&edit_target),
        MutationCommandKind::SetTask,
        outcome,
        |target| {
            MutationTargetRef::TaskItem(TaskItemTargetRef {
                kind: MutationTargetKind::TaskItem,
                loc: target.loc.to_string(),
                block_index: target.loc.block_index(),
                child_path: target.loc.child_path().to_vec(),
                span: target.span,
            })
        },
    )
}
