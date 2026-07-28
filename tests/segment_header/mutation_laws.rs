//! Field-complete segment-header corruption laws.

use std::error::Error;

use keep::{SegmentHeader, SegmentHeaderError};

use super::empty_segment;

#[test]
fn every_fixed_header_field_has_an_exact_first_refusal() -> Result<(), Box<dyn Error>> {
    let segment = empty_segment()?;
    let header = segment
        .get(..SegmentHeader::ENCODED_LENGTH)
        .ok_or("empty-segment fixture lacks its header")?;
    let expected_magic = *b"KEEP:SEGMENT:V1\0";
    let mut observed_magic = expected_magic;
    let first_magic_byte = observed_magic
        .first_mut()
        .ok_or("segment magic must not be empty")?;
    *first_magic_byte = 0;

    assert_mutation(
        header,
        0,
        &observed_magic,
        SegmentHeaderError::InvalidMagic {
            expected: expected_magic,
            observed: observed_magic,
        },
    )?;
    assert_mutation(
        header,
        18,
        &1u16.to_be_bytes(),
        SegmentHeaderError::UnknownFlags {
            expected: 0,
            observed: 1,
        },
    )?;
    assert_mutation(
        header,
        20,
        &63u16.to_be_bytes(),
        SegmentHeaderError::HeaderLength {
            expected: 64,
            observed: 63,
        },
    )?;
    assert_mutation(
        header,
        22,
        &111u16.to_be_bytes(),
        SegmentHeaderError::RecordHeaderLength {
            expected: 112,
            observed: 111,
        },
    )?;
    assert_mutation(
        header,
        24,
        &127u16.to_be_bytes(),
        SegmentHeaderError::SealLength {
            expected: 128,
            observed: 127,
        },
    )?;
    assert_mutation(
        header,
        26,
        &1u16.to_be_bytes(),
        SegmentHeaderError::ReservedU16 {
            offset: 26,
            expected: 0,
            observed: 1,
        },
    )?;
    Ok(())
}

#[test]
fn every_header_bound_and_algorithm_has_an_exact_refusal() -> Result<(), Box<dyn Error>> {
    let segment = empty_segment()?;
    let header = segment
        .get(..SegmentHeader::ENCODED_LENGTH)
        .ok_or("empty-segment fixture lacks its header")?;

    assert_mutation(
        header,
        28,
        &67_108_863u64.to_be_bytes(),
        SegmentHeaderError::MaximumRecordPayloadLength {
            expected: 67_108_864,
            observed: 67_108_863,
        },
    )?;
    assert_mutation(
        header,
        36,
        &1_073_741_823u64.to_be_bytes(),
        SegmentHeaderError::MaximumSegmentLength {
            expected: 1_073_741_824,
            observed: 1_073_741_823,
        },
    )?;
    assert_mutation(
        header,
        44,
        &1_048_575u32.to_be_bytes(),
        SegmentHeaderError::MaximumRecordCount {
            expected: 1_048_576,
            observed: 1_048_575,
        },
    )?;
    assert_mutation(
        header,
        48,
        &[2],
        SegmentHeaderError::RecordChecksumAlgorithm {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        header,
        49,
        &[2],
        SegmentHeaderError::SegmentDigestAlgorithm {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        header,
        50,
        &[1; 14],
        SegmentHeaderError::ReservedBytes {
            offset: 50,
            expected: [0; 14],
            observed: [1; 14],
        },
    )?;
    Ok(())
}

fn assert_mutation(
    canonical: &[u8],
    offset: usize,
    replacement: &[u8],
    expected: SegmentHeaderError,
) -> Result<(), Box<dyn Error>> {
    let end = offset
        .checked_add(replacement.len())
        .ok_or("segment header mutation end overflow")?;
    let mut mutated = canonical.to_vec();
    let target = mutated
        .get_mut(offset..end)
        .ok_or("segment header mutation is out of bounds")?;
    target.copy_from_slice(replacement);
    assert_eq!(SegmentHeader::decode(&mutated), Err(expected));
    Ok(())
}
