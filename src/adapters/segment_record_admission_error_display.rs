//! Human-readable complete segment-record admission failures.

use std::error::Error;
use std::fmt;

use super::SegmentRecordAdmissionError;

impl fmt::Display for SegmentRecordAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChunkHash { source } => {
                write!(formatter, "segment record chunk hashing failed: {source}")
            }
            Self::Header { source } => {
                write!(
                    formatter,
                    "segment record header construction failed: {source}"
                )
            }
            Self::ChunkIdentityMismatch { .. } => {
                formatter.write_str("segment record chunk identity does not match its payload")
            }
            Self::Layout { source } => {
                write!(
                    formatter,
                    "segment record layout admission failed: {source}"
                )
            }
            Self::PayloadLengthHostWidth { observed } => write!(
                formatter,
                "segment record payload host length {observed} cannot fit the wire coordinate"
            ),
            Self::PayloadLengthMismatch { expected, observed } => write!(
                formatter,
                "segment record payload length {observed} does not equal header length {expected}"
            ),
            Self::RecordLengthArithmetic { observed } => write!(
                formatter,
                "segment record framing arithmetic failed for length {observed}"
            ),
        }
    }
}

impl Error for SegmentRecordAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChunkHash { source } => Some(source),
            Self::Header { source } => Some(source),
            Self::Layout { source } => Some(source),
            Self::ChunkIdentityMismatch { .. }
            | Self::PayloadLengthHostWidth { .. }
            | Self::PayloadLengthMismatch { .. }
            | Self::RecordLengthArithmetic { .. } => None,
        }
    }
}
