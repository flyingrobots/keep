//! This module owns observed resource use for one verified retention closure.

/// Exact successful resource accounting for one complete retained root.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionClosureUsage {
    node_count: u64,
    maximum_depth: u16,
    encoded_bytes: u64,
    physical_bytes: u64,
}

impl RetentionClosureUsage {
    /// Returns the unique first-scheduled record count.
    #[must_use]
    pub const fn node_count(self) -> u64 {
        self.node_count
    }

    /// Returns the maximum catalog-record edge depth reached.
    #[must_use]
    pub const fn maximum_depth(self) -> u16 {
        self.maximum_depth
    }

    /// Returns the unique structured layout payload bytes decoded.
    #[must_use]
    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }

    /// Returns complete record bytes charged to reconstruction work.
    #[must_use]
    pub const fn physical_bytes(self) -> u64 {
        self.physical_bytes
    }

    pub(crate) const fn from_verified(
        node_count: u64,
        maximum_depth: u16,
        encoded_bytes: u64,
        physical_bytes: u64,
    ) -> Self {
        Self {
            node_count,
            maximum_depth,
            encoded_bytes,
            physical_bytes,
        }
    }
}
