//! This module owns source inventory orchestration and the 500-line law.

mod repository_path;
mod source_error;

use std::collections::BTreeSet;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::git_inventory::{GitPath, paths as git_paths};
use crate::repository_file::{OpenRepositoryFileError, RepositoryRoot};
use repository_path::RepositoryPath;
pub(super) use source_error::SourceStructureError;

const SOURCE_MODULE_HARD_LIMIT_LINES: u64 = 500;
const SOURCE_SUFFIXES: [[u8; 2]; 3] = [*b"py", *b"rs", *b"sh"];
const PRESENT_PATH_ARGUMENTS: [&str; 5] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
];

pub(super) fn check(repository_root: &Path) -> Result<(), SourceStructureError> {
    let source_root =
        RepositoryRoot::open(repository_root).map_err(|source| SourceStructureError::Inspect {
            path: repository_root.to_owned(),
            source,
        })?;
    let paths = source_paths(repository_root)?;
    verify_source_root(&source_root, repository_root)?;
    let violations = source_violations(&source_root, paths)?;
    if violations.is_empty() {
        Ok(())
    } else {
        Err(SourceStructureError::Violations {
            maximum: SOURCE_MODULE_HARD_LIMIT_LINES,
            paths: violations,
        })
    }
}

fn verify_source_root(
    source_root: &RepositoryRoot,
    repository_root: &Path,
) -> Result<(), SourceStructureError> {
    match source_root.is_current_path() {
        Ok(true) => Ok(()),
        Ok(false) => Err(SourceStructureError::RepositoryRootChanged(
            repository_root.to_owned(),
        )),
        Err(source) => Err(SourceStructureError::Inspect {
            path: repository_root.to_owned(),
            source,
        }),
    }
}

fn source_paths(repository_root: &Path) -> Result<Vec<RepositoryPath>, SourceStructureError> {
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
    select_source_paths(&present, &deleted)
}

fn select_source_paths(
    present: &BTreeSet<GitPath>,
    deleted: &BTreeSet<GitPath>,
) -> Result<Vec<RepositoryPath>, SourceStructureError> {
    present
        .difference(deleted)
        .filter(|path| is_source_module(path.as_bytes()))
        .map(admit_source_path)
        .collect()
}

fn admit_source_path(path: &GitPath) -> Result<RepositoryPath, SourceStructureError> {
    let text = String::from_utf8(path.as_bytes().to_vec()).map_err(|source| {
        SourceStructureError::GitPathEncoding {
            operation: "source path admission",
            source,
        }
    })?;
    RepositoryPath::admit(text)
}

fn source_violations(
    source_root: &RepositoryRoot,
    paths: Vec<RepositoryPath>,
) -> Result<Vec<String>, SourceStructureError> {
    let mut violations = Vec::new();
    for relative in paths {
        let lines = source_line_count(source_root, &relative)?;
        if lines == SourceLineCount::Exceeded {
            violations.push(relative.as_str().to_owned());
        }
    }
    Ok(violations)
}

fn source_line_count(
    source_root: &RepositoryRoot,
    relative: &RepositoryPath,
) -> Result<SourceLineCount, SourceStructureError> {
    source_line_count_with(source_root, relative, RepositoryRoot::open_file)
}

fn source_line_count_with(
    source_root: &RepositoryRoot,
    relative: &RepositoryPath,
    open_source: impl FnOnce(&RepositoryRoot, &Path) -> Result<std::fs::File, OpenRepositoryFileError>,
) -> Result<SourceLineCount, SourceStructureError> {
    let path = source_root.display_path(relative.as_path());
    let file = open_source(source_root, relative.as_path()).map_err(|error| match error {
        OpenRepositoryFileError::Io(source) => SourceStructureError::Inspect {
            path: path.clone(),
            source,
        },
        OpenRepositoryFileError::NonRegular => SourceStructureError::NonRegular(path.clone()),
    })?;
    line_count(BufReader::new(file))
        .map_err(|source| SourceStructureError::Inspect { path, source })
}

fn is_source_module(path: &[u8]) -> bool {
    let Some(file_name) = path.rsplit(|byte| *byte == b'/').next() else {
        return false;
    };
    let mut components = file_name.rsplitn(2, |byte| *byte == b'.');
    let Some(suffix) = components.next() else {
        return false;
    };
    let Some(stem) = components.next() else {
        return false;
    };
    !stem.is_empty() && SOURCE_SUFFIXES.iter().any(|candidate| suffix == candidate)
}

const fn exceeds_hard_limit(lines: u64) -> bool {
    lines > SOURCE_MODULE_HARD_LIMIT_LINES
}

fn line_count(mut reader: impl BufRead) -> Result<SourceLineCount, io::Error> {
    let mut counter = LineCounter::default();
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        for byte in buffer {
            if let Some(lines) = counter.observe(*byte)? {
                return Ok(lines);
            }
        }
        let consumed = buffer.len();
        reader.consume(consumed);
    }
    counter.finish()
}

#[derive(Default)]
struct LineCounter {
    completed: u64,
    in_line: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceLineCount {
    Exceeded,
    Within(u64),
}

impl LineCounter {
    fn observe(&mut self, byte: u8) -> Result<Option<SourceLineCount>, io::Error> {
        if !self.in_line {
            let current = self.next_line()?;
            if exceeds_hard_limit(current) {
                return Ok(Some(SourceLineCount::Exceeded));
            }
            self.in_line = true;
        }
        if byte == b'\n' {
            self.completed = self.next_line()?;
            self.in_line = false;
        }
        Ok(None)
    }

    fn finish(&self) -> Result<SourceLineCount, io::Error> {
        if self.in_line {
            self.next_line().map(SourceLineCount::Within)
        } else {
            Ok(SourceLineCount::Within(self.completed))
        }
    }

    fn next_line(&self) -> Result<u64, io::Error> {
        self.completed
            .checked_add(1)
            .ok_or_else(|| io::Error::other("source line count overflow"))
    }
}

#[cfg(test)]
mod tests;
