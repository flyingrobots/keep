//! This module owns byte-exact copies of a quiescent store for portability laws.
//!
//! A copy is what a backup or a move to another disk produces: every
//! directory and regular file recreated with the same bytes under a new root,
//! so every inode, and the root's own device-and-inode pair, differs.

use std::fs;
use std::path::Path;

use super::writer_fixture::TestResult;

/// Copies the complete store at `source` into the empty directory `destination`.
pub(super) fn copy_store(source: &Path, destination: &Path) -> TestResult<()> {
    let mut pending = vec![(source.to_path_buf(), destination.to_path_buf())];
    while let Some((from, to)) = pending.pop() {
        for entry in fs::read_dir(&from)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let target = to.join(entry.file_name());
            if kind.is_dir() {
                fs::create_dir(&target)?;
                pending.push((entry.path(), target));
            } else if kind.is_file() {
                fs::copy(entry.path(), &target)?;
            } else {
                return Err(format!("{} is not a store entry", entry.path().display()).into());
            }
        }
    }
    Ok(())
}
