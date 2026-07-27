//! Ordered admission of untrusted decoded entry coordinates.

use super::admitted::check_entry_limit;
use super::validation::{
    checked_end, entry_index, validate_aggregate, validate_cardinality, validate_offset,
    validate_positive_length, validate_profile_length,
};
use super::{AdmittedLayout, LayoutEntry, LayoutEntryLimit, LayoutValidationError};
use crate::{BlobId, ChunkId, ChunkLength, ChunkOffset, RegisteredStorageProfile};

impl AdmittedLayout {
    pub(crate) fn from_decoded_coordinates(
        target: BlobId,
        profile: RegisteredStorageProfile,
        entry_count: usize,
        coordinates: impl IntoIterator<Item = (u64, u32, [u8; 32])>,
        entry_limit: LayoutEntryLimit,
    ) -> Result<Self, LayoutValidationError> {
        check_entry_limit(entry_count, entry_limit)?;
        validate_cardinality(target, entry_count)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(entry_count)
            .map_err(|source| LayoutValidationError::Allocation { source })?;
        let mut expected_offset = 0_u64;
        for (offset, length, digest) in coordinates {
            let position = entries.len();
            if position >= entry_count {
                let observed = position
                    .checked_add(1)
                    .ok_or(LayoutValidationError::EntryIndexOutOfRange { observed: position })?;
                return Err(LayoutValidationError::EntryCountMismatch {
                    expected: entry_count,
                    observed,
                });
            }
            let index = entry_index(position)?;
            validate_offset(index, expected_offset, offset)?;
            validate_positive_length(index, length)?;
            validate_profile_length(position, entry_count, index, length, profile)?;
            expected_offset = checked_end(index, offset, length)?;
            let chunk_length = ChunkLength::from_validated(length);
            let id = ChunkId::from_validated_parts(chunk_length, digest);
            let entry_offset = ChunkOffset::from_validated(offset);
            entries.push(LayoutEntry::from_validated_parts(entry_offset, id));
        }
        if entries.len() != entry_count {
            return Err(LayoutValidationError::EntryCountMismatch {
                expected: entry_count,
                observed: entries.len(),
            });
        }
        validate_aggregate(target, expected_offset)?;
        Ok(Self::from_validated_entries(target, profile, entries))
    }
}
