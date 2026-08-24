use std::collections::HashSet;
use std::str::FromStr;

use crate::core_error::{CoreError, EtagTarget};
use crate::document::Document;
use crate::edit::{EditOutcome, EditPreservation};
use crate::fingerprint::{TargetEtag, TargetEtagGuard};
use crate::model::{MutationDisposition, SourceSpan, TaskStatus};
use crate::parser::{BlockInfo, TaskItemInfo};
use crate::section::{SectionIndex, SectionTarget};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskLoc {
    block_index: u32,
    child_path: Vec<u32>,
}

impl TaskLoc {
    pub fn block_index(&self) -> u32 {
        self.block_index
    }

    pub fn child_path(&self) -> &[u32] {
        &self.child_path
    }
}

impl FromStr for TaskLoc {
    type Err = CoreError;

    fn from_str(loc: &str) -> Result<Self, Self::Err> {
        let parts = loc.split('.').collect::<Vec<_>>();
        if parts.len() < 2 {
            return Err(CoreError::InvalidTaskLoc {
                loc: loc.to_string(),
            });
        }
        let indices = parts
            .iter()
            .map(|part| {
                part.parse::<u32>().map_err(|_| CoreError::InvalidTaskLoc {
                    loc: loc.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            block_index: indices[0],
            child_path: indices[1..].to_vec(),
        })
    }
}

impl std::fmt::Display for TaskLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.block_index)?;
        for child in &self.child_path {
            write!(f, ".{child}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct TaskQuery {
    pub status: Option<TaskStatus>,
    pub contains: Option<String>,
    pub under: Option<SectionTarget>,
}

#[derive(Clone, Debug)]
pub struct TaskRead {
    pub task: TaskRecord,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRecord {
    pub loc: TaskLoc,
    pub task_index: u32,
    pub status: TaskStatus,
    pub depth: u32,
    pub nearest_heading: Option<String>,
    pub nearest_heading_block_index: Option<u32>,
    pub span: SourceSpan,
    pub etag: TargetEtag,
    pub summary_text: String,
}

#[derive(Clone, Debug)]
pub struct SetTaskEdit {
    pub loc: TaskLoc,
    pub status: TaskStatus,
    pub expect_etag: Option<TargetEtagGuard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskEditTarget {
    pub loc: TaskLoc,
    pub span: SourceSpan,
}

pub fn tasks(document: &Document, query: &TaskQuery) -> Result<Vec<TaskRecord>, CoreError> {
    let selected = query
        .under
        .as_ref()
        .map(|selector| {
            SectionIndex::new(document)
                .resolve(selector)
                .map(|section| {
                    section
                        .block_indices
                        .iter()
                        .copied()
                        .collect::<HashSet<_>>()
                })
        })
        .transpose()?;
    let mut entries = Vec::new();
    let mut nearest_heading = (None, None);

    for block in document.blocks() {
        if let Some(heading) = &block.heading {
            nearest_heading = (Some(heading.text.clone()), Some(block.index));
        }
        if selected
            .as_ref()
            .is_some_and(|indices| !indices.contains(&block.index))
        {
            continue;
        }
        if block.task_items.is_empty() {
            continue;
        }
        for item in &block.task_items {
            if query.status.is_some_and(|status| item.status != status) {
                continue;
            }
            if query
                .contains
                .as_ref()
                .is_some_and(|text| !item.summary_text.contains(text))
            {
                continue;
            }
            entries.push(task_record(document, block, item, &nearest_heading));
        }
    }
    Ok(entries)
}

pub fn task(document: &Document, loc: &TaskLoc) -> Result<TaskRead, CoreError> {
    let (block, item) = resolve_task(document, loc)?;
    let heading = nearest_heading(document.blocks(), block.index);
    Ok(TaskRead {
        task: task_record(document, block, item, &heading),
        content: document.slice_unchecked(&item.span).to_string(),
    })
}

pub fn set_task(
    document: &Document,
    edit: &SetTaskEdit,
) -> Result<EditOutcome<TaskEditTarget>, CoreError> {
    let (_, item) = resolve_task(document, &edit.loc)?;
    let task_span = item.span;
    let current = document.slice_unchecked(&task_span);
    if let Some(expected) = edit.expect_etag.as_ref() {
        let actual = TargetEtag::for_bytes(current.as_bytes());
        if expected.as_str() != actual.as_str() {
            return Err(CoreError::TargetEtagMismatch {
                target: EtagTarget::Task(edit.loc.to_string()),
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        let duplicates = document
            .blocks()
            .iter()
            .flat_map(|block| &block.task_items)
            .filter(|candidate| {
                TargetEtag::for_bytes(document.slice_unchecked(&candidate.span).as_bytes()).as_str()
                    == expected.as_str()
            })
            .count();
        if duplicates > 1 {
            return Err(CoreError::TargetEtagAmbiguous {
                target_kind: "task",
                expected: expected.to_string(),
                count: duplicates,
            });
        }
    }

    let disposition = if item.status == edit.status {
        MutationDisposition::NoChange
    } else {
        MutationDisposition::Replaced
    };
    let symbol = item.symbol_byte_offset as usize;
    let source = document.source().as_bytes();
    if symbol == 0
        || symbol + 1 >= source.len()
        || source[symbol - 1] != b'['
        || source[symbol + 1] != b']'
    {
        return Err(CoreError::TaskNotFound {
            loc: edit.loc.to_string(),
        });
    }
    let content = if disposition == MutationDisposition::NoChange {
        document.source().to_string()
    } else {
        let mut output = source.to_vec();
        output[symbol] = match edit.status {
            TaskStatus::Done => b'x',
            TaskStatus::Pending => b' ',
        };
        String::from_utf8(output).map_err(|error| CoreError::ParseFailed(error.to_string()))?
    };

    Ok(EditOutcome {
        base_revision: document.revision().clone(),
        target: TaskEditTarget {
            loc: edit.loc.clone(),
            span: task_span,
        },
        disposition,
        guarded: edit.expect_etag.is_some(),
        line_endings: document.line_ending_style(),
        preservation: EditPreservation {
            preserves_non_target_bytes: true,
            target_span_before: Some(task_span),
            target_span_after: Some(task_span),
        },
        content,
    })
}

fn resolve_task<'a>(
    document: &'a Document,
    loc: &TaskLoc,
) -> Result<(&'a BlockInfo, &'a TaskItemInfo), CoreError> {
    let block = document
        .blocks()
        .get(loc.block_index as usize)
        .ok_or_else(|| CoreError::TaskBlockOutOfRange {
            loc: loc.to_string(),
            block_index: loc.block_index,
            block_count: document.blocks().len() as u32,
        })?;
    if block.task_items.is_empty() {
        return Err(CoreError::NotTaskList {
            block_index: loc.block_index,
        });
    }
    let item = block
        .task_items
        .iter()
        .find(|item| item.child_path == loc.child_path)
        .ok_or_else(|| CoreError::TaskNotFound {
            loc: loc.to_string(),
        })?;
    Ok((block, item))
}

fn nearest_heading(blocks: &[BlockInfo], before_index: u32) -> (Option<String>, Option<u32>) {
    blocks[..before_index as usize]
        .iter()
        .rev()
        .find_map(|block| {
            block
                .heading
                .as_ref()
                .map(|heading| (Some(heading.text.clone()), Some(block.index)))
        })
        .unwrap_or((None, None))
}

fn task_record(
    document: &Document,
    block: &BlockInfo,
    item: &TaskItemInfo,
    heading: &(Option<String>, Option<u32>),
) -> TaskRecord {
    TaskRecord {
        loc: TaskLoc {
            block_index: block.index,
            child_path: item.child_path.clone(),
        },
        task_index: item.task_index,
        status: item.status,
        depth: item.depth,
        nearest_heading: heading.0.clone(),
        nearest_heading_block_index: heading.1,
        span: item.span,
        etag: TargetEtag::for_bytes(document.slice_unchecked(&item.span).as_bytes()),
        summary_text: item.summary_text.clone(),
    }
}
