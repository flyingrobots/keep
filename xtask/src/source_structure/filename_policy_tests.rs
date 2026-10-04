//! This module owns full-checker forbidden-name and diagnostic regression laws.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::SourceStructureError;
use crate::test_directory::TestDirectory;

#[test]
fn public_source_checker_refuses_an_inventoried_forbidden_basename() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("source-forbidden-name")?;
    let output = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(directory.path())
        .output()?;
    if !output.status.success() {
        return Err("fixture Git initialization failed".into());
    }
    fs::create_dir(directory.path().join("src"))?;
    fs::write(
        directory.path().join("src/utils.rs"),
        b"//! This module owns the forbidden-name fixture.\n",
    )?;
    assert!(matches!(super::check(directory.path()),
        Err(SourceStructureError::ForbiddenFilename(path)) if path == Path::new("src/utils.rs")));
    directory.close()?;
    Ok(())
}

#[test]
fn forbidden_name_diagnostics_escape_parent_directory_controls() {
    let error = SourceStructureError::ForbiddenFilename("src/first\nforged/utils.rs".into());
    assert_eq!(
        error.to_string(),
        "repository source uses forbidden filename `src/first\\nforged/utils.rs`"
    );
}
