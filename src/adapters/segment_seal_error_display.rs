//! Human-readable segment-seal admission failures.

use std::error::Error;
use std::fmt;

use super::SegmentSealError;

impl fmt::Display for SegmentSealError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(result) = display_structure(self, formatter) {
            return result;
        }
        if let Some(result) = display_lengths(self, formatter) {
            return result;
        }
        display_integrity(self, formatter).unwrap_or(Err(fmt::Error))
    }
}

impl Error for SegmentSealError {}

fn display_structure(
    error: &SegmentSealError,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    let result = match error {
        SegmentSealError::WrongLength { expected, observed } => write!(
            formatter,
            "segment seal length must be {expected} bytes, observed {observed}"
        ),
        SegmentSealError::InvalidMagic { expected, observed } => write!(
            formatter,
            "segment seal magic must be {expected:?}, observed {observed:?}"
        ),
        SegmentSealError::UnsupportedVersion { expected, observed } => write!(
            formatter,
            "segment seal version must be {expected}, observed {observed}"
        ),
        SegmentSealError::UnknownFlags { expected, observed } => write!(
            formatter,
            "segment seal flags must be {expected:#06x}, observed {observed:#06x}"
        ),
        SegmentSealError::SealLength { expected, observed } => write!(
            formatter,
            "embedded segment seal length must be {expected}, observed {observed}"
        ),
        SegmentSealError::ReservedU16 { expected, observed } => write!(
            formatter,
            "reserved segment seal u16 must be {expected}, observed {observed}"
        ),
        SegmentSealError::RecordCountOutOfBounds { maximum, observed } => write!(
            formatter,
            "segment seal record count must not exceed {maximum}, observed {observed}"
        ),
        SegmentSealError::ReservedU32 { expected, observed } => write!(
            formatter,
            "reserved segment seal u32 must be {expected}, observed {observed}"
        ),
        _ => return None,
    };
    Some(result)
}

fn display_lengths(
    error: &SegmentSealError,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    let result = match error {
        SegmentSealError::PrefixLengthHostWidth { observed } => write!(
            formatter,
            "segment prefix length {observed} cannot be represented by the seal format"
        ),
        SegmentSealError::BytesBeforeSeal { expected, observed } => write!(
            formatter,
            "segment seal prefix length must be {expected}, observed {observed}"
        ),
        SegmentSealError::LengthArithmetic { bytes_before_seal } => write!(
            formatter,
            "segment length arithmetic overflowed after {bytes_before_seal} pre-seal bytes"
        ),
        SegmentSealError::SegmentLengthOutOfBounds { maximum, observed } => write!(
            formatter,
            "segment length must not exceed {maximum}, observed {observed}"
        ),
        SegmentSealError::SegmentLength { expected, observed } => write!(
            formatter,
            "segment length must be {expected}, observed {observed}"
        ),
        SegmentSealError::RecordBytes { expected, observed } => write!(
            formatter,
            "segment record byte count must be {expected}, observed {observed}"
        ),
        _ => return None,
    };
    Some(result)
}

fn display_integrity(
    error: &SegmentSealError,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    let result = match error {
        SegmentSealError::SealChecksumAlgorithm { expected, observed } => write!(
            formatter,
            "segment seal checksum algorithm must be {expected}, observed {observed}"
        ),
        SegmentSealError::SegmentDigestAlgorithm { expected, observed } => write!(
            formatter,
            "segment digest algorithm must be {expected}, observed {observed}"
        ),
        SegmentSealError::ReservedBytes { expected, observed } => write!(
            formatter,
            "reserved segment seal bytes must be {expected:?}, observed {observed:?}"
        ),
        SegmentSealError::DigestLengthArithmetic { prefix_length } => write!(
            formatter,
            "segment digest framing overflowed for prefix length {prefix_length}"
        ),
        SegmentSealError::SegmentDigestMismatch { expected, observed } => write!(
            formatter,
            "segment digest mismatch: expected {:?}, observed {:?}",
            expected.as_bytes(),
            observed.as_bytes()
        ),
        SegmentSealError::SealChecksumMismatch { expected, observed } => write!(
            formatter,
            "segment seal checksum mismatch: expected {expected:?}, observed {observed:?}"
        ),
        _ => return None,
    };
    Some(result)
}
