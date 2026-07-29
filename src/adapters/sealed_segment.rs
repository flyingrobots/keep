//! Explicitly flushed and synchronized immutable segment stage.

use super::{ClosedSegment, SegmentDigest, SegmentStage};

/// A complete segment stage whose record prefix and sealed bytes were each
/// flushed and synchronized in protocol order.
///
/// The stage remains unpublished. This type exposes no mutable stage handle,
/// makes no directory-durability claim, and is not a catalog reference.
#[must_use]
pub struct SealedSegment<S>
where
    S: SegmentStage,
{
    _stage: S,
    record_count: u32,
    segment_length: u64,
    digest: SegmentDigest,
}

impl<S> SealedSegment<S>
where
    S: SegmentStage,
{
    /// Closes the owned writable stage and returns storage-agnostic metadata.
    ///
    /// The stage has already completed both required flush-and-sync
    /// transitions. This consuming operation drops the only stage value Keep
    /// owns before returning a handle-free [`ClosedSegment`] receipt. A
    /// filesystem publisher requires its own authority-bound selection method;
    /// this generic receipt cannot authorize a retained filesystem stage.
    pub fn close(self) -> ClosedSegment {
        let (stage, record_count, segment_length, digest) = self.into_parts();
        drop(stage);
        ClosedSegment::admitted(record_count, segment_length, digest)
    }

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

    pub(super) fn into_parts(self) -> (S, u32, u64, SegmentDigest) {
        let Self {
            _stage: stage,
            record_count,
            segment_length,
            digest,
        } = self;
        (stage, record_count, segment_length, digest)
    }

    pub(super) const fn admitted(
        stage: S,
        record_count: u32,
        segment_length: u64,
        digest: SegmentDigest,
    ) -> Self {
        Self {
            _stage: stage,
            record_count,
            segment_length,
            digest,
        }
    }
}
