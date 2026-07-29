//! This module owns sync-capable capability directory admission.

use std::io;

use cap_fs_ext::{
    FollowSymlinks, OpenOptionsFollowExt, OpenOptionsMaybeDirExt, OpenOptionsSyncExt,
};
use cap_std::fs::{Dir, OpenOptions};

pub(super) fn open(parent: &Dir, name: &str) -> io::Result<Dir> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .maybe_dir(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    let file = parent.open_with(name, &options)?;
    if !file.metadata()?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "sync-capable target is not a directory",
        ));
    }
    Ok(Dir::from_std_file(file.into_std()))
}
