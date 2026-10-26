//! This boundary module owns GC retirement intent framing constants and
//! integrity domains.

pub(super) const HEADER_LENGTH: usize = 320;
pub(super) const CANDIDATE_WIDTH: usize = 72;
pub(super) const TRAILER_LENGTH: usize = 64;
pub(super) const CANDIDATE_SET_DIGEST_OFFSET: usize = 288;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:GC:INTENT2\0";
pub(super) const VERSION: u16 = 2;
pub(super) const RECORD_HEADER_LENGTH: u16 = 320;
pub(super) const RECORD_CANDIDATE_WIDTH: u16 = 72;
const CANDIDATE_SET_DOMAIN: &[u8] = b"keep.gc-candidate-set/v2\0";
const INTENT_DOMAIN: &[u8] = b"keep.gc-retirement-intent/v2\0";
const CHECKSUM_DOMAIN: &[u8] = b"keep.gc-retirement-intent-checksum/v2\0";

/// Canonical encoded length for `candidate_count` candidates, or `None` on
/// arithmetic overflow.
pub(super) fn canonical_length(candidate_count: u32) -> Option<usize> {
    usize::try_from(candidate_count)
        .ok()?
        .checked_mul(CANDIDATE_WIDTH)?
        .checked_add(HEADER_LENGTH)?
        .checked_add(TRAILER_LENGTH)
}

pub(super) fn candidate_set_digest(candidate_count: u32, candidates: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CANDIDATE_SET_DOMAIN);
    hasher.update(&candidate_count.to_be_bytes());
    hasher.update(candidates);
    *hasher.finalize().as_bytes()
}

pub(super) fn intent_digest(preimage: &[u8]) -> [u8; 32] {
    hash(INTENT_DOMAIN, preimage)
}

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    hash(CHECKSUM_DOMAIN, preimage)
}

fn hash(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}
