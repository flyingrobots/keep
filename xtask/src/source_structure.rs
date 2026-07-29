//! This module owns source inventory orchestration and the 500-line law.

mod python_source;
mod repository_path;
mod source_error;
mod source_file;
mod source_inventory;
mod source_kind;

use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::repository_file::RepositoryRoot;
use repository_path::RepositoryPath;
pub(super) use source_error::SourceStructureError;
use source_file::{AdmittedSource, FileExecution, SourceFileAdmission};
#[cfg(test)]
use source_inventory::{
    PRESENT_PATH_ARGUMENTS, select as select_source_inventory, select_source_paths,
};
use source_inventory::{SourceInventory, collect as source_paths};
use source_kind::is_extensionless_file;

const SOURCE_MODULE_HARD_LIMIT_LINES: u64 = 500;

pub(super) fn check(repository_root: &Path) -> Result<(), SourceStructureError> {
    let source_root =
        RepositoryRoot::open(repository_root).map_err(|source| SourceStructureError::Inspect {
            path: repository_root.to_owned(),
            source,
        })?;
    let paths = source_paths(repository_root)?;
    verify_source_root(&source_root, repository_root)?;
    let violations = inventory_violations(&source_root, paths)?;
    verify_source_root(&source_root, repository_root)?;
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

fn inventory_violations(
    source_root: &RepositoryRoot,
    inventory: SourceInventory,
) -> Result<Vec<std::path::PathBuf>, SourceStructureError> {
    let mut violations = Vec::new();
    for relative in inventory.executable_candidates {
        let SourceFileAdmission::Regular(source) =
            AdmittedSource::admit(source_root, relative.as_path())?
        else {
            continue;
        };
        if source.execution() == FileExecution::Executable
            && source_line_count(source_root, &source)? == SourceLineCount::Exceeded
        {
            violations.push(relative.as_path().to_owned());
        }
    }
    violations.extend(source_violations(source_root, inventory.modules)?);
    violations.sort();
    Ok(violations)
}

fn source_violations(
    source_root: &RepositoryRoot,
    paths: Vec<RepositoryPath>,
) -> Result<Vec<std::path::PathBuf>, SourceStructureError> {
    let mut violations = Vec::new();
    for relative in paths {
        let SourceFileAdmission::Regular(source) =
            AdmittedSource::admit(source_root, relative.as_path())?
        else {
            return Err(SourceStructureError::NonRegular(
                source_root.display_path(relative.as_path()),
            ));
        };
        if is_extensionless_file(relative.as_str().as_bytes())
            && source.execution() == FileExecution::NonExecutable
        {
            continue;
        }
        let lines = source_line_count(source_root, &source)?;
        if lines == SourceLineCount::Exceeded {
            violations.push(relative.as_path().to_owned());
        }
    }
    Ok(violations)
}

fn source_line_count(
    source_root: &RepositoryRoot,
    source: &AdmittedSource,
) -> Result<SourceLineCount, SourceStructureError> {
    let lines = line_count(BufReader::new(source.file())).map_err(|error| {
        SourceStructureError::Inspect {
            path: source.path().to_owned(),
            source: error,
        }
    })?;
    source.verify_current(source_root)?;
    Ok(lines)
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
#[path = "source_structure/executable_candidate_tests.rs"]
mod executable_candidate_tests;
#[cfg(test)]
#[path = "source_structure/pure_rust_tests.rs"]
mod pure_rust_tests;
#[cfg(test)]
mod tests;
