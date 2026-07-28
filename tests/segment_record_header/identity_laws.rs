//! Kind-specific logical-identity corruption laws.

use std::error::Error;

use keep::{LayoutIdBinaryParseError, SegmentRecordHeader, SegmentRecordHeaderError};

use super::{
    FIRST_RECORD_OFFSET, ONE_ZERO_BUNDLE_SEGMENT_HEX, ONE_ZERO_SEGMENT_HEX, SECOND_RECORD_OFFSET,
    assert_mutation, mutate_header, record_header,
};

#[test]
fn chunk_identity_slot_refuses_every_kind_specific_disagreement() -> Result<(), Box<dyn Error>> {
    let header = record_header(ONE_ZERO_SEGMENT_HEX, FIRST_RECORD_OFFSET)?;

    assert_mutation(
        &header,
        48,
        &0u32.to_be_bytes(),
        SegmentRecordHeaderError::ZeroChunkLength { observed: 0 },
    )?;
    assert_mutation(
        &header,
        48,
        &2u32.to_be_bytes(),
        SegmentRecordHeaderError::ChunkPayloadLengthMismatch {
            identity_length: 2,
            payload_length: 1,
        },
    )?;
    let mut observed_tail = [0_u8; 24];
    let first = observed_tail
        .first_mut()
        .ok_or("chunk identity tail must not be empty")?;
    *first = 1;
    assert_mutation(
        &header,
        84,
        &observed_tail,
        SegmentRecordHeaderError::NonzeroChunkIdentityTail {
            expected: [0; 24],
            observed: observed_tail,
        },
    )?;
    assert_mutation(
        &header,
        108,
        &[1; 4],
        SegmentRecordHeaderError::ReservedBytes {
            offset: 108,
            expected: [0; 4],
            observed: [1; 4],
        },
    )?;
    Ok(())
}

#[test]
fn layout_identity_slot_preserves_nested_coordinate_failures() -> Result<(), Box<dyn Error>> {
    let header = record_header(ONE_ZERO_BUNDLE_SEGMENT_HEX, SECOND_RECORD_OFFSET)?;
    let mut invalid_magic = *b"KEEP:LAYOUT:ID\0\0";
    let first = invalid_magic
        .first_mut()
        .ok_or("layout identity magic must not be empty")?;
    *first = 0;

    assert_mutation(
        &header,
        48,
        &invalid_magic,
        SegmentRecordHeaderError::LayoutIdentity {
            source: LayoutIdBinaryParseError::InvalidMagic {
                observed: invalid_magic,
            },
        },
    )?;
    let mutated = mutate_header(&header, 48, &invalid_magic)?;
    let error = SegmentRecordHeader::decode(&mutated)
        .err()
        .ok_or("invalid layout identity was unexpectedly admitted")?;
    let source = error
        .source()
        .ok_or("layout identity error did not preserve its source")?;
    assert_eq!(
        source.downcast_ref::<LayoutIdBinaryParseError>(),
        Some(&LayoutIdBinaryParseError::InvalidMagic {
            observed: invalid_magic,
        })
    );
    assert_mutation(
        &header,
        68,
        &264u64.to_be_bytes(),
        SegmentRecordHeaderError::LayoutPayloadLengthMismatch {
            identity_length: 264,
            payload_length: 220,
        },
    )?;
    Ok(())
}

#[test]
fn layout_payload_bounds_are_kind_specific() -> Result<(), Box<dyn Error>> {
    let header = record_header(ONE_ZERO_BUNDLE_SEGMENT_HEX, SECOND_RECORD_OFFSET)?;

    assert_mutation(
        &header,
        22,
        &36u16.to_be_bytes(),
        SegmentRecordHeaderError::IdentityLength {
            record_kind: 2,
            expected: 60,
            observed: 36,
        },
    )?;
    assert_mutation(
        &header,
        24,
        &1u64.to_be_bytes(),
        SegmentRecordHeaderError::PayloadLengthOutOfBounds {
            record_kind: 2,
            minimum: 176,
            maximum: 46_137_520,
            observed: 1,
        },
    )?;
    Ok(())
}
