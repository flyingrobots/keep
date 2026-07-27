//! This module owns the bounded `b3sum` adapter for identity digest evidence.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::GoldenError;
use super::digest_port::IdentityDigestOracle;

const ALGORITHM: u8 = 1;
const B3SUM: &str = "b3sum";
const DATA_MAGIC: [u8; 16] = *b"KEEP:BLOB:DATA\0\0";
const DIAGNOSTIC_LIMIT_BYTES: usize = 65_536;
const DIGEST_BYTES: usize = 32;
const OUTPUT_LIMIT_BYTES: usize = DIGEST_BYTES + 1;
const VERSION: u16 = 1;

pub(super) struct B3sumOracle;

struct B3sumProcess {
    child: Child,
    stdin: ChildStdin,
    stdout_worker: JoinHandle<Result<BoundedBytes, io::Error>>,
    diagnostic_worker: JoinHandle<Result<BoundedBytes, io::Error>>,
}

impl IdentityDigestOracle for B3sumOracle {
    fn identity_digest(&self, payload: &[u8]) -> Result<[u8; 32], GoldenError> {
        let length = u64::try_from(payload.len()).map_err(|source| {
            GoldenError::violation(format!("payload length cannot be represented: {source}"))
        })?;
        let process = start_b3sum()?;
        collect_b3sum(process, payload, length)
    }
}

fn start_b3sum() -> Result<B3sumProcess, GoldenError> {
    let mut child = Command::new(B3SUM)
        .args(["--raw", "--no-mmap", "--num-threads", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| GoldenError::io("start b3sum", Path::new(B3SUM), source))?;
    let stdin = take_stdin(&mut child)?;
    let stdout = take_stdout(&mut child)?;
    let stderr = take_stderr(&mut child)?;
    let stdout_worker = match start_reader("xtask-b3sum-output", stdout, OUTPUT_LIMIT_BYTES) {
        Ok(worker) => worker,
        Err(error) => {
            cleanup_child(&mut child)?;
            return Err(error);
        }
    };
    let diagnostic_worker =
        match start_reader("xtask-b3sum-diagnostic", stderr, DIAGNOSTIC_LIMIT_BYTES) {
            Ok(worker) => worker,
            Err(error) => {
                let cleanup = cleanup_child(&mut child);
                let join = join_reader(stdout_worker, "output");
                cleanup?;
                join?;
                return Err(error);
            }
        };
    Ok(B3sumProcess {
        child,
        stdin,
        stdout_worker,
        diagnostic_worker,
    })
}

fn take_stdin(child: &mut Child) -> Result<ChildStdin, GoldenError> {
    let stdin = child.stdin.take();
    stdin.map_or_else(|| missing_pipe(child, "stdin"), Ok)
}

fn take_stdout(child: &mut Child) -> Result<impl io::Read + Send + 'static, GoldenError> {
    let stdout = child.stdout.take();
    stdout.map_or_else(|| missing_pipe(child, "stdout"), Ok)
}

fn take_stderr(child: &mut Child) -> Result<impl io::Read + Send + 'static, GoldenError> {
    let stderr = child.stderr.take();
    stderr.map_or_else(|| missing_pipe(child, "stderr"), Ok)
}

fn missing_pipe<T>(child: &mut Child, stream: &'static str) -> Result<T, GoldenError> {
    cleanup_child(child)?;
    Err(GoldenError::violation(format!(
        "b3sum {stream} pipe is absent"
    )))
}

fn start_reader(
    name: &str,
    reader: impl io::Read + Send + 'static,
    maximum: usize,
) -> Result<JoinHandle<Result<BoundedBytes, io::Error>>, GoldenError> {
    thread::Builder::new()
        .name(String::from(name))
        .spawn(move || bounded_bytes(reader, maximum))
        .map_err(|source| GoldenError::io("start b3sum output reader", Path::new(B3SUM), source))
}

