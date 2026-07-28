//! Canonical version-1 segment header codec.

use super::{SegmentHeaderError, segment_header_decoder, segment_header_encoding};

pub(super) const MAGIC: [u8; 16] = *b"KEEP:SEGMENT:V1\0";
pub(super) const FORMAT_VERSION: u16 = 1;
pub(super) const FLAGS: u16 = 0;
pub(super) const HEADER_LENGTH: u16 = 64;
pub(super) const RECORD_HEADER_LENGTH: u16 = 112;
pub(super) const SEAL_LENGTH: u16 = 128;
pub(super) const MAXIMUM_RECORD_PAYLOAD_LENGTH: u64 = 67_108_864;
pub(super) const MAXIMUM_SEGMENT_LENGTH: u64 = 1_073_741_824;
pub(super) const MAXIMUM_RECORD_COUNT: u32 = 1_048_576;
pub(super) const BLAKE3_256: u8 = 1;
pub(super) const ENCODED_LENGTH: usize = 64;

/// The exact admitted header for `keep.segment-store/v1`.
///
/// Version 1 has one canonical header. Decoding performs no allocation or I/O
/// and admits only the exact immutable format coordinates and bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentHeader(());

impl SegmentHeader {
    /// Exact encoded header length.
    pub const ENCODED_LENGTH: usize = ENCODED_LENGTH;

    /// Decodes and admits one exact version-1 segment header.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentHeaderError`] at the first failed fixed-width field in
    /// byte order. Length is checked before any field is read.
    pub fn decode(encoded: &[u8]) -> Result<Self, SegmentHeaderError> {
        segment_header_decoder::decode(encoded)
    }

    /// Encodes the exact canonical version-1 header.
    #[must_use]
    pub const fn encode(self) -> [u8; Self::ENCODED_LENGTH] {
        segment_header_encoding::canonical_bytes()
    }

    /// Returns the immutable maximum record-payload length.
    #[must_use]
    pub const fn maximum_record_payload_length(self) -> u64 {
        MAXIMUM_RECORD_PAYLOAD_LENGTH
    }

    /// Returns the immutable maximum complete segment length.
    #[must_use]
    pub const fn maximum_segment_length(self) -> u64 {
        MAXIMUM_SEGMENT_LENGTH
    }

    /// Returns the immutable maximum record count.
    #[must_use]
    pub const fn maximum_record_count(self) -> u32 {
        MAXIMUM_RECORD_COUNT
    }

    pub(super) const fn admitted() -> Self {
        Self(())
    }
}
