//! This module owns source inventory orchestration and the 500-line law.

mod git_path_inventory;
mod source_error;
mod source_file;

use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader};
use std::path::{Component, Path};

use git_path_inventory::git_paths;
pub(super) use source_error::SourceStructureError;
use source_file::{OpenSourceError, SourceRoot};

const SOURCE_MODULE_HARD_LIMIT_LINES: u64 = 500;
const SOURCE_SUFFIXES: [&str; 3] = ["py", "rs", "sh"];
const PRESENT_PATH_ARGUMENTS: [&str; 5] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
];

pub(super) fn check(repository_root: &Path) -> Result<(), SourceStructureError> {
    let source_root =
        SourceRoot::open(repository_root).map_err(|source| SourceStructureError::Inspect {
            path: repository_root.to_owned(),
            source,
        })?;
    let paths = source_paths(repository_root)?;
    let violations = source_violations(&source_root, paths)?;
    if violations.is_empty() {
        Ok(())
    } else {
        Err(SourceStructureError::Violations(violations))
    }
}

fn source_paths(repository_root: &Path) -> Result<Vec<String>, SourceStructureError> {
    let present = git_paths(
        repository_root,
        &PRESENT_PATH_ARGUMENTS,
        "git ls-files present",
    )?;
    let deleted = git_paths(
        repository_root,
        &["ls-files", "-z", "--deleted"],
        "git ls-files deleted",
    )?;
    Ok(present
        .difference(&deleted)
        .filter(|path| is_source_module(Path::new(path)))
        .cloned()
        .collect())
}

fn source_violations(
    source_root: &SourceRoot,
    paths: Vec<String>,
) -> Result<Vec<(String, u64)>, SourceStructureError> {
    let mut violations = Vec::new();
    for relative in paths {
        let lines = source_line_count(source_root, &relative)?;
        if exceeds_hard_limit(lines) {
            violations.push((relative, lines));
        }
    }
    Ok(violations)
}

fn source_line_count(
    source_root: &SourceRoot,
    relative: &str,
) -> Result<u64, SourceStructureError> {
    source_line_count_with(source_root, relative, SourceRoot::open_file)
}

fn source_line_count_with(
    source_root: &SourceRoot,
    relative: &str,
    open_source: impl FnOnce(&SourceRoot, &Path) -> Result<std::fs::File, OpenSourceError>,
) -> Result<u64, SourceStructureError> {
    let relative_path = admitted_relative_path(relative)?;
    let path = source_root.display_path(relative_path);
    let file = open_source(source_root, relative_path).map_err(|error| match error {
        OpenSourceError::Io(source) => SourceStructureError::Inspect {
            path: path.clone(),
            source,
        },
        OpenSourceError::NonRegular => SourceStructureError::NonRegular(path.clone()),
    })?;
    line_count(BufReader::new(file))
        .map_err(|source| SourceStructureError::Inspect { path, source })
}

fn admitted_relative_path(path: &str) -> Result<&Path, SourceStructureError> {
    let relative = Path::new(path);
    if relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(relative)
    } else {
        Err(SourceStructureError::InvalidPath(path.to_owned()))
    }
}

fn is_source_module(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|suffix| SOURCE_SUFFIXES.contains(&suffix))
}

const fn exceeds_hard_limit(lines: u64) -> bool {
    lines > SOURCE_MODULE_HARD_LIMIT_LINES
}

fn line_count(mut reader: impl BufRead) -> Result<u64, io::Error> {
    let mut completed_lines = 0_u64;
    let mut in_line = false;
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        for byte in buffer {
            if !in_line {
                let current_line = completed_lines
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("source line count overflow"))?;
                if exceeds_hard_limit(current_line) {
                    return Ok(current_line);
                }
                in_line = true;
            }
            if *byte == b'\n' {
                completed_lines = completed_lines
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("source line count overflow"))?;
                in_line = false;
            }
        }
        let consumed = buffer.len();
        reader.consume(consumed);
    }
    if in_line {
        completed_lines
            .checked_add(1)
            .ok_or_else(|| io::Error::other("source line count overflow"))
    } else {
        Ok(completed_lines)
    }
}

#[cfg(test)]
mod tests;
