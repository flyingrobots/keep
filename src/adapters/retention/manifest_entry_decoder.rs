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
        let entry = admit_entry(bytes, index, previous)?;
        previous = Some(entry.namespace());
        entries.push(entry);
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

/// Admits complete entries and constraints decidable from the partial suffix.
pub(super) fn admit_prefix(encoded: &[u8]) -> Result<(), RetentionManifestDecodeError> {
    let mut previous = None;
    for (position, bytes) in encoded.chunks(ENTRY_WIDTH).enumerate() {
        let index =
            u32::try_from(position).map_err(|_| RetentionManifestDecodeError::LengthOverflow)?;
        if bytes.len() < ENTRY_WIDTH {
            return admit_partial_entry(bytes, index, previous);
        }
        previous = Some(admit_entry(bytes, index, previous)?.namespace());
    }
    Ok(())
}

fn admit_entry(
    bytes: &[u8],
    index: u32,
    previous: Option<RetentionNamespaceDigest>,
) -> Result<RetentionManifestEntry, RetentionManifestDecodeError> {
    let namespace = RetentionNamespaceDigest::from_hash(read_array(bytes, 0)?);
    let root_generation = admit_generation(bytes, index)?;
    let root_digest = RetentionRootDigest::from_hash(read_array(bytes, 40)?);
    admit_order(namespace.as_bytes(), index, previous)?;
    Ok(RetentionManifestEntry::new(
        namespace,
        root_generation,
        root_digest,
    ))
}

fn admit_partial_entry(
    bytes: &[u8],
    index: u32,
    previous: Option<RetentionNamespaceDigest>,
) -> Result<(), RetentionManifestDecodeError> {
    if bytes.len() >= 40 {
        let _generation = admit_generation(bytes, index)?;
    }
    // All unknown namespace suffix bytes set to 0xff give the greatest possible
    // completion. If even this cannot follow the predecessor, no completion can.
    let mut maximum = [u8::MAX; 32];
    for (target, observed) in maximum.iter_mut().zip(bytes) {
        *target = *observed;
    }
    admit_order(&maximum, index, previous)
}

fn admit_generation(
    bytes: &[u8],
    index: u32,
) -> Result<RootGeneration, RetentionManifestDecodeError> {
    RootGeneration::new(read_u64(bytes, 32)?)
        .map_err(|source| RetentionManifestDecodeError::RootGeneration { index, source })
}

fn admit_order(
    namespace: &[u8; 32],
    index: u32,
    previous: Option<RetentionNamespaceDigest>,
) -> Result<(), RetentionManifestDecodeError> {
    if let Some(prior) = previous
        && namespace <= prior.as_bytes()
    {
        return Err(RetentionManifestDecodeError::NonCanonicalEntryOrder { index });
    }
    Ok(())
}
