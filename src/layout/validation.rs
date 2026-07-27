//! One-pass semantic flat-layout validation.

use super::{LayoutEntry, LayoutEntryLimit, LayoutValidationError, admitted::check_entry_limit};
use crate::{BlobId, RegisteredStorageProfile};

pub(super) fn validate_entries(
    target: BlobId,
    profile: RegisteredStorageProfile,
    entries: &[LayoutEntry],
    entry_limit: LayoutEntryLimit,
) -> Result<(), LayoutValidationError> {
    check_entry_limit(entries.len(), entry_limit)?;
    validate_cardinality(target, entries.len())?;
    let mut expected_offset = 0_u64;
    for (position, entry) in entries.iter().enumerate() {
        let index = entry_index(position)?;
        let observed_offset = entry.offset().get();
        validate_offset(index, expected_offset, observed_offset)?;
        let length = entry.chunk_id().length().get();
        validate_profile_length(position, entries.len(), index, length, profile)?;
        expected_offset = checked_end(index, observed_offset, length)?;
    }
    validate_aggregate(target, expected_offset)
}

pub(super) const fn validate_cardinality(
    target: BlobId,
    entry_count: usize,
) -> Result<(), LayoutValidationError> {
    if target.is_empty() && entry_count != 0 {
        return Err(LayoutValidationError::EmptyBlobHasEntries {
            observed: entry_count,
        });
    }
    if !target.is_empty() && entry_count == 0 {
        return Err(LayoutValidationError::NonemptyBlobHasNoEntries);
    }
    Ok(())
}

pub(super) fn entry_index(position: usize) -> Result<u32, LayoutValidationError> {
    u32::try_from(position)
        .map_err(|_source| LayoutValidationError::EntryIndexOutOfRange { observed: position })
}

pub(super) const fn validate_positive_length(
    index: u32,
    observed: u32,
) -> Result<(), LayoutValidationError> {
    if observed == 0 {
        return Err(LayoutValidationError::ZeroChunkLength { index });
    }
    Ok(())
}

pub(super) fn validate_offset(
    index: u32,
    expected: u64,
    observed: u64,
) -> Result<(), LayoutValidationError> {
    if index == 0 && observed != 0 {
        return Err(LayoutValidationError::FirstOffsetNotZero { observed });
    }
    match observed.cmp(&expected) {
        std::cmp::Ordering::Less => Err(LayoutValidationError::Overlap {
            index,
            expected,
            observed,
        }),
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Greater => Err(LayoutValidationError::Gap {
            index,
            expected,
            observed,
        }),
    }
}

pub(super) fn validate_profile_length(
    position: usize,
    entry_count: usize,
    index: u32,
    observed: u32,
    profile: RegisteredStorageProfile,
) -> Result<(), LayoutValidationError> {
    let is_final = position.checked_add(1) == Some(entry_count);
    let minimum = if is_final {
        1
    } else {
        profile.minimum_chunk_length().get()
    };
    let maximum = profile.maximum_chunk_length().get();
    if (minimum..=maximum).contains(&observed) {
        return Ok(());
    }
    Err(LayoutValidationError::ProfileLengthOutOfBounds {
        index,
        minimum,
        maximum,
        observed,
    })
}

pub(super) fn checked_end(
    index: u32,
    offset: u64,
    length: u32,
) -> Result<u64, LayoutValidationError> {
    offset
        .checked_add(u64::from(length))
        .ok_or(LayoutValidationError::OffsetOverflow {
            index,
            offset,
            length,
        })
}

pub(super) const fn validate_aggregate(
    target: BlobId,
    observed: u64,
) -> Result<(), LayoutValidationError> {
    if observed != target.logical_length().get() {
        return Err(LayoutValidationError::AggregateLengthMismatch {
            expected: target.logical_length(),
            observed,
        });
    }
    Ok(())
}
