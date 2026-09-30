//! This boundary module owns the fixed layout of one verification receipt.

pub(super) const ENCODED_LENGTH: usize = 384;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:VERIFY:RCPT";
pub(super) const VERSION: u16 = 1;
pub(super) const RECORD_LENGTH: u16 = 384;
pub(super) const CHECKSUM_OFFSET: usize = 352;
pub(super) const RESERVED_OFFSET: usize = 324;
pub(super) const RESERVED_LENGTH: usize = 28;
pub(super) const IDENTITY_SLOT: usize = 60;
pub(super) const SUBJECT_OFFSET: usize = 48;
pub(super) const LAYOUT_OFFSET: usize = 108;
pub(super) const TARGET_OFFSET: usize = 168;
pub(super) const CATALOG_GENERATION_OFFSET: usize = 228;
pub(super) const CATALOG_DIGEST_OFFSET: usize = 236;
pub(super) const LIVENESS_GENERATION_OFFSET: usize = 268;
pub(super) const MANIFEST_DIGEST_OFFSET: usize = 276;
pub(super) const EVIDENCE_INDEX_OFFSET: usize = 308;
pub(super) const CHUNKS_VERIFIED_OFFSET: usize = 316;
const CHECKSUM_DOMAIN: &[u8] = b"keep.verification-receipt-checksum/v1\0";

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}
