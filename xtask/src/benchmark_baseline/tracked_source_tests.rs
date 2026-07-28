//! Exact tracked-source comparison laws.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use super::matches_head;
use crate::test_directory::TestDirectory;

#[test]
fn clean_worktree_matches_head_without_mutating_the_real_index() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-clean-source")?;
    git(directory.path(), &["init", "--quiet"])?;
    git(directory.path(), &["config", "user.name", "Keep Tests"])?;
    git(
        directory.path(),
        &["config", "user.email", "keep-tests@example.invalid"],
    )?;
    fs::write(directory.path().join("tracked.txt"), b"exact\n")?;
    git(directory.path(), &["add", "tracked.txt"])?;
    git(directory.path(), &["commit", "--quiet", "-m", "fixture"])?;
    let index = directory.path().join(".git/index");
    let before = fs::read(&index)?;

    assert!(matches_head(directory.path())?);
    assert_eq!(fs::read(&index)?, before);
    directory.close()?;
    Ok(())
}

fn git(repository: &Path, arguments: &[&str]) -> Result<(), io::Error> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "fixture Git command failed with {:?}",
            status.code()
        )))
    }
}
