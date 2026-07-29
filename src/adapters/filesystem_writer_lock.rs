//! Persistent, capability-relative filesystem writer exclusion.

use std::fs::{File, TryLockError};
use std::io;
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};

use super::{WriterLockAcquireError, WriterLockAcquirePhase};

#[cfg(test)]
#[path = "filesystem_writer_lock_tests.rs"]
mod tests;

const LOCK_FILE_NAME: &str = "writer.lock";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl FileIdentity {
    fn read(file: &cap_std::fs::File) -> Result<Self, WriterLockAcquireError> {
        file.metadata()
            .map(|metadata| Self::from(&metadata))
            .map_err(|source| {
                WriterLockAcquireError::io(WriterLockAcquirePhase::VerifyFileIdentity, source)
            })
    }
}

impl From<&Metadata> for FileIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

/// Exclusive kernel-managed writer authority over one pinned store root.
///
/// The guard retains both the opened root capability and lock-file handle.
/// Dropping it closes the handle and releases the process-scoped kernel lock;
/// it never deletes, renames, truncates, or replaces `writer.lock`.
#[must_use]
pub struct FilesystemWriterLock {
    directory: Dir,
    lock_file: File,
}

impl FilesystemWriterLock {
    /// Tries to acquire exclusive writer authority without blocking.
    ///
    /// The store root is pinned before `writer.lock` is opened relative to it.
    /// The lock entry must already exist as a regular file and is opened
    /// without following symbolic links. After nonblocking kernel acquisition,
    /// the adapter reopens the directory entry and proves that it still names
    /// the locked device and inode before returning authority.
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
        let lock_file = open_existing(&directory)?;
        Self::acquire(directory, lock_file)
    }

    pub(super) fn initialize_in(directory: Dir) -> Result<Self, WriterLockAcquireError> {
        let lock_file = open_or_create(&directory)?;
        let guard = Self::acquire(directory, lock_file)?;
        guard.lock_file.sync_all().map_err(|source| {
            WriterLockAcquireError::io(WriterLockAcquirePhase::SynchronizeFile, source)
        })?;
        Ok(guard)
    }

    fn acquire(
        directory: Dir,
        lock_file: cap_std::fs::File,
    ) -> Result<Self, WriterLockAcquireError> {
        let metadata = lock_file.metadata().map_err(|source| {
            WriterLockAcquireError::io(WriterLockAcquirePhase::InspectFile, source)
        })?;
        if !metadata.is_file() {
            return Err(WriterLockAcquireError::NotRegular);
        }
        let expected_identity = FileIdentity::from(&metadata);
        let lock_file = lock_file.into_std();
        match lock_file.try_lock() {
            Ok(()) => {
                verify_current_identity(&directory, expected_identity)?;
                Ok(Self {
                    directory,
                    lock_file,
                })
            }
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

fn verify_current_identity(
    directory: &Dir,
    expected: FileIdentity,
) -> Result<(), WriterLockAcquireError> {
    let observed_file = directory
        .open_with(LOCK_FILE_NAME, &lock_options())
        .map_err(|source| {
            WriterLockAcquireError::io(WriterLockAcquirePhase::VerifyFileIdentity, source)
        })?;
    let observed = FileIdentity::read(&observed_file)?;
    if observed == expected {
        return Ok(());
    }
    Err(WriterLockAcquireError::io(
        WriterLockAcquirePhase::VerifyFileIdentity,
        io::Error::new(
            io::ErrorKind::InvalidData,
            "writer.lock changed identity during acquisition",
        ),
    ))
}

fn open_or_create(directory: &Dir) -> Result<cap_std::fs::File, WriterLockAcquireError> {
    let mut options = lock_options();
    options.create_new(true);
    match directory.open_with(LOCK_FILE_NAME, &options) {
        Ok(file) => Ok(file),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => open_existing(directory),
        Err(source) => Err(WriterLockAcquireError::io(
            WriterLockAcquirePhase::OpenFile,
            source,
        )),
    }
}

fn open_existing(directory: &Dir) -> Result<cap_std::fs::File, WriterLockAcquireError> {
    directory
        .open_with(LOCK_FILE_NAME, &lock_options())
        .map_err(|source| WriterLockAcquireError::io(WriterLockAcquirePhase::OpenFile, source))
}

fn lock_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    options
}
