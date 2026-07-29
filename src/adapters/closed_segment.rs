//! This module owns proof that a synchronized segment stage is closed.

use super::SegmentDigest;

/// Metadata retained after the sealed stage's owned writable handle is closed.
///
/// Values can be created only by consuming [`crate::SealedSegment`]. The
/// receipt carries no file handle and cannot mutate or publish the stage.
#[must_use]
#[derive(Debug)]
pub struct ClosedSegment {
    record_count: u32,
    segment_length: u64,
    digest: SegmentDigest,
}

impl ClosedSegment {
    /// Returns the exact sealed record count.
    #[must_use]
    pub const fn record_count(&self) -> u32 {
        self.record_count
    }

    /// Returns the exact complete segment byte count.
    #[must_use]
    pub const fn segment_length(&self) -> u64 {
        self.segment_length
    }

    /// Returns the physical immutable-segment digest.
    #[must_use]
    pub const fn digest(&self) -> SegmentDigest {
        self.digest
    }

    pub(super) const fn admitted(
        record_count: u32,
        segment_length: u64,
        digest: SegmentDigest,
    ) -> Self {
        Self {
            record_count,
            segment_length,
            digest,
        }
    }
}
