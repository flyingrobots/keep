//! This module owns refusal on platforms without a production adapter.

use std::{io, path::Path};

use cap_std::fs::Dir;

use super::filesystem_directory_coordinate::FilesystemDirectoryCoordinate;
use super::platform_filesystem::PlatformFilesystem;

pub(super) struct UnsupportedFilesystem;

impl PlatformFilesystem for UnsupportedFilesystem {
    fn open_root(&self, _path: &Path) -> io::Result<Dir> {
        Err(unsupported())
    }

    fn admit_directory(&self, _directory: &Dir) -> io::Result<FilesystemDirectoryCoordinate> {
        Err(unsupported())
    }
}

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem initialization requires an admitted Linux ext4 or macOS APFS profile",
    )
}
