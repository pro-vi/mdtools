use std::path::Path;

use crate::cli::{TasksArgs, SetTaskArgs};
use crate::errors::CommandError;
use crate::model::*;
use crate::multifile;
use crate::output;
use crate::parser::ParsedDocument;

// --- Loc helpers ---

fn build_loc(block_index: u32, child_path: &[u32]) -> String {
    let path_str: Vec<String> = child_path.iter().map(|i| i.to_string()).collect();
    format!("{}.{}", block_index, path_str.join("."))
}

struct ParsedLoc {
    block_index: u32,
    child_path: Vec<u32>,
}

fn parse_loc(loc: &str) -> Result<ParsedLoc, CommandError> {
    let parts: Vec<&str> = loc.split('.').collect();
    if parts.len() < 2 {
        return Err(CommandError::invalid_task_loc(loc));
    }
    let mut indices = Vec::with_capacity(parts.len());
    for part in &parts {
        indices.push(
            part.parse::<u32>()
                .map_err(|_| CommandError::invalid_task_loc(loc))?,
        );
    }
    Ok(ParsedLoc {
        block_index: indices[0],
        child_path: indices[1..].to_vec(),
    })
}

// --- Nearest heading lookup ---

fn find_nearest_heading(blocks: &[crate::parser::BlockInfo], before_index: u32) -> (Option<String>, Option<u32>) {
    for block in blocks[..before_index as usize].iter().rev() {
        if let Some(ref h) = block.heading {
            return (Some(h.text.clone()), Some(block.index));
        }
    }
    (None, None)
}

// --- Read command ---

pub fn run_tasks(args: &TasksArgs, json: bool) -> Result<(), CommandError> {
    let file_set = multifile::resolve_paths(&args.files, args.recursive)?;
    let multi = file_set.is_multi();
    let mut file_results = Vec::new();

    // Collect all results (for uniform JSON output)
    let collect_fn = |file: &Path| -> Result<TaskFileResult, CommandError> {
        let source = std::fs::read_to_string(file)?;
        let doc = ParsedDocument::parse(source)?;
        let file_str = file.to_string_lossy().to_string();

        let mut tasks = Vec::new();
        for block in &doc.blocks {
            if block.task_items.is_empty() {
                continue;
            }
            let (heading_text, heading_idx) = find_nearest_heading(&doc.blocks, block.index);

            for item in &block.task_items {
                if let Some(ref filter) = args.status {
                    if item.status != *filter {
                        continue;
                    }
                }
                tasks.push(TaskEntry {
                    loc: build_loc(block.index, &item.child_path),
                    block_index: block.index,
                    child_path: item.child_path.clone(),
                    task_index: item.task_index,
                    status: item.status,
                    depth: item.depth,
                    nearest_heading: heading_text.clone(),
                    nearest_heading_block_index: heading_idx,
                    span: item.span,
                    summary_text: item.summary_text.clone(),
                });
            }
        }

        Ok(TaskFileResult { file: file_str, tasks })
    };

    if json {
        // Collect all, preserving partial results on per-file errors
        let mut error_count = 0u32;
        for path in &file_set.paths {
            match collect_fn(path) {
                Ok(fr) => file_results.push(fr),
                Err(e) => {
                    if multi {
                        multifile::report_file_error(path, &e);
                        error_count += 1;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        let result = TasksResult {
            schema_version: SCHEMA_VERSION.to_string(),
            results: file_results,
        };
        output::write_json(&result)?;
        if error_count > 0 {
            Err(CommandError {
                exit_code: crate::errors::MdExitCode::ParseError,
                message: format!("{} file(s) failed", error_count),
            })
        } else {
            Ok(())
        }
    } else {
        // Text output
        multifile::for_each_file(&file_set, |file| {
            let fr = collect_fn(file)?;
            for task in &fr.tasks {
                let heading = task.nearest_heading.as_deref().unwrap_or("");
                let text = output::escape_text_field(&task.summary_text);
                if multi {
                    println!(
                        "{}:\t{}\t{}\t{}\t{}-{}\t{}\t{}",
                        fr.file, task.loc, task.status, task.depth,
                        task.span.line_start, task.span.line_end,
                        heading, text,
                    );
                } else {
                    println!(
                        "{}\t{}\t{}\t{}-{}\t{}\t{}",
                        task.loc, task.status, task.depth,
                        task.span.line_start, task.span.line_end,
                        heading, text,
                    );
                }
            }
            Ok(())
        })
    }
}

// --- Mutation command ---

pub fn run_set_task(args: &SetTaskArgs, json: bool) -> Result<(), CommandError> {
    let parsed = parse_loc(&args.loc)?;
    let source = std::fs::read_to_string(&args.file)?;
    let doc = ParsedDocument::parse(source)?;

    // Resolve block
    let block = doc.blocks.get(parsed.block_index as usize).ok_or_else(|| {
        CommandError::block_out_of_range(parsed.block_index, doc.blocks.len() as u32)
    })?;

    if block.task_items.is_empty() {
        return Err(CommandError::not_a_task_list(parsed.block_index));
    }

    // Find the task item matching the child_path
    let task_item = block
        .task_items
        .iter()
        .find(|ti| ti.child_path == parsed.child_path)
        .ok_or_else(|| CommandError::task_item_not_found(&args.loc))?;

    let line_endings = doc.line_ending_style();
    let task_span = task_item.span;

    // Check idempotent
    let disposition = if task_item.status == args.status {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let changed = disposition != MutationDisposition::NoChange;

    // Validate brackets around symbol
    let sym_offset = task_item.symbol_byte_offset as usize;
    let src = doc.source.as_bytes();
    if sym_offset == 0
        || sym_offset + 1 >= src.len()
        || src[sym_offset - 1] != b'['
        || src[sym_offset + 1] != b']'
    {
        return Err(CommandError::task_item_not_found(&args.loc));
    }

    // Build output
    let output_doc = if changed {
        let replacement_byte = match args.status {
            TaskStatus::Done => b'x',
            TaskStatus::Pending => b' ',
        };
        let mut out = doc.source.as_bytes().to_vec();
        out[sym_offset] = replacement_byte;
        String::from_utf8(out).map_err(|_| CommandError::io("UTF-8 error after mutation"))?
    } else {
        doc.source.clone()
    };

    let target = MutationTargetRef::TaskItem(TaskItemTargetRef {
        kind: MutationTargetKind::TaskItem,
        loc: args.loc.clone(),
        block_index: parsed.block_index,
        child_path: parsed.child_path.clone(),
        span: task_span,
    });

    let span_after = if changed { Some(task_span) } else { Some(task_span) };

    let build_result = |content: Option<String>| MutationResult {
        schema_version: SCHEMA_VERSION.to_string(),
        file: args.file.to_string_lossy().to_string(),
        command: MutationCommandKind::SetTask,
        target: target.clone(),
        disposition,
        changed,
        line_endings,
        invariant: SourcePreservationInvariant {
            preserves_non_target_bytes: true,
            target_span_before: Some(task_span),
            target_span_after: span_after,
        },
        content,
    };

    if args.in_place {
        if changed {
            std::fs::write(&args.file, &output_doc)?;
        }
        if json {
            output::write_json(&build_result(None))?;
        }
    } else if json {
        output::write_json(&build_result(Some(output_doc)))?;
    } else {
        print!("{}", output_doc);
    }
    Ok(())
}
