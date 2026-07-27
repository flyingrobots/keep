//! This module owns source inventory orchestration and the 500-line law.

mod git_path_inventory;
mod git_path_stream;
mod repository_path;
mod source_error;
mod source_file;

use std::io::{self, BufRead, BufReader};
use std::path::Path;

use git_path_inventory::git_paths;
use repository_path::RepositoryPath;
pub(super) use source_error::{GitOutputUnit, SourceStructureError};
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
        Err(SourceStructureError::Violations {
            maximum: SOURCE_MODULE_HARD_LIMIT_LINES,
            paths: violations,
        })
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
    Ok(present
        .difference(&deleted)
        .filter(|path| is_source_module(path.as_str()))
        .cloned()
        .collect())
}

fn source_violations(
    source_root: &SourceRoot,
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
    source_root: &SourceRoot,
    relative: &RepositoryPath,
) -> Result<SourceLineCount, SourceStructureError> {
    source_line_count_with(source_root, relative, SourceRoot::open_file)
}

fn source_line_count_with(
    source_root: &SourceRoot,
    relative: &RepositoryPath,
    open_source: impl FnOnce(&SourceRoot, &Path) -> Result<std::fs::File, OpenSourceError>,
) -> Result<SourceLineCount, SourceStructureError> {
    let path = source_root.display_path(relative.as_path());
    let file = open_source(source_root, relative.as_path()).map_err(|error| match error {
        OpenSourceError::Io(source) => SourceStructureError::Inspect {
            path: path.clone(),
            source,
        },
        OpenSourceError::NonRegular => SourceStructureError::NonRegular(path.clone()),
    })?;
    line_count(BufReader::new(file))
        .map_err(|source| SourceStructureError::Inspect { path, source })
}

fn is_source_module(path: &str) -> bool {
    let Some(file_name) = path.rsplit('/').next() else {
        return false;
    };
    let Some((stem, suffix)) = file_name.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty() && SOURCE_SUFFIXES.contains(&suffix)
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
