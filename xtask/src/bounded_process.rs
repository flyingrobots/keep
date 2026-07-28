//! This module owns bounded external child-process collection.

mod capture;
mod cleanup;
mod deadline;
mod error;
mod process_group;
mod reader;

use std::process::Command;

pub(crate) use capture::capture;
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
) -> Result<ProcessOutput, ProcessError> {
    let status = command.status().map_err(|source| ProcessError::Io {
        program,
        action: "wait for",
        source,
    })?;
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
