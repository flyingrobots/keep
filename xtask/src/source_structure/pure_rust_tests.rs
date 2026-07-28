//! This module owns the pure-Rust source-admission regression law.

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::git_inventory::GitPath;
use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

#[test]
fn python_source_is_refused_by_the_pure_rust_boundary() {
    for path in [
        ".py",
        "scripts/.PYW",
        "scripts/check.py",
        "scripts/check.PY",
        "scripts/check.pyw",
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
    let directory = TestDirectory::create("extensionless-python")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    let script = repository.join("check");
    fs::write(&script, b"#!/usr/bin/env python3\nprint('forbidden')\n")?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    let present = BTreeSet::from([GitPath::new(b"check".to_vec())]);

    let paths = super::select_source_paths(&present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&repository)?;
    let result = super::source_violations(&source_root, paths);

    assert!(matches!(
        result,
        Err(super::SourceStructureError::PythonSource(ref path)) if path == "check"
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn extension_bearing_executable_python_is_refused_by_the_pure_rust_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("extension-bearing-python")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    let script = repository.join("check.sh");
    fs::write(&script, b"#!/usr/bin/python3\nprint('forbidden')\n")?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    let present = BTreeSet::from([GitPath::new(b"check.sh".to_vec())]);

    let paths = super::select_source_paths(&present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&repository)?;
    let result = super::source_violations(&source_root, paths);

    assert!(matches!(
        result,
        Err(super::SourceStructureError::PythonSource(ref path)) if path == "check.sh"
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn extensionless_executable_source_obeys_the_line_limit() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::create("extensionless-source-limit")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    let script = repository.join("check");
    fs::write(&script, format!("#!/bin/sh\n{}", ":\n".repeat(500)))?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    let present = BTreeSet::from([GitPath::new(b"check".to_vec())]);

    let paths = super::select_source_paths(&present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&repository)?;
    let violations = super::source_violations(&source_root, paths)?;

    assert_eq!(violations, vec![String::from("check")]);
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn extensionless_nonexecutable_text_is_not_a_source_module()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("extensionless-text")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    fs::write(
        repository.join("NOTICE"),
        b"#!/usr/bin/env python3\nnot executable\n",
    )?;
    let mut permissions = fs::metadata(repository.join("NOTICE"))?.permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(repository.join("NOTICE"), permissions)?;
    let present = BTreeSet::from([GitPath::new(b"NOTICE".to_vec())]);

    let paths = super::select_source_paths(&present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&repository)?;
    assert!(super::source_violations(&source_root, paths)?.is_empty());

    drop(source_root);
    directory.close()?;
    Ok(())
}
