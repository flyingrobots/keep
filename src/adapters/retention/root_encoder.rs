//! This boundary module owns canonical version-2 retention root encoding.

use super::{CanonicalRetentionRoot, RetentionRootEncodeError};
use crate::{RetentionRoot, RetentionRootDigest};

const HEADER_LENGTH: usize = 192;
const ANCHOR_WIDTH: usize = 119;
const TRAILER_LENGTH: usize = 64;

struct EncodingPlan {
    total_length: usize,
    digest_preimage_length: usize,
    anchor_set_digest: [u8; 32],
}

pub(super) fn encode(
    root: &RetentionRoot,
) -> Result<CanonicalRetentionRoot, RetentionRootEncodeError> {
    let plan = plan(root)?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(plan.total_length)
        .map_err(|source| RetentionRootEncodeError::Allocation { source })?;
    write_header(&mut encoded, root, &plan)?;
    write_body(&mut encoded, root);
    require_length(&encoded, plan.digest_preimage_length)?;
    let digest = hash(b"keep.retention-root/v2\0", &encoded);
    encoded.extend_from_slice(&digest);
    let checksum = hash(b"keep.retention-root-checksum/v2\0", &encoded);
    encoded.extend_from_slice(&checksum);
    require_length(&encoded, plan.total_length)?;
    Ok(CanonicalRetentionRoot::admitted(
        encoded,
        RetentionRootDigest::from_hash(digest),
    ))
}

fn plan(root: &RetentionRoot) -> Result<EncodingPlan, RetentionRootEncodeError> {
    let anchor_bytes = usize::try_from(root.anchor_count())
        .map_err(|_| RetentionRootEncodeError::LengthOverflow)?
        .checked_mul(ANCHOR_WIDTH)
        .ok_or(RetentionRootEncodeError::LengthOverflow)?;
    let digest_preimage_length = HEADER_LENGTH
        .checked_add(root.namespace().as_bytes().len())
        .and_then(|length| length.checked_add(anchor_bytes))
        .ok_or(RetentionRootEncodeError::LengthOverflow)?;
    let total_length = digest_preimage_length
        .checked_add(TRAILER_LENGTH)
        .ok_or(RetentionRootEncodeError::LengthOverflow)?;
    Ok(EncodingPlan {
        total_length,
        digest_preimage_length,
        anchor_set_digest: anchor_set_digest(root),
    })
}

fn anchor_set_digest(root: &RetentionRoot) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-anchor-set/v2\0");
    hasher.update(&root.anchor_count().to_be_bytes());
    for anchor in root.anchors() {
        hasher.update(&anchor.blob_id().encode_binary());
        hasher.update(&anchor.layout_id().encode_binary());
    }
    *hasher.finalize().as_bytes()
}

fn write_header(
    encoded: &mut Vec<u8>,
    root: &RetentionRoot,
    plan: &EncodingPlan,
) -> Result<(), RetentionRootEncodeError> {
    encoded.extend_from_slice(b"KEEP:RET:ROOT2\0\0");
    push_u16(encoded, 2);
    push_u16(encoded, 192);
    push_u32(encoded, 0);
    push_u64(
        encoded,
        u64::try_from(plan.total_length).map_err(|_| RetentionRootEncodeError::LengthOverflow)?,
    );
    push_u64(encoded, root.generation().get());
    push_u16(
        encoded,
        u16::try_from(root.namespace().as_bytes().len())
            .map_err(|_| RetentionRootEncodeError::LengthOverflow)?,
    );
    push_u16(encoded, 119);
    push_u32(encoded, root.anchor_count());
    write_policy(encoded, root);
    encoded.extend_from_slice(&predecessor_bytes(root));
    encoded.extend_from_slice(&plan.anchor_set_digest);
    encoded.extend_from_slice(&[0_u8; 12]);
    require_length(encoded, HEADER_LENGTH)
}

fn write_policy(encoded: &mut Vec<u8>, root: &RetentionRoot) {
    let profile = root.profile();
    let limits = root.limits();
    push_u32(encoded, profile.identity());
    push_u32(encoded, profile.version());
    encoded.extend_from_slice(profile.digest());
    push_u64(encoded, limits.nodes());
    push_u16(encoded, limits.depth());
    push_u16(encoded, 0);
    push_u64(encoded, limits.encoded_bytes());
    push_u64(encoded, limits.physical_bytes());
}

fn predecessor_bytes(root: &RetentionRoot) -> [u8; 32] {
    root.predecessor()
        .map_or([0_u8; 32], |digest| *digest.as_bytes())
}

fn write_body(encoded: &mut Vec<u8>, root: &RetentionRoot) {
    encoded.extend_from_slice(root.namespace().as_bytes());
    for anchor in root.anchors() {
        encoded.extend_from_slice(&anchor.blob_id().encode_binary());
        encoded.extend_from_slice(&anchor.layout_id().encode_binary());
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

const fn require_length(encoded: &[u8], expected: usize) -> Result<(), RetentionRootEncodeError> {
    if encoded.len() == expected {
        Ok(())
    } else {
        Err(RetentionRootEncodeError::ConstructionLength {
            expected,
            observed: encoded.len(),
        })
    }
}
