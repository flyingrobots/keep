//! This module owns hermetic Git commands for repository regression fixtures.

use std::env;
use std::error::Error;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::bounded_process;

const GIT_FIXTURE_DEADLINE: Duration = Duration::from_mins(2);

/// Runs one Git fixture command under the repository's bounded process policy.
///
/// The call blocks for at most two minutes, starts Git in a dedicated process
/// group, discards both output streams, and terminates descendants on timeout
/// or interruption. The child inherits only the admitted executable search
/// path, deterministic locale, and null system and global Git configuration.
/// Process failures retain their typed source; a nonzero Git status reports the
/// attempted arguments without admitting tool output.
pub(crate) fn run_git(root: &Path, arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let path = env::var_os("PATH")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "PATH is unavailable"))?;
    let mut command = Command::new("git");
    command
        .args(arguments)
        .current_dir(root)
        .env_clear()
        .env("PATH", path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let output = bounded_process::status(
        "repository Git fixture",
        &mut command,
        Some(GIT_FIXTURE_DEADLINE),
    )?;
    if output.succeeded {
        Ok(())
    } else {
        Err(format!("git fixture command failed: {arguments:?}").into())
    }
}
