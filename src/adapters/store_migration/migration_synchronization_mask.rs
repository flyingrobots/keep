//! This module owns completed migration synchronization evidence.

/// Closed set of mandatory migration synchronization transitions.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MigrationSynchronizationMask(u64);

impl MigrationSynchronizationMask {
    pub(super) const COMPLETE_BITS: u64 = 0x03ff;

    /// Returns the complete synchronization bit set.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    pub(super) const fn complete() -> Self {
        Self(Self::COMPLETE_BITS)
    }
}
