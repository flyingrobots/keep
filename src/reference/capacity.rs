//! Materialized chunk-byte capacity for the reference adapter.

/// Maximum exact chunk bytes the non-durable reference store may own.
///
/// Layout metadata is bounded separately by [`crate::LayoutEntryLimit`].
/// A zero-byte capacity can still publish the empty blob.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReferenceStoreCapacity(usize);

impl ReferenceStoreCapacity {
    /// Constructs an explicit host-memory byte capacity.
    #[must_use]
    pub const fn new(bytes: usize) -> Self {
        Self(bytes)
    }

    /// Returns the maximum materialized chunk bytes.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}
