//! This module owns source-root and source-file replacement-race tests.

use std::fs;
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};

use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

use super::super::repository_path::RepositoryPath;
use super::super::source_file::{AdmittedSource, FileExecution, SourceFileAdmission};
use super::super::{SourceLineCount, source_line_count, verify_source_root};

#[test]
fn source_scan_keeps_the_admitted_repository_root() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("source-root")?;
    let root = directory.path().join("repository");
    let retained_root = directory.path().join("retained");
    fs::create_dir(&root)?;
    fs::write(root.join("source.rs"), "safe\n")?;
    let source_root = RepositoryRoot::open(&root)?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;

    fs::rename(&root, &retained_root)?;
    fs::create_dir(&root)?;
    fs::write(root.join("source.rs"), "replacement\n".repeat(501))?;

    let source = regular_source(AdmittedSource::admit(&source_root, relative.as_path())?)?;
    assert_eq!(
        source_line_count(&source_root, &source)?,
        SourceLineCount::Within(1)
    );
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn source_scan_keeps_one_admitted_file_identity() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("source-file-identity")?;
    let root = directory.path().join("repository");
    fs::create_dir(&root)?;
    let source_path = root.join("source");
    let retained_path = root.join("retained");
    fs::write(&source_path, "#!/bin/sh\nsafe\n")?;
    fs::set_permissions(&source_path, fs::Permissions::from_mode(0o755))?;
    let source_root = RepositoryRoot::open(&root)?;
    let relative = RepositoryPath::admit(String::from("source"))?;

    let source = regular_source(AdmittedSource::admit(&source_root, relative.as_path())?)?;
    assert_eq!(source.execution(), FileExecution::Executable);
    fs::rename(&source_path, &retained_path)?;
    fs::write(&source_path, "replacement\n".repeat(501))?;

    let result = source_line_count(&source_root, &source);
    assert!(matches!(
        result,
        Err(super::super::SourceStructureError::SourceFileChanged(ref path))
            if path == &source_path
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn source_scan_refuses_in_place_mutation() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("source-file-mutation")?;
    let root = directory.path().join("repository");
    fs::create_dir(&root)?;
    let source_path = root.join("source.rs");
    fs::write(&source_path, "safe\n")?;
    let source_root = RepositoryRoot::open(&root)?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;
    let source = regular_source(AdmittedSource::admit(&source_root, relative.as_path())?)?;

    fs::write(&source_path, "mutated\n".repeat(501))?;

    let result = source_line_count(&source_root, &source);
    assert!(matches!(
        result,
        Err(super::super::SourceStructureError::SourceFileChanged(ref path))
            if path == &source_path
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn source_scan_detects_a_replaced_repository_root() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("source-identity")?;
    let root = directory.path().join("repository");
    let retained_root = directory.path().join("retained");
    fs::create_dir(&root)?;
    let source_root = RepositoryRoot::open(&root)?;

    fs::rename(&root, &retained_root)?;
    fs::create_dir(&root)?;
    let result = verify_source_root(&source_root, &root);
    assert!(matches!(
        result,
        Err(super::super::SourceStructureError::RepositoryRootChanged(ref path)) if path == &root
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[test]
fn source_open_refuses_replacement_symlink() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("source-replacement")?;
    let root = directory.path().join("repository");
    fs::create_dir(&root)?;
    let source_path = root.join("source.rs");
    let retained_path = root.join("retained.rs");
    let target_path = root.join("target.rs");
    fs::write(&source_path, "safe\n")?;
    fs::write(&target_path, "outside\n".repeat(501))?;
    let source_root = RepositoryRoot::open(&root)?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;

    fs::rename(&source_path, &retained_path)?;
    symlink(&target_path, &source_path)?;
    let admission = AdmittedSource::admit(&source_root, relative.as_path())?;
    assert!(matches!(admission, SourceFileAdmission::NonRegular));
    drop(source_root);
    directory.close()?;
    Ok(())
}

fn regular_source(admission: SourceFileAdmission) -> Result<AdmittedSource, io::Error> {
    match admission {
        SourceFileAdmission::Regular(source) => Ok(source),
        SourceFileAdmission::NonRegular => Err(io::Error::other("expected a regular source file")),
    }
}
