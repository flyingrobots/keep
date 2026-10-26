//! This module owns persistent migration reader-fence admission.

use std::io;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, OpenOptions};

use crate::adapters::ReaderFenceRefusal;

const READER_LOCK: &str = "reader.lock";

pub(super) fn create(root: &Dir) -> io::Result<File> {
    let mut options = options();
    options.create_new(true);
    root.open_with(READER_LOCK, &options)
}

pub(super) fn open(root: &Dir) -> io::Result<File> {
    root.open_with(READER_LOCK, &options())
}

pub(super) fn verify(root: &Dir, file: &File) -> io::Result<()> {
    let handle = file.metadata()?;
    let entry = root.symlink_metadata(READER_LOCK)?;
    ReaderFenceRefusal::admit(&handle, &entry)
}

fn options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    options
}
