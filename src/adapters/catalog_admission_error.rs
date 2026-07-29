//! Catalog-to-segment admission failures.

use std::collections::TryReserveError;

use super::{
    CatalogAllocationPhase, CatalogDecodeError, SegmentDigest, SegmentReadError,
    SegmentRecordChecksum, SegmentRecordIdentity,
};

/// Failure to bind a checksummed catalog to exact admitted segment records.
#[derive(Debug)]
pub enum CatalogAdmissionError {
    /// Revalidating an immutable catalog entry failed.
    Catalog {
        /// Exact nested catalog failure.
        source: CatalogDecodeError,
    },
    /// The bounded catalog count cannot fit the host allocation width.
    EntryCountHostWidth {
        /// Verified catalog entry count.
        observed: u64,
    },
    /// Caller input supplied more segments than the catalog could reference.
    SegmentCountOutOfBounds {
        /// Largest useful segment count.
        maximum: u64,
        /// Caller-supplied segment count.
        observed: usize,
    },
    /// A bounded segment-index or record-binding allocation failed.
    Allocation {
        /// Semantic allocation phase.
        phase: CatalogAllocationPhase,
        /// Exact requested element capacity.
        requested: usize,
        /// Allocator refusal.
        source: TryReserveError,
    },
    /// Caller input repeated one physical segment digest.
    DuplicateSegment {
        /// Repeated physical segment coordinate.
        digest: SegmentDigest,
    },
    /// The catalog named a segment absent from caller input.
    MissingSegment {
        /// Required physical segment coordinate.
        digest: SegmentDigest,
    },
    /// Revalidating an admitted segment's immutable records failed.
    Segment {
        /// Physical segment being scanned.
        digest: SegmentDigest,
        /// Exact nested segment failure.
        source: Box<SegmentReadError>,
    },
    /// A physical coordinate was not one complete top-level record span.
    LocationNotTopLevel {
        /// Logical catalog key.
        identity: SegmentRecordIdentity,
        /// Physical segment coordinate.
        segment_digest: SegmentDigest,
        /// Declared absolute record offset.
        record_offset: u64,
        /// Declared complete-record length.
        record_length: u64,
    },
    /// The selected record carried a different logical identity.
    RecordIdentityMismatch {
        /// Identity declared by the catalog.
        expected: SegmentRecordIdentity,
        /// Identity verified from the selected record.
        observed: SegmentRecordIdentity,
    },
    /// The selected record carried a different checksum.
    RecordChecksumMismatch {
        /// Checksum declared by the catalog.
        expected: SegmentRecordChecksum,
        /// Checksum verified from the selected record.
        observed: SegmentRecordChecksum,
    },
}
