//! This module owns calibration of the subprocess readiness protocol.
//! Size: medium; real local sockets and a controlled child, no network egress.
//! Oracle: the specified readiness byte must survive an observed sender exit.
//! This calibrates test support; it is not Keep storage or durability evidence.
//! Delete only when stronger process-driver calibration subsumes this protocol.

use std::env;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::{Command, Stdio};

use super::readiness::wait_for_signal;
use super::{readiness_listener, wait_for_ready};
use crate::test_directory::TestDirectory;

const GATE: &str = "KEEP_READINESS_RELEASE_SOCKET";
const READY: &str = "KEEP_READINESS_SIGNAL_SOCKET";
const CHILD: &str =
    "bounded_process::process_group::readiness_tests::readiness_child_waits_for_release";

#[test]
fn queued_readiness_survives_exit_between_observations() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("readiness-queued-exit")?;
    let gate_path = directory.path().join("gate");
    let ready_path = directory.path().join("ready");
    let gate = UnixListener::bind(&gate_path)?;
    let ready = readiness_listener(&ready_path)?;
    let mut child = Command::new(env::current_exe()?)
        .args(["--exact", CHILD])
        .env_clear()
        .env(GATE, &gate_path)
        .env(READY, &ready_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let (mut release, _) = gate.accept()?;

    // Schedule: first signal poll is empty; release the sender at exit polling.
    // Waiting for actual exit guarantees its written signal is already queued.
    let result = wait_for_signal(&ready, || {
        release.write_all(b"g")?;
        child.wait().map(Some)
    });

    assert_eq!(
        result.map_err(|source| source.kind()),
        Ok(()),
        "a queued valid readiness signal must be admitted after its sender exits"
    );
    directory.close()?;
    Ok(())
}

#[test]
fn an_invalid_readiness_byte_refuses() -> Result<(), Box<dyn std::error::Error>> {
    // Exhaust the byte domain in ascending order; the first failure is minimal.
    for byte in u8::MIN..=u8::MAX {
        if byte == b'r' {
            continue;
        }
        let directory = TestDirectory::create(&format!("readiness-invalid-{byte}"))?;
        let path = directory.path().join("ready");
        let listener = readiness_listener(&path)?;
        let mut sender = UnixStream::connect(&path)?;
        sender.write_all(&[byte])?;

        let result = wait_for_signal(&listener, || Ok(None));

        assert!(
            matches!(result, Err(source) if source.kind() == io::ErrorKind::InvalidData),
            "readiness byte {byte} must refuse with InvalidData"
        );
        directory.close()?;
    }
    Ok(())
}

// Subprocess fixture only: the parent calibration owns the assertions.
#[test]
fn readiness_child_waits_for_release() -> Result<(), io::Error> {
    let Some(gate) = env::var_os(GATE) else {
        return Ok(());
    };
    let ready = env::var_os(READY)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing readiness socket"))?;
    let mut release = UnixStream::connect(gate)?;
    let mut signal = [0_u8; 1];
    release.read_exact(&mut signal)?;
    if signal != [b'g'] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid release signal",
        ));
    }
    UnixStream::connect(ready)?.write_all(b"r")
}

#[test]
fn readiness_refuses_child_exit_without_a_signal() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("readiness-exit")?;
    let listener = readiness_listener(&directory.path().join("ready"))?;
    let mut child = Command::new(env::current_exe()?)
        .args(["--exact", CHILD])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let result = wait_for_ready(&listener, &mut child);
    directory.close()?;

    assert!(matches!(
        result,
        Err(source) if source.kind() == io::ErrorKind::UnexpectedEof
    ));
    Ok(())
}
