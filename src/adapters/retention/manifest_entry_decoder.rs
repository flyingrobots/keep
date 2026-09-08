//! This boundary module owns canonical retention manifest entry decoding.

use super::RetentionManifestDecodeError;
use super::manifest_field_decoder::require_exact;
use crate::{
    RetentionManifestEntry, RetentionNamespaceDigest, RetentionRootDigest, RootGeneration,
};

const ENTRY_WIDTH: usize = 72;

pub(super) fn decode(
    encoded: &[u8],
    entry_count: u32,
) -> Result<Vec<RetentionManifestEntry>, RetentionManifestDecodeError> {
    let capacity =
        usize::try_from(entry_count).map_err(|_| RetentionManifestDecodeError::LengthOverflow)?;
    let expected_length = capacity
        .checked_mul(ENTRY_WIDTH)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    require_exact(encoded, expected_length)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(capacity)
        .map_err(|source| RetentionManifestDecodeError::Allocation { source })?;
    let mut previous = None;
    for (position, bytes) in encoded.chunks_exact(ENTRY_WIDTH).enumerate() {
        let index =
            u32::try_from(position).map_err(|_| RetentionManifestDecodeError::LengthOverflow)?;
        let namespace = RetentionNamespaceDigest::from_hash(read_array(bytes, 0)?);
        let root_generation = RootGeneration::new(read_u64(bytes, 32)?)
            .map_err(|source| RetentionManifestDecodeError::RootGeneration { index, source })?;
        let root_digest = RetentionRootDigest::from_hash(read_array(bytes, 40)?);
        if let Some(prior) = previous
            && namespace <= prior
        {
            return Err(RetentionManifestDecodeError::NonCanonicalEntryOrder { index });
        }
        entries.push(RetentionManifestEntry::new(
            namespace,
            root_generation,
            root_digest,
        ));
        previous = Some(namespace);
    }
    Ok(entries)
}

fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, RetentionManifestDecodeError> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

fn read_array<const WIDTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], RetentionManifestDecodeError> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionManifestDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| RetentionManifestDecodeError::Truncated {
        expected: end,
        observed: encoded.len(),
    })
}
