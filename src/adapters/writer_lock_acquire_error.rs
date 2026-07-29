//! Persistent writer-lock acquisition failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::WriterLockAcquirePhase;

/// Failure to acquire exclusive writer authority over one store root.
#[derive(Debug)]
pub enum WriterLockAcquireError {
    /// Another handle or process currently owns the kernel lock.
    Busy,
    /// `writer.lock` was opened but was not a regular file.
    NotRegular,
    /// One acquisition phase failed at the filesystem boundary.
    Io {
        /// Exact operation that failed.
        phase: WriterLockAcquirePhase,
        /// Preserved filesystem source.
        source: io::Error,
    },
}

impl WriterLockAcquireError {
    pub(super) const fn io(phase: WriterLockAcquirePhase, source: io::Error) -> Self {
        Self::Io { phase, source }
    }
}

impl fmt::Display for WriterLockAcquireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => formatter.write_str("writer lock is held by another writer"),
            Self::NotRegular => formatter.write_str("writer lock entry is not a regular file"),
            Self::Io { phase, .. } => write!(formatter, "writer lock {phase} failed"),
        }
    }
}

impl Error for WriterLockAcquireError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Busy | Self::NotRegular => None,
        }
    }
}
