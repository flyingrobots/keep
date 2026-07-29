//! This module owns bounded external child-process collection.

mod capture;
mod capture_limit;
mod cleanup;
mod deadline;
mod error;
mod interrupt;
mod process_group;
mod reader;

use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use capture::wait_for_child;
pub(crate) use capture::{capture, capture_with, capture_with_limits};
pub(crate) use capture_limit::CaptureLimits;
use deadline::ProcessDeadline;
pub(crate) use error::ProcessError;
use interrupt::InterruptGuard;
use reader::ReaderWorker;

/// The completed child status and any output retained by the selected mode.
///
/// Captured execution retains at most the selected limit for each output
/// stream. Inherited execution leaves both byte vectors empty because the child
/// writes directly to the parent's configured streams.
pub(crate) struct ProcessOutput {
    /// The platform exit code, or `None` when the process ended by signal.
    pub(crate) code: Option<i32>,
    /// Whether the platform status represents successful termination.
    pub(crate) succeeded: bool,
    /// Captured standard output, or an empty vector in inherited mode.
    pub(crate) stdout: Vec<u8>,
    /// Captured standard error, or an empty vector in inherited mode.
    pub(crate) stderr: Vec<u8>,
}

/// Runs a child synchronously while inheriting its configured output streams.
///
/// The optional deadline bounds the complete wait. The child starts in a
/// dedicated process group so timeout or polling failure terminates descendants
/// before this function returns. Spawn, poll, wait, timeout, and cleanup
/// failures retain their typed [`ProcessError`] boundary.
pub(crate) fn status(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
) -> Result<ProcessOutput, ProcessError> {
    let deadline = ProcessDeadline::new(program, deadline)?;
    let interrupts = InterruptGuard::begin(program)?;
    command.process_group(0);
    let mut child = command.spawn().map_err(|source| ProcessError::Io {
        program,
        action: "spawn",
        source,
    })?;
    let status = wait_for_child(program, &mut child, &deadline, &interrupts)?;
    Ok(ProcessOutput {
        code: status.code(),
        succeeded: status.success(),
        stdout: Vec::new(),
        stderr: Vec::new(),
    })
}

#[cfg(test)]
#[path = "bounded_process/tests.rs"]
mod tests;
