//! This boundary module owns canonical version-2 retention manifest encoding.

use super::{CanonicalRetentionManifest, RetentionManifestEncodeError};
use crate::{RetentionManifest, RetentionManifestDigest};

const HEADER_LENGTH: usize = 160;
const ENTRY_WIDTH: usize = 72;
const TRAILER_LENGTH: usize = 64;

struct EncodingPlan {
    total_length: usize,
    digest_preimage_length: usize,
    entry_set_digest: [u8; 32],
}

pub(super) fn encode(
    manifest: &RetentionManifest,
) -> Result<CanonicalRetentionManifest, RetentionManifestEncodeError> {
    let plan = plan(manifest)?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(plan.total_length)
        .map_err(|source| RetentionManifestEncodeError::Allocation { source })?;
    write_header(&mut encoded, manifest, &plan)?;
    write_entries(&mut encoded, manifest);
    require_length(&encoded, plan.digest_preimage_length)?;
    let digest = hash(b"keep.retention-manifest/v2\0", &encoded);
    encoded.extend_from_slice(&digest);
    let checksum = hash(b"keep.retention-manifest-checksum/v2\0", &encoded);
    encoded.extend_from_slice(&checksum);
    require_length(&encoded, plan.total_length)?;
    Ok(CanonicalRetentionManifest::admitted(
        encoded,
        RetentionManifestDigest::from_hash(digest),
    ))
}

fn plan(manifest: &RetentionManifest) -> Result<EncodingPlan, RetentionManifestEncodeError> {
    let entry_bytes = usize::try_from(manifest.entry_count())
        .map_err(|_| RetentionManifestEncodeError::LengthOverflow)?
        .checked_mul(ENTRY_WIDTH)
        .ok_or(RetentionManifestEncodeError::LengthOverflow)?;
    let digest_preimage_length = HEADER_LENGTH
        .checked_add(entry_bytes)
        .ok_or(RetentionManifestEncodeError::LengthOverflow)?;
    let total_length = digest_preimage_length
        .checked_add(TRAILER_LENGTH)
        .ok_or(RetentionManifestEncodeError::LengthOverflow)?;
    Ok(EncodingPlan {
        total_length,
        digest_preimage_length,
        entry_set_digest: entry_set_digest(manifest),
    })
}

fn entry_set_digest(manifest: &RetentionManifest) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-manifest-entries/v2\0");
    hasher.update(&manifest.entry_count().to_be_bytes());
    for entry in manifest.entries() {
        hasher.update(entry.namespace().as_bytes());
        hasher.update(&entry.root_generation().get().to_be_bytes());
        hasher.update(entry.root_digest().as_bytes());
    }
    *hasher.finalize().as_bytes()
}

fn write_header(
    encoded: &mut Vec<u8>,
    manifest: &RetentionManifest,
    plan: &EncodingPlan,
) -> Result<(), RetentionManifestEncodeError> {
    encoded.extend_from_slice(b"KEEP:RET:LIVE2\0\0");
    push_u16(encoded, 2);
    push_u16(encoded, 160);
    push_u32(encoded, 0);
    push_u64(
        encoded,
        u64::try_from(plan.total_length)
            .map_err(|_| RetentionManifestEncodeError::LengthOverflow)?,
    );
    push_u64(encoded, manifest.generation().get());
    push_u16(encoded, 72);
    push_u16(encoded, 0);
    push_u32(encoded, manifest.entry_count());
    encoded.extend_from_slice(&predecessor_bytes(manifest));
    encoded.extend_from_slice(&plan.entry_set_digest);
    encoded.extend_from_slice(&[0_u8; 48]);
    require_length(encoded, HEADER_LENGTH)
}

fn predecessor_bytes(manifest: &RetentionManifest) -> [u8; 32] {
    manifest
        .predecessor()
        .map_or([0_u8; 32], |digest| *digest.as_bytes())
}

fn write_entries(encoded: &mut Vec<u8>, manifest: &RetentionManifest) {
    for entry in manifest.entries() {
        encoded.extend_from_slice(entry.namespace().as_bytes());
        push_u64(encoded, entry.root_generation().get());
        encoded.extend_from_slice(entry.root_digest().as_bytes());
    }
}

fn hash(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

fn push_u16(encoded: &mut Vec<u8>, value: u16) {
    encoded.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(encoded: &mut Vec<u8>, value: u32) {
    encoded.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(encoded: &mut Vec<u8>, value: u64) {
    encoded.extend_from_slice(&value.to_be_bytes());
}

const fn require_length(
    encoded: &[u8],
    expected: usize,
) -> Result<(), RetentionManifestEncodeError> {
    if encoded.len() == expected {
        Ok(())
    } else {
        Err(RetentionManifestEncodeError::ConstructionLength {
            expected,
            observed: encoded.len(),
        })
    }
}
