//! This module owns deadline-bounded child-output collection.

use std::io;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::{InterruptGuard, ProcessDeadline, ProcessError};

const INTERRUPT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

/// Owns one bounded stream reader and its single-result channel.
pub(super) struct ReaderWorker {
    completed: bool,
    handle: JoinHandle<()>,
    program: &'static str,
    receiver: Receiver<Result<BoundedBytes, io::Error>>,
    stream: &'static str,
}

impl ReaderWorker {
    /// Starts a named reader that retains at most `maximum` stream bytes.
    ///
    /// Thread creation failure remains a typed process I/O error. The worker
    /// sends exactly one bounded result and performs no unbounded buffering.
    pub(super) fn start(
        program: &'static str,
        stream: &'static str,
        reader: impl io::Read + Send + 'static,
        maximum: usize,
    ) -> Result<Self, ProcessError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name(format!("xtask-{stream}-reader"))
            .spawn(move || {
                drop(sender.send(bounded_bytes(reader, maximum)));
            })
            .map_err(|source| ProcessError::Io {
                program,
                action: "start output reader",
                source,
            })?;
        Ok(Self {
            completed: false,
            handle,
            program,
            receiver,
            stream,
        })
    }

    /// Waits for the bounded result while polling the shared deadline and signal guard.
    ///
    /// This call blocks only for the smaller of the remaining deadline and the
    /// fixed interrupt interval, so timeout and interruption remain observable.
    pub(super) fn receive(
        &mut self,
        deadline: &ProcessDeadline,
        interrupts: &InterruptGuard,
    ) -> Result<BoundedBytes, ProcessError> {
        loop {
            if let Some(error) = interrupts.refusal(self.program) {
                return Err(error);
            }
            let (wait, duration) = receive_wait(deadline, self.program)?;
            match self.receiver.recv_timeout(wait) {
                Ok(result) => {
                    self.completed = true;
                    return result.map_err(|source| ProcessError::Io {
                        program: self.program,
                        action: "read child output",
                        source,
                    });
                }
                Err(RecvTimeoutError::Timeout) if duration.is_none() => {}
                Err(RecvTimeoutError::Timeout) => {
                    deadline.remaining(self.program)?;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    self.completed = true;
                    return Err(reader_panic(self.program, self.stream));
                }
            }
        }
    }

    /// Retires one reader without an unbounded thread join.
    ///
    /// A result already received proves the worker crossed its only fallible
    /// read boundary. Otherwise retirement waits only for `grace`; a stalled
    /// reader becomes a typed timeout and its thread handle is detached.
    pub(super) fn retire(self, grace: Duration) -> Result<(), ProcessError> {
        let result = if self.completed {
            Ok(())
        } else {
            match self.receiver.recv_timeout(grace) {
                Ok(Ok(_output)) => Ok(()),
                Ok(Err(source)) => Err(ProcessError::Io {
                    program: self.program,
                    action: "read child output",
                    source,
                }),
                Err(RecvTimeoutError::Disconnected) => Err(reader_panic(self.program, self.stream)),
                Err(RecvTimeoutError::Timeout) => Err(ProcessError::Io {
                    program: self.program,
                    action: "retire output reader",
                    source: io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("{} reader did not retire", self.stream),
                    ),
                }),
            }
        };
        drop(self.handle);
        result
    }
}

fn receive_wait(
    deadline: &ProcessDeadline,
    program: &'static str,
) -> Result<(std::time::Duration, Option<std::time::Duration>), ProcessError> {
    match deadline.remaining(program)? {
        Some((remaining, duration)) => Ok((remaining.min(INTERRUPT_POLL_INTERVAL), Some(duration))),
        None => Ok((INTERRUPT_POLL_INTERVAL, None)),
    }
}

const fn reader_panic(program: &'static str, stream: &'static str) -> ProcessError {
    ProcessError::ReaderPanic { program, stream }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    struct BlockingReader(mpsc::Receiver<()>);

    impl io::Read for BlockingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, io::Error> {
            self.0
                .recv()
                .map_err(|source| io::Error::new(io::ErrorKind::BrokenPipe, source))?;
            Ok(0)
        }
    }

    #[test]
    fn stalled_reader_retirement_obeys_the_cleanup_deadline()
    -> Result<(), Box<dyn std::error::Error>> {
        let (release, blocked) = mpsc::channel();
        let worker = ReaderWorker::start("reader-test", "stdout", BlockingReader(blocked), 1)?;
        let error = worker
            .retire(Duration::ZERO)
            .err()
            .ok_or_else(|| io::Error::other("a stalled reader retired successfully"))?;

        assert!(matches!(
            error,
            ProcessError::Io {
                program: "reader-test",
                action: "retire output reader",
                ref source,
            } if source.kind() == io::ErrorKind::TimedOut
        ));
        release.send(())?;
        Ok(())
    }
}
