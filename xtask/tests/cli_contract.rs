//! Subprocess-level contract tests for repository maintenance commands.

#![cfg(feature = "repository-tasks")]

use std::io;
use std::path::Path;
use std::process::{Command, Output};

#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

const CARGO_CONFIGURATION: &str = include_str!("../../.cargo/config.toml");

#[test]
fn repository_tasks_require_the_committed_dependency_graph() {
    assert!(CARGO_CONFIGURATION.contains("xtask = \"run --quiet --locked --package xtask --\""));
}

#[test]
fn successful_verification_is_silent() -> Result<(), io::Error> {
    let output = invoke(&["verify"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn repository_alias_preserves_silent_success() -> Result<(), io::Error> {
    let output = invoke_alias(&["golden-file-worldline-check"])?;
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
          <benchmark-baseline|golden-file-worldline-check|prepare-fuzz-corpus|\
          source-structure-check|verify>\n"
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
fn command_diagnostics_escape_terminal_controls() -> Result<(), io::Error> {
    let output = invoke(&["first\nError: forged\rrewrite\u{1b}[31m"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"Error: unknown xtask command `first\\nError: forged\\rrewrite\\u{1b}[31m`\n"
    );
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

#[cfg(unix)]
#[test]
fn non_utf8_arguments_are_typed_refusals() -> Result<(), io::Error> {
    let invalid = OsString::from_vec(vec![0xff]);
    let command_output = invoke_os(std::slice::from_ref(&invalid))?;
    assert_eq!(command_output.status.code(), Some(1));
    assert!(command_output.stdout.is_empty());
    assert_eq!(
        command_output.stderr,
        b"Error: xtask command is not valid Unicode\n"
    );

    let extra_output = invoke_os(&[OsString::from("verify"), invalid])?;
    assert_eq!(extra_output.status.code(), Some(1));
    assert!(extra_output.stdout.is_empty());
    assert_eq!(
        extra_output.stderr,
        b"Error: unexpected xtask argument is not valid Unicode\n"
    );
    Ok(())
}

fn invoke(arguments: &[&str]) -> Result<Output, io::Error> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(arguments)
        .output()
}

fn invoke_alias(arguments: &[&str]) -> Result<Output, io::Error> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("xtask manifest has no repository parent"))?;
    Command::new(env!("CARGO"))
        .arg("xtask")
        .args(arguments)
        .current_dir(repository_root)
        .output()
}

#[cfg(unix)]
fn invoke_os(arguments: &[OsString]) -> Result<Output, io::Error> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(arguments)
        .output()
}
