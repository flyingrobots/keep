//! Human-readable complete segment-record decoding failures.

use std::error::Error;
use std::fmt;

use super::SegmentRecordDecodeError;

impl fmt::Display for SegmentRecordDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { expected, observed } => write!(
                formatter,
                "segment record ended at {observed} bytes before its {expected}-byte header"
            ),
            Self::Header { source } => write!(formatter, "invalid segment record header: {source}"),
            Self::RecordLengthHostWidth { observed } => write!(
                formatter,
                "segment record length {observed} exceeds the host width"
            ),
            Self::TruncatedRecord { expected, observed } => write!(
                formatter,
                "segment record ended at {observed} bytes before declared length {expected}"
            ),
            Self::TrailingData { expected, observed } => write!(
                formatter,
                "segment record has {observed} bytes after declared length {expected}"
            ),
            Self::PayloadLengthHostWidth { observed } => write!(
                formatter,
                "segment record payload length {observed} exceeds the host width"
            ),
            Self::RecordLengthArithmetic { observed } => write!(
                formatter,
                "segment record framing arithmetic failed for length {observed}"
            ),
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("segment record checksum does not match its header and payload")
            }
        }
    }
}

impl Error for SegmentRecordDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Header { source } => Some(source),
            _ => None,
        }
    }
}
