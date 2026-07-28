//! This module owns deadline-bounded child-output collection.

use std::io;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};

use crate::process_output::{BoundedBytes, bounded_bytes};

use super::{ProcessDeadline, ProcessError};

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

    pub(super) fn collect(self, deadline: &ProcessDeadline) -> Result<BoundedBytes, ProcessError> {
        let Self {
            handle,
            program,
            receiver,
            stream,
        } = self;
        let result = match deadline.remaining(program)? {
            Some((remaining, duration)) => receiver
                .recv_timeout(remaining)
                .map_err(|error| receive_error(program, stream, duration, error))?,
            None => receiver.recv().map_err(|_| reader_panic(program, stream))?,
        };
        handle
            .join()
            .map_err(|_panic| reader_panic(program, stream))?;
        result.map_err(|source| ProcessError::Io {
            program,
            action: "read child output",
            source,
        })
    }
}

const fn receive_error(
    program: &'static str,
    stream: &'static str,
    duration: std::time::Duration,
    error: RecvTimeoutError,
) -> ProcessError {
    match error {
        RecvTimeoutError::Timeout => ProcessError::Timeout { program, duration },
        RecvTimeoutError::Disconnected => reader_panic(program, stream),
    }
}

const fn reader_panic(program: &'static str, stream: &'static str) -> ProcessError {
    ProcessError::ReaderPanic { program, stream }
}
