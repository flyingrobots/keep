//! Exact tracked-worktree comparison against the captured `HEAD` tree.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use super::BenchmarkBaselineError;
use super::process::{ProcessOutput, require_success, run, run_status};

const CREATION_ATTEMPTS: u16 = 1_024;
const DIAGNOSTIC_LIMIT: usize = 65_536;
const INVENTORY_LIMIT: usize = 1_048_576;
static NEXT_INDEX: AtomicU64 = AtomicU64::new(0);

pub(super) fn matches_head(repository_root: &Path) -> Result<bool, BenchmarkBaselineError> {
    let index = TemporaryIndex::create()?;
    let read_tree = git(
        repository_root,
        index.path(),
        &["read-tree", "HEAD"],
        INVENTORY_LIMIT,
    )?;
    require_silent(&read_tree)?;
    refresh(repository_root, index.path())?;
    let diff = git(
        repository_root,
        index.path(),
        &[
            "diff-files",
            "--raw",
            "-z",
            "--no-ext-diff",
            "--ignore-submodules=none",
            "--",
        ],
        INVENTORY_LIMIT,
    )?;
    require_no_diagnostics(&diff)?;
    Ok(diff.stdout.is_empty())
}

fn refresh(repository_root: &Path, index: &Path) -> Result<(), BenchmarkBaselineError> {
    let output = run_status(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .env("GIT_INDEX_FILE", index)
            .args(["update-index", "--really-refresh", "-q"]),
        "git",
        INVENTORY_LIMIT,
        DIAGNOSTIC_LIMIT,
    )?;
    match output.status.code() {
        Some(0 | 1) => Ok(()),
        _other => require_success(output, "git").map(|_output| ()),
    }
}

fn git(
    repository_root: &Path,
    index: &Path,
    arguments: &[&str],
    stdout_limit: usize,
) -> Result<ProcessOutput, BenchmarkBaselineError> {
    run(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .env("GIT_INDEX_FILE", index)
            .args(arguments),
        "git",
        stdout_limit,
        DIAGNOSTIC_LIMIT,
    )
}

fn require_silent(output: &ProcessOutput) -> Result<(), BenchmarkBaselineError> {
    require_no_diagnostics(output)?;
    if output.stdout.is_empty() {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "tracked-source-read-tree-wrote-output",
        })
    }
}

const fn require_no_diagnostics(output: &ProcessOutput) -> Result<(), BenchmarkBaselineError> {
    if output.stderr.is_empty() {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "successful-tracked-source-command-wrote-diagnostics",
        })
    }
}

struct TemporaryIndex {
    directory: PathBuf,
    path: PathBuf,
}

impl TemporaryIndex {
    fn create() -> Result<Self, BenchmarkBaselineError> {
        for _ in 0_u16..CREATION_ATTEMPTS {
            let sequence = NEXT_INDEX
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                    current.checked_add(1)
                })
                .map_err(|_| io_error("allocate temporary source index", Path::new("temporary")))?;
            let directory = std::env::temp_dir().join(format!(
                "keep-benchmark-index-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    let path = directory.join("index");
                    return Ok(Self { directory, path });
                }
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => {
                    return Err(BenchmarkBaselineError::Io {
                        action: "create temporary source index directory",
                        target: directory,
                        source,
                    });
                }
            }
        }
        Err(io_error(
            "create temporary source index directory",
            Path::new("temporary"),
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryIndex {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.directory));
    }
}

fn io_error(action: &'static str, target: &Path) -> BenchmarkBaselineError {
    BenchmarkBaselineError::Io {
        action,
        target: target.to_path_buf(),
        source: io::Error::other(action),
    }
}

#[cfg(test)]
#[path = "tracked_source_tests.rs"]
mod tests;
