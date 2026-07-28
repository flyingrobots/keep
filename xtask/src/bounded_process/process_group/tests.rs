//! This module owns child process-group cleanup regression evidence.

use std::env;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::{DESCENDANT_PARENT, DESCENDANT_READY, DESCENDANT_SOCKET, wait_for_ready};
use crate::bounded_process::cleanup::cleanup_process;
use crate::bounded_process::{ProcessError, capture};
use crate::test_directory::TestDirectory;

const CHILD_PROCESS: &str =
    "bounded_process::process_group::child_tests::process_child_leaves_descendant_pipe_open";

#[test]
fn inherited_descendant_pipe_obeys_the_process_deadline() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::create("process-descendant-deadline")?;
    let ready = directory.path().join("ready");
    let socket = directory.path().join("descendant.sock");
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args(["--exact", CHILD_PROCESS])
        .env(DESCENDANT_PARENT, "1")
        .env(DESCENDANT_READY, &ready)
        .env(DESCENDANT_SOCKET, &socket)
        .stdin(Stdio::null());

    let result = capture(
        "test process",
        &mut command,
        Some(Duration::from_millis(25)),
    );

    assert!(matches!(
        result,
        Err(ProcessError::Timeout {
            program: "test process",
            duration,
        }) if duration == Duration::from_millis(25)
    ));
    directory.close()?;
    Ok(())
}

#[test]
fn cleanup_terminates_the_entire_child_process_group() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("process-group-cleanup")?;
    let ready = directory.path().join("ready");
    let socket = directory.path().join("descendant.sock");
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args(["--exact", CHILD_PROCESS])
        .env(DESCENDANT_PARENT, "1")
        .env(DESCENDANT_READY, &ready)
        .env(DESCENDANT_SOCKET, &socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut child = command.spawn()?;
    wait_for_ready(&ready)?;

    let error = cleanup_process(
        &mut child,
        ProcessError::Timeout {
            program: "test process",
            duration: Duration::from_millis(25),
        },
    );

    assert!(matches!(error, ProcessError::Timeout { .. }));
    let descendant_survived = descendant_survived_cleanup(&socket)?;
    directory.close()?;
    assert!(
        !descendant_survived,
        "cleanup returned while a descendant remained reachable"
    );
    Ok(())
}

fn descendant_survived_cleanup(socket: &Path) -> Result<bool, io::Error> {
    let expires = Instant::now()
        .checked_add(Duration::from_millis(500))
        .ok_or_else(|| io::Error::other("descendant cleanup deadline overflow"))?;
    loop {
        match UnixStream::connect(socket) {
            Ok(mut stream) if Instant::now() >= expires => {
                stream.write_all(b"x")?;
                return Ok(true);
            }
            Ok(mut stream) => match stream.write_all(b"p") {
                Ok(()) => thread::yield_now(),
                Err(source)
                    if matches!(
                        source.kind(),
                        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
                    ) =>
                {
                    return Ok(false);
                }
                Err(source) => return Err(source),
            },
            Err(source)
                if matches!(
                    source.kind(),
                    io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
                ) =>
            {
                return Ok(false);
            }
            Err(source) => return Err(source),
        }
    }
}
