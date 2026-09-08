//! This boundary module owns retention manifest integrity verification.

use super::RetentionManifestDecodeError;

pub(super) fn verify(
    encoded: &[u8],
    digest_offset: usize,
    checksum_offset: usize,
) -> Result<[u8; 32], RetentionManifestDecodeError> {
    let observed_checksum = read_digest(encoded, checksum_offset)?;
    let checksum_preimage =
        encoded
            .get(..checksum_offset)
            .ok_or(RetentionManifestDecodeError::Truncated {
                expected: checksum_offset,
                observed: encoded.len(),
            })?;
    let expected_checksum = hash(b"keep.retention-manifest-checksum/v2\0", checksum_preimage);
    if observed_checksum != expected_checksum {
        return Err(RetentionManifestDecodeError::ChecksumMismatch {
            expected: expected_checksum,
            observed: observed_checksum,
        });
    }

    let observed_digest = read_digest(encoded, digest_offset)?;
    let digest_preimage =
        encoded
            .get(..digest_offset)
            .ok_or(RetentionManifestDecodeError::Truncated {
                expected: digest_offset,
                observed: encoded.len(),
            })?;
    let expected_digest = hash(b"keep.retention-manifest/v2\0", digest_preimage);
    if observed_digest != expected_digest {
        return Err(RetentionManifestDecodeError::ManifestDigestMismatch {
            expected: expected_digest,
            observed: observed_digest,
        });
    }
    Ok(expected_digest)
}

pub(super) fn verify_entry_set(
    entry_count: u32,
    entries: &[u8],
    observed: [u8; 32],
) -> Result<(), RetentionManifestDecodeError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-manifest-entries/v2\0");
    hasher.update(&entry_count.to_be_bytes());
    hasher.update(entries);
    let expected = *hasher.finalize().as_bytes();
    if observed == expected {
        Ok(())
    } else {
        Err(RetentionManifestDecodeError::EntrySetDigestMismatch { expected, observed })
    }
}

fn read_digest(encoded: &[u8], offset: usize) -> Result<[u8; 32], RetentionManifestDecodeError> {
    let end = offset
        .checked_add(32)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionManifestDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    <[u8; 32]>::try_from(bytes).map_err(|_| RetentionManifestDecodeError::Truncated {
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
