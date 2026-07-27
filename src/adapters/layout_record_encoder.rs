//! Canonical version-1 flat-layout record encoder.

use super::{CanonicalLayoutRecord, LayoutEncodeError};
use crate::layout::{AdmittedLayout, LayoutId, LayoutRecordLength};

const MAGIC: &[u8; 16] = b"KEEP:LAYOUT:PLAN";
const FORMAT_VERSION: u16 = 1;
const LAYOUT_CODEC: u16 = 1;
const FLAGS: u32 = 0;
const HEADER_LENGTH: u16 = 144;
const ENTRY_LENGTH: u16 = 44;
const CHECKSUM_LENGTH: u64 = 32;
const CHECKSUM_ALGORITHM: u8 = 1;
const CHUNK_HASH_ALGORITHM: u8 = 1;
const CHUNK_IDENTITY_VERSION: u16 = 1;
const PROFILE_IDENTITY_VERSION: u16 = 1;
const PROFILE_HASH_ALGORITHM: u8 = 1;
const RESERVED: [u8; 6] = [0_u8; 6];
const CHECKSUM_DOMAIN: &[u8; 16] = b"KEEP:LAYOUT:SUM\0";
const LAYOUT_ID_DOMAIN: &[u8; 16] = b"KEEP:LAYOUT:ID\0\0";

pub(super) fn encode_layout(
    layout: &AdmittedLayout,
) -> Result<CanonicalLayoutRecord, LayoutEncodeError> {
    let entry_count = u32::try_from(layout.entries().len()).map_err(|source| {
        LayoutEncodeError::EntryCountOutOfRange {
            observed: layout.entries().len(),
            source,
        }
    })?;
    let record_length = record_length(entry_count)?;
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

fn record_length(entry_count: u32) -> Result<LayoutRecordLength, LayoutEncodeError> {
    let entry_bytes = u64::from(entry_count)
        .checked_mul(u64::from(ENTRY_LENGTH))
        .ok_or(LayoutEncodeError::RecordLengthOutOfRange { entry_count })?;
    let with_header = u64::from(HEADER_LENGTH)
        .checked_add(entry_bytes)
        .ok_or(LayoutEncodeError::RecordLengthOutOfRange { entry_count })?;
    let raw = with_header
        .checked_add(CHECKSUM_LENGTH)
        .ok_or(LayoutEncodeError::RecordLengthOutOfRange { entry_count })?;
    LayoutRecordLength::from_wire(raw)
        .ok_or(LayoutEncodeError::RecordLengthOutOfRange { entry_count })
}

fn append_header(
    bytes: &mut Vec<u8>,
    layout: &AdmittedLayout,
    record_length: LayoutRecordLength,
    entry_count: u32,
) {
    bytes.extend_from_slice(MAGIC);
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

pub(super) fn record_checksum(
    record_without_checksum: &[u8],
    record_without_checksum_length: u64,
) -> [u8; 32] {
    let mut state = blake3::Hasher::new();
    state.update(CHECKSUM_DOMAIN);
    state.update(&FORMAT_VERSION.to_be_bytes());
    state.update(&[CHECKSUM_ALGORITHM]);
    state.update(record_without_checksum);
    state.update(&record_without_checksum_length.to_be_bytes());
    *state.finalize().as_bytes()
}

pub(super) fn calculate_layout_id(record: &[u8], record_length: LayoutRecordLength) -> LayoutId {
    let mut state = blake3::Hasher::new();
    state.update(LAYOUT_ID_DOMAIN);
    state.update(&FORMAT_VERSION.to_be_bytes());
    state.update(&LAYOUT_CODEC.to_be_bytes());
    state.update(record);
    state.update(&record_length.get().to_be_bytes());
    LayoutId::from_validated_parts(record_length, *state.finalize().as_bytes())
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
