//! Finite specification-derived sweep over partial seal framing.

use std::error::Error;

use keep::{
    RecoverySegmentStage, RecoverySegmentTruncation, SegmentReadPolicy, SegmentSealError,
    classify_recovery_segment_stage,
};

use super::{seal_cause, support};

// Size: small. Oracle: v1 fixed-field table, independently encoded below.
// Exhaustive over each listed byte's first contradiction and every short seal end.
// Delete only if a stronger generated public boundary law subsumes these fields.
#[test]
fn available_fixed_seal_contradictions_refuse_every_later_short_boundary()
-> Result<(), Box<dyn Error>> {
    let canonical = support::decode_hex(
        include_str!("../../conformance/segment-store/v1/one-zero-segment.hex").trim(),
    )?;
    let start = canonical
        .len()
        .checked_sub(128)
        .ok_or("fixture lacks seal")?;
    for &(offset, expected) in CONTRADICTIONS {
        let mut changed = canonical.clone();
        let position = start
            .checked_add(offset)
            .ok_or("mutation offset overflow")?;
        *changed
            .get_mut(position)
            .ok_or("mutation outside fixture")? ^= 1;
        for observed in 16_usize..128 {
            let end = start.checked_add(observed).ok_or("prefix end overflow")?;
            let bytes = changed.get(..end).ok_or("prefix outside fixture")?;
            let state = classify_recovery_segment_stage(bytes, SegmentReadPolicy::MAXIMUM);
            if observed > offset {
                let error = state.err().ok_or_else(|| {
                    format!("offset={offset}, observed={observed}: corrupt seal admitted")
                })?;
                assert_eq!(
                    seal_cause(&error),
                    Some(&expected),
                    "offset={offset}, observed={observed}"
                );
            } else {
                assert!(
                    matches!(state?, RecoverySegmentStage::Truncated(RecoverySegmentTruncation::Seal {
                    offset: actual, required: 128, observed: length,
                }) if actual == u64::try_from(start)? && length == observed),
                    "unobserved mutation offset={offset}, observed={observed} must not become corruption"
                );
            }
        }
    }
    Ok(())
}

use SegmentSealError::{ReservedU16, ReservedU32, SealLength, UnknownFlags, UnsupportedVersion};

const CONTRADICTIONS: &[(usize, SegmentSealError)] = &[
    (
        16,
        UnsupportedVersion {
            expected: 1,
            observed: 257,
        },
    ),
    (
        17,
        UnsupportedVersion {
            expected: 1,
            observed: 0,
        },
    ),
    (
        18,
        UnknownFlags {
            expected: 0,
            observed: 256,
        },
    ),
    (
        19,
        UnknownFlags {
            expected: 0,
            observed: 1,
        },
    ),
    (
        20,
        SealLength {
            expected: 128,
            observed: 384,
        },
    ),
    (
        21,
        SealLength {
            expected: 128,
            observed: 129,
        },
    ),
    (
        22,
        ReservedU16 {
            expected: 0,
            observed: 256,
        },
    ),
    (
        23,
        ReservedU16 {
            expected: 0,
            observed: 1,
        },
    ),
    (
        28,
        ReservedU32 {
            expected: 0,
            observed: 16_777_216,
        },
    ),
    (
        29,
        ReservedU32 {
            expected: 0,
            observed: 65_536,
        },
    ),
    (
        30,
        ReservedU32 {
            expected: 0,
            observed: 256,
        },
    ),
    (
        31,
        ReservedU32 {
            expected: 0,
            observed: 1,
        },
    ),
    (
        56,
        SegmentSealError::SealChecksumAlgorithm {
            expected: 1,
            observed: 0,
        },
    ),
    (
        57,
        SegmentSealError::SegmentDigestAlgorithm {
            expected: 1,
            observed: 0,
        },
    ),
    (
        58,
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [1, 0, 0, 0, 0, 0],
        },
    ),
    (
        59,
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [0, 1, 0, 0, 0, 0],
        },
    ),
    (
        60,
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [0, 0, 1, 0, 0, 0],
        },
    ),
    (
        61,
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [0, 0, 0, 1, 0, 0],
        },
    ),
    (
        62,
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [0, 0, 0, 0, 1, 0],
        },
    ),
    (
        63,
        SegmentSealError::ReservedBytes {
            expected: [0; 6],
            observed: [0, 0, 0, 0, 0, 1],
        },
    ),
];
