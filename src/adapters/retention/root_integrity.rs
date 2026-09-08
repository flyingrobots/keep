//! This boundary module owns retention root digest and checksum verification.

use super::RetentionRootDecodeError;
use crate::RetentionAnchorSetDigest;

pub(super) fn verify(
    encoded: &[u8],
    digest_offset: usize,
    checksum_offset: usize,
) -> Result<[u8; 32], RetentionRootDecodeError> {
    let observed_checksum = read_digest(encoded, checksum_offset)?;
    let checksum_preimage =
        encoded
            .get(..checksum_offset)
            .ok_or(RetentionRootDecodeError::Truncated {
                expected: checksum_offset,
                observed: encoded.len(),
            })?;
    let expected_checksum = hash(b"keep.retention-root-checksum/v2\0", checksum_preimage);
    if observed_checksum != expected_checksum {
        return Err(RetentionRootDecodeError::ChecksumMismatch {
            expected: expected_checksum,
            observed: observed_checksum,
        });
    }

    let observed_digest = read_digest(encoded, digest_offset)?;
    let digest_preimage =
        encoded
            .get(..digest_offset)
            .ok_or(RetentionRootDecodeError::Truncated {
                expected: digest_offset,
                observed: encoded.len(),
            })?;
    let expected_digest = hash(b"keep.retention-root/v2\0", digest_preimage);
    if observed_digest != expected_digest {
        return Err(RetentionRootDecodeError::RootDigestMismatch {
            expected: expected_digest,
            observed: observed_digest,
        });
    }
    Ok(expected_digest)
}

pub(super) fn verify_anchor_set(
    anchor_count: u32,
    anchors: &[u8],
    observed: [u8; 32],
) -> Result<RetentionAnchorSetDigest, RetentionRootDecodeError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-anchor-set/v2\0");
    hasher.update(&anchor_count.to_be_bytes());
    hasher.update(anchors);
    let expected = *hasher.finalize().as_bytes();
    if observed == expected {
        Ok(RetentionAnchorSetDigest::from_verified(expected))
    } else {
        Err(RetentionRootDecodeError::AnchorSetDigestMismatch { expected, observed })
    }
}

fn read_digest(encoded: &[u8], offset: usize) -> Result<[u8; 32], RetentionRootDecodeError> {
    let end = offset
        .checked_add(32)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionRootDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    <[u8; 32]>::try_from(bytes).map_err(|_| RetentionRootDecodeError::Truncated {
        expected: end,
        observed: encoded.len(),
    })
}

fn hash(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}
