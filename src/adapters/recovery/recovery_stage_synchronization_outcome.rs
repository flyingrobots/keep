//! This module owns staged-file synchronization outcomes during recovery.

/// Whether complete-stage recovery synchronized a stage or found it absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageSynchronizationOutcome {
    /// The exact present stage was verified and synchronized.
    Synchronized,
    /// The fixed stage was absent and recovery continued from the pool.
    AlreadyAbsent,
}
