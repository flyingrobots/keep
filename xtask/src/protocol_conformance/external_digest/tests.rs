//! This module owns external-digest process deadline regression evidence.

use std::env;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use super::{
    B3SUM, B3sumProcess, ConformanceError, collect_with_timeout, start_reader, take_stderr,
    take_stdin, take_stdout,
};

const BLOCKING_CHILD: &str = "KEEP_XTASK_B3SUM_BLOCKING_CHILD";

#[test]
fn blocked_stdin_write_obeys_process_deadline() -> Result<(), ConformanceError> {
    let executable = env::current_exe().map_err(|source| {
        ConformanceError::io("locate digest test executable", "current", source)
    })?;
    let mut child = Command::new(executable)
        .args([
            "--exact",
            "protocol_conformance::external_digest::tests::digest_child_does_not_read_stdin",
        ])
        .env(BLOCKING_CHILD, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| {
            ConformanceError::io("start digest test executable", "current", source)
        })?;
    let stdin = take_stdin(&mut child)?;
    let stdout = take_stdout(&mut child)?;
    let stderr = take_stderr(&mut child)?;
    let process = B3sumProcess {
        child,
        stdin,
        stdout_worker: start_reader("output", stdout, 32)?,
        stderr_worker: start_reader("diagnostic", stderr, 65_536)?,
    };
    let preimage = vec![0_u8; 1_048_576];
    assert!(matches!(
        collect_with_timeout(process, &[&preimage], Duration::from_millis(50)),
        Err(ConformanceError::ProcessTimeout { program, duration })
            if program == B3SUM && duration == Duration::from_millis(50)
    ));
    Ok(())
}

#[test]
fn digest_child_does_not_read_stdin() {
    if env::var_os(BLOCKING_CHILD).is_none() {
        return;
    }
    let (sender, receiver) = mpsc::channel::<()>();
    assert!(matches!(
        receiver.recv_timeout(Duration::from_secs(1)),
        Err(RecvTimeoutError::Timeout)
    ));
    drop(sender);
}
