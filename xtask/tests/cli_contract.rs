//! Subprocess-level contract tests for repository maintenance commands.

#![cfg(feature = "repository-tasks")]

use std::io;
use std::process::{Command, Output};

#[test]
fn successful_verification_is_silent() -> Result<(), io::Error> {
    let output = invoke(&["verify"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn missing_command_returns_the_versioned_usage_contract() -> Result<(), io::Error> {
    let output = invoke(&[])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"Error: usage: cargo xtask \
          <golden-file-worldline-check|source-structure-check|verify>\n"
    );
    Ok(())
}

#[test]
fn unknown_command_is_a_silent_stdout_refusal() -> Result<(), io::Error> {
    let output = invoke(&["unknown"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"Error: unknown xtask command `unknown`\n");
    Ok(())
}

#[test]
fn extra_argument_is_a_silent_stdout_refusal() -> Result<(), io::Error> {
    let output = invoke(&["verify", "extra"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"Error: unexpected xtask argument `extra`\n");
    Ok(())
}

fn invoke(arguments: &[&str]) -> Result<Output, io::Error> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(arguments)
        .output()
}
