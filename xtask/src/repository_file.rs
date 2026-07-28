//! This module owns capability-relative, no-follow repository-file admission.
//!
//! Repository tasks are intentionally supported only on Unix hosts. This module
//! binds an opened repository root to Unix device and inode identity so that path
//! replacement cannot silently redirect a read. Supporting another host requires
//! an equivalent stable directory-identity contract before enabling these tasks.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadAccessPolicy {
    Enabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BlockingIoPolicy {
    Refuse,
}

#[derive(Clone, Copy)]
pub(crate) struct RepositoryReadPolicy {
    read_access: ReadAccessPolicy,
    blocking_io: BlockingIoPolicy,
}

pub(crate) const REPOSITORY_READ_POLICY: RepositoryReadPolicy = RepositoryReadPolicy {
    read_access: ReadAccessPolicy::Enabled,
    blocking_io: BlockingIoPolicy::Refuse,
};

pub(crate) struct RepositoryRoot {
    directory: Dir,
    identity: DirectoryIdentity,
    path: PathBuf,
}

#[derive(Eq, PartialEq)]
struct DirectoryIdentity {
    device: u64,
    inode: u64,
}

pub(crate) enum OpenRepositoryFileError {
    Io(io::Error),
    NonRegular,
}

impl RepositoryRoot {
    pub(crate) fn open(path: &Path) -> Result<Self, io::Error> {
        let directory = Dir::open_ambient_dir(path, ambient_authority())?;
        let identity = DirectoryIdentity::from(&directory.dir_metadata()?);
        Ok(Self {
            directory,
            identity,
            path: path.to_owned(),
        })
    }

    pub(crate) fn display_path(&self, relative: &Path) -> PathBuf {
        self.path.join(relative)
    }

    pub(crate) fn is_current_path(&self) -> Result<bool, io::Error> {
        let current = Dir::open_ambient_dir(&self.path, ambient_authority())?;
        let identity = DirectoryIdentity::from(&current.dir_metadata()?);
        Ok(self.identity == identity)
    }

    pub(crate) fn open_file(&self, relative: &Path) -> Result<File, OpenRepositoryFileError> {
        let file = self
            .directory
            .open_with(relative, &REPOSITORY_READ_POLICY.options())
            .map_err(OpenRepositoryFileError::Io)?
            .into_std();
        let metadata = file.metadata().map_err(OpenRepositoryFileError::Io)?;
        if metadata.is_file() {
            Ok(file)
        } else {
            Err(OpenRepositoryFileError::NonRegular)
        }
    }
}

impl RepositoryReadPolicy {
    #[cfg(test)]
    pub(super) const fn read_access(self) -> ReadAccessPolicy {
        self.read_access
    }

    #[cfg(test)]
    pub(super) const fn blocking_io(self) -> BlockingIoPolicy {
        self.blocking_io
    }

    fn options(self) -> OpenOptions {
        let mut options = OpenOptions::new();
        match self.read_access {
            ReadAccessPolicy::Enabled => {
                options.read(true);
            }
        }
        options.follow(FollowSymlinks::No);
        match self.blocking_io {
            BlockingIoPolicy::Refuse => {
                options.nonblock(true);
            }
        }
        options
    }
}

impl From<&cap_std::fs::Metadata> for DirectoryIdentity {
    fn from(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}
