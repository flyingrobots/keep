//! This module owns hermetic Git commands for corpus regression repositories.

use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::bounded_process;

const GIT_FIXTURE_DEADLINE: Duration = Duration::from_mins(2);

pub(in crate::documentation_integrity) fn run_git(
    root: &Path,
    arguments: &[&str],
) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("git");
    command
        .args(arguments)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let output = bounded_process::status(
        "documentation Git fixture",
        &mut command,
        Some(GIT_FIXTURE_DEADLINE),
    )?;
    if output.succeeded {
        Ok(())
    } else {
        Err(format!("git fixture command failed: {arguments:?}").into())
    }
}
