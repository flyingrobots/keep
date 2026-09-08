//! This boundary module owns canonical version-2 format-marker encoding.

use super::{
    CanonicalStoreFormatMarker, StoreFormatDefinitionDigest, format_marker_decoder as format,
};
use crate::RetentionManifest;

pub(super) fn version_two() -> CanonicalStoreFormatMarker {
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
    let (definition_digest, remaining) = remaining.split_at_mut(32);
    definition_digest.copy_from_slice(StoreFormatDefinitionDigest::VERSION_TWO.as_bytes());
    let (maximum_namespace_count, remaining) = remaining.split_at_mut(4);
    maximum_namespace_count.copy_from_slice(&RetentionManifest::MAXIMUM_ENTRY_COUNT.to_be_bytes());
    let (_reserved, _complete) = remaining.split_at_mut(4);
    checksum_slot.copy_from_slice(&format::checksum(preimage));
    let digest = format::digest(&encoded);
    CanonicalStoreFormatMarker::admitted(encoded, digest)
}
