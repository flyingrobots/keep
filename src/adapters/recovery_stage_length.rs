//! This module owns validated recovery-stage byte lengths.

/// Exact bounded byte length of one observed fixed recovery stage.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageLength(u64);

impl RecoveryStageLength {
    pub(super) const fn from_validated(value: u64) -> Self {
        Self(value)
    }

    /// Returns the exact byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
