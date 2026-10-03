//! This module owns repository fault-observation hooks without stage authority.

use std::io;

/// An observation boundary around a real stage durability operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentStageDurabilityEvent {
    /// Before flushing buffered bytes.
    BeforeFlush,
    /// After a successful flush.
    AfterFlush,
    /// Before synchronizing persisted bytes.
    BeforeSynchronize,
    /// After a successful synchronization.
    AfterSynchronize,
}

/// Repository fault hooks that can interrupt operations but cannot access storage.
///
/// The wrapper always performs actual writes, flushes and synchronization itself.
/// Observers can block or fail at a boundary, or limit a write to a strict prefix.
/// No hook receives a writable stage or can authorize a sealed receipt.
pub trait SegmentStageObserver {
    /// Selects how many of the requested bytes may reach the next actual write.
    ///
    /// # Errors
    /// Returns the injected pre-write failure. Limits beyond `requested` are refused.
    fn before_write(&mut self, requested: usize) -> io::Result<usize>;

    /// Observes the actual successful write count before the caller continues.
    ///
    /// # Errors
    /// Returns an injected failure after the write; this does not undo its effects.
    fn after_write(&mut self, written: usize) -> io::Result<()>;

    /// Observes a boundary before or after an actual durability operation.
    ///
    /// # Errors
    /// Returns the injected failure at this boundary, without rolling back effects.
    fn durability(&mut self, event: SegmentStageDurabilityEvent) -> io::Result<()>;
}
