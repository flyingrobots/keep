//! This module owns bounded Git path streaming and diagnostic collection.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::git_path_stream::{GitPathRecord, read_paths};
use super::{GitOutputUnit, SourceStructureError};

const GIT_DIAGNOSTIC_LIMIT_BYTES: usize = 65_536;

struct GitProcess {
    child: Child,
    diagnostic_worker: JoinHandle<Result<BoundedBytes, io::Error>>,
    stdout: ChildStdout,
}

/// Lists repository paths without allowing either child pipe to block.
///
/// Standard error is drained concurrently before standard output is read.
/// Collection reads standard output before requesting termination, waits for
/// the child before joining the diagnostic reader, and then preserves the
/// established error precedence.
pub(super) fn git_paths(
    repository_root: &Path,
    arguments: &[&str],
    operation: &'static str,
) -> Result<BTreeSet<GitPathRecord>, SourceStructureError> {
    let process = start_git(repository_root, arguments, operation)?;
    let paths = read_paths(process.stdout, operation);
    collect_git_result(process.child, process.diagnostic_worker, paths, operation)
}

fn start_git(
    repository_root: &Path,
    arguments: &[&str],
    operation: &'static str,
) -> Result<GitProcess, SourceStructureError> {
    let mut child = Command::new("git")
        .args(arguments)
        .current_dir(repository_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| SourceStructureError::RunGit {
            operation,
            action: "start",
            source,
        })?;
    let Some(stdout) = child.stdout.take() else {
        cleanup_child(&mut child, operation)?;
        return Err(SourceStructureError::GitPipe {
            operation,
            stream: "stdout",
        });
    };
    let Some(stderr) = child.stderr.take() else {
        cleanup_child(&mut child, operation)?;
        return Err(SourceStructureError::GitPipe {
            operation,
            stream: "stderr",
        });
    };
    let diagnostic_worker = thread::Builder::new()
        .name(String::from("xtask-git-diagnostic"))
        .spawn(move || bounded_bytes(stderr, GIT_DIAGNOSTIC_LIMIT_BYTES));
    let diagnostic_worker = match diagnostic_worker {
        Ok(worker) => worker,
        Err(source) => {
            cleanup_child(&mut child, operation)?;
            return Err(SourceStructureError::RunGit {
                operation,
                action: "start the diagnostic reader for",
                source,
            });
        }
    };
    Ok(GitProcess {
        child,
        diagnostic_worker,
        stdout,
    })
}

fn collect_git_result(
    mut child: Child,
    diagnostic_worker: JoinHandle<Result<BoundedBytes, io::Error>>,
    paths: Result<BTreeSet<GitPathRecord>, SourceStructureError>,
    operation: &'static str,
) -> Result<BTreeSet<GitPathRecord>, SourceStructureError> {
    let stop = if paths.is_err() {
        request_stop(&mut child, operation)
    } else {
        Ok(())
    };
    let status = child.wait().map_err(|source| SourceStructureError::RunGit {
        operation,
        action: "wait for",
        source,
    });
    let diagnostic = diagnostic_worker
        .join()
        .map_err(|_| SourceStructureError::GitWorker { operation })?;

    stop?;
    let paths = paths?;
    let status = status?;
    let diagnostic = diagnostic.map_err(|source| SourceStructureError::RunGit {
        operation,
        action: "read diagnostics from",
        source,
    })?;
    if diagnostic.exceeded {
        return Err(SourceStructureError::GitOutputBound {
            operation,
            stream: "diagnostic bytes",
            maximum: GIT_DIAGNOSTIC_LIMIT_BYTES,
            unit: GitOutputUnit::Bytes,
        });
    }
    if !status.success() {
        return Err(git_failure(operation, status.code(), diagnostic.bytes));
    }
    Ok(paths)
}

fn request_stop(child: &mut Child, operation: &'static str) -> Result<(), SourceStructureError> {
    child
        .kill()
        .or_else(|source| {
            if source.kind() == io::ErrorKind::InvalidInput {
                Ok(())
            } else {
                Err(source)
            }
        })
        .map_err(|source| SourceStructureError::RunGit {
            operation,
            action: "stop",
            source,
        })
}

fn cleanup_child(child: &mut Child, operation: &'static str) -> Result<(), SourceStructureError> {
    let stop = request_stop(child, operation);
    let wait = child.wait().map_err(|source| SourceStructureError::RunGit {
        operation,
        action: "wait for",
        source,
    });
    stop?;
    wait.map(|_| ())
}

fn git_failure(
    operation: &'static str,
    code: Option<i32>,
    diagnostic: Vec<u8>,
) -> SourceStructureError {
    match String::from_utf8(diagnostic) {
        Ok(stderr) => SourceStructureError::GitFailed {
            operation,
            code,
            stderr,
        },
        Err(source) => SourceStructureError::GitDiagnosticEncoding {
            operation,
            code,
            source,
        },
    }
}

#[cfg(test)]
mod tests;
