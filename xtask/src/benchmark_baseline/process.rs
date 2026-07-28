//! Bounded, deadlock-free child-process collection for the baseline task.

use std::io;
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::BenchmarkBaselineError;

pub(super) struct ProcessOutput {
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

pub(super) fn run(
    command: &mut Command,
    program: &'static str,
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<ProcessOutput, BenchmarkBaselineError> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| process_io(program, "start", source))?;
    let Some(stdout) = child.stdout.take() else {
        cleanup(&mut child, program)?;
        return Err(BenchmarkBaselineError::MissingPipe {
            program,
            stream: "stdout",
        });
    };
    let Some(stderr) = child.stderr.take() else {
        cleanup(&mut child, program)?;
        return Err(BenchmarkBaselineError::MissingPipe {
            program,
            stream: "stderr",
        });
    };
    let stdout_worker = match start_reader(program, "stdout", stdout, stdout_limit) {
        Ok(worker) => worker,
        Err(error) => {
            cleanup(&mut child, program)?;
            return Err(error);
        }
    };
    let stderr_worker = match start_reader(program, "stderr", stderr, stderr_limit) {
        Ok(worker) => worker,
        Err(error) => {
            let cleanup = cleanup(&mut child, program);
            let join = join_reader(program, "stdout", stdout_worker);
            cleanup?;
            join?;
            return Err(error);
        }
    };
    let status = match child.wait() {
        Ok(status) => status,
        Err(source) => {
            let cleanup = cleanup(&mut child, program);
            let stdout_join = join_reader(program, "stdout", stdout_worker);
            let stderr_join = join_reader(program, "stderr", stderr_worker);
            cleanup?;
            stdout_join?;
            stderr_join?;
            return Err(process_io(program, "wait for", source));
        }
    };
    let stdout = join_reader(program, "stdout", stdout_worker);
    let stderr = join_reader(program, "stderr", stderr_worker);
    let stdout = stdout?;
    let stderr = stderr?;
    refuse_exceeded(program, "stdout", stdout_limit, &stdout)?;
    refuse_exceeded(program, "stderr", stderr_limit, &stderr)?;
    if !status.success() {
        let stderr = String::from_utf8(stderr.bytes)
            .map_err(|source| BenchmarkBaselineError::DiagnosticEncoding { program, source })?;
        return Err(BenchmarkBaselineError::ProcessFailed {
            program,
            code: status.code(),
            stderr,
        });
    }
    Ok(ProcessOutput {
        stdout: stdout.bytes,
        stderr: stderr.bytes,
    })
}

fn start_reader(
    program: &'static str,
    stream: &'static str,
    reader: impl io::Read + Send + 'static,
    maximum: usize,
) -> Result<JoinHandle<Result<BoundedBytes, io::Error>>, BenchmarkBaselineError> {
    thread::Builder::new()
        .name(format!("xtask-{program}-{stream}"))
        .spawn(move || bounded_bytes(reader, maximum))
        .map_err(|source| process_io(program, "start output reader for", source))
}

fn join_reader(
    program: &'static str,
    stream: &'static str,
    worker: JoinHandle<Result<BoundedBytes, io::Error>>,
) -> Result<BoundedBytes, BenchmarkBaselineError> {
    worker
        .join()
        .map_err(|_| BenchmarkBaselineError::ReaderThread { program, stream })?
        .map_err(|source| process_io(program, "read output from", source))
}

const fn refuse_exceeded(
    program: &'static str,
    stream: &'static str,
    maximum: usize,
    output: &BoundedBytes,
) -> Result<(), BenchmarkBaselineError> {
    if output.exceeded {
        Err(BenchmarkBaselineError::OutputBound {
            program,
            stream,
            maximum,
        })
    } else {
        Ok(())
    }
}

const fn process_io(
    program: &'static str,
    action: &'static str,
    source: io::Error,
) -> BenchmarkBaselineError {
    BenchmarkBaselineError::ProcessIo {
        program,
        action,
        source,
    }
}

fn cleanup(
    child: &mut std::process::Child,
    program: &'static str,
) -> Result<(), BenchmarkBaselineError> {
    child
        .kill()
        .or_else(|source| {
            if source.kind() == io::ErrorKind::InvalidInput {
                Ok(())
            } else {
                Err(source)
            }
        })
        .map_err(|source| process_io(program, "stop", source))?;
    child
        .wait()
        .map_err(|source| process_io(program, "wait for stopped", source))
        .map(|_status| ())
}
