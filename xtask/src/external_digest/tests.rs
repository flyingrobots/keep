//! This module owns bounded external digest regression evidence.

use std::env;
use std::process::Command;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use crate::bounded_process::{ProcessError, ProcessOutput};

use super::{B3SUM, ExternalDigestError, execute, validate};

const BLOCKING_CHILD: &str = "KEEP_XTASK_DIGEST_BLOCKING_CHILD";

#[test]
fn child_that_never_reads_digest_input_obeys_the_complete_deadline()
-> Result<(), Box<dyn std::error::Error>> {
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "external_digest::tests::digest_child_does_not_read_stdin",
        ])
        .env(BLOCKING_CHILD, "1");
    let input = vec![0_u8; 1_048_576];
    let duration = Duration::from_millis(50);

    assert!(matches!(
        execute(&mut command, &[&input], duration),
        Err(ExternalDigestError::Process {
            source: ProcessError::Timeout {
                program: B3SUM,
                duration: observed,
            },
        }) if observed == duration
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

#[test]
fn failed_digest_preserves_utf8_and_non_utf8_diagnostics() {
    let utf8 = validate(ProcessOutput {
        code: Some(7),
        succeeded: false,
        stdout: Vec::new(),
        stderr: b"refused".to_vec(),
    });
    assert!(matches!(
        utf8,
        Err(ExternalDigestError::Failed {
            code: Some(7),
            ref stderr,
        }) if stderr == "refused"
    ));

    let non_utf8 = validate(ProcessOutput {
        code: Some(9),
        succeeded: false,
        stdout: Vec::new(),
        stderr: vec![0xff],
    });
    assert!(matches!(
        non_utf8,
        Err(ExternalDigestError::DiagnosticEncoding { code: Some(9), .. })
    ));
}

#[test]
fn successful_digest_requires_silent_exact_width_output() {
    let diagnostic = validate(ProcessOutput {
        code: Some(0),
        succeeded: true,
        stdout: vec![0_u8; 32],
        stderr: b"unexpected".to_vec(),
    });
    assert!(matches!(
        diagnostic,
        Err(ExternalDigestError::UnexpectedDiagnostic)
    ));

    let width = validate(ProcessOutput {
        code: Some(0),
        succeeded: true,
        stdout: vec![0_u8; 31],
        stderr: Vec::new(),
    });
    assert!(matches!(
        width,
        Err(ExternalDigestError::Width { observed: 31 })
    ));
}
