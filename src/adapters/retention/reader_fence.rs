//! This module owns the shared reader fence over one version-two store root.

use std::io;

use cap_fs_ext::MetadataExt;
use cap_std::fs::{Dir, File, Metadata};
use rustix::fs::{FlockOperation, flock};

use crate::adapters::filesystem_exact_record;

const READER_LOCK: &str = "reader.lock";

/// A shared kernel lock on `reader.lock` held for one snapshot's lifetime.
///
/// Collection acquires the store writer authority and then an exclusive lock
/// on the same file, so while any fence is held no published segment, root,
/// or manifest can be deleted. Publication proceeds beside fences because it
/// only adds immutable successors. Dropping the fence releases only the
/// kernel lock; the persistent file is never deleted.
#[must_use]
pub struct ReaderFence {
    _file: File,
}

impl ReaderFence {
    /// Acquires the shared fence, waiting while collection holds it exclusively.
    ///
    /// `reader.lock` must be a regular zero-length file reached without
    /// following links; its identity is verified after the open so a swapped
    /// entry refuses.
    pub(super) fn acquire(root: &Dir) -> io::Result<Self> {
        let file = filesystem_exact_record::open_read(root, READER_LOCK)?;
        verify(root, &file)?;
        flock(&file, FlockOperation::LockShared)?;
        verify(root, &file)?;
        Ok(Self { _file: file })
    }
}

fn verify(root: &Dir, file: &File) -> io::Result<()> {
    let handle = file.metadata()?;
    let entry = root.symlink_metadata(READER_LOCK)?;
    if handle.is_file()
        && entry.is_file()
        && handle.len() == 0
        && entry.len() == 0
        && identity(&handle) == identity(&entry)
    {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "reader fence kind, length, or identity disagreed",
        ))
    }
}

fn identity(metadata: &Metadata) -> (u64, u64) {
    (metadata.dev(), metadata.ino())
}
