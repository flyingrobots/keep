//! This module owns the pure-Rust source-admission regression law.

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::bounded_process;
use crate::git_inventory::GitPath;
use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

#[test]
fn python_source_is_refused_by_the_pure_rust_boundary() {
    for path in [
        ".py",
        "scripts/.PYW",
        "scripts/check.py",
        "scripts/check.Py",
        "scripts/check.PY",
        "scripts/check.pyw",
        "scripts/check.pYw",
        "scripts/check.PYW",
    ] {
        let present = BTreeSet::from([GitPath::new(path.as_bytes().to_vec())]);
        assert!(matches!(
            super::select_source_paths(&present, &BTreeSet::new()),
            Err(super::SourceStructureError::PythonSource(ref observed))
                if observed == path
        ));
    }
}

#[test]
fn extensionless_executable_python_is_refused_by_the_pure_rust_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = SourceFixture::create(
        "extensionless-python",
        "check",
        b"#!/usr/bin/env python3\nprint('forbidden')\n",
        FixtureMode::Executable,
    )?;

    let paths = super::select_source_paths(&fixture.present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&fixture.repository)?;
    let result = super::source_violations(&source_root, paths);

    assert!(matches!(
        result,
        Err(super::SourceStructureError::PythonSource(ref path)) if path == "check"
    ));
    drop(source_root);
    fixture.close()?;
    Ok(())
}

#[test]
fn extension_bearing_executable_python_is_refused_by_the_pure_rust_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = SourceFixture::create(
        "extension-bearing-python",
        "check.sh",
        b"#!/usr/bin/python3\nprint('forbidden')\n",
        FixtureMode::Executable,
    )?;

    let paths = super::select_source_paths(&fixture.present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&fixture.repository)?;
    let result = super::source_violations(&source_root, paths);

    assert!(matches!(
        result,
        Err(super::SourceStructureError::PythonSource(ref path)) if path == "check.sh"
    ));
    drop(source_root);
    fixture.close()?;
    Ok(())
}

#[test]
fn extensionless_executable_source_obeys_the_line_limit() -> Result<(), Box<dyn std::error::Error>>
{
    let limit = usize::try_from(super::SOURCE_MODULE_HARD_LIMIT_LINES)?;
    let contents = format!("#!/bin/sh\n{}", ":\n".repeat(limit));
    let fixture = SourceFixture::create(
        "extensionless-source-limit",
        "check",
        contents.as_bytes(),
        FixtureMode::Executable,
    )?;

    let paths = super::select_source_paths(&fixture.present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&fixture.repository)?;
    let violations = super::source_violations(&source_root, paths)?;

    assert_eq!(violations, vec![String::from("check")]);
    drop(source_root);
    fixture.close()?;
    Ok(())
}

#[test]
fn extensionless_nonexecutable_text_is_not_a_source_module()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = SourceFixture::create(
        "extensionless-text",
        "NOTICE",
        b"#!/usr/bin/env python3\nnot executable\n",
        FixtureMode::NonExecutable,
    )?;

    let paths = super::select_source_paths(&fixture.present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&fixture.repository)?;
    assert!(super::source_violations(&source_root, paths)?.is_empty());

    drop(source_root);
    fixture.close()?;
    Ok(())
}

#[test]
fn tracked_executable_mode_must_match_the_worktree() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("tracked-executable-mode")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    run_git(&repository, &["init", "--quiet", "--template="])?;
    fs::write(
        repository.join("script"),
        "#!/usr/bin/env python3\nprint('forbidden')\n",
    )?;
    run_git(&repository, &["add", "script"])?;
    run_git(&repository, &["update-index", "--chmod=+x", "script"])?;

    let result = super::check(&repository);

    assert!(matches!(
        result,
        Err(super::SourceStructureError::ExecutionModeChanged {
            ref path,
            tracked: "executable",
            worktree: "nonexecutable",
        }) if path == &repository.join("script")
    ));
    directory.close()?;
    Ok(())
}

#[derive(Clone, Copy)]
enum FixtureMode {
    Executable,
    NonExecutable,
}

struct SourceFixture {
    directory: TestDirectory,
    present: BTreeSet<GitPath>,
    repository: PathBuf,
}

impl SourceFixture {
    fn create(
        case: &str,
        path: &str,
        contents: &[u8],
        mode: FixtureMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = TestDirectory::create(case)?;
        let repository = directory.path().join("repository");
        fs::create_dir(&repository)?;
        let script = repository.join(path);
        fs::write(&script, contents)?;
        let mut permissions = fs::metadata(&script)?.permissions();
        permissions.set_mode(match mode {
            FixtureMode::Executable => 0o755,
            FixtureMode::NonExecutable => 0o644,
        });
        fs::set_permissions(&script, permissions)?;
        Ok(Self {
            directory,
            present: BTreeSet::from([GitPath::new(path.as_bytes().to_vec())]),
            repository,
        })
    }

    fn close(self) -> Result<(), Box<dyn std::error::Error>> {
        self.directory.close()?;
        Ok(())
    }
}

fn run_git(
    repository: &std::path::Path,
    arguments: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::var_os("PATH").ok_or("test PATH is unavailable")?;
    let mut command = Command::new("git");
    command
        .args(arguments)
        .current_dir(repository)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .env("PATH", path)
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    let output = bounded_process::status(
        "git test fixture",
        &mut command,
        Some(Duration::from_secs(10)),
    )?;
    if output.succeeded {
        Ok(())
    } else {
        Err(format!("git test fixture failed with status {:?}", output.code).into())
    }
}
