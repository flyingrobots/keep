//! This boundary module owns shared migration-receipt framing and integrity.

pub(super) const CHECKSUM_OFFSET: usize = 224;
pub(super) const ENCODED_LENGTH: usize = 256;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:MIG:REC2\0\0\0";
pub(super) const RECORD_LENGTH: u16 = 256;
pub(super) const VERSION: u16 = 2;
const CHECKSUM_DOMAIN: &[u8] = b"keep.store-migration-receipt-checksum/v2\0";

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}
