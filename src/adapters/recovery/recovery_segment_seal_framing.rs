//! This module owns contradiction checks for available fixed seal framing.

use super::recovery_fixed_field_prefix::observed_field;
use super::segment_seal::{ALGORITHM, FLAGS, SEAL_LENGTH, VERSION};
use crate::adapters::SegmentSealError;

// Canonical completion is only an instrument for detecting fixed-field
// contradictions. It does not admit absent bytes or prove seal completion.
pub(super) fn validate(encoded: &[u8]) -> Result<(), SegmentSealError> {
    require_u16(encoded, 16, VERSION, |expected, observed| {
        SegmentSealError::UnsupportedVersion { expected, observed }
    })?;
    require_u16(encoded, 18, FLAGS, |expected, observed| {
        SegmentSealError::UnknownFlags { expected, observed }
    })?;
    require_u16(encoded, 20, SEAL_LENGTH, |expected, observed| {
        SegmentSealError::SealLength { expected, observed }
    })?;
    require_u16(encoded, 22, 0, |expected, observed| {
        SegmentSealError::ReservedU16 { expected, observed }
    })?;
    let observed = u32::from_be_bytes(observed_field(encoded, 28, [0; 4]));
    if observed != 0 {
        return Err(SegmentSealError::ReservedU32 {
            expected: 0,
            observed,
        });
    }
    validate_algorithms(encoded)
}

fn validate_algorithms(encoded: &[u8]) -> Result<(), SegmentSealError> {
    let checksum = observed_field(encoded, 56, [ALGORITHM]);
    if checksum != [ALGORITHM] {
        return Err(SegmentSealError::SealChecksumAlgorithm {
            expected: ALGORITHM,
            observed: u8::from_be_bytes(checksum),
        });
    }
    let digest = observed_field(encoded, 57, [ALGORITHM]);
    if digest != [ALGORITHM] {
        return Err(SegmentSealError::SegmentDigestAlgorithm {
            expected: ALGORITHM,
            observed: u8::from_be_bytes(digest),
        });
    }
    let expected = [0; 6];
    let observed = observed_field(encoded, 58, expected);
    if observed != expected {
        return Err(SegmentSealError::ReservedBytes { expected, observed });
    }
    Ok(())
}

fn require_u16(
    encoded: &[u8],
    offset: usize,
    expected: u16,
    error: fn(u16, u16) -> SegmentSealError,
) -> Result<(), SegmentSealError> {
    let observed = u16::from_be_bytes(observed_field(encoded, offset, expected.to_be_bytes()));
    if observed == expected {
        Ok(())
    } else {
        Err(error(expected, observed))
    }
}
