//! This module owns Git process deadline and diagnostic regression evidence.

use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;

use std::os::unix::process::CommandExt;

use super::{GIT_DIAGNOSTIC_LIMIT_BYTES, git_failure, paths_with_deadline, process_failure};
use crate::bounded_process::ProcessError;
use crate::git_inventory::GitInventoryError;

const PARKED_CHILD: &str = "KEEP_XTASK_PARKED_GIT_CHILD";
const PARKED_CHILD_TEST: &str = "git_inventory::process::tests::process_child_parks_indefinitely";

#[test]
fn git_diagnostic_encoding_failure_retains_exit_status() {
    let error = git_failure("test diagnostics", Some(9), vec![u8::MAX]);
    assert!(matches!(
        error,
        GitInventoryError::DiagnosticEncoding {
            operation: "test diagnostics",
            code: Some(9),
            ..
        }
    ));
}

#[test]
fn git_diagnostic_limit_maps_to_the_inventory_boundary() {
    let error = process_failure(
        "test diagnostics",
        ProcessError::OutputLimit {
            program: "git",
            stream: "stderr",
            maximum: GIT_DIAGNOSTIC_LIMIT_BYTES,
        },
    );
    assert!(matches!(
        error,
        GitInventoryError::OutputBound {
            operation: "test diagnostics",
            stream: "diagnostic bytes",
            maximum: GIT_DIAGNOSTIC_LIMIT_BYTES,
            ..
        }
    ));
}

#[test]
fn stalled_git_process_obeys_the_inventory_deadline() -> Result<(), Box<dyn std::error::Error>> {
    let executable = env::current_exe()?;
    let child = Command::new(executable)
        .args(["--exact", PARKED_CHILD_TEST])
        .env(PARKED_CHILD, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()?;
    let deadline = Duration::from_millis(25);

    let result = paths_with_deadline(&[], "test Git inventory", deadline, move |_command| {
        Ok(child)
    });

    assert!(matches!(
        result,
        Err(GitInventoryError::Process {
            operation: "test Git inventory",
            source: ProcessError::Timeout {
                program: "git",
                duration,
            },
        }) if duration == deadline
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
