//! This module owns bounded cargo-fuzz child-process collection.

mod error;

use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(super) use error::ProcessError;

use crate::process_output::{BoundedBytes, bounded_bytes};

const OUTPUT_LIMIT: usize = 1_048_576;

pub(super) struct ProcessOutput {
    pub(super) succeeded: bool,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

pub(super) fn status(command: &mut Command) -> Result<ProcessOutput, ProcessError> {
    let status = command.status().map_err(|source| ProcessError::Io {
        action: "wait for",
        source,
    })?;
    Ok(ProcessOutput {
        succeeded: status.success(),
        stdout: Vec::new(),
        stderr: Vec::new(),
    })
}

pub(super) fn capture(
    command: &mut Command,
    deadline: Option<Duration>,
) -> Result<ProcessOutput, ProcessError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|source| ProcessError::Io {
        action: "spawn",
        source,
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or(ProcessError::MissingStream("stdout"));
    let stderr = child
        .stderr
        .take()
        .ok_or(ProcessError::MissingStream("stderr"));
    let (stdout, stderr) = match (stdout, stderr) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        (Err(error), _) | (_, Err(error)) => return cleanup(&mut child, error),
    };
    let stdout_reader = start_reader("stdout", stdout)?;
    let stderr_reader = match start_reader("stderr", stderr) {
        Ok(reader) => reader,
        Err(error) => return cleanup(&mut child, error),
    };
    let status = wait_for_child(&mut child, deadline);
    let stdout = join_reader("stdout", stdout_reader)?;
    let stderr = join_reader("stderr", stderr_reader)?;
    let status = status?;
    refuse_exceeded("stdout", &stdout)?;
    refuse_exceeded("stderr", &stderr)?;
    Ok(ProcessOutput {
        succeeded: status.success(),
        stdout: stdout.bytes,
        stderr: stderr.bytes,
    })
}

fn wait_for_child(
    child: &mut Child,
    deadline: Option<Duration>,
) -> Result<ExitStatus, ProcessError> {
    let Some(duration) = deadline else {
        return child.wait().map_err(|source| ProcessError::Io {
            action: "wait",
            source,
        });
    };
    let expires = Instant::now()
        .checked_add(duration)
        .ok_or(ProcessError::Timeout(duration))?;
    loop {
        match child.try_wait().map_err(|source| ProcessError::Io {
            action: "poll",
            source,
        })? {
            Some(status) => return Ok(status),
            None if Instant::now() >= expires => {
                return cleanup(child, ProcessError::Timeout(duration));
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn start_reader(
    stream: &'static str,
    reader: impl io::Read + Send + 'static,
) -> Result<JoinHandle<Result<BoundedBytes, io::Error>>, ProcessError> {
    thread::Builder::new()
        .name(format!("fuzz-{stream}-reader"))
        .spawn(move || bounded_bytes(reader, OUTPUT_LIMIT))
        .map_err(|source| ProcessError::Io {
            action: "start output reader",
            source,
        })
}

fn join_reader(
    stream: &'static str,
    worker: JoinHandle<Result<BoundedBytes, io::Error>>,
) -> Result<BoundedBytes, ProcessError> {
    worker
        .join()
        .map_err(|_panic| ProcessError::ReaderPanic(stream))?
        .map_err(|source| ProcessError::Io {
            action: "read child output",
            source,
        })
}

const fn refuse_exceeded(stream: &'static str, output: &BoundedBytes) -> Result<(), ProcessError> {
    if output.exceeded {
        Err(ProcessError::OutputLimit {
            stream,
            maximum: OUTPUT_LIMIT,
        })
    } else {
        Ok(())
    }
}

fn cleanup<T>(child: &mut std::process::Child, primary: ProcessError) -> Result<T, ProcessError> {
    let kill = child.kill();
    let wait = child.wait();
    if let Err(source) = kill
        && source.kind() != io::ErrorKind::InvalidInput
    {
        return Err(ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "kill",
            source,
        });
    }
    if let Err(source) = wait {
        return Err(ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "wait",
            source,
        });
    }
    Err(primary)
}
