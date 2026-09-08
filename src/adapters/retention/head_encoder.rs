//! This boundary module owns canonical version-2 retention-head encoding.

use super::{CanonicalRetentionHead, head_decoder as format};
use crate::RetentionHead;

pub(super) fn encode(head: &RetentionHead) -> CanonicalRetentionHead {
    let mut encoded = [0_u8; format::ENCODED_LENGTH];
    let (preimage, checksum_slot) = encoded.split_at_mut(format::CHECKSUM_OFFSET);
    let (magic, remaining) = preimage.split_at_mut(16);
    magic.copy_from_slice(&format::MAGIC);
    let (version, remaining) = remaining.split_at_mut(2);
    version.copy_from_slice(&format::VERSION.to_be_bytes());
    let (record_length, remaining) = remaining.split_at_mut(2);
    record_length.copy_from_slice(&format::RECORD_LENGTH.to_be_bytes());
    let (flags, remaining) = remaining.split_at_mut(4);
    flags.copy_from_slice(&0_u32.to_be_bytes());
    let (generation, remaining) = remaining.split_at_mut(8);
    generation.copy_from_slice(&head.generation().get().to_be_bytes());
    let (manifest_length, remaining) = remaining.split_at_mut(8);
    manifest_length.copy_from_slice(&head.manifest_length().get().to_be_bytes());
    let (manifest_digest, remaining) = remaining.split_at_mut(32);
    manifest_digest.copy_from_slice(head.manifest_digest().as_bytes());
    let (predecessor, remaining) = remaining.split_at_mut(32);
    predecessor.copy_from_slice(
        &head
            .predecessor()
            .map_or([0_u8; 32], |digest| *digest.as_bytes()),
    );
    let (_reserved, _complete) = remaining.split_at_mut(8);
    checksum_slot.copy_from_slice(&format::checksum(preimage));
    CanonicalRetentionHead::admitted(encoded)
}
