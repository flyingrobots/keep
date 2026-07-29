//! This module owns deadline-bounded Git path process execution.

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::bounded_process::{self, CaptureLimits, ProcessError};

use super::path_stream::{GIT_PATH_STREAM_LIMIT_BYTES, GitPath, read_paths};
use super::{GitInventoryError, GitOutputUnit};

const GIT_DEADLINE: Duration = Duration::from_mins(2);
const GIT_DIAGNOSTIC_LIMIT_BYTES: usize = 65_536;
const GIT_CAPTURE_LIMITS: CaptureLimits =
    CaptureLimits::new(GIT_PATH_STREAM_LIMIT_BYTES, GIT_DIAGNOSTIC_LIMIT_BYTES);

/// Lists paths through an injected capability-bound spawn operation.
///
/// The adapter materializes at most the 16 MiB path-stream bound before
/// deterministic NUL-framed decoding.
pub(crate) fn paths_with(
    arguments: &[&str],
    operation: &'static str,
    spawn: impl FnOnce(&mut Command) -> Result<Child, io::Error>,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    paths_with_deadline(arguments, operation, GIT_DEADLINE, spawn)
}

fn paths_with_deadline(
    arguments: &[&str],
    operation: &'static str,
    deadline: Duration,
    spawn: impl FnOnce(&mut Command) -> Result<Child, io::Error>,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    let path = env::var_os("PATH").ok_or_else(|| GitInventoryError::Run {
        operation,
        action: "read PATH for",
        source: io::Error::new(io::ErrorKind::NotFound, "PATH is unavailable"),
    })?;
    let mut command = git_command(arguments, &path);
    let output = bounded_process::capture_with_limits(
        "git",
        &mut command,
        Some(deadline),
        GIT_CAPTURE_LIMITS,
        spawn,
    )
    .map_err(|source| process_failure(operation, source))?;
    let paths = read_paths(output.stdout.as_slice(), operation)?;
    if output.succeeded {
        Ok(paths)
    } else {
        Err(git_failure(operation, output.code, output.stderr))
    }
}

fn git_command(arguments: &[&str], path: &OsStr) -> Command {
    let mut command = Command::new("git");
    command
        .args(arguments)
        .stdin(Stdio::null())
        .env_clear()
        .env("PATH", path)
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0");
    command
}

fn process_failure(operation: &'static str, source: ProcessError) -> GitInventoryError {
    match source {
        ProcessError::OutputLimit {
            stream: "stdout",
            maximum,
            ..
        } => output_bound(operation, "path stream bytes", maximum),
        ProcessError::OutputLimit {
            stream: "stderr",
            maximum,
            ..
        } => output_bound(operation, "diagnostic bytes", maximum),
        source => GitInventoryError::Process { operation, source },
    }
}

const fn output_bound(
    operation: &'static str,
    stream: &'static str,
    maximum: usize,
) -> GitInventoryError {
    GitInventoryError::OutputBound {
        operation,
        stream,
        maximum,
        unit: GitOutputUnit::Bytes,
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
