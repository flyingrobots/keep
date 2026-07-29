//! This module owns failed child-process and reader teardown.

use std::io;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use super::process_group::ProcessGroup;
use super::{ProcessError, ReaderWorker};

const CLEANUP_DEADLINE: Duration = Duration::from_secs(2);
const CLEANUP_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub(super) fn cleanup_process(child: &mut Child, primary: ProcessError) -> ProcessError {
    drop(child.stdin.take());
    let process_group = ProcessGroup::for_child(child).and_then(ProcessGroup::terminate);
    let kill = match child.kill() {
        Err(source) if source.kind() == io::ErrorKind::InvalidInput => Ok(()),
        result => result,
    };
    let reap = reap_child(child);
    let primary = with_cleanup(primary, "terminate child process group", process_group);
    let primary = with_cleanup(primary, "kill child process", kill);
    with_cleanup(primary, "reap child process", reap)
}

pub(super) fn retire_readers(
    stdout: ReaderWorker,
    stderr: ReaderWorker,
    primary: ProcessError,
) -> ProcessError {
    let primary = retire_after_cleanup(stdout, primary);
    retire_after_cleanup(stderr, primary)
}

pub(super) fn retire_after_cleanup(reader: ReaderWorker, primary: ProcessError) -> ProcessError {
    match reader.retire(CLEANUP_DEADLINE) {
        Ok(()) => primary,
        Err(additional) => ProcessError::Additional {
            primary: Box::new(primary),
            additional: Box::new(additional),
        },
    }
}

fn reap_child(child: &mut Child) -> Result<(), io::Error> {
    wait_until_reaped(CLEANUP_DEADLINE, || {
        child.try_wait().map(|status| status.is_some())
    })
}

fn wait_until_reaped(
    deadline: Duration,
    mut poll: impl FnMut() -> Result<bool, io::Error>,
) -> Result<(), io::Error> {
    let expires = Instant::now()
        .checked_add(deadline)
        .ok_or_else(|| cleanup_timeout(deadline))?;
    loop {
        if poll()? {
            return Ok(());
        }
        let remaining = expires
            .checked_duration_since(Instant::now())
            .ok_or_else(|| cleanup_timeout(deadline))?;
        if remaining.is_zero() {
            return Err(cleanup_timeout(deadline));
        }
        thread::sleep(CLEANUP_POLL_INTERVAL.min(remaining));
    }
}

fn with_cleanup(
    primary: ProcessError,
    action: &'static str,
    result: Result<(), io::Error>,
) -> ProcessError {
    match result {
        Ok(()) => primary,
        Err(source) => ProcessError::Cleanup {
            primary: Box::new(primary),
            action,
            source,
        },
    }
}

fn cleanup_timeout(deadline: Duration) -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        format!("child was not reaped within {deadline:?}"),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn stalled_child_reap_obeys_the_cleanup_deadline() -> Result<(), String> {
        let mut polls = 0_u8;
        let error = wait_until_reaped(Duration::ZERO, || {
            polls = polls.saturating_add(1);
            Ok(false)
        })
        .err()
        .ok_or_else(|| String::from("an unreaped child outlived the cleanup deadline"))?;

        assert_eq!(polls, 1);
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        Ok(())
    }
}
