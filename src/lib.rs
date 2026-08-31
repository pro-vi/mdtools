#![doc = include_str!("../README.crate.md")]

pub mod core_error;
pub mod document;
mod edit;
#[cfg(feature = "file")]
pub mod file;
pub mod fingerprint;
pub mod fragment;
mod frontmatter;
pub mod index;
mod model;
pub use model::{
    BlockKind, ColumnAlignment, DocumentStats, FrontmatterFormat, HeadingMatchMode,
    LineEndingStyle, LinkKind, MutationDisposition, SearchMatchMode, SourceSpan, TaskStatus,
    SCHEMA_VERSION,
};
mod parser;
pub mod patch;
pub mod protocol;
pub mod read;
pub mod revision;
mod search;
mod section;
mod section_edit;
mod source;
mod stats;
mod table;
pub mod target;
