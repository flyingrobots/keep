//! Blocking durability capability for one exclusively owned segment stage.

use std::io::{self, Write};

/// Blocking write and synchronization capability for one segment stage.
///
/// Before admission by [`crate::StagedSegment::begin`], an implementation must
/// refer to one exclusively owned, empty staging object positioned at byte
/// zero. It must retain that object for the staged state's lifetime.
/// `synchronize` must not report success until all preceding writes to that
/// object satisfy the implementation's file-durability contract.
///
/// A raw [`std::fs::File`] deliberately does not implement this trait because
/// its ownership, length, and current offset do not prove those preconditions.
pub trait SegmentStage: Write {
    /// Synchronizes all preceding stage writes.
    ///
    /// # Errors
    ///
    /// Returns the underlying durability failure without weakening it to a
    /// flush-only claim.
    fn synchronize(&mut self) -> io::Result<()>;
}
