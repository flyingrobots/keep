//! Persistent, capability-relative filesystem writer exclusion.

use std::fs::{File, TryLockError};
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

use super::{WriterLockAcquireError, WriterLockAcquirePhase};

const LOCK_FILE_NAME: &str = "writer.lock";

/// Exclusive kernel-managed writer authority over one pinned store root.
///
/// The guard retains both the opened root capability and lock-file handle.
/// Dropping it closes the handle and releases the process-scoped kernel lock;
/// it never deletes, renames, truncates, or replaces `writer.lock`.
#[must_use]
pub struct FilesystemWriterLock {
    directory: Dir,
    _lock_file: File,
}

impl FilesystemWriterLock {
    /// Tries to acquire exclusive writer authority without blocking.
    ///
    /// The store root is pinned before `writer.lock` is opened relative to it.
    /// The lock entry must already exist as a regular file and is opened
    /// without following symbolic links.
    ///
    /// # Errors
    ///
    /// Returns [`WriterLockAcquireError::Busy`] when another handle or process
    /// owns the lock. Other failures preserve their exact acquisition phase and
    /// I/O source. A missing lock file is never created by this operation.
    pub fn try_acquire(store_root: &Path) -> Result<Self, WriterLockAcquireError> {
        let directory =
            Dir::open_ambient_dir(store_root, ambient_authority()).map_err(|source| {
                WriterLockAcquireError::io(WriterLockAcquirePhase::OpenRoot, source)
            })?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .follow(FollowSymlinks::No)
            .nonblock(true);
        let lock_file = directory
            .open_with(LOCK_FILE_NAME, &options)
            .map_err(|source| {
                WriterLockAcquireError::io(WriterLockAcquirePhase::OpenFile, source)
            })?;
        let metadata = lock_file.metadata().map_err(|source| {
            WriterLockAcquireError::io(WriterLockAcquirePhase::InspectFile, source)
        })?;
        if !metadata.is_file() {
            return Err(WriterLockAcquireError::NotRegular);
        }
        let lock_file = lock_file.into_std();
        match lock_file.try_lock() {
            Ok(()) => Ok(Self {
                directory,
                _lock_file: lock_file,
            }),
            Err(TryLockError::WouldBlock) => Err(WriterLockAcquireError::Busy),
            Err(TryLockError::Error(source)) => Err(WriterLockAcquireError::io(
                WriterLockAcquirePhase::Acquire,
                source,
            )),
        }
    }

    pub(super) fn clone_directory(&self) -> std::io::Result<Dir> {
        self.directory.try_clone()
    }
}
