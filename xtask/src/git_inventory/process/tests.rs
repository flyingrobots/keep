//! This module owns Git process deadline and diagnostic regression evidence.

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::process::{Command, Stdio};
use std::time::Duration;

use std::os::unix::process::CommandExt;

use super::{
    GIT_DIAGNOSTIC_LIMIT_BYTES, git_command, git_failure, inventory_output, paths_with_deadline,
    process_failure,
};
use crate::bounded_process::{ProcessError, ProcessOutput};
use crate::git_inventory::GitInventoryError;

const PARKED_CHILD: &str = "KEEP_XTASK_PARKED_GIT_CHILD";
const PARKED_CHILD_TEST: &str = "git_inventory::process::tests::process_child_parks_indefinitely";

#[test]
fn git_inventory_receives_only_reviewed_environment() {
    let command = git_command(&["status"], OsString::from("/reviewed/tools").as_os_str());
    let observed = command
        .get_envs()
        .map(|(name, value)| (name.to_owned(), value.map(OsString::from)))
        .collect::<BTreeMap<_, _>>();
    let expected = BTreeMap::from([
        (
            OsString::from("GIT_CONFIG_COUNT"),
            Some(OsString::from("0")),
        ),
        (
            OsString::from("GIT_CONFIG_GLOBAL"),
            Some(OsString::from("/dev/null")),
        ),
        (
            OsString::from("GIT_CONFIG_NOSYSTEM"),
            Some(OsString::from("1")),
        ),
        (
            OsString::from("GIT_OPTIONAL_LOCKS"),
            Some(OsString::from("0")),
        ),
        (OsString::from("LC_ALL"), Some(OsString::from("C"))),
        (
            OsString::from("PATH"),
            Some(OsString::from("/reviewed/tools")),
        ),
    ]);

    assert_eq!(observed, expected);
}

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
fn failed_git_status_precedes_stdout_decoding() {
    let result = inventory_output(
        "test failure precedence",
        ProcessOutput {
            code: Some(23),
            succeeded: false,
            stdout: b"unterminated".to_vec(),
            stderr: b"fatal: reviewed failure\n".to_vec(),
        },
    );

    assert!(matches!(
        result,
        Err(GitInventoryError::Failed {
            operation: "test failure precedence",
            code: Some(23),
            ref stderr,
        }) if stderr == "fatal: reviewed failure\n"
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
