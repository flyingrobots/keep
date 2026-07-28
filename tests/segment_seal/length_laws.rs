//! Segment-seal length and algorithm corruption laws.

use std::error::Error;

use keep::SegmentSealError;

use super::{assert_mutation, canonical_empty_parts};

#[test]
fn seal_derived_lengths_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;

    assert_mutation(
        &parts.prefix,
        &parts.seal,
        32,
        &63_u64.to_be_bytes(),
        SegmentSealError::BytesBeforeSeal {
            expected: 64,
            observed: 63,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        40,
        &191_u64.to_be_bytes(),
        SegmentSealError::SegmentLength {
            expected: 192,
            observed: 191,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        48,
        &1_u64.to_be_bytes(),
        SegmentSealError::RecordBytes {
            expected: 0,
            observed: 1,
        },
    )?;
    Ok(())
}

#[test]
fn seal_algorithms_and_reserved_bytes_have_exact_first_refusals() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;

    assert_mutation(
        &parts.prefix,
        &parts.seal,
        56,
        &[2],
        SegmentSealError::SealChecksumAlgorithm {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        57,
        &[2],
        SegmentSealError::SegmentDigestAlgorithm {
            expected: 1,
            observed: 2,
        },
    )?;
    assert_mutation(
        &parts.prefix,
        &parts.seal,
        58,
        &[1; 6],
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [1; 6],
        },
    )?;
    Ok(())
}
