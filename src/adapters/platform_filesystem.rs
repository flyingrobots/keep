//! This module owns the native filesystem mechanism interface.
//!
//! This infrastructure boundary stays below the semantic storage ports. Its
//! capability handles never enter domain identity or transition APIs.

use std::{io, path::Path};

use cap_std::fs::Dir;

use super::filesystem_root_identity::FilesystemRootIdentity;

pub(super) trait PlatformFilesystem {
    /// Pins a readable directory without traversing symbolic links.
    fn open_root(&self, path: &Path) -> io::Result<Dir>;

    /// Requires a supported writable profile and observes its live coordinates.
    fn admit_directory(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity>;

    /// Observes descriptor-bound coordinates without admitting a new profile.
    fn root_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity>;

    /// Supplies historical repository-fixture coordinates, never production proof.
    #[cfg(any(test, feature = "repository-tasks"))]
    fn repository_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity>;
}
