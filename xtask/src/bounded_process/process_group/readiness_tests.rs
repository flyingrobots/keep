//! This module owns deterministic readiness synchronization evidence.

use std::env;
use std::io;
use std::process::{Command, Stdio};

use super::{readiness_listener, wait_for_ready};
use crate::test_directory::TestDirectory;

#[test]
fn readiness_wait_does_not_depend_on_wall_clock() {
    let process_group = include_str!("../process_group.rs");

    assert!(
        !process_group.contains("std::time::"),
        "readiness must be bounded by the child lifecycle, not wall-clock time"
    );
}

#[test]
fn readiness_refuses_child_exit_without_a_signal() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("readiness-exit")?;
    let listener = readiness_listener(&directory.path().join("ready"))?;
    let mut child = Command::new(env::current_exe()?)
        .args([
            "--exact",
            "bounded_process::process_group::readiness_tests::readiness_wait_does_not_depend_on_wall_clock",
        ])
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
