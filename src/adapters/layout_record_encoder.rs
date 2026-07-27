//! Canonical version-1 flat-layout record encoder.

use super::layout_record_format::{
    CHECKSUM_ALGORITHM, CHECKSUM_LENGTH, CHUNK_HASH_ALGORITHM, CHUNK_IDENTITY_VERSION,
    ENTRY_LENGTH, FLAGS, FORMAT_VERSION, HEADER_LENGTH, LAYOUT_CODEC, MAGIC,
    PROFILE_HASH_ALGORITHM, PROFILE_IDENTITY_VERSION, RESERVED, calculate_layout_id,
    record_checksum, record_length,
};
use super::{CanonicalLayoutRecord, LayoutEncodeError};
use crate::layout::{AdmittedLayout, LayoutRecordLength};

pub(super) fn encode_layout(
    layout: &AdmittedLayout,
) -> Result<CanonicalLayoutRecord, LayoutEncodeError> {
    let entry_count = u32::try_from(layout.entries().len()).map_err(|source| {
        LayoutEncodeError::EntryCountOutOfRange {
            observed: layout.entries().len(),
            source,
        }
    })?;
    let record_length = record_length(entry_count)
        .ok_or(LayoutEncodeError::RecordLengthOutOfRange { entry_count })?;
    let capacity = usize::try_from(record_length.get()).map_err(|source| {
        LayoutEncodeError::HostLengthOutOfRange {
            observed: record_length.get(),
            source,
        }
    })?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|source| LayoutEncodeError::Allocation {
            requested: capacity,
            source,
        })?;
    append_header(&mut bytes, layout, record_length, entry_count);
    append_entries(&mut bytes, layout);
    let checksum_input_length = record_length
        .get()
        .checked_sub(CHECKSUM_LENGTH)
        .ok_or(LayoutEncodeError::RecordLengthOutOfRange { entry_count })?;
    let checksum = record_checksum(&bytes, checksum_input_length);
    bytes.extend_from_slice(&checksum);
    verify_emitted_length(&bytes, record_length)?;
    let id = calculate_layout_id(&bytes, record_length);
    Ok(CanonicalLayoutRecord::from_parts(bytes, id))
}

fn append_header(
    bytes: &mut Vec<u8>,
    layout: &AdmittedLayout,
    record_length: LayoutRecordLength,
    entry_count: u32,
) {
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
    bytes.extend_from_slice(&LAYOUT_CODEC.to_be_bytes());
    bytes.extend_from_slice(&FLAGS.to_be_bytes());
    bytes.extend_from_slice(&HEADER_LENGTH.to_be_bytes());
    bytes.extend_from_slice(&ENTRY_LENGTH.to_be_bytes());
    bytes.extend_from_slice(&record_length.get().to_be_bytes());
    bytes.extend_from_slice(&entry_count.to_be_bytes());
    bytes.push(CHECKSUM_ALGORITHM);
    bytes.push(CHUNK_HASH_ALGORITHM);
    bytes.extend_from_slice(&CHUNK_IDENTITY_VERSION.to_be_bytes());
    bytes.extend_from_slice(&layout.target().encode_binary());
    bytes.extend_from_slice(&PROFILE_IDENTITY_VERSION.to_be_bytes());
    bytes.push(PROFILE_HASH_ALGORITHM);
    bytes.extend_from_slice(layout.profile().id().digest());
    bytes.extend_from_slice(&RESERVED);
}

fn append_entries(bytes: &mut Vec<u8>, layout: &AdmittedLayout) {
    for entry in layout.entries() {
        bytes.extend_from_slice(&entry.offset().get().to_be_bytes());
        bytes.extend_from_slice(&entry.chunk_id().length().get().to_be_bytes());
        bytes.extend_from_slice(entry.chunk_id().digest());
    }
}

fn verify_emitted_length(
    bytes: &[u8],
    expected: LayoutRecordLength,
) -> Result<(), LayoutEncodeError> {
    if usize::try_from(expected.get()) == Ok(bytes.len()) {
        return Ok(());
    }
    Err(LayoutEncodeError::InvariantLength {
        expected: expected.get(),
        observed: bytes.len(),
    })
}
