//! Checked flat-layout count, length, and host-width admission.

use super::layout_record_format::{CHECKSUM_LENGTH, record_length};
use super::{LayoutDecodeError, LayoutDecodePolicy};
use crate::layout::{LayoutEntryLimit, LayoutRecordLength};

pub(super) struct ValidatedFraming {
    pub(super) record_length: LayoutRecordLength,
    pub(super) entry_count: u32,
    pub(super) entry_capacity: usize,
    pub(super) checksum_start: usize,
}

pub(super) fn validate_framing(
    declared_record_length: u64,
    entry_count: u32,
    input_length: usize,
    policy: LayoutDecodePolicy,
) -> Result<ValidatedFraming, LayoutDecodeError> {
    validate_entry_bounds(entry_count, policy)?;
    validate_record_bound(declared_record_length)?;
    let calculated = record_length(entry_count)
        .ok_or(LayoutDecodeError::RecordLengthArithmetic { entry_count })?;
    let declared_host = usize::try_from(declared_record_length).map_err(|source| {
        LayoutDecodeError::HostRecordLengthOutOfRange {
            observed: declared_record_length,
            source,
        }
    })?;
    validate_cross_lengths(
        declared_record_length,
        declared_host,
        calculated,
        entry_count,
        input_length,
    )?;
    let entry_capacity =
        usize::try_from(entry_count).map_err(|source| LayoutDecodeError::EntryCountHostWidth {
            observed: entry_count,
            source,
        })?;
    let checksum_width = usize::try_from(CHECKSUM_LENGTH).map_err(|source| {
        LayoutDecodeError::HostRecordLengthOutOfRange {
            observed: CHECKSUM_LENGTH,
            source,
        }
    })?;
    let checksum_start = declared_host
        .checked_sub(checksum_width)
        .ok_or(LayoutDecodeError::RecordLengthArithmetic { entry_count })?;
    Ok(ValidatedFraming {
        record_length: calculated,
        entry_count,
        entry_capacity,
        checksum_start,
    })
}

const fn validate_entry_bounds(
    observed: u32,
    policy: LayoutDecodePolicy,
) -> Result<(), LayoutDecodeError> {
    let protocol_maximum = LayoutEntryLimit::MAXIMUM.get();
    if observed > protocol_maximum {
        return Err(LayoutDecodeError::EntryCountLimitExceeded {
            maximum: protocol_maximum,
            observed,
        });
    }
    if observed > policy.entry_limit().get() {
        return Err(LayoutDecodeError::ConfiguredEntryLimitExceeded {
            maximum: policy.entry_limit().get(),
            observed,
        });
    }
    Ok(())
}

const fn validate_record_bound(observed: u64) -> Result<(), LayoutDecodeError> {
    if observed > LayoutRecordLength::MAXIMUM.get() {
        return Err(LayoutDecodeError::RecordLengthLimitExceeded {
            maximum: LayoutRecordLength::MAXIMUM.get(),
            observed,
        });
    }
    Ok(())
}

fn validate_cross_lengths(
    declared: u64,
    declared_host: usize,
    calculated: LayoutRecordLength,
    entry_count: u32,
    input_length: usize,
) -> Result<(), LayoutDecodeError> {
    if declared != calculated.get() {
        if declared_host == input_length {
            return Err(LayoutDecodeError::EntryCountLengthMismatch {
                entry_count,
                expected: calculated.get(),
                observed: declared,
            });
        }
        return Err(LayoutDecodeError::RecordLengthMismatch {
            expected: calculated.get(),
            observed: declared,
        });
    }
    match input_length.cmp(&declared_host) {
        std::cmp::Ordering::Less => Err(LayoutDecodeError::TruncatedRecord {
            expected: declared,
            observed: input_length,
        }),
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Greater => Err(LayoutDecodeError::TrailingData {
            expected: declared,
            observed: input_length,
        }),
    }
}
