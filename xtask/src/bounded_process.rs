//! This module owns bounded external child-process collection.

mod capture;
mod cleanup;
mod deadline;
mod error;
mod process_group;
mod reader;

use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

pub(crate) use capture::capture;
use capture::wait_for_child;
use deadline::ProcessDeadline;
pub(crate) use error::ProcessError;
use reader::ReaderWorker;

pub(crate) struct ProcessOutput {
    pub(crate) code: Option<i32>,
    pub(crate) succeeded: bool,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

pub(crate) fn status(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
) -> Result<ProcessOutput, ProcessError> {
    let deadline = ProcessDeadline::new(program, deadline)?;
    command.process_group(0);
    let mut child = command.spawn().map_err(|source| ProcessError::Io {
        program,
        action: "spawn",
        source,
    })?;
    let status = wait_for_child(program, &mut child, &deadline)?;
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
