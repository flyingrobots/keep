//! Record framing and coordinate corruption laws.

use std::error::Error;

use keep::{SegmentRecordHeader, SegmentRecordHeaderError};

use super::{FIRST_RECORD_OFFSET, ONE_ZERO_SEGMENT_HEX, assert_mutation, record_header};

#[test]
fn record_header_refuses_wrong_width_before_field_admission() -> Result<(), Box<dyn Error>> {
    let header = canonical_chunk_header()?;
    let truncated = header
        .get(..SegmentRecordHeader::ENCODED_LENGTH - 1)
        .ok_or("record header lacks a truncation target")?;

    assert_eq!(
        SegmentRecordHeader::decode(truncated),
        Err(SegmentRecordHeaderError::WrongLength {
            expected: SegmentRecordHeader::ENCODED_LENGTH,
            observed: SegmentRecordHeader::ENCODED_LENGTH - 1,
        })
    );
    Ok(())
}

#[test]
fn record_magic_version_and_kind_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let header = canonical_chunk_header()?;
    let expected_magic = *b"KEEP:SEG:RECORD\0";
    let mut observed_magic = expected_magic;
    let first = observed_magic
        .first_mut()
        .ok_or("record magic must not be empty")?;
    *first = 0;

    assert_mutation(
        &header,
        0,
        &observed_magic,
        SegmentRecordHeaderError::InvalidMagic {
            expected: expected_magic,
            observed: observed_magic,
        },
    )?;
    assert_mutation(
        &header,
        16,
        &2u16.to_be_bytes(),
        SegmentRecordHeaderError::UnsupportedVersion {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &header,
        18,
        &[3],
        SegmentRecordHeaderError::UnknownRecordKind { observed: 3 },
    )?;
    Ok(())
}

#[test]
fn record_fixed_width_prefix_fields_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let header = canonical_chunk_header()?;

    assert_mutation(
        &header,
        19,
        &[1],
        SegmentRecordHeaderError::UnknownFlags {
            expected: 0,
            observed: 1,
        },
    )?;
    assert_mutation(
        &header,
        20,
        &111u16.to_be_bytes(),
        SegmentRecordHeaderError::HeaderLength {
            expected: 112,
            observed: 111,
        },
    )?;
    assert_mutation(
        &header,
        22,
        &35u16.to_be_bytes(),
        SegmentRecordHeaderError::IdentityLength {
            record_kind: 1,
            expected: 36,
            observed: 35,
        },
    )?;
    Ok(())
}

#[test]
fn record_payload_and_complete_lengths_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let header = canonical_chunk_header()?;

    assert_mutation(
        &header,
        24,
        &0u64.to_be_bytes(),
        SegmentRecordHeaderError::PayloadLengthOutOfBounds {
            record_kind: 1,
            minimum: 1,
            maximum: 67_108_864,
            observed: 0,
        },
    )?;
    assert_mutation(
        &header,
        32,
        &144u64.to_be_bytes(),
        SegmentRecordHeaderError::RecordLength {
            expected: 145,
            observed: 144,
        },
    )?;
    Ok(())
}

#[test]
fn record_algorithms_and_reserved_bytes_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let header = canonical_chunk_header()?;

    assert_mutation(
        &header,
        40,
        &[2],
        SegmentRecordHeaderError::RecordChecksumAlgorithm {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &header,
        41,
        &2u16.to_be_bytes(),
        SegmentRecordHeaderError::IdentityVersion {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &header,
        43,
        &[2],
        SegmentRecordHeaderError::IdentityAlgorithm {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &header,
        44,
        &[1; 4],
        SegmentRecordHeaderError::ReservedBytes {
            offset: 44,
            expected: [0; 4],
            observed: [1; 4],
        },
    )?;
    Ok(())
}

fn canonical_chunk_header() -> Result<Vec<u8>, Box<dyn Error>> {
    record_header(ONE_ZERO_SEGMENT_HEX, FIRST_RECORD_OFFSET)
}
