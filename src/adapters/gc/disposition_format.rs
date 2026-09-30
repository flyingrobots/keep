//! This boundary module owns recovery-disposition receipt framing and
//! integrity.

pub(super) const CHECKSUM_OFFSET: usize = 288;
pub(super) const ENCODED_LENGTH: usize = 320;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:REC:DISP2\0\0";
pub(super) const RECORD_LENGTH: u16 = 320;
pub(super) const VERSION: u16 = 2;
pub(super) const HEADER_RESERVED_OFFSET: usize = 30;
pub(super) const TRAILER_RESERVED_OFFSET: usize = 280;
pub(super) const TRAILER_RESERVED_LENGTH: usize = 8;
const CHECKSUM_DOMAIN: &[u8] = b"keep.recovery-disposition-receipt-checksum/v2\0";
const ARTIFACT_DOMAIN: &[u8] = b"keep.recovery-disposition-artifact/v2\0";

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}

/// Digests the exact bytes of one disposed artifact under the registered
/// artifact domain.
pub(super) fn artifact_content_digest(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ARTIFACT_DOMAIN);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}
