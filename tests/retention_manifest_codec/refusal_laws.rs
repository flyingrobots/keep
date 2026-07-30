//! Framing, integrity, and semantic refusal laws for retention manifests.

use std::io;

use keep::{AdmittedRetentionManifest, RetentionManifestDecodeError};

use super::{
    CHECKSUM_OFFSET, ENTRY_BODY_OFFSET, ENTRY_SET_DIGEST_OFFSET, MANIFEST_DIGEST_OFFSET,
    ONE_ROOT_MANIFEST, fixture_bytes,
};

#[test]
fn manifest_framing_and_integrity_have_exact_first_refusals()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        AdmittedRetentionManifest::decode(&truncated),
        Err(RetentionManifestDecodeError::Truncated {
            expected: 296,
            observed: 295,
        })
    ));

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(
        AdmittedRetentionManifest::decode(&trailing),
        Err(RetentionManifestDecodeError::TrailingData {
            expected: 296,
            observed: 297,
        })
    ));

    let mut checksum_corruption = bytes.clone();
    let last = checksum_corruption
        .last_mut()
        .ok_or_else(|| io::Error::other("frozen manifest is empty"))?;
    *last ^= 1;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&checksum_corruption),
        Err(RetentionManifestDecodeError::ChecksumMismatch { .. })
    ));

    let mut digest_corruption = bytes;
    let digest_byte = digest_corruption
        .get_mut(MANIFEST_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("frozen manifest lacks its digest"))?;
    *digest_byte ^= 1;
    refresh_checksum(&mut digest_corruption)?;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&digest_corruption),
        Err(RetentionManifestDecodeError::ManifestDigestMismatch { .. })
    ));
    Ok(())
}

#[test]
fn complete_integrity_precedes_manifest_semantics() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    bytes
        .get_mut(32..40)
        .ok_or_else(|| io::Error::other("frozen manifest lacks generation bytes"))?
        .fill(0);
    assert!(matches!(
        AdmittedRetentionManifest::decode(&bytes),
        Err(RetentionManifestDecodeError::ChecksumMismatch { .. })
    ));

    refresh_manifest_digest_and_checksum(&mut bytes)?;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&bytes),
        Err(RetentionManifestDecodeError::LivenessGeneration { .. })
    ));
    Ok(())
}

#[test]
fn entry_set_integrity_precedes_nested_root_generation_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    let first_entry_byte = bytes
        .get_mut(ENTRY_BODY_OFFSET)
        .ok_or_else(|| io::Error::other("frozen manifest lacks its entry body"))?;
    *first_entry_byte ^= 1;
    refresh_manifest_digest_and_checksum(&mut bytes)?;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&bytes),
        Err(RetentionManifestDecodeError::EntrySetDigestMismatch { .. })
    ));

    bytes
        .get_mut(ENTRY_BODY_OFFSET + 32..ENTRY_BODY_OFFSET + 40)
        .ok_or_else(|| io::Error::other("frozen manifest lacks root generation bytes"))?
        .fill(0);
    refresh_entry_set_digest(&mut bytes)?;
    refresh_manifest_digest_and_checksum(&mut bytes)?;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&bytes),
        Err(RetentionManifestDecodeError::RootGeneration { index: 0, .. })
    ));
    Ok(())
}

fn refresh_entry_set_digest(bytes: &mut [u8]) -> Result<(), io::Error> {
    let entries = bytes
        .get(ENTRY_BODY_OFFSET..MANIFEST_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("retention manifest lacks its entry body"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-manifest-entries/v2\0");
    hasher.update(&1_u32.to_be_bytes());
    hasher.update(entries);
    let digest = *hasher.finalize().as_bytes();
    bytes
        .get_mut(ENTRY_SET_DIGEST_OFFSET..ENTRY_SET_DIGEST_OFFSET + 32)
        .ok_or_else(|| io::Error::other("retention manifest lacks its entry-set digest"))?
        .copy_from_slice(&digest);
    Ok(())
}

fn refresh_manifest_digest_and_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let preimage = bytes
        .get(..MANIFEST_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("retention manifest lacks its digest preimage"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-manifest/v2\0");
    hasher.update(preimage);
    let digest = *hasher.finalize().as_bytes();
    bytes
        .get_mut(MANIFEST_DIGEST_OFFSET..CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("retention manifest lacks its digest"))?
        .copy_from_slice(&digest);
    refresh_checksum(bytes)
}

fn refresh_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let (preimage, trailer) = bytes
        .split_at_mut_checked(CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("retention manifest lacks a checksum"))?;
    let checksum_slot = trailer
        .get_mut(..blake3::OUT_LEN)
        .ok_or_else(|| io::Error::other("retention manifest checksum is truncated"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-manifest-checksum/v2\0");
    hasher.update(preimage);
    checksum_slot.copy_from_slice(hasher.finalize().as_bytes());
    Ok(())
}
