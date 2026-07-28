//! This module owns bounded child-process regression evidence.

use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use super::{ProcessError, capture};

const DESCENDANT_CHILD: &str = "KEEP_XTASK_DESCENDANT_CHILD";
const DESCENDANT_PARENT: &str = "KEEP_XTASK_DESCENDANT_PARENT";
const OUTPUT_CHILD: &str = "KEEP_XTASK_BOUNDED_OUTPUT_CHILD";

#[test]
fn external_output_is_drained_but_refused_above_the_bound() -> Result<(), Box<dyn std::error::Error>>
{
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "bounded_process::tests::process_child_writes_excess_output",
        ])
        .env(OUTPUT_CHILD, "1")
        .stdin(Stdio::null());

    let result = capture("test process", &mut command, Some(Duration::from_secs(5)));

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
fn process_child_writes_excess_output() -> Result<(), io::Error> {
    if env::var_os(OUTPUT_CHILD).is_none() {
        return Ok(());
    }
    let bytes = vec![b'x'; 1_048_577];
    let mut output = io::stdout().lock();
    output.write_all(&bytes)?;
    output.flush()
}

#[test]
fn inherited_descendant_pipe_obeys_the_process_deadline() -> Result<(), Box<dyn std::error::Error>>
{
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "bounded_process::tests::process_child_leaves_descendant_pipe_open",
        ])
        .env(DESCENDANT_PARENT, "1")
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
    Ok(())
}

#[test]
fn process_child_leaves_descendant_pipe_open() -> Result<(), io::Error> {
    if env::var_os(DESCENDANT_PARENT).is_none() {
        return Ok(());
    }
    let executable = env::current_exe()?;
    drop(
        Command::new(executable)
            .args([
                "--exact",
                "bounded_process::tests::process_descendant_holds_pipe",
            ])
            .env(DESCENDANT_CHILD, "1")
            .spawn()?,
    );
    Ok(())
}

#[test]
fn process_descendant_holds_pipe() {
    if env::var_os(DESCENDANT_CHILD).is_none() {
        return;
    }
    let (sender, receiver) = mpsc::channel::<()>();
    assert!(matches!(
        receiver.recv_timeout(Duration::from_millis(250)),
        Err(RecvTimeoutError::Timeout)
    ));
    drop(sender);
}
