//! This module owns bounded child-process capture and collection.

use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use super::cleanup::{cleanup_process, retire_after_cleanup, retire_readers};
use super::input::write_input;
use super::{
    CaptureLimits, InterruptGuard, ProcessDeadline, ProcessError, ProcessOutput, ReaderWorker,
};
use crate::process_output::BoundedBytes;

const DEFAULT_CAPTURE_LIMITS: CaptureLimits = CaptureLimits::new(1_048_576, 1_048_576);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Runs a child synchronously and captures bounded standard output and error.
///
/// Each stream is drained concurrently and retains at most one mebibyte. The
/// optional deadline covers child execution and reader collection. Failures
/// terminate the child's dedicated process group, retire both readers within a
/// fixed cleanup deadline, and retain primary and cleanup errors in
/// [`ProcessError`].
pub(crate) fn capture(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
) -> Result<ProcessOutput, ProcessError> {
    capture_with(program, command, deadline, Command::spawn)
}

/// Runs one captured child through an injected spawn boundary.
///
/// This variant retains the default one-mebibyte limit for each stream while
/// allowing a capability-bound caller to own the actual spawn operation.
pub(crate) fn capture_with(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
    spawn: impl FnOnce(&mut Command) -> Result<Child, std::io::Error>,
) -> Result<ProcessOutput, ProcessError> {
    capture_with_limits(program, command, deadline, DEFAULT_CAPTURE_LIMITS, spawn)
}

/// Runs one captured child with exact independent stream limits.
///
/// The deadline covers child execution and both reader workers. Every failure
/// terminates the dedicated process group and bounds worker retirement.
pub(crate) fn capture_with_limits(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
    limits: CaptureLimits,
    spawn: impl FnOnce(&mut Command) -> Result<Child, std::io::Error>,
) -> Result<ProcessOutput, ProcessError> {
    let deadline = ProcessDeadline::new(program, deadline)?;
    let interrupts = InterruptGuard::begin(program)?;
    CapturedProcess::start(program, command, spawn, interrupts, limits)?.finish(program, &deadline)
}

/// Runs one captured child with bounded streaming input and exact stream limits.
///
/// The deadline clock starts before synchronous spawn. After spawn returns, its
/// remaining time bounds every nonblocking stdin write, both output readers,
/// and child execution. Failed-operation teardown uses a separate fixed
/// deadline for child reaping and reader retirement. Input slices are streamed
/// directly without constructing a combined preimage allocation.
pub(crate) fn capture_with_input_limits(
    program: &'static str,
    command: &mut Command,
    input: &[&[u8]],
    deadline: Option<Duration>,
    limits: CaptureLimits,
) -> Result<ProcessOutput, ProcessError> {
    let deadline = ProcessDeadline::new(program, deadline)?;
    let interrupts = InterruptGuard::begin(program)?;
    command.stdin(Stdio::piped());
    CapturedProcess::start(program, command, Command::spawn, interrupts, limits)?
        .finish_with_input(program, input, &deadline)
}

struct CapturedProcess {
    child: Child,
    interrupts: InterruptGuard,
    limits: CaptureLimits,
    stderr: ReaderWorker,
    stdout: ReaderWorker,
}

impl CapturedProcess {
    fn start(
        program: &'static str,
        command: &mut Command,
        spawn: impl FnOnce(&mut Command) -> Result<Child, std::io::Error>,
        interrupts: InterruptGuard,
        limits: CaptureLimits,
    ) -> Result<Self, ProcessError> {
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let mut child = spawn(command).map_err(|source| ProcessError::Io {
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
            (Err(error), _) | (_, Err(error)) => {
                return Err(cleanup_process(&mut child, error));
            }
        };
        let stdout = match ReaderWorker::start(program, "stdout", stdout, limits.stdout_bytes()) {
            Ok(reader) => reader,
            Err(error) => return Err(cleanup_process(&mut child, error)),
        };
        let stderr = match ReaderWorker::start(program, "stderr", stderr, limits.stderr_bytes()) {
            Ok(reader) => reader,
            Err(error) => {
                let error = cleanup_process(&mut child, error);
                return Err(retire_after_cleanup(stdout, error));
            }
        };
        Ok(Self {
            child,
            interrupts,
            limits,
            stderr,
            stdout,
        })
    }

    fn finish(
        mut self,
        program: &'static str,
        deadline: &ProcessDeadline,
    ) -> Result<ProcessOutput, ProcessError> {
        let stdout = match self.stdout.receive(deadline, &self.interrupts) {
            Ok(output) => output,
            Err(error) => return Err(self.cleanup_readers(error)),
        };
        let stderr = match self.stderr.receive(deadline, &self.interrupts) {
            Ok(output) => output,
            Err(error) => return Err(self.cleanup_readers(error)),
        };
        drop(self.stdout);
        drop(self.stderr);
        let status = wait_for_child(program, &mut self.child, deadline, &self.interrupts)?;
        refuse_exceeded(program, "stdout", self.limits.stdout_bytes(), &stdout).and_then(|()| {
            refuse_exceeded(program, "stderr", self.limits.stderr_bytes(), &stderr)
        })?;
        if let Some(error) = self.interrupts.refusal(program) {
            return Err(error);
        }
        Ok(ProcessOutput {
            code: status.code(),
            succeeded: status.success(),
            stdout: stdout.bytes,
            stderr: stderr.bytes,
        })
    }

    fn finish_with_input(
        mut self,
        program: &'static str,
        input: &[&[u8]],
        deadline: &ProcessDeadline,
    ) -> Result<ProcessOutput, ProcessError> {
        let Some(mut stdin) = self.child.stdin.take() else {
            let error = ProcessError::MissingStream {
                program,
                stream: "stdin",
            };
            return Err(self.cleanup_readers(error));
        };
        if let Err(error) = write_input(program, &mut stdin, input, deadline, &self.interrupts) {
            return Err(self.cleanup_readers(error));
        }
        drop(stdin);
        self.finish(program, deadline)
    }

    fn cleanup_readers(self, error: ProcessError) -> ProcessError {
        let mut child = self.child;
        let error = cleanup_process(&mut child, error);
        retire_readers(self.stdout, self.stderr, error)
    }
}

pub(super) fn wait_for_child(
    program: &'static str,
    child: &mut Child,
    deadline: &ProcessDeadline,
    interrupts: &InterruptGuard,
) -> Result<ExitStatus, ProcessError> {
    loop {
        if let Some(error) = interrupts.refusal(program) {
            return Err(cleanup_process(child, error));
        }
        match child.try_wait() {
            Err(source) => {
                return Err(cleanup_process(
                    child,
                    ProcessError::Io {
                        program,
                        action: "poll",
                        source,
                    },
                ));
            }
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                let interval = match deadline.remaining(program) {
                    Ok(Some((remaining, _duration))) => PROCESS_POLL_INTERVAL.min(remaining),
                    Ok(None) => PROCESS_POLL_INTERVAL,
                    Err(error) => return Err(cleanup_process(child, error)),
                };
                thread::sleep(interval);
            }
        }
    }
}

const fn refuse_exceeded(
    program: &'static str,
    stream: &'static str,
    maximum: usize,
    output: &BoundedBytes,
) -> Result<(), ProcessError> {
    if output.exceeded {
        Err(ProcessError::OutputLimit {
            program,
            stream,
            maximum,
        })
    } else {
        Ok(())
    }
}
