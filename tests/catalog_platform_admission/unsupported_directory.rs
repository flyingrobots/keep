//! This module owns per-process tmpfs scratch for unsupported-platform refusal laws.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Private scratch on the Linux runner's explicitly unsupported tmpfs mount.
pub(super) struct UnsupportedDirectory {
    path: PathBuf,
}

impl UnsupportedDirectory {
    pub(super) fn create() -> io::Result<Self> {
        let path =
            Path::new("/dev/shm").join(format!("keep-catalog-platform-{}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn remove(self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }
}

impl Drop for UnsupportedDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
