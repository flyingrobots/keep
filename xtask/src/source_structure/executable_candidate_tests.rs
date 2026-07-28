//! This module owns executable admission outside known source suffixes.

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::git_inventory::GitPath;
use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

#[test]
fn non_source_suffix_cannot_hide_executable_python() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("hidden-python")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    let script = repository.join("check.txt");
    fs::write(&script, b"#!/usr/bin/python3\nprint('forbidden')\n")?;
    make_executable(&script)?;
    let present = BTreeSet::from([GitPath::new(b"check.txt".to_vec())]);

    let paths = super::select_source_inventory(&present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&repository)?;
    let result = super::inventory_violations(&source_root, paths);

    assert!(matches!(
        result,
        Err(super::SourceStructureError::PythonSource(ref path))
            if path == Path::new("check.txt")
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn non_executable_text_remains_outside_the_python_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("non-executable-text")?;
    let repository = directory.path().join("repository");
    fs::create_dir(&repository)?;
    fs::write(
        repository.join("check.txt"),
        b"#!/usr/bin/python3\nnot executable\n",
    )?;
    let present = BTreeSet::from([GitPath::new(b"check.txt".to_vec())]);

    let paths = super::select_source_inventory(&present, &BTreeSet::new())?;
    let source_root = RepositoryRoot::open(&repository)?;
    let violations = super::inventory_violations(&source_root, paths)?;

    assert!(violations.is_empty());
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn non_utf8_executable_candidate_preserves_path_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let path_bytes = b"check-\xff.txt";
    let present = BTreeSet::from([GitPath::new(path_bytes.to_vec())]);

    let inventory = super::select_source_inventory(&present, &BTreeSet::new())?;

    assert_eq!(
        inventory
            .executable_candidates
            .first()
            .map(|path| path.as_os_str().as_bytes()),
        Some(path_bytes.as_slice())
    );
    Ok(())
}

fn make_executable(path: &Path) -> Result<(), std::io::Error> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}
