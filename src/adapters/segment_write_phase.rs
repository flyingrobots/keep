//! Typed segment-stage write and durability boundaries.

/// Exact byte class being written to a segment stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentWritePhase {
    /// Fixed 64-byte segment header.
    Header,
    /// Fixed 112-byte record header.
    RecordHeader,
    /// Exact borrowed record payload.
    RecordPayload,
    /// Fixed 32-byte record checksum.
    RecordChecksum,
    /// Fixed 128-byte segment seal.
    Seal,
}

/// Exact segment-stage durability boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentDurabilityPhase {
    /// Complete reusable header-and-record prefix.
    RecordPrefix,
    /// Complete segment including its terminal seal.
    SealedSegment,
}
