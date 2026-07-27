//! This module owns capability-relative, no-follow source-file admission.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

pub(super) struct SourceRoot {
    directory: Dir,
    path: PathBuf,
}

pub(super) enum OpenSourceError {
    Io(io::Error),
    NonRegular,
}

impl SourceRoot {
    pub(super) fn open(path: &Path) -> Result<Self, io::Error> {
        let directory = Dir::open_ambient_dir(path, ambient_authority())?;
        Ok(Self {
            directory,
            path: path.to_owned(),
        })
    }

    pub(super) fn display_path(&self, relative: &Path) -> PathBuf {
        self.path.join(relative)
    }

    pub(super) fn open_file(&self, relative: &Path) -> Result<File, OpenSourceError> {
        let file = self
            .directory
            .open_with(relative, &nonblocking_read_options())
            .map_err(OpenSourceError::Io)?
            .into_std();
        let metadata = file.metadata().map_err(OpenSourceError::Io)?;
        if metadata.is_file() {
            Ok(file)
        } else {
            Err(OpenSourceError::NonRegular)
        }
    }
}

pub(super) fn nonblocking_read_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    options
}
