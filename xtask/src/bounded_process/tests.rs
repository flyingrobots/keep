//! This module owns bounded child-process regression evidence.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::time::Duration;

use super::{ProcessError, capture, status};
use crate::test_directory::TestDirectory;

const PARKED_CHILD: &str = "KEEP_XTASK_PARKED_CHILD";

#[test]
fn external_output_is_drained_but_refused_above_the_bound() -> Result<(), Box<dyn std::error::Error>>
{
    let repository = TestDirectory::create("bounded-process-output")?;
    let blob = repository.path().join("oversized.bin");
    let global_config = repository.path().join("global.gitconfig");
    let template = repository.path().join("empty-template");
    fs::write(&blob, vec![b'x'; 1_048_577])?;
    fs::write(&global_config, [])?;
    fs::create_dir(&template)?;
    let initialized = fixture_git_command(&repository)?
        .args(["init", "--quiet"])
        .arg(template_argument(&template))
        .status()?;
    if !initialized.success() {
        return Err(io::Error::other("cannot initialize fixture repository").into());
    }
    let hashed = fixture_git_command(&repository)?
        .args(["hash-object", "-w", "oversized.bin"])
        .output()?;
    if !hashed.status.success() {
        return Err(io::Error::other("cannot hash fixture blob").into());
    }
    let object_id = str::from_utf8(&hashed.stdout)?.trim();
    let mut command = fixture_git_command(&repository)?;
    command
        .args(["cat-file", "blob", object_id])
        .stdin(Stdio::null());

    let result = capture("test process", &mut command, Some(Duration::from_secs(5)));
    repository.close()?;

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

fn fixture_git_command(repository: &TestDirectory) -> Result<Command, io::Error> {
    let path = env::var_os("PATH").ok_or_else(|| io::Error::other("PATH is unavailable"))?;
    let global_config = repository.path().join("global.gitconfig");
    let mut command = Command::new("git");
    command
        .env_clear()
        .env("PATH", path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", global_config)
        .env("LC_ALL", "C")
        .current_dir(repository.path());
    Ok(command)
}

fn template_argument(template: &Path) -> OsString {
    let mut argument = OsString::from("--template=");
    argument.push(template);
    argument
}

#[test]
fn fixture_template_argument_preserves_non_utf8_paths() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let template = OsString::from_vec(b"/tmp/non-utf8-\xff".to_vec());
    let argument = template_argument(Path::new(&template));

    assert_eq!(
        argument.as_os_str().as_bytes(),
        b"--template=/tmp/non-utf8-\xff"
    );
}

#[test]
fn inherited_process_obeys_the_process_deadline() -> Result<(), Box<dyn std::error::Error>> {
    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "bounded_process::tests::process_child_parks_indefinitely",
        ])
        .env(PARKED_CHILD, "1")
        .stdin(Stdio::null());

    let result = status(
        "test process",
        &mut command,
        Some(Duration::from_millis(50)),
    );

    assert!(matches!(
        result,
        Err(ProcessError::Timeout {
            program: "test process",
            duration,
        }) if duration == Duration::from_millis(50)
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
