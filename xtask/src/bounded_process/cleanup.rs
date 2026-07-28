//! This module owns failed child-process and reader teardown.

use std::io;
use std::process::Child;

use super::process_group::ProcessGroup;
use super::{ProcessError, ReaderWorker};

pub(super) fn cleanup_process(child: &mut Child, primary: ProcessError) -> ProcessError {
    let process_group = ProcessGroup::for_child(child).and_then(ProcessGroup::terminate);
    let kill = child.kill();
    let wait = child.wait();
    if let Err(source) = process_group {
        return ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "terminate child process group",
            source,
        };
    }
    if let Err(source) = kill
        && source.kind() != io::ErrorKind::InvalidInput
    {
        return ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "kill child process",
            source,
        };
    }
    if let Err(source) = wait {
        return ProcessError::Cleanup {
            primary: Box::new(primary),
            action: "reap child process",
            source,
        };
    }
    primary
}

pub(super) fn join_readers(
    stdout: ReaderWorker,
    stderr: ReaderWorker,
    primary: ProcessError,
) -> ProcessError {
    let primary = join_after_cleanup(stdout, primary);
    join_after_cleanup(stderr, primary)
}

pub(super) fn join_after_cleanup(reader: ReaderWorker, primary: ProcessError) -> ProcessError {
    match reader.join() {
        Ok(()) => primary,
        Err(additional) => ProcessError::Additional {
            primary: Box::new(primary),
            additional: Box::new(additional),
        },
    }
}
