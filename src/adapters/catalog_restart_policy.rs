//! Explicit restart-loading resource policy.

use super::{CatalogRestartByteLimit, SegmentReadPolicy};

/// Bounds for one published catalog restart load.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogRestartPolicy {
    segment_read: SegmentReadPolicy,
    retained_segment_bytes: CatalogRestartByteLimit,
}

impl CatalogRestartPolicy {
    /// Creates a policy from segment grammar and aggregate retention bounds.
    pub const fn new(
        segment_read: SegmentReadPolicy,
        retained_segment_bytes: CatalogRestartByteLimit,
    ) -> Self {
        Self {
            segment_read,
            retained_segment_bytes,
        }
    }

    pub(super) const fn segment_read(self) -> SegmentReadPolicy {
        self.segment_read
    }

    pub(super) const fn retained_segment_bytes(self) -> CatalogRestartByteLimit {
        self.retained_segment_bytes
    }
}
