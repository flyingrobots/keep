//! This boundary module owns the blocking storage capability port GC
//! execution drives, one capability per phase.

use std::io;

/// Durable capabilities GC execution calls in [`GcExecutionPhase::ALL`] order.
///
/// Each owns its complete effect and the synchronization that makes it
/// durable; the intent, receipt, and candidate list belong to the
/// implementation's context. Every method returns the exact filesystem
/// failure or the implementation's typed refusal as an [`io::Error`].
///
/// [`GcExecutionPhase::ALL`]: super::GcExecutionPhase::ALL
#[expect(
    clippy::missing_errors_doc,
    reason = "every capability returns the exact filesystem failure or refusal"
)]
pub trait GcExecutionStorage {
    /// Exclusively creates `gc/intent.next` with the complete canonical intent.
    fn write_intent_stage(&mut self) -> io::Result<()>;
    /// Synchronizes the intent stage and reverifies its bytes.
    fn synchronize_intent_stage(&mut self) -> io::Result<()>;
    /// Links the intent stage to `gc/intent` without replacement.
    fn link_intent(&mut self) -> io::Result<()>;
    /// Synchronizes `gc` after the intent link.
    fn synchronize_gc_after_intent(&mut self) -> io::Result<()>;
    /// Removes the retained intent stage after proving its link.
    fn remove_intent_stage(&mut self) -> io::Result<()>;
    /// Synchronizes `gc` after intent-stage cleanup.
    fn synchronize_gc_after_intent_cleanup(&mut self) -> io::Result<()>;
    /// Reopens candidate `index` without following links, verifies its exact
    /// identity and length against the intent, and unlinks only that entry.
    fn unlink_candidate(&mut self, index: usize) -> io::Result<()>;
    /// Synchronizes `segments` after candidate `index` was unlinked.
    fn synchronize_segment_pool(&mut self, index: usize) -> io::Result<()>;
    /// Exclusively creates `gc/receipt.next` with the complete canonical
    /// receipt after proving every candidate absent from the pool.
    fn write_receipt_stage(&mut self) -> io::Result<()>;
    /// Synchronizes the receipt stage and reverifies its bytes.
    fn synchronize_receipt_stage(&mut self) -> io::Result<()>;
    /// Renames the receipt stage onto `gc/receipt`, replacing any prior
    /// retirement's receipt atomically.
    fn replace_receipt(&mut self) -> io::Result<()>;
    /// Synchronizes `gc` after the receipt replacement.
    fn synchronize_gc_after_receipt(&mut self) -> io::Result<()>;
    /// Removes the completed `gc/intent` after proving the receipt completes it.
    fn remove_intent(&mut self) -> io::Result<()>;
    /// Synchronizes `gc` after the intent removal.
    fn synchronize_gc_after_intent_removal(&mut self) -> io::Result<()>;
}
