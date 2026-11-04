//! This module owns Darwin directory observations and local APFS admission.

use std::{
    fs::File,
    io,
    path::{Component, Path},
};

use cap_fs_ext::MetadataExt;
use cap_std::fs::Dir;
use rustix::fs::{CWD, Mode, OFlags, fstatfs, openat};

use super::filesystem_root_identity::{DARWIN_COORDINATE_DOMAIN, FilesystemRootIdentity};
use super::platform_filesystem::PlatformFilesystem;

pub(super) struct DarwinFilesystem;

impl PlatformFilesystem for DarwinFilesystem {
    fn open_root(&self, path: &Path) -> io::Result<Dir> {
        let start = if path.is_absolute() { "/" } else { "." };
        let mut current = openat(CWD, start, directory_flags(), Mode::empty())?;
        for component in path.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(name) => {
                    current = openat(&current, name, directory_flags(), Mode::empty())?;
                }
                Component::ParentDir => {
                    current = openat(&current, "..", directory_flags(), Mode::empty())?;
                }
                Component::Prefix(_) => return Err(unsupported("unsupported root prefix")),
            }
        }
        Ok(Dir::from_std_file(File::from(current)))
    }

    fn admit_directory(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        // Darwin's public mount flags. Native APFS admission excludes snapshots
        // mounted read-only and network filesystems even if a name resembles APFS.
        const READ_ONLY: u32 = 0x0000_0001;
        const LOCAL: u32 = 0x0000_1000;
        let observed = fstatfs(directory)?;
        let apfs = observed
            .f_fstypename
            .into_iter()
            .take(5)
            .eq([97, 112, 102, 115, 0]);
        if !apfs || observed.f_flags & READ_ONLY != 0 || observed.f_flags & LOCAL == 0 {
            return Err(unsupported(
                "directory is not on a writable local APFS volume",
            ));
        }
        self.root_identity(directory)
    }

    fn root_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        let metadata = directory.dir_metadata()?;
        let device = u32::try_from(metadata.dev())
            .map_err(|source| io::Error::new(io::ErrorKind::Unsupported, source))?;
        // Each mounted APFS volume has its own st_dev, and fsid only repeats it
        // beside the filesystem type, so one observation serves both coordinates.
        let device = DARWIN_COORDINATE_DOMAIN | u64::from(device);
        Ok(FilesystemRootIdentity::new(device, device, metadata.ino()))
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    fn repository_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        self.root_identity(directory)
    }
}

fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}

fn unsupported(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[cfg(test)]
#[path = "darwin_filesystem_tests.rs"]
mod tests;
