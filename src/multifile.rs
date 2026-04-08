use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::errors::{CommandError, MdExitCode};

pub struct FileSet {
    pub paths: Vec<PathBuf>,
    forced_multi: bool,
}

impl FileSet {
    pub fn is_multi(&self) -> bool {
        self.paths.len() > 1 || self.forced_multi
    }
}

/// Resolve CLI path arguments into a sorted list of .md files.
/// If a path is a directory, enumerate .md files within it.
/// If recursive is true, walk subdirectories.
pub fn resolve_paths(paths: &[PathBuf], recursive: bool) -> Result<FileSet, CommandError> {
    let mut result = Vec::new();
    let mut had_directory = false;

    for path in paths {
        if path.is_dir() {
            had_directory = true;
            if recursive {
                for entry in WalkDir::new(path)
                    .sort_by_file_name()
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() && has_md_extension(entry.path()) {
                        result.push(entry.into_path());
                    }
                }
            } else {
                let mut dir_files: Vec<PathBuf> = std::fs::read_dir(path)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && has_md_extension(p))
                    .collect();
                dir_files.sort();
                result.extend(dir_files);
            }
        } else {
            result.push(path.clone());
        }
    }

    if result.is_empty() {
        return Err(CommandError::io("no .md files found"));
    }

    Ok(FileSet {
        paths: result,
        forced_multi: had_directory,
    })
}

fn has_md_extension(path: &Path) -> bool {
    path.extension()
        .map_or(false, |ext| ext == "md" || ext == "markdown")
}

/// Report a per-file error to stderr with filename prefix.
pub fn report_file_error(file: &Path, err: &CommandError) {
    eprintln!("{}: {}", file.display(), err);
}

/// Run a per-file callback over a resolved FileSet, handling the
/// single-file shortcut and error aggregation.
///
/// Callers capture `file_set.is_multi()` in their closure if they
/// need it for output formatting.
pub fn for_each_file<F>(file_set: &FileSet, mut f: F) -> Result<(), CommandError>
where
    F: FnMut(&Path) -> Result<(), CommandError>,
{
    if !file_set.is_multi() {
        return f(&file_set.paths[0]);
    }

    let mut error_count = 0u32;
    let mut worst_code = MdExitCode::Success;
    for path in &file_set.paths {
        if let Err(e) = f(path) {
            report_file_error(path, &e);
            if (e.exit_code as u8) > (worst_code as u8) {
                worst_code = e.exit_code;
            }
            error_count += 1;
        }
    }
    if error_count == 0 {
        Ok(())
    } else {
        Err(CommandError {
            exit_code: worst_code,
            message: format!("{} file(s) failed", error_count),
        })
    }
}
