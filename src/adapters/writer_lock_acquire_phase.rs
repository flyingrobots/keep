//! Persistent writer-lock acquisition phases.

use std::fmt;

/// Exact filesystem operation attempted while acquiring writer authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriterLockAcquirePhase {
    /// Pin the caller-selected store root.
    OpenRoot,
    /// Open `writer.lock` relative to the pinned root without following links.
    OpenFile,
    /// Verify that the opened lock handle names a regular file.
    InspectFile,
    /// Acquire the nonblocking exclusive kernel lock.
    Acquire,
    /// Verify that the locked handle still names the admitted directory entry.
    VerifyFileIdentity,
    /// Synchronize an initialization-created writer file before admission.
    SynchronizeFile,
}

impl fmt::Display for WriterLockAcquirePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OpenRoot => "root open",
            Self::OpenFile => "file open",
            Self::InspectFile => "file inspection",
            Self::Acquire => "kernel acquisition",
            Self::VerifyFileIdentity => "file identity verification",
            Self::SynchronizeFile => "file synchronization",
        })
    }
}
