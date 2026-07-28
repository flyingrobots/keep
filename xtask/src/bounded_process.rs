//! This module owns bounded external child-process collection.

mod deadline;
mod error;
mod reader;

use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use deadline::ProcessDeadline;
pub(crate) use error::ProcessError;
use reader::ReaderWorker;

use crate::process_output::BoundedBytes;

const OUTPUT_LIMIT: usize = 1_048_576;

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

pub(crate) fn capture(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
) -> Result<ProcessOutput, ProcessError> {
    let deadline = ProcessDeadline::new(program, deadline)?;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|source| ProcessError::Io {
        program,
        action: "spawn",
        source,
    })?;
    let stdout = child.stdout.take().ok_or(ProcessError::MissingStream {
        program,
        stream: "stdout",
    });
    let stderr = child.stderr.take().ok_or(ProcessError::MissingStream {
        program,
        stream: "stderr",
    });
    let (stdout, stderr) = match (stdout, stderr) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        (Err(error), _) | (_, Err(error)) => return Err(cleanup(&mut child, error)),
    };
    let stdout_reader = match ReaderWorker::start(program, "stdout", stdout, OUTPUT_LIMIT) {
        Ok(reader) => reader,
        Err(error) => return Err(cleanup(&mut child, error)),
    };
    let stderr_reader = match ReaderWorker::start(program, "stderr", stderr, OUTPUT_LIMIT) {
        Ok(reader) => reader,
        Err(error) => {
            let error = cleanup(&mut child, error);
            drop(stdout_reader);
            return Err(error);
        }
    };
    let status = wait_for_child(program, &mut child, &deadline)?;
    let stdout = stdout_reader.collect(&deadline)?;
    let stderr = stderr_reader.collect(&deadline)?;
    refuse_exceeded(program, "stdout", &stdout)?;
    refuse_exceeded(program, "stderr", &stderr)?;
    Ok(ProcessOutput {
        code: status.code(),
        succeeded: status.success(),
        stdout: stdout.bytes,
        stderr: stderr.bytes,
    })
}

fn wait_for_child(
    program: &'static str,
    child: &mut Child,
    deadline: &ProcessDeadline,
) -> Result<ExitStatus, ProcessError> {
    let ProcessDeadline::Bounded { duration, expires } = deadline else {
        return match child.wait() {
            Ok(status) => Ok(status),
            Err(source) => Err(cleanup(
                child,
                ProcessError::Io {
                    program,
                    action: "wait",
                    source,
                },
            )),
        };
    };
    loop {
        match child.try_wait() {
            Err(source) => {
                return Err(cleanup(
                    child,
                    ProcessError::Io {
                        program,
                        action: "poll",
                        source,
                    },
                ));
            }
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= *expires => {
                return Err(cleanup(
                    child,
                    ProcessError::Timeout {
                        program,
                        duration: *duration,
                    },
                ));
            }
            Ok(None) => thread::sleep(
                Duration::from_millis(10).min(expires.saturating_duration_since(Instant::now())),
            ),
        }
    }
}

const fn refuse_exceeded(
    program: &'static str,
    stream: &'static str,
    output: &BoundedBytes,
) -> Result<(), ProcessError> {
    if output.exceeded {
        Err(ProcessError::OutputLimit {
            program,
            stream,
            maximum: OUTPUT_LIMIT,
        })
    } else {
        Ok(())
    }
}

fn cleanup(child: &mut std::process::Child, primary: ProcessError) -> ProcessError {
    let kill = child.kill();
    let wait = child.wait();
    if let Err(source) = kill
        && source.kind() != io::ErrorKind::InvalidInput
    {
        return ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "kill",
            source,
        };
    }
    if let Err(source) = wait {
        return ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "wait",
            source,
        };
    }
    primary
}

#[cfg(test)]
#[path = "bounded_process/tests.rs"]
mod tests;
