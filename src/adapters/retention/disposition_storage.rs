//! This module owns the blocking storage capability port for one disposition.

use std::io;

/// Durable capabilities a disposition executes, one per phase, in order.
///
/// Each capability owns its complete effect and the synchronization that
/// makes it durable. The receipt bytes and the retained stage belong to the
/// implementation's context; the executor only sequences phases.
pub trait RecoveryDispositionStorage {
    /// Exclusively creates `recovery/disposition.next` with the complete
    /// canonical receipt.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn write_disposition_stage(&mut self) -> io::Result<()>;

    /// Synchronizes the stage and reverifies its bytes.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn synchronize_disposition_stage(&mut self) -> io::Result<()>;

    /// Links the stage into `recovery/dispositions` under the artifact's
    /// canonical name without replacement.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a conflicting-entry refusal.
    fn link_disposition_receipt(&mut self) -> io::Result<()>;

    /// Synchronizes `recovery/dispositions`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn synchronize_dispositions(&mut self) -> io::Result<()>;

    /// Removes the retained stage after proving the linked receipt.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the link is
    /// not proven.
    fn remove_disposition_stage(&mut self) -> io::Result<()>;

    /// Synchronizes `recovery`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn synchronize_recovery(&mut self) -> io::Result<()>;

    /// Removes the disposed retention stage after proving its pool link.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the link is
    /// not proven.
    fn remove_retained_stage(&mut self) -> io::Result<()>;

    /// Synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn synchronize_retention_after_disposition(&mut self) -> io::Result<()>;

    /// Unlinks the retired artifact from its immutable pool after proving
    /// its exact bytes, and removes its namespace directory when that leaves
    /// it empty (`Retire` only).
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the entry's
    /// bytes are not the disposed artifact's.
    fn remove_pool_entry(&mut self) -> io::Result<()>;

    /// Synchronizes the pool the artifact left (`Retire` only).
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn synchronize_pool(&mut self) -> io::Result<()>;
}
