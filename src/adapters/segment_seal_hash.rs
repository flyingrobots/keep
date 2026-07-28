//! Canonical physical segment digest and seal checksum calculation.

use blake3::Hasher;

use super::{SegmentDigest, SegmentSealError};

const VERSION: u16 = 1;
const ALGORITHM: u8 = 1;
const DIGEST_DOMAIN: &[u8] = b"KEEP:SEGMENT:DIGEST\0";
const CHECKSUM_DOMAIN: &[u8] = b"KEEP:SEGMENT:SEAL:SUM\0";

pub(super) fn segment_digest(
    prefix: &[u8],
    seal: &[u8],
) -> Result<SegmentDigest, SegmentSealError> {
    let q = seal.get(..64).ok_or(SegmentSealError::WrongLength {
        expected: 128,
        observed: seal.len(),
    })?;
    let input_length = prefix
        .len()
        .checked_add(q.len())
        .and_then(|length| u64::try_from(length).ok())
        .ok_or(SegmentSealError::DigestLengthArithmetic {
            prefix_length: prefix.len(),
        })?;
    Ok(SegmentDigest::from_validated(hash_parts(
        DIGEST_DOMAIN,
        &[prefix, q],
        input_length,
    )))
}

pub(super) fn seal_checksum(seal: &[u8]) -> Result<[u8; 32], SegmentSealError> {
    let covered = seal.get(..96).ok_or(SegmentSealError::WrongLength {
        expected: 128,
        observed: seal.len(),
    })?;
    Ok(hash_parts(CHECKSUM_DOMAIN, &[covered], 96))
}

fn hash_parts(domain: &[u8], parts: &[&[u8]], length: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(domain);
    hasher.update(&VERSION.to_be_bytes());
    hasher.update(&[ALGORITHM]);
    for part in parts {
        hasher.update(part);
    }
    hasher.update(&length.to_be_bytes());
    *hasher.finalize().as_bytes()
}
