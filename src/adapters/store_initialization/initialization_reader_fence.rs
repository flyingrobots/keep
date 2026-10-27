//! This module owns creation of the persistent reader fence during initialization.

use std::io;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};

use super::initialization_namespace::READER_LOCK;
use crate::adapters::filesystem_catalog_artifact;
use crate::adapters::filesystem_namespace_refusal::{
    self as namespace, FilesystemEntryKind, FilesystemNamespaceRefusal,
};

/// Creates or admits zero-length `reader.lock`, then synchronizes the root.
///
/// The open is exclusive, follows no link, and does not block, so a FIFO or
/// device planted at the name refuses by kind instead of hanging. An existing
/// entry is admitted only as a zero-length regular file.
pub(super) fn admit_reader_fence(root: &Dir) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    match root.open_with(READER_LOCK, &options) {
        Ok(created) => drop(created),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
        Err(source) => return Err(source),
    }
    let metadata = root.symlink_metadata(READER_LOCK)?;
    namespace::require_kind(metadata.file_type(), FilesystemEntryKind::File)?;
    if metadata.len() == 0 {
        filesystem_catalog_artifact::synchronize_directory(root)
    } else {
        Err(FilesystemNamespaceRefusal::UnexpectedEntry.into_io())
    }
}
