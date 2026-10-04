//! This boundary module owns GC retirement receipt framing and integrity.

pub(super) const CHECKSUM_OFFSET: usize = 288;
pub(super) const ENCODED_LENGTH: usize = 320;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:GC:RECEIPT2";
pub(super) const RECORD_LENGTH: u16 = 320;
pub(super) const VERSION: u16 = 2;
pub(super) const RESERVED_OFFSET: usize = 240;
pub(super) const RESERVED_LENGTH: usize = 48;
// The fixed grammar records the same byte length as its owned representation.
const _: () = assert!(RECORD_LENGTH == 320 && ENCODED_LENGTH == 320);
const CHECKSUM_DOMAIN: &[u8] = b"keep.gc-retirement-receipt-checksum/v2\0";

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}
