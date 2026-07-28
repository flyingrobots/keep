//! Human-readable segment-header diagnostics.

use std::error::Error;
use std::fmt;

use super::SegmentHeaderError;

impl fmt::Display for SegmentHeaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "segment header length {observed} does not equal {expected}"
            ),
            Self::InvalidMagic { expected, observed } => write!(
                formatter,
                "segment header magic {observed:02x?} does not equal {expected:02x?}"
            ),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "segment header version {observed} is not supported; expected {expected}"
            ),
            Self::UnknownFlags { expected, observed } => write!(
                formatter,
                "segment header flags {observed} do not equal {expected}"
            ),
            Self::HeaderLength { expected, observed } => write!(
                formatter,
                "segment header field length {observed} does not equal {expected}"
            ),
            Self::RecordHeaderLength { expected, observed } => write!(
                formatter,
                "record header field length {observed} does not equal {expected}"
            ),
            Self::SealLength { expected, observed } => write!(
                formatter,
                "segment seal field length {observed} does not equal {expected}"
            ),
            Self::ReservedU16 {
                offset,
                expected,
                observed,
            } => write!(
                formatter,
                "segment header reserved field at offset {offset} is {observed}, expected {expected}"
            ),
            Self::MaximumRecordPayloadLength { expected, observed } => write!(
                formatter,
                "record payload bound {observed} does not equal {expected}"
            ),
            Self::MaximumSegmentLength { expected, observed } => write!(
                formatter,
                "segment length bound {observed} does not equal {expected}"
            ),
            Self::MaximumRecordCount { expected, observed } => write!(
                formatter,
                "segment record-count bound {observed} does not equal {expected}"
            ),
            Self::RecordChecksumAlgorithm { expected, observed } => write!(
                formatter,
                "record checksum algorithm {observed} is not supported; expected {expected}"
            ),
            Self::SegmentDigestAlgorithm { expected, observed } => write!(
                formatter,
                "segment digest algorithm {observed} is not supported; expected {expected}"
            ),
            Self::ReservedBytes {
                offset,
                expected,
                observed,
            } => write!(
                formatter,
                "segment header reserved bytes at offset {offset} are {observed:02x?}, expected {expected:02x?}"
            ),
        }
    }
}

impl Error for SegmentHeaderError {}
