//! Canonical fixed-width catalog-entry decoder.

use super::{
    CatalogEntryDecodeError, DecodedCatalogEntry, SegmentDigest, SegmentRecordChecksum,
    SegmentRecordIdentity, SegmentRecordLength,
    catalog_entry_fields::{self, read_array, read_u8, read_u16, read_u64},
};
use crate::{ChunkId, ChunkLength, LayoutId};

pub(super) const ENCODED_LENGTH: usize = catalog_entry_fields::ENCODED_LENGTH;
const FLAGS: u8 = 0;
const CHUNK_KIND: u8 = 1;
const LAYOUT_KIND: u8 = 2;
const CHUNK_IDENTITY_LENGTH: u16 = 36;
const LAYOUT_IDENTITY_LENGTH: u16 = 60;
const SEGMENT_HEADER_LENGTH: u64 = 64;
const RECORD_FRAMING_LENGTH: u64 = 144;
const MAXIMUM_RECORD_PAYLOAD_LENGTH: u64 = 67_108_864;

pub(super) fn decode(encoded: &[u8]) -> Result<DecodedCatalogEntry, CatalogEntryDecodeError> {
    if encoded.len() != ENCODED_LENGTH {
        return Err(CatalogEntryDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        });
    }
    let kind = read_u8(encoded, 0)?;
    let flags = read_u8(encoded, 1)?;
    if flags != FLAGS {
        return Err(CatalogEntryDecodeError::Flags {
            expected: FLAGS,
            observed: flags,
        });
    }
    let identity_length = read_u16(encoded, 2)?;
    let identity_slot = read_array(encoded, 4)?;
    let segment_digest = read_array(encoded, 64)?;
    let record_offset = read_u64(encoded, 96)?;
    let record_length = read_u64(encoded, 104)?;
    let payload_length = read_u64(encoded, 112)?;
    let checksum = read_array(encoded, 120)?;
    let reserved = read_array(encoded, 152)?;
    let identity = decode_identity(kind, identity_length, identity_slot, payload_length)?;
    validate_payload_bounds(kind, payload_length)?;
    validate_location(record_offset, record_length, payload_length)?;
    let expected = [0_u8; 8];
    if reserved != expected {
        return Err(CatalogEntryDecodeError::Reserved {
            expected,
            observed: reserved,
        });
    }
    Ok(DecodedCatalogEntry::new(
        identity,
        SegmentDigest::from_validated(segment_digest),
        record_offset,
        SegmentRecordLength::from_validated(record_length),
        SegmentRecordChecksum::from_validated(checksum),
    ))
}

fn decode_identity(
    kind: u8,
    identity_length: u16,
    identity: [u8; 60],
    payload_length: u64,
) -> Result<SegmentRecordIdentity, CatalogEntryDecodeError> {
    match kind {
        CHUNK_KIND => decode_chunk(identity_length, identity, payload_length),
        LAYOUT_KIND => decode_layout(identity_length, identity, payload_length),
        observed => Err(CatalogEntryDecodeError::UnknownRecordKind { observed }),
    }
}

fn decode_chunk(
    identity_length: u16,
    identity: [u8; 60],
    payload_length: u64,
) -> Result<SegmentRecordIdentity, CatalogEntryDecodeError> {
    require_identity_length(CHUNK_KIND, CHUNK_IDENTITY_LENGTH, identity_length)?;
    let length_bytes = read_array(&identity, 0)?;
    let digest = read_array(&identity, 4)?;
    let observed_tail = read_array(&identity, 36)?;
    let expected_tail = [0_u8; 24];
    if observed_tail != expected_tail {
        return Err(CatalogEntryDecodeError::NonzeroChunkIdentityTail {
            expected: expected_tail,
            observed: observed_tail,
        });
    }
    let length_value = u32::from_be_bytes(length_bytes);
    let length =
        ChunkLength::from_wire(length_value).ok_or(CatalogEntryDecodeError::ZeroChunkLength {
            observed: length_value,
        })?;
    if u64::from(length_value) != payload_length {
        return Err(CatalogEntryDecodeError::ChunkPayloadLengthMismatch {
            identity_length: length_value,
            payload_length,
        });
    }
    Ok(SegmentRecordIdentity::Chunk(ChunkId::from_validated_parts(
        length, digest,
    )))
}

fn decode_layout(
    identity_length: u16,
    identity: [u8; 60],
    payload_length: u64,
) -> Result<SegmentRecordIdentity, CatalogEntryDecodeError> {
    require_identity_length(LAYOUT_KIND, LAYOUT_IDENTITY_LENGTH, identity_length)?;
    let layout = LayoutId::parse_binary(&identity)
        .map_err(|source| CatalogEntryDecodeError::LayoutIdentity { source })?;
    let identity_length = layout.plan_length().get();
    if identity_length != payload_length {
        return Err(CatalogEntryDecodeError::LayoutPayloadLengthMismatch {
            identity_length,
            payload_length,
        });
    }
    Ok(SegmentRecordIdentity::Layout(layout))
}

const fn require_identity_length(
    record_kind: u8,
    expected: u16,
    observed: u16,
) -> Result<(), CatalogEntryDecodeError> {
    if observed == expected {
        Ok(())
    } else {
        Err(CatalogEntryDecodeError::IdentityLength {
            record_kind,
            expected,
            observed,
        })
    }
}

const fn validate_payload_bounds(kind: u8, observed: u64) -> Result<(), CatalogEntryDecodeError> {
    let minimum = 1;
    let maximum = MAXIMUM_RECORD_PAYLOAD_LENGTH;
    if matches!(kind, CHUNK_KIND | LAYOUT_KIND) && (observed < minimum || observed > maximum) {
        return Err(CatalogEntryDecodeError::PayloadLengthOutOfBounds {
            minimum,
            maximum,
            observed,
        });
    }
    Ok(())
}

fn validate_location(
    record_offset: u64,
    record_length: u64,
    payload_length: u64,
) -> Result<(), CatalogEntryDecodeError> {
    if record_offset < SEGMENT_HEADER_LENGTH {
        return Err(CatalogEntryDecodeError::RecordOffset {
            minimum: SEGMENT_HEADER_LENGTH,
            observed: record_offset,
        });
    }
    let expected = payload_length
        .checked_add(RECORD_FRAMING_LENGTH)
        .ok_or(CatalogEntryDecodeError::RecordLengthArithmetic { payload_length })?;
    if record_length != expected {
        return Err(CatalogEntryDecodeError::RecordLengthMismatch {
            payload_length,
            expected,
            observed: record_length,
        });
    }
    record_offset.checked_add(record_length).ok_or(
        CatalogEntryDecodeError::RecordSpanArithmetic {
            record_offset,
            record_length,
        },
    )?;
    Ok(())
}
