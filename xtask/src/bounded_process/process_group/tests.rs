//! This module owns child process-group cleanup regression evidence.

use std::env;
use std::io;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, kill_process};

use super::{
    DESCENDANT_CHILD_READY, DESCENDANT_PARENT, DESCENDANT_READY, DESCENDANT_SOCKET,
    INTERRUPT_SUPERVISOR, readiness_listener, wait_for_ready,
};
use crate::bounded_process::cleanup::cleanup_process;
use crate::bounded_process::{ProcessError, capture_with};
use crate::test_directory::TestDirectory;

const CHILD_PROCESS: &str =
    "bounded_process::process_group::child_tests::process_child_leaves_descendant_pipe_open";
const SUPERVISOR_PROCESS: &str = "bounded_process::process_group::child_tests::process_supervisor_captures_descendant_until_interrupted";

fn spawn_ready_descendant(
    executable: &Path,
    ready: &Path,
    child_ready: &Path,
    socket: &Path,
) -> Result<Child, Box<dyn std::error::Error>> {
    let listener = readiness_listener(ready)?;
    let mut child = Command::new(executable)
        .args(["--exact", CHILD_PROCESS])
        .env(DESCENDANT_PARENT, "1")
        .env(DESCENDANT_CHILD_READY, child_ready)
        .env(DESCENDANT_READY, ready)
        .env(DESCENDANT_SOCKET, socket)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()?;
    if let Err(source) = wait_for_ready(&listener, &mut child) {
        let error = cleanup_process(
            &mut child,
            ProcessError::Io {
                program: "test process",
                action: "wait for descendant readiness in",
                source,
            },
        );
        return Err(Box::new(error));
    }
    Ok(child)
}

#[test]
fn terminal_interrupt_terminates_the_isolated_descendant_group()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("process-group-interrupt")?;
    let ready = directory.path().join("ready");
    let child_ready = directory.path().join("child-ready");
    let socket = directory.path().join("descendant.sock");
    let listener = readiness_listener(&ready)?;
    let executable = env::current_exe()?;
    let mut supervisor = Command::new(executable)
        .args(["--exact", SUPERVISOR_PROCESS])
        .env(INTERRUPT_SUPERVISOR, "1")
        .env(DESCENDANT_CHILD_READY, &child_ready)
        .env(DESCENDANT_READY, &ready)
        .env(DESCENDANT_SOCKET, &socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    wait_for_ready(&listener, &mut supervisor)?;
    let descendant = UnixStream::connect(&socket)?;
    let supervisor_pid =
        Pid::from_raw(i32::try_from(supervisor.id())?).ok_or("supervisor process ID is zero")?;
    kill_process(supervisor_pid, Signal::INT)?;

    let status = wait_for_exit(&mut supervisor)?;
    require_descendant_disconnect(descendant)?;
    directory.close()?;
    assert!(
        status.success(),
        "interrupted supervisor did not exit cleanly: {status:?}"
    );
    Ok(())
}

#[test]
fn inherited_descendant_pipe_obeys_the_process_deadline() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::create("pd")?;
    let ready = directory.path().join("r");
    let child_ready = directory.path().join("cr");
    let socket = directory.path().join("d");
    let executable = env::current_exe()?;
    let child = spawn_ready_descendant(&executable, &ready, &child_ready, &socket)?;

    let mut admitted = Command::new("pre-spawned descendant fixture");
    let result = capture_with(
        "test process",
        &mut admitted,
        Some(Duration::from_millis(25)),
        |_command| Ok(child),
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
    let child_ready = directory.path().join("child-ready");
    let socket = directory.path().join("descendant.sock");
    let listener = readiness_listener(&ready)?;
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args(["--exact", CHILD_PROCESS])
        .env(DESCENDANT_PARENT, "1")
        .env(DESCENDANT_CHILD_READY, &child_ready)
        .env(DESCENDANT_READY, &ready)
        .env(DESCENDANT_SOCKET, &socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut child = command.spawn()?;
    wait_for_ready(&listener, &mut child)?;
    let descendant = UnixStream::connect(&socket)?;

    let error = cleanup_process(
        &mut child,
        ProcessError::Timeout {
            program: "test process",
            duration: Duration::from_millis(25),
        },
    );

    assert!(matches!(error, ProcessError::Timeout { .. }));
    require_descendant_disconnect(descendant)?;
    directory.close()?;
    Ok(())
}

fn require_descendant_disconnect(mut descendant: UnixStream) -> Result<(), io::Error> {
    descendant.set_read_timeout(Some(Duration::from_millis(250)))?;
    let mut byte = [0_u8; 1];
    match io::Read::read_exact(&mut descendant, &mut byte) {
        Err(source)
            if matches!(
                source.kind(),
                io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset
            ) =>
        {
            Ok(())
        }
        Err(source) => Err(source),
        Ok(()) => Err(io::Error::other(
            "terminated descendant unexpectedly wrote to its witness socket",
        )),
    }
}

#[test]
fn descendant_disconnect_witness_has_a_finite_read_deadline() -> Result<(), io::Error> {
    let (descendant, _retained_writer) = UnixStream::pair()?;

    let Err(error) = require_descendant_disconnect(descendant) else {
        return Err(io::Error::other(
            "an open idle witness did not reach its read deadline",
        ));
    };

    assert!(matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    ));
    Ok(())
}

fn wait_for_exit(child: &mut std::process::Child) -> Result<std::process::ExitStatus, io::Error> {
    let expires = Instant::now()
        .checked_add(Duration::from_secs(2))
        .ok_or_else(|| io::Error::other("supervisor exit deadline overflow"))?;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= expires {
            child.kill()?;
            return child.wait();
        }
        thread::yield_now();
    }
}
