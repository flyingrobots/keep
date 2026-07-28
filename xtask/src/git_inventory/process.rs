//! This module owns bounded Git path process execution and collection.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::path_stream::{GitPath, read_paths};
use super::{GitInventoryError, GitOutputUnit};

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
pub(crate) fn paths(
    repository_root: &Path,
    arguments: &[&str],
    operation: &'static str,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    paths_with(arguments, operation, |command| {
        command.current_dir(repository_root).spawn()
    })
}

pub(crate) fn paths_with(
    arguments: &[&str],
    operation: &'static str,
    spawn: impl FnOnce(&mut Command) -> Result<Child, io::Error>,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    let process = start_git(arguments, operation, spawn)?;
    let paths = read_paths(process.stdout, operation);
    collect_git_result(process.child, process.diagnostic_worker, paths, operation)
}

fn start_git(
    arguments: &[&str],
    operation: &'static str,
    spawn: impl FnOnce(&mut Command) -> Result<Child, io::Error>,
) -> Result<GitProcess, GitInventoryError> {
    let mut command = Command::new("git");
    command
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = spawn(&mut command).map_err(|source| GitInventoryError::Run {
        operation,
        action: "start",
        source,
    })?;
    let Some(stdout) = child.stdout.take() else {
        let primary = GitInventoryError::Pipe {
            operation,
            stream: "stdout",
        };
        return Err(cleanup_child(&mut child, operation, primary));
    };
    let Some(stderr) = child.stderr.take() else {
        let primary = GitInventoryError::Pipe {
            operation,
            stream: "stderr",
        };
        return Err(cleanup_child(&mut child, operation, primary));
    };
    let diagnostic_worker = thread::Builder::new()
        .name(String::from("xtask-git-diagnostic"))
        .spawn(move || bounded_bytes(stderr, GIT_DIAGNOSTIC_LIMIT_BYTES));
    let diagnostic_worker = match diagnostic_worker {
        Ok(worker) => worker,
        Err(source) => {
            let primary = GitInventoryError::Run {
                operation,
                action: "start the diagnostic reader for",
                source,
            };
            return Err(cleanup_child(&mut child, operation, primary));
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
    paths: Result<BTreeSet<GitPath>, GitInventoryError>,
    operation: &'static str,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    let stop = if paths.is_err() {
        request_stop(&mut child, operation)
    } else {
        Ok(())
    };
    let status = child.wait().map_err(|source| GitInventoryError::Run {
        operation,
        action: "wait for",
        source,
    });
    let diagnostic = diagnostic_worker.join();
    let paths = match paths {
        Ok(paths) => paths,
        Err(primary) => {
            return Err(preserve_collection_failure(
                primary, stop, status, diagnostic, operation,
            ));
        }
    };
    let status = match status {
        Ok(status) => status,
        Err(primary) => {
            return Err(preserve_diagnostic_failure(primary, diagnostic, operation));
        }
    };
    let diagnostic = diagnostic
        .map_err(|_| GitInventoryError::Worker { operation })?
        .map_err(|source| GitInventoryError::Run {
            operation,
            action: "read diagnostics from",
            source,
        })?;
    if diagnostic.exceeded {
        return Err(GitInventoryError::OutputBound {
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

fn request_stop(child: &mut Child, operation: &'static str) -> Result<(), GitInventoryError> {
    child
        .kill()
        .or_else(|source| {
            if source.kind() == io::ErrorKind::InvalidInput {
                Ok(())
            } else {
                Err(source)
            }
        })
        .map_err(|source| GitInventoryError::Run {
            operation,
            action: "stop",
            source,
        })
}

fn cleanup_child(
    child: &mut Child,
    operation: &'static str,
    primary: GitInventoryError,
) -> GitInventoryError {
    let stop = request_stop(child, operation);
    let wait = child.wait().map_err(|source| GitInventoryError::Run {
        operation,
        action: "wait for",
        source,
    });
    let primary = preserve_error(primary, stop);
    preserve_error(primary, wait.map(|_| ()))
}

fn preserve_collection_failure(
    primary: GitInventoryError,
    stop: Result<(), GitInventoryError>,
    status: Result<std::process::ExitStatus, GitInventoryError>,
    diagnostic: thread::Result<Result<BoundedBytes, io::Error>>,
    operation: &'static str,
) -> GitInventoryError {
    let primary = preserve_error(primary, stop);
    let primary = preserve_error(primary, status.map(|_| ()));
    preserve_diagnostic_failure(primary, diagnostic, operation)
}

fn preserve_diagnostic_failure(
    primary: GitInventoryError,
    diagnostic: thread::Result<Result<BoundedBytes, io::Error>>,
    operation: &'static str,
) -> GitInventoryError {
    let cleanup = match diagnostic {
        Ok(Ok(_)) => return primary,
        Ok(Err(source)) => GitInventoryError::Run {
            operation,
            action: "read diagnostics from",
            source,
        },
        Err(_) => GitInventoryError::Worker { operation },
    };
    preserve_error(primary, Err(cleanup))
}

fn preserve_error(
    primary: GitInventoryError,
    cleanup: Result<(), GitInventoryError>,
) -> GitInventoryError {
    match cleanup {
        Ok(()) => primary,
        Err(cleanup) => GitInventoryError::Cleanup {
            primary: Box::new(primary),
            cleanup: Box::new(cleanup),
        },
    }
}

fn git_failure(
    operation: &'static str,
    code: Option<i32>,
    diagnostic: Vec<u8>,
) -> GitInventoryError {
    match String::from_utf8(diagnostic) {
        Ok(stderr) => GitInventoryError::Failed {
            operation,
            code,
            stderr,
        },
        Err(source) => GitInventoryError::DiagnosticEncoding {
            operation,
            code,
            source,
        },
    }
}

#[cfg(test)]
mod tests;
