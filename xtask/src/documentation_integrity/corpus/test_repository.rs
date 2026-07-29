//! This module owns hermetic Git commands for corpus regression repositories.

use std::error::Error;
use std::path::Path;
use std::process::Command;

pub(in crate::documentation_integrity) fn run_git(
    root: &Path,
    arguments: &[&str],
) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("git fixture command failed: {arguments:?}").into())
    }
}
