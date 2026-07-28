//! This module owns deadline-bounded child-output collection.

use std::io;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::{InterruptGuard, ProcessDeadline, ProcessError};

const INTERRUPT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

pub(super) struct ReaderWorker {
    handle: JoinHandle<()>,
    program: &'static str,
    receiver: Receiver<Result<BoundedBytes, io::Error>>,
    stream: &'static str,
}

impl ReaderWorker {
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
