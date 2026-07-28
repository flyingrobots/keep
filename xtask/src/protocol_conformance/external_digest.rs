//! This module owns the bounded external `b3sum` conformance witness.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::ConformanceError;

const B3SUM: &str = "b3sum";
const DIAGNOSTIC_LIMIT: usize = 65_536;
const DIGEST_BYTES: usize = 32;
const TIMEOUT: Duration = Duration::from_secs(10);

struct B3sumProcess {
    child: Child,
    stdin: ChildStdin,
    stdout_worker: JoinHandle<Result<BoundedBytes, io::Error>>,
    stderr_worker: JoinHandle<Result<BoundedBytes, io::Error>>,
}

pub(super) fn digest(parts: &[&[u8]]) -> Result<[u8; DIGEST_BYTES], ConformanceError> {
    let process = start()?;
    collect(process, parts)
}

fn start() -> Result<B3sumProcess, ConformanceError> {
    let mut child = Command::new(B3SUM)
        .args(["--raw", "--no-mmap", "--num-threads", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ConformanceError::io("start b3sum", Path::new(B3SUM), source))?;
    let stdin = take_stdin(&mut child)?;
    let stdout = take_stdout(&mut child)?;
    let stderr = take_stderr(&mut child)?;
    let stdout_worker = match start_reader("output", stdout, DIGEST_BYTES) {
        Ok(worker) => worker,
        Err(error) => return Err(cleanup(&mut child, error)),
    };
    let stderr_worker = match start_reader("diagnostic", stderr, DIAGNOSTIC_LIMIT) {
        Ok(worker) => worker,
        Err(error) => {
            let error = cleanup(&mut child, error);
            drop(join_reader(stdout_worker, "output"));
            return Err(error);
        }
    };
    Ok(B3sumProcess {
        child,
        stdin,
        stdout_worker,
        stderr_worker,
    })
}

fn take_stdin(child: &mut Child) -> Result<ChildStdin, ConformanceError> {
    child.stdin.take().ok_or_else(|| {
        cleanup(
            child,
            ConformanceError::violation("b3sum stdin pipe is absent"),
        )
    })
}

fn take_stdout(child: &mut Child) -> Result<impl io::Read + Send + 'static, ConformanceError> {
    child.stdout.take().ok_or_else(|| {
        cleanup(
            child,
            ConformanceError::violation("b3sum stdout pipe is absent"),
        )
    })
}

fn take_stderr(child: &mut Child) -> Result<impl io::Read + Send + 'static, ConformanceError> {
    child.stderr.take().ok_or_else(|| {
        cleanup(
            child,
            ConformanceError::violation("b3sum stderr pipe is absent"),
        )
    })
}

fn start_reader(
    stream: &'static str,
    reader: impl io::Read + Send + 'static,
    maximum: usize,
) -> Result<JoinHandle<Result<BoundedBytes, io::Error>>, ConformanceError> {
    thread::Builder::new()
        .name(format!("conformance-b3sum-{stream}"))
        .spawn(move || bounded_bytes(reader, maximum))
        .map_err(|source| ConformanceError::io("start b3sum reader", Path::new(B3SUM), source))
}

fn collect(process: B3sumProcess, parts: &[&[u8]]) -> Result<[u8; DIGEST_BYTES], ConformanceError> {
    let B3sumProcess {
        mut child,
        mut stdin,
        stdout_worker,
        stderr_worker,
    } = process;
    if let Err(error) = write_parts(&mut stdin, parts) {
        let primary = cleanup(&mut child, error);
        drop(join_reader(stdout_worker, "output"));
        drop(join_reader(stderr_worker, "diagnostic"));
        return Err(primary);
    }
    drop(stdin);
    let status = wait(&mut child);
    let output = join_reader(stdout_worker, "output");
    let diagnostic = join_reader(stderr_worker, "diagnostic");
    let status = status?;
    validate(status, output?, diagnostic?)
}

fn write_parts(stdin: &mut ChildStdin, parts: &[&[u8]]) -> Result<(), ConformanceError> {
    for part in parts {
        stdin
            .write_all(part)
            .map_err(|source| ConformanceError::io("write b3sum preimage", B3SUM, source))?;
    }
    Ok(())
}

fn wait(child: &mut Child) -> Result<ExitStatus, ConformanceError> {
    let expires = Instant::now()
        .checked_add(TIMEOUT)
        .ok_or_else(|| cleanup(child, timeout()))?;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= expires => return Err(cleanup(child, timeout())),
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(source) => {
                return Err(cleanup(
                    child,
                    ConformanceError::io("poll b3sum", B3SUM, source),
                ));
            }
        }
    }
}

const fn timeout() -> ConformanceError {
    ConformanceError::ProcessTimeout {
        program: B3SUM,
        duration: TIMEOUT,
    }
}

fn join_reader(
    worker: JoinHandle<Result<BoundedBytes, io::Error>>,
    stream: &'static str,
) -> Result<BoundedBytes, ConformanceError> {
    worker
        .join()
        .map_err(|_| ConformanceError::ReaderPanic {
            program: B3SUM,
            stream,
        })?
        .map_err(|source| ConformanceError::io("read b3sum output", B3SUM, source))
}

fn validate(
    status: ExitStatus,
    output: BoundedBytes,
    diagnostic: BoundedBytes,
) -> Result<[u8; DIGEST_BYTES], ConformanceError> {
    refuse_exceeded("output", &output, DIGEST_BYTES)?;
    refuse_exceeded("diagnostic", &diagnostic, DIAGNOSTIC_LIMIT)?;
    if !status.success() {
        return Err(process_failure(status.code(), diagnostic.bytes));
    }
    if !diagnostic.bytes.is_empty() {
        return Err(ConformanceError::violation(
            "b3sum wrote diagnostics despite successful exit",
        ));
    }
    output
        .bytes
        .try_into()
        .map_err(|_| ConformanceError::violation("b3sum returned a noncanonical digest width"))
}

const fn refuse_exceeded(
    stream: &'static str,
    output: &BoundedBytes,
    maximum: usize,
) -> Result<(), ConformanceError> {
    if output.exceeded {
        Err(ConformanceError::ProcessOutputBound {
            program: B3SUM,
            stream,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn process_failure(code: Option<i32>, diagnostic: Vec<u8>) -> ConformanceError {
    match String::from_utf8(diagnostic) {
        Ok(stderr) => ConformanceError::ProcessFailed {
            program: B3SUM,
            code,
            stderr,
        },
        Err(source) => ConformanceError::ProcessDiagnosticEncoding {
            program: B3SUM,
            code,
            source,
        },
    }
}

fn cleanup(child: &mut Child, primary: ConformanceError) -> ConformanceError {
    let kill = child.kill();
    let wait = child.wait();
    if let Err(source) = kill
        && source.kind() != io::ErrorKind::InvalidInput
    {
        return ConformanceError::cleanup(primary, "kill b3sum", source);
    }
    if let Err(source) = wait {
        return ConformanceError::cleanup(primary, "wait for b3sum", source);
    }
    primary
}
