//! This module owns the shared reader fence over one version-two store root.

use std::io;

use cap_fs_ext::MetadataExt;
use cap_std::fs::{Dir, File, Metadata};
use rustix::fs::{FlockOperation, flock};

use super::{ReaderFenceKind, ReaderFenceRefusal};
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
    file: File,
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
        Ok(Self { file })
    }

    /// Acquires the fence exclusively without waiting, for collection and
    /// disposition under writer authority.
    ///
    /// Any reader holding the shared fence refuses the acquisition with
    /// [`io::ErrorKind::WouldBlock`]; the caller reports that readers are
    /// active rather than waiting on them.
    pub(in crate::adapters) fn acquire_exclusive(root: &Dir) -> io::Result<Self> {
        let file = filesystem_exact_record::open_read(root, READER_LOCK)?;
        verify(root, &file)?;
        flock(&file, FlockOperation::NonBlockingLockExclusive)?;
        verify(root, &file)?;
        Ok(Self { file })
    }

    /// Returns the locked file's device and inode identity.
    pub(in crate::adapters) fn identity(&self) -> io::Result<(u64, u64)> {
        self.file.metadata().map(|metadata| identity(&metadata))
    }
}

fn verify(root: &Dir, file: &File) -> io::Result<()> {
    let handle = file.metadata()?;
    let entry = root.symlink_metadata(READER_LOCK)?;
    if !handle.is_file() || !entry.is_file() {
        return Err(refused(ReaderFenceRefusal::Kind {
            expected: ReaderFenceKind::RegularFile,
            observed_handle: kind(&handle),
            observed_entry: kind(&entry),
        }));
    }
    if handle.len() != 0 || entry.len() != 0 {
        return Err(refused(ReaderFenceRefusal::Length {
            expected: 0,
            observed_handle: handle.len(),
            observed_entry: entry.len(),
        }));
    }
    if identity(&handle) != identity(&entry) {
        return Err(refused(ReaderFenceRefusal::Identity {
            expected: identity(&handle),
            observed: identity(&entry),
        }));
    }
    Ok(())
}

fn kind(metadata: &Metadata) -> ReaderFenceKind {
    if metadata.is_file() {
        ReaderFenceKind::RegularFile
    } else {
        ReaderFenceKind::Other
    }
}

fn refused(refusal: ReaderFenceRefusal) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, refusal)
}

fn identity(metadata: &Metadata) -> (u64, u64) {
    (metadata.dev(), metadata.ino())
}
