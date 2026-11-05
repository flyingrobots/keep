//! This module owns the native filesystem mechanism interface.
//!
//! This infrastructure boundary stays below the semantic storage ports. Its
//! capability handles never enter domain identity or transition APIs.

use std::{io, path::Path};

use cap_std::fs::Dir;

use super::filesystem_directory_coordinate::FilesystemDirectoryCoordinate;

pub(super) trait PlatformFilesystem {
    /// Pins a readable directory without traversing symbolic links.
    fn open_root(&self, path: &Path) -> io::Result<Dir>;

    /// Requires a supported writable profile and observes its live coordinates.
    fn admit_directory(&self, directory: &Dir) -> io::Result<FilesystemDirectoryCoordinate>;
}
