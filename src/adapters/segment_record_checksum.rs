//! Typed segment-record checksum and canonical calculation.

use super::{SegmentRecordHeader, framed_blake3};

const DOMAIN: &[u8] = b"KEEP:SEG:RECORD:SUM\0";

/// BLAKE3-256 checksum binding one complete segment-record header and payload.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentRecordChecksum([u8; 32]);

impl SegmentRecordChecksum {
    /// Returns the exact 32 checksum bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) fn calculate(
        header: SegmentRecordHeader,
        payload: &[u8],
        covered_length: u64,
    ) -> Self {
        Self(framed_blake3::hash(
            DOMAIN,
            &[&header.encode(), payload],
            covered_length,
        ))
    }

    pub(super) const fn from_validated(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
