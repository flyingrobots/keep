//! This module owns immutable-pool admission outcomes during recovery.

/// Whether recovery linked an exact artifact or admitted an existing one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStagePoolOutcome {
    /// The exact stage was linked into its immutable pool.
    Linked,
    /// The immutable pool coordinate already existed and required verification.
    AlreadyPresent,
}
