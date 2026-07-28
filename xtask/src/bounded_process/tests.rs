//! This module owns bounded child-process regression evidence.

use std::env;
use std::fs;
use std::io;
use std::process::{Command, Stdio};
use std::str;
use std::time::Duration;

use super::{ProcessError, capture, status};
use crate::test_directory::TestDirectory;

const PARKED_CHILD: &str = "KEEP_XTASK_PARKED_CHILD";

#[test]
fn external_output_is_drained_but_refused_above_the_bound() -> Result<(), Box<dyn std::error::Error>>
{
    let repository = TestDirectory::create("bounded-process-output")?;
    let blob = repository.path().join("oversized.bin");
    fs::write(&blob, vec![b'x'; 1_048_577])?;
    let initialized = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(repository.path())
        .status()?;
    if !initialized.success() {
        return Err(io::Error::other("cannot initialize fixture repository").into());
    }
    let hashed = Command::new("git")
        .args(["hash-object", "-w", "oversized.bin"])
        .current_dir(repository.path())
        .output()?;
    if !hashed.status.success() {
        return Err(io::Error::other("cannot hash fixture blob").into());
    }
    let object_id = str::from_utf8(&hashed.stdout)?.trim();
    let mut command = Command::new("git");
    command
        .args(["cat-file", "blob", object_id])
        .current_dir(repository.path())
        .stdin(Stdio::null());

    let result = capture("test process", &mut command, Some(Duration::from_secs(5)));
    repository.close()?;

    assert!(matches!(
        result,
        Err(ProcessError::OutputLimit {
            program: "test process",
            stream: "stdout",
            maximum: 1_048_576,
        })
    ));
    Ok(())
}

#[test]
fn inherited_process_obeys_the_process_deadline() -> Result<(), Box<dyn std::error::Error>> {
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "bounded_process::tests::process_child_parks_indefinitely",
        ])
        .env(PARKED_CHILD, "1")
        .stdin(Stdio::null());

    let result = status(
        "test process",
        &mut command,
        Some(Duration::from_millis(50)),
    );

    assert!(matches!(
        result,
        Err(ProcessError::Timeout {
            program: "test process",
            duration,
        }) if duration == Duration::from_millis(50)
    ));
    Ok(())
}

#[test]
fn process_child_parks_indefinitely() {
    if env::var_os(PARKED_CHILD).is_none() {
        return;
    }
    loop {
        std::thread::park();
    }
}
