//! Shared immutable version-1 flat-layout wire coordinates.

use crate::layout::{LayoutId, LayoutRecordLength};

pub(super) const MAGIC: [u8; 16] = *b"KEEP:LAYOUT:PLAN";
pub(super) const FORMAT_VERSION: u16 = 1;
pub(super) const LAYOUT_CODEC: u16 = 1;
pub(super) const FLAGS: u32 = 0;
pub(super) const HEADER_LENGTH: u16 = 144;
pub(super) const ENTRY_LENGTH: u16 = 44;
pub(super) const CHECKSUM_LENGTH: u64 = 32;
pub(super) const CHECKSUM_ALGORITHM: u8 = 1;
pub(super) const CHUNK_HASH_ALGORITHM: u8 = 1;
pub(super) const CHUNK_IDENTITY_VERSION: u16 = 1;
pub(super) const PROFILE_IDENTITY_VERSION: u16 = 1;
pub(super) const PROFILE_HASH_ALGORITHM: u8 = 1;
pub(super) const RESERVED: [u8; 6] = [0_u8; 6];
const CHECKSUM_DOMAIN: &[u8; 16] = b"KEEP:LAYOUT:SUM\0";
const LAYOUT_ID_DOMAIN: &[u8; 16] = b"KEEP:LAYOUT:ID\0\0";

pub(super) fn record_length(entry_count: u32) -> Option<LayoutRecordLength> {
    let entry_bytes = u64::from(entry_count).checked_mul(u64::from(ENTRY_LENGTH))?;
    let with_header = u64::from(HEADER_LENGTH).checked_add(entry_bytes)?;
    let raw = with_header.checked_add(CHECKSUM_LENGTH)?;
    LayoutRecordLength::from_wire(raw)
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
