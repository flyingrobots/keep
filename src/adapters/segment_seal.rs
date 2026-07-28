//! Canonical version-1 segment seal codec.

use super::{SegmentDigest, SegmentSealError, segment_seal_decoder, segment_seal_encoding};

pub(super) const MAGIC: [u8; 16] = *b"KEEP:SEGMENT:END";
pub(super) const VERSION: u16 = 1;
pub(super) const FLAGS: u16 = 0;
pub(super) const SEAL_LENGTH: u16 = 128;
pub(super) const ALGORITHM: u8 = 1;
pub(super) const ENCODED_LENGTH: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SegmentSealCoordinates {
    record_count: u32,
    bytes_before_seal: u64,
    segment_length: u64,
    record_bytes: u64,
}

/// An admitted canonical `keep.segment-store/v1` seal.
///
/// Decoding binds the supplied exact pre-seal bytes to the seal's physical
/// digest and checksum. It does not independently parse or admit the segment
/// header and records contained in that prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentSeal {
    coordinates: SegmentSealCoordinates,
    digest: SegmentDigest,
    checksum: [u8; 32],
}

impl SegmentSeal {
    /// Exact encoded seal length.
    pub const ENCODED_LENGTH: usize = ENCODED_LENGTH;

    /// Decodes and verifies one exact seal against its pre-seal bytes.
    ///
    /// This operation performs no allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentSealError`] at the first failed structural, length,
    /// algorithm, physical-digest, or seal-checksum law.
    pub fn decode(prefix: &[u8], encoded: &[u8]) -> Result<Self, SegmentSealError> {
        segment_seal_decoder::decode(prefix, encoded)
    }

    /// Encodes this admitted seal into its exact canonical bytes.
    #[must_use]
    pub const fn encode(self) -> [u8; Self::ENCODED_LENGTH] {
        segment_seal_encoding::encode(self)
    }

    /// Returns the exact number of complete records.
    #[must_use]
    pub const fn record_count(self) -> u32 {
        self.coordinates.record_count
    }

    /// Returns the exact header-plus-record byte count.
    #[must_use]
    pub const fn bytes_before_seal(self) -> u64 {
        self.coordinates.bytes_before_seal
    }

    /// Returns the exact complete segment byte count.
    #[must_use]
    pub const fn segment_length(self) -> u64 {
        self.coordinates.segment_length
    }

    /// Returns the exact concatenated complete-record byte count.
    #[must_use]
    pub const fn record_bytes(self) -> u64 {
        self.coordinates.record_bytes
    }

    /// Returns the physical immutable-segment digest.
    #[must_use]
    pub const fn digest(self) -> SegmentDigest {
        self.digest
    }

    pub(super) const fn admitted(
        coordinates: SegmentSealCoordinates,
        digest: SegmentDigest,
        checksum: [u8; 32],
    ) -> Self {
        Self {
            coordinates,
            digest,
            checksum,
        }
    }

    pub(super) const fn checksum(self) -> [u8; 32] {
        self.checksum
    }
}

impl SegmentSealCoordinates {
    pub(super) const fn new(
        record_count: u32,
        bytes_before_seal: u64,
        segment_length: u64,
        record_bytes: u64,
    ) -> Self {
        Self {
            record_count,
            bytes_before_seal,
            segment_length,
            record_bytes,
        }
    }
}
