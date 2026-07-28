//! Typed segment-record checksum and canonical calculation.

use blake3::Hasher;

use super::SegmentRecordHeader;

const DOMAIN: &[u8] = b"KEEP:SEG:RECORD:SUM\0";
const FRAMING_VERSION: u16 = 1;
const ALGORITHM: u8 = 1;

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
        let mut hasher = Hasher::new();
        hasher.update(DOMAIN);
        hasher.update(&FRAMING_VERSION.to_be_bytes());
        hasher.update(&[ALGORITHM]);
        hasher.update(&header.encode());
        hasher.update(payload);
        hasher.update(&covered_length.to_be_bytes());
        Self(*hasher.finalize().as_bytes())
    }

    pub(super) const fn from_validated(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
