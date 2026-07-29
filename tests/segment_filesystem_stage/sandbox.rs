//! Deterministic filesystem sandbox for filesystem adapter laws.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Process-isolated directory beneath Cargo's test scratch root.
pub(super) struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    /// Creates an empty named sandbox for the current test process.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure from scratch-root creation,
    /// removal of same-process stale test evidence, or sandbox creation.
    pub(super) fn create(name: &str) -> io::Result<Self> {
        let root = option_env!("CARGO_TARGET_TMPDIR").map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("target/tmp"),
            PathBuf::from,
        );
        fs::create_dir_all(&root)?;
        let path = root.join(format!("keep-{name}-{}", std::process::id()));
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(source),
        }
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    /// Returns the sandbox root.
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    /// Removes the complete sandbox after all test handles are closed.
    ///
    /// # Errors
    ///
    /// Returns the exact recursive-removal filesystem failure.
    pub(super) fn remove(self) -> io::Result<()> {
        fs::remove_dir_all(self.path)
    }
}
