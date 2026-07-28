//! This module owns subprocess fixtures for process-group regression tests.

use std::env;
use std::fs;
use std::io;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::Command;

use super::{
    DESCENDANT_CHILD, DESCENDANT_PARENT, DESCENDANT_READY, DESCENDANT_SOCKET, INTERRUPT_SUPERVISOR,
    wait_for_ready,
};
use crate::bounded_process::{ProcessError, capture};

#[test]
fn process_supervisor_captures_descendant_until_interrupted() -> Result<(), io::Error> {
    if env::var_os(INTERRUPT_SUPERVISOR).is_none() {
        return Ok(());
    }
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "bounded_process::process_group::child_tests::process_child_leaves_descendant_pipe_open",
        ])
        .env(DESCENDANT_PARENT, "1")
        .env(
            DESCENDANT_READY,
            env::var_os(DESCENDANT_READY).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing descendant ready path")
            })?,
        )
        .env(
            DESCENDANT_SOCKET,
            env::var_os(DESCENDANT_SOCKET).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing descendant socket path")
            })?,
        );
    match capture("interrupt fixture", &mut command, None) {
        Err(ProcessError::Interrupted {
            program: "interrupt fixture",
            signal: "SIGINT",
        }) => Ok(()),
        Err(error) => Err(io::Error::other(format!(
            "unexpected interrupt refusal: {error}"
        ))),
        Ok(_) => Err(io::Error::other(
            "interrupt fixture unexpectedly completed successfully",
        )),
    }
}

#[test]
fn process_child_leaves_descendant_pipe_open() -> Result<(), io::Error> {
    if env::var_os(DESCENDANT_PARENT).is_none() {
        return Ok(());
    }
    let executable = env::current_exe()?;
    let ready = env::var_os(DESCENDANT_READY).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "missing descendant ready path")
    })?;
    drop(
        Command::new(executable)
            .args([
                "--exact",
                "bounded_process::process_group::child_tests::process_descendant_holds_pipe",
            ])
            .env(DESCENDANT_CHILD, "1")
            .env(DESCENDANT_READY, &ready)
            .env(
                DESCENDANT_SOCKET,
                env::var_os(DESCENDANT_SOCKET).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "missing descendant socket path",
                    )
                })?,
            )
            .spawn()?,
    );
    wait_for_ready(Path::new(&ready))?;
    Ok(())
}

#[test]
fn process_descendant_holds_pipe() -> Result<(), io::Error> {
    if env::var_os(DESCENDANT_CHILD).is_none() {
        return Ok(());
    }
    let socket = env::var_os(DESCENDANT_SOCKET).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing descendant socket path",
        )
    })?;
    let ready = env::var_os(DESCENDANT_READY).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "missing descendant ready path")
    })?;
    let listener = UnixListener::bind(socket)?;
    fs::write(ready, b"ready")?;
    loop {
        let (mut stream, _) = listener.accept()?;
        let mut command = [0_u8; 1];
        io::Read::read_exact(&mut stream, &mut command)?;
        if command == [b'x'] {
            return Ok(());
        }
    }
}
