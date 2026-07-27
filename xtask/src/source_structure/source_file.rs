//! This module owns capability-relative, no-follow source-file admission.

use std::fs::File;
use std::io;
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

pub(super) enum OpenSourceError {
    Io(io::Error),
    NonRegular,
}

pub(super) fn open_source_file(
    repository_root: &Path,
    relative: &Path,
) -> Result<File, OpenSourceError> {
    let directory =
        Dir::open_ambient_dir(repository_root, ambient_authority()).map_err(OpenSourceError::Io)?;
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = directory
        .open_with(relative, &options)
        .map_err(OpenSourceError::Io)?
        .into_std();
    let metadata = file.metadata().map_err(OpenSourceError::Io)?;
    if metadata.is_file() {
        Ok(file)
    } else {
        Err(OpenSourceError::NonRegular)
    }
}
