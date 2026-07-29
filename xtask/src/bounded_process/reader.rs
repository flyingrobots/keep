//! This module owns deadline-bounded child-output collection.

use std::io;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::{InterruptGuard, ProcessDeadline, ProcessError};

const INTERRUPT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

/// Owns one bounded stream reader and its single-result channel.
pub(super) struct ReaderWorker {
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
        &self,
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
                    return Err(reader_panic(self.program, self.stream));
                }
            }
        }
    }

    /// Joins the completed reader and maps a worker panic to a typed failure.
    pub(super) fn join(self) -> Result<(), ProcessError> {
        self.handle
            .join()
            .map_err(|_panic| reader_panic(self.program, self.stream))
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
