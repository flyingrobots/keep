//! This module owns persistent migration reader-fence admission.

use std::io;

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, Metadata, OpenOptions};

use super::filesystem_migration_namespace_directory::ambiguous;

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
    if handle.is_file()
        && entry.is_file()
        && handle.len() == 0
        && entry.len() == 0
        && FileIdentity::from(&handle) == FileIdentity::from(&entry)
    {
        Ok(())
    } else {
        Err(ambiguous(
            "reader fence kind, length, or identity disagreed",
        ))
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl From<&Metadata> for FileIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}
