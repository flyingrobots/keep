//! Human-readable segment-record-header admission failures.

use std::error::Error;
use std::fmt;

use super::SegmentRecordHeaderError;

impl fmt::Display for SegmentRecordHeaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match display_group(self) {
            DisplayGroup::Framing => format_framing(self, formatter),
            DisplayGroup::Coordinate => format_coordinate(self, formatter),
            DisplayGroup::Identity => format_identity(self, formatter),
        }
    }
}

impl Error for SegmentRecordHeaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutIdentity { source } => Some(source),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum DisplayGroup {
    Framing,
    Coordinate,
    Identity,
}

const fn display_group(error: &SegmentRecordHeaderError) -> DisplayGroup {
    match error {
        SegmentRecordHeaderError::WrongLength { .. }
        | SegmentRecordHeaderError::InvalidMagic { .. }
        | SegmentRecordHeaderError::UnsupportedVersion { .. }
        | SegmentRecordHeaderError::UnknownRecordKind { .. }
        | SegmentRecordHeaderError::UnknownFlags { .. }
        | SegmentRecordHeaderError::HeaderLength { .. }
        | SegmentRecordHeaderError::IdentityLength { .. }
        | SegmentRecordHeaderError::PayloadLengthOutOfBounds { .. }
        | SegmentRecordHeaderError::RecordLengthArithmetic { .. }
        | SegmentRecordHeaderError::RecordLength { .. } => DisplayGroup::Framing,
        SegmentRecordHeaderError::RecordChecksumAlgorithm { .. }
        | SegmentRecordHeaderError::IdentityVersion { .. }
        | SegmentRecordHeaderError::IdentityAlgorithm { .. }
        | SegmentRecordHeaderError::ReservedBytes { .. } => DisplayGroup::Coordinate,
        SegmentRecordHeaderError::ZeroChunkLength { .. }
        | SegmentRecordHeaderError::ChunkPayloadLengthMismatch { .. }
        | SegmentRecordHeaderError::NonzeroChunkIdentityTail { .. }
        | SegmentRecordHeaderError::LayoutIdentity { .. }
        | SegmentRecordHeaderError::LayoutPayloadLengthMismatch { .. } => DisplayGroup::Identity,
    }
}

fn format_framing(
    error: &SegmentRecordHeaderError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        SegmentRecordHeaderError::WrongLength { expected, observed } => {
            write!(
                formatter,
                "record header length {observed} does not equal {expected}"
            )
        }
        SegmentRecordHeaderError::InvalidMagic { .. } => {
            formatter.write_str("invalid segment-record magic")
        }
        SegmentRecordHeaderError::UnsupportedVersion { expected, observed } => write!(
            formatter,
            "record version {observed} is unsupported; version {expected} is required"
        ),
        SegmentRecordHeaderError::UnknownRecordKind { observed } => {
            write!(formatter, "record kind {observed} is unsupported")
        }
        SegmentRecordHeaderError::UnknownFlags { expected, observed } => {
            write!(formatter, "record flags {observed} do not equal {expected}")
        }
        SegmentRecordHeaderError::HeaderLength { expected, observed } => {
            write!(
                formatter,
                "record header field length {observed} does not equal {expected}"
            )
        }
        SegmentRecordHeaderError::IdentityLength {
            record_kind,
            expected,
            observed,
        } => write!(
            formatter,
            "record kind {record_kind} identity length {observed} does not equal {expected}"
        ),
        SegmentRecordHeaderError::PayloadLengthOutOfBounds {
            record_kind,
            minimum,
            maximum,
            observed,
        } => write!(
            formatter,
            "record kind {record_kind} payload length {observed} is outside {minimum}..={maximum}"
        ),
        SegmentRecordHeaderError::RecordLengthArithmetic { payload_length } => write!(
            formatter,
            "record length arithmetic failed for payload length {payload_length}"
        ),
        SegmentRecordHeaderError::RecordLength { expected, observed } => {
            write!(
                formatter,
                "record length {observed} does not equal {expected}"
            )
        }
        _ => Err(fmt::Error),
    }
}

fn format_coordinate(
    error: &SegmentRecordHeaderError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        SegmentRecordHeaderError::RecordChecksumAlgorithm { expected, observed } => write!(
            formatter,
            "record checksum algorithm {observed} does not equal {expected}"
        ),
        SegmentRecordHeaderError::IdentityVersion { expected, observed } => write!(
            formatter,
            "record identity version {observed} does not equal {expected}"
        ),
        SegmentRecordHeaderError::IdentityAlgorithm { expected, observed } => write!(
            formatter,
            "record identity algorithm {observed} does not equal {expected}"
        ),
        SegmentRecordHeaderError::ReservedBytes {
            offset, observed, ..
        } => write!(
            formatter,
            "record reserved bytes at offset {offset} are nonzero: {observed:02x?}"
        ),
        _ => Err(fmt::Error),
    }
}

fn format_identity(
    error: &SegmentRecordHeaderError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        SegmentRecordHeaderError::ZeroChunkLength { observed } => {
            write!(
                formatter,
                "chunk identity length {observed} is not positive"
            )
        }
        SegmentRecordHeaderError::ChunkPayloadLengthMismatch {
            identity_length,
            payload_length,
        } => write!(
            formatter,
            "chunk identity length {identity_length} disagrees with payload length {payload_length}"
        ),
        SegmentRecordHeaderError::NonzeroChunkIdentityTail { observed, .. } => write!(
            formatter,
            "unused chunk identity bytes are nonzero: {observed:02x?}"
        ),
        SegmentRecordHeaderError::LayoutIdentity { source } => {
            write!(formatter, "invalid record layout identity: {source}")
        }
        SegmentRecordHeaderError::LayoutPayloadLengthMismatch {
            identity_length,
            payload_length,
        } => write!(
            formatter,
            "layout identity length {identity_length} disagrees with payload length {payload_length}"
        ),
        _ => Err(fmt::Error),
    }
}