fn collect_b3sum(
    process: B3sumProcess,
    payload: &[u8],
    length: u64,
) -> Result<[u8; 32], GoldenError> {
    let B3sumProcess {
        mut child,
        stdin,
        stdout_worker,
        diagnostic_worker,
    } = process;
    if let Err(write_error) = write_preimage(stdin, payload, length) {
        cleanup_child(&mut child)?;
        join_reader(stdout_worker, "output")?;
        join_reader(diagnostic_worker, "diagnostic")?;
        return Err(write_error);
    }
    let status = child
        .wait()
        .map_err(|source| GoldenError::io("wait for b3sum", Path::new(B3SUM), source))?;
    let output = join_reader(stdout_worker, "output")?;
    let diagnostic = join_reader(diagnostic_worker, "diagnostic")?;
    validate_b3sum(status.code(), status.success(), output, diagnostic)
}

fn write_preimage(mut stdin: ChildStdin, payload: &[u8], length: u64) -> Result<(), GoldenError> {
    for bytes in [
        DATA_MAGIC.as_slice(),
        VERSION.to_be_bytes().as_slice(),
        [ALGORITHM].as_slice(),
        payload,
        length.to_be_bytes().as_slice(),
    ] {
        stdin
            .write_all(bytes)
            .map_err(|source| GoldenError::io("write b3sum preimage", Path::new(B3SUM), source))?;
    }
    Ok(())
}

fn join_reader(
    worker: JoinHandle<Result<BoundedBytes, io::Error>>,
    stream: &'static str,
) -> Result<BoundedBytes, GoldenError> {
    worker
        .join()
        .map_err(|_| GoldenError::violation(format!("b3sum {stream} reader panicked")))?
        .map_err(|source| GoldenError::io("read b3sum output", Path::new(B3SUM), source))
}

fn validate_b3sum(
    code: Option<i32>,
    success: bool,
    output: BoundedBytes,
    diagnostic: BoundedBytes,
) -> Result<[u8; 32], GoldenError> {
    refuse_exceeded("output", &output, OUTPUT_LIMIT_BYTES)?;
    refuse_exceeded("diagnostic", &diagnostic, DIAGNOSTIC_LIMIT_BYTES)?;
    if !success {
        return Err(process_failure(code, diagnostic.bytes));
    }
    if !diagnostic.bytes.is_empty() {
        return Err(GoldenError::violation(
            "b3sum wrote diagnostics despite successful exit",
        ));
    }
    output
        .bytes
        .try_into()
        .map_err(|_| GoldenError::violation("b3sum returned a noncanonical digest width"))
}

const fn refuse_exceeded(
    stream: &'static str,
    output: &BoundedBytes,
    maximum: usize,
) -> Result<(), GoldenError> {
    if output.exceeded {
        Err(GoldenError::ProcessOutputBound {
            program: B3SUM,
            stream,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn process_failure(code: Option<i32>, diagnostic: Vec<u8>) -> GoldenError {
    match String::from_utf8(diagnostic) {
        Ok(stderr) => GoldenError::ProcessFailed {
            program: B3SUM,
            code,
            stderr,
        },
        Err(source) => GoldenError::ProcessDiagnosticEncoding {
            program: B3SUM,
            code,
            source,
        },
    }
}

fn cleanup_child(child: &mut Child) -> Result<(), GoldenError> {
    let stop = child.kill().or_else(|source| {
        if source.kind() == io::ErrorKind::InvalidInput {
            Ok(())
        } else {
            Err(source)
        }
    });
    let wait = child.wait();
    stop.map_err(|source| GoldenError::io("stop b3sum", Path::new(B3SUM), source))?;
    wait.map_err(|source| GoldenError::io("wait for b3sum", Path::new(B3SUM), source))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{B3sumOracle, IdentityDigestOracle};
    use crate::golden_file_worldline::identity_oracle::digest;

    #[test]
    fn external_oracle_agrees_on_the_identity_preimage() {
        let payload = b"independent digest boundary";
        let external = B3sumOracle.identity_digest(payload);
        let internal = digest(payload);
        assert!(matches!(
            (external, internal),
            (Ok(external), Ok(internal)) if external == *internal.as_bytes()
        ));
    }
}
