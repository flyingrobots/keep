//! This module owns subprocess fixtures for process-group regression tests.

use std::env;
use std::io::{self, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;

use super::{
    DESCENDANT_CHILD, DESCENDANT_CHILD_READY, DESCENDANT_PARENT, DESCENDANT_READY,
    DESCENDANT_SOCKET, INTERRUPT_SUPERVISOR, readiness_listener, wait_for_ready,
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
            DESCENDANT_CHILD_READY,
            env::var_os(DESCENDANT_CHILD_READY).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "missing descendant child-ready path",
                )
            })?,
        )
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
    let child_ready = env::var_os(DESCENDANT_CHILD_READY).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing descendant child-ready path",
        )
    })?;
    let listener = readiness_listener(std::path::Path::new(&child_ready))?;
    let mut descendant = Command::new(executable)
        .args([
            "--exact",
            "bounded_process::process_group::child_tests::process_descendant_holds_pipe",
        ])
        .env(DESCENDANT_CHILD, "1")
        .env(DESCENDANT_READY, &child_ready)
        .env(
            DESCENDANT_SOCKET,
            env::var_os(DESCENDANT_SOCKET).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "missing descendant socket path",
                )
            })?,
        )
        .spawn()?;
    wait_for_ready(&listener, &mut descendant)?;
    let mut readiness = UnixStream::connect(ready)?;
    readiness.write_all(b"r")?;
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
    let mut readiness = UnixStream::connect(ready)?;
    readiness.write_all(b"r")?;
    loop {
        let (mut stream, _) = listener.accept()?;
        let mut command = [0_u8; 1];
        io::Read::read_exact(&mut stream, &mut command)?;
        if command == [b'x'] {
            return Ok(());
        }
    }
}
