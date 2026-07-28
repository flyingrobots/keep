//! This module owns bounded child-process capture and collection.

use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::cleanup::{cleanup_process, join_after_cleanup, join_readers};
use super::{ProcessDeadline, ProcessError, ProcessOutput, ReaderWorker};
use crate::process_output::BoundedBytes;

const OUTPUT_LIMIT: usize = 1_048_576;

pub(crate) fn capture(
    program: &'static str,
    command: &mut Command,
    deadline: Option<Duration>,
) -> Result<ProcessOutput, ProcessError> {
    let deadline = ProcessDeadline::new(program, deadline)?;
    CapturedProcess::start(program, command)?.finish(program, &deadline)
}

struct CapturedProcess {
    child: Child,
    stderr: ReaderWorker,
    stdout: ReaderWorker,
}

impl CapturedProcess {
    fn start(program: &'static str, command: &mut Command) -> Result<Self, ProcessError> {
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
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
            (Err(error), _) | (_, Err(error)) => {
                return Err(cleanup_process(&mut child, error));
            }
        };
        let stdout = match ReaderWorker::start(program, "stdout", stdout, OUTPUT_LIMIT) {
            Ok(reader) => reader,
            Err(error) => return Err(cleanup_process(&mut child, error)),
        };
        let stderr = match ReaderWorker::start(program, "stderr", stderr, OUTPUT_LIMIT) {
            Ok(reader) => reader,
            Err(error) => {
                let error = cleanup_process(&mut child, error);
                return Err(join_after_cleanup(stdout, error));
            }
        };
        Ok(Self {
            child,
            stderr,
            stdout,
        })
    }

    fn finish(
        mut self,
        program: &'static str,
        deadline: &ProcessDeadline,
    ) -> Result<ProcessOutput, ProcessError> {
        let status = match wait_for_child(program, &mut self.child, deadline) {
            Ok(status) => status,
            Err(error) => return Err(join_readers(self.stdout, self.stderr, error)),
        };
        let stdout = match self.stdout.receive(deadline) {
            Ok(output) => output,
            Err(error) => return Err(self.cleanup_readers(error)),
        };
        let stderr = match self.stderr.receive(deadline) {
            Ok(output) => output,
            Err(error) => return Err(self.cleanup_readers(error)),
        };
        if let Err(error) = self.stdout.join() {
            let error = cleanup_process(&mut self.child, error);
            return Err(join_after_cleanup(self.stderr, error));
        }
        if let Err(error) = self.stderr.join() {
            return Err(cleanup_process(&mut self.child, error));
        }
        refuse_exceeded(program, "stdout", &stdout)
            .and_then(|()| refuse_exceeded(program, "stderr", &stderr))
            .map_err(|error| cleanup_process(&mut self.child, error))?;
        Ok(ProcessOutput {
            code: status.code(),
            succeeded: status.success(),
            stdout: stdout.bytes,
            stderr: stderr.bytes,
        })
    }

    fn cleanup_readers(self, error: ProcessError) -> ProcessError {
        let mut child = self.child;
        let error = cleanup_process(&mut child, error);
        join_readers(self.stdout, self.stderr, error)
    }
}

fn wait_for_child(
    program: &'static str,
    child: &mut Child,
    deadline: &ProcessDeadline,
) -> Result<ExitStatus, ProcessError> {
    let ProcessDeadline::Bounded { duration, expires } = deadline else {
        return child.wait().map_err(|source| {
            cleanup_process(
                child,
                ProcessError::Io {
                    program,
                    action: "wait",
                    source,
                },
            )
        });
    };
    loop {
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
            Ok(None) if Instant::now() >= *expires => {
                return Err(cleanup_process(
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
