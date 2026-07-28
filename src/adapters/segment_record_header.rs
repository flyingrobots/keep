//! Canonical version-1 segment-record header codec.

use super::{
    SegmentRecordHeaderError, SegmentRecordIdentity, SegmentRecordLength,
    SegmentRecordPayloadLength, segment_record_header_admission, segment_record_header_decoder,
    segment_record_header_encoding,
};
use crate::{ChunkId, LayoutId};

pub(super) const MAGIC: [u8; 16] = *b"KEEP:SEG:RECORD\0";
pub(super) const RECORD_VERSION: u16 = 1;
pub(super) const FLAGS: u8 = 0;
pub(super) const HEADER_LENGTH: u16 = 112;
pub(super) const CHECKSUM_LENGTH: u64 = 32;
pub(super) const CHECKSUM_ALGORITHM: u8 = 1;
pub(super) const IDENTITY_VERSION: u16 = 1;
pub(super) const IDENTITY_ALGORITHM: u8 = 1;
pub(super) const ENCODED_LENGTH: usize = 112;

/// An admitted canonical header for one `keep.segment-store/v1` record.
///
/// Admission validates exact framing and converts the kind-specific identity
/// slot into a typed logical identity. It performs no allocation or I/O and
/// makes no claim about payload checksum or content identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentRecordHeader {
    identity: SegmentRecordIdentity,
    payload_length: SegmentRecordPayloadLength,
    record_length: SegmentRecordLength,
}

impl SegmentRecordHeader {
    /// Exact encoded record-header length.
    pub const ENCODED_LENGTH: usize = ENCODED_LENGTH;

    /// Decodes and admits one exact version-1 record header.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordHeaderError`] at the first failed field in byte
    /// order. Exact width is checked before any field is read.
    pub fn decode(encoded: &[u8]) -> Result<Self, SegmentRecordHeaderError> {
        segment_record_header_decoder::decode(encoded)
    }

    /// Constructs the canonical record header for a chunk identity.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordHeaderError`] when the chunk length exceeds the
    /// segment protocol bound or checked record-length arithmetic fails.
    pub fn for_chunk(identity: ChunkId) -> Result<Self, SegmentRecordHeaderError> {
        segment_record_header_admission::from_identity(SegmentRecordIdentity::Chunk(identity))
    }

    /// Constructs the canonical record header for a flat-layout identity.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordHeaderError`] if checked record-length
    /// arithmetic fails.
    pub fn for_layout(identity: LayoutId) -> Result<Self, SegmentRecordHeaderError> {
        segment_record_header_admission::from_identity(SegmentRecordIdentity::Layout(identity))
    }

    /// Encodes this header into its exact canonical bytes.
    #[must_use]
    pub const fn encode(self) -> [u8; Self::ENCODED_LENGTH] {
        segment_record_header_encoding::encode(self)
    }

    /// Returns the admitted logical identity and record kind.
    #[must_use]
    pub const fn identity(self) -> SegmentRecordIdentity {
        self.identity
    }

    /// Returns the exact payload byte count.
    #[must_use]
    pub const fn payload_length(self) -> SegmentRecordPayloadLength {
        self.payload_length
    }

    /// Returns the exact complete record byte count.
    #[must_use]
    pub const fn record_length(self) -> SegmentRecordLength {
        self.record_length
    }

    pub(super) const fn admitted(
        identity: SegmentRecordIdentity,
        payload_length: SegmentRecordPayloadLength,
        record_length: SegmentRecordLength,
    ) -> Self {
        Self {
            identity,
            payload_length,
            record_length,
        }
    }
}

#[cfg(test)]
#[path = "segment_record_header_tests.rs"]
mod tests;
