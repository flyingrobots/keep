//! Segment-seal framing and coordinate corruption laws.

use std::error::Error;

use keep::{SegmentSeal, SegmentSealError};

use super::{assert_mutation, canonical_empty_parts};

#[test]
fn segment_seal_refuses_every_noncanonical_width_before_fields() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;
    let truncated = parts
        .seal
        .get(..SegmentSeal::ENCODED_LENGTH - 1)
        .ok_or("segment seal lacks a truncation target")?;
    let mut extended = parts.seal.clone();
    extended.push(0);

    assert_eq!(
        SegmentSeal::decode(&parts.prefix, truncated),
        Err(SegmentSealError::WrongLength {
            expected: SegmentSeal::ENCODED_LENGTH,
            observed: SegmentSeal::ENCODED_LENGTH - 1,
        })
    );
    assert_eq!(
        SegmentSeal::decode(&parts.prefix, &extended),
        Err(SegmentSealError::WrongLength {
            expected: SegmentSeal::ENCODED_LENGTH,
            observed: SegmentSeal::ENCODED_LENGTH + 1,
        })
    );
    Ok(())
}

#[test]
fn seal_magic_version_flags_and_width_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;
    let expected_magic = *b"KEEP:SEGMENT:END";
    let mut observed_magic = expected_magic;
    let first = observed_magic
        .first_mut()
        .ok_or("segment seal magic must not be empty")?;
    *first = 0;

    assert_mutation(
        &parts.prefix,
        &parts.seal,
        0,
        &observed_magic,
        SegmentSealError::InvalidMagic {
            expected: expected_magic,
            observed: observed_magic,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        16,
        &2_u16.to_be_bytes(),
        SegmentSealError::UnsupportedVersion {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        18,
        &1_u16.to_be_bytes(),
        SegmentSealError::UnknownFlags {
            expected: 0,
            observed: 1,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        20,
        &127_u16.to_be_bytes(),
        SegmentSealError::SealLength {
            expected: 128,
            observed: 127,
        },
    )?;
    Ok(())
}

#[test]
fn seal_reserved_and_record_count_fields_have_exact_refusals() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;

    assert_mutation(
        &parts.prefix,
        &parts.seal,
        22,
        &1_u16.to_be_bytes(),
        SegmentSealError::ReservedU16 {
            expected: 0,
            observed: 1,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        24,
        &1_048_577_u32.to_be_bytes(),
        SegmentSealError::RecordCountOutOfBounds {
            maximum: 1_048_576,
            observed: 1_048_577,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        28,
        &1_u32.to_be_bytes(),
        SegmentSealError::ReservedU32 {
            expected: 0,
            observed: 1,
        },
    )?;
    Ok(())
}
