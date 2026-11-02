//! This module owns refusal on platforms without a production adapter.

use std::{io, path::Path};

use cap_std::fs::Dir;

use super::filesystem_root_identity::FilesystemRootIdentity;
use super::platform_filesystem::PlatformFilesystem;

pub(super) struct UnsupportedFilesystem;

impl PlatformFilesystem for UnsupportedFilesystem {
    fn open_root(&self, _path: &Path) -> io::Result<Dir> {
        Err(unsupported())
    }

    fn admit_directory(&self, _directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        Err(unsupported())
    }

    fn root_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        #[cfg(any(test, feature = "repository-tasks"))]
        return self.repository_identity(directory);

        #[cfg(not(any(test, feature = "repository-tasks")))]
        {
            let _ = directory;
            Err(unsupported())
        }
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    fn repository_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        use cap_fs_ext::MetadataExt;

        let metadata = directory.dir_metadata()?;
        Ok(FilesystemRootIdentity::new(
            metadata.dev(),
            metadata.dev(),
            metadata.ino(),
        ))
    }
}

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem initialization currently requires the admitted Linux ext4 profile",
    )
}
