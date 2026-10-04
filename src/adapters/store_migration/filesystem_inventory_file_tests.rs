//! Identity-stable migration artifact read laws.

use std::error::Error;
use std::fs;
use std::io;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_inventory_file::{
    self, FilesystemInventoryFileError, FilesystemInventoryFilePolicy,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{CatalogRestartArtifact, CatalogRestartPhase};

#[test]
fn replacement_after_read_refuses_the_opened_artifact() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-artifact-replacement")?;
    let name = "artifact";
    fs::write(sandbox.path().join(name), b"old bytes")?;
    let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;

    let result = filesystem_inventory_file::read_with(&directory, name, policy(9), || {
        replace(sandbox.path(), name);
    });
    assert!(matches!(result, Err(FilesystemInventoryFileError::Changed)));
    drop(directory);
    sandbox.remove()?;
    Ok(())
}

fn replace(root: &std::path::Path, name: &str) {
    let renamed = fs::rename(root.join(name), root.join("replaced"));
    let written = fs::write(root.join(name), b"old bytes");
    assert!(renamed.is_ok(), "test replacement rename failed");
    assert!(written.is_ok(), "test replacement write failed");
}

#[test]
fn regular_file_read_returns_exact_bytes() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-artifact-read")?;
    let name = "artifact";
    fs::write(sandbox.path().join(name), b"exact")?;
    let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;

    let bytes = filesystem_inventory_file::read(&directory, name, policy(5)).map_err(file_error)?;
    assert_eq!(bytes, b"exact");
    drop(directory);
    sandbox.remove()?;
    Ok(())
}

fn file_error(error: FilesystemInventoryFileError) -> io::Error {
    match error {
        FilesystemInventoryFileError::Artifact(source) => io::Error::other(source),
        FilesystemInventoryFileError::Changed => io::Error::other("artifact changed"),
    }
}

const fn policy(maximum_length: u64) -> FilesystemInventoryFilePolicy {
    FilesystemInventoryFilePolicy::new(
        CatalogRestartArtifact::Catalog,
        CatalogRestartPhase::OpenCatalog,
        CatalogRestartPhase::ReadCatalog,
        maximum_length,
    )
}
