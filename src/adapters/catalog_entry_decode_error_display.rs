//! Human-readable catalog-entry decoding diagnostics.

use std::error::Error;
use std::fmt;

use super::CatalogEntryDecodeError;

impl fmt::Display for CatalogEntryDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "wrong catalog-entry length: expected {expected}, observed {observed}"
            ),
            Self::UnknownRecordKind { observed } => {
                write!(formatter, "unknown catalog record kind {observed}")
            }
            Self::Flags { expected, observed } => write!(
                formatter,
                "noncanonical catalog-entry flags: expected {expected}, observed {observed}"
            ),
            Self::IdentityLength {
                record_kind,
                expected,
                observed,
            } => write!(
                formatter,
                "wrong kind-{record_kind} identity length: expected {expected}, observed {observed}"
            ),
            Self::ZeroChunkLength { observed } => {
                write!(
                    formatter,
                    "catalog chunk length must be positive, observed {observed}"
                )
            }
            Self::NonzeroChunkIdentityTail { .. } => {
                formatter.write_str("nonzero catalog chunk identity tail")
            }
            Self::ChunkPayloadLengthMismatch {
                identity_length,
                payload_length,
            } => write!(
                formatter,
                "catalog chunk identity length {identity_length} disagrees with payload {payload_length}"
            ),
            Self::LayoutIdentity { source } => {
                write!(formatter, "invalid catalog layout identity: {source}")
            }
            Self::LayoutPayloadLengthMismatch {
                identity_length,
                payload_length,
            } => write!(
                formatter,
                "catalog layout identity length {identity_length} disagrees with payload {payload_length}"
            ),
            Self::PayloadLengthOutOfBounds {
                minimum,
                maximum,
                observed,
            } => write!(
                formatter,
                "catalog payload length {observed} is outside {minimum}..={maximum}"
            ),
            Self::RecordOffset { minimum, observed } => {
                write!(
                    formatter,
                    "catalog record offset {observed} precedes {minimum}"
                )
            }
            Self::RecordLengthMismatch {
                payload_length,
                expected,
                observed,
            } => write!(
                formatter,
                "catalog payload {payload_length} requires record length {expected}, observed {observed}"
            ),
            Self::RecordLengthArithmetic { payload_length } => write!(
                formatter,
                "catalog record-length arithmetic overflowed for payload {payload_length}"
            ),
            Self::RecordSpanArithmetic {
                record_offset,
                record_length,
            } => write!(
                formatter,
                "catalog record span overflows at {record_offset} plus {record_length}"
            ),
            Self::Reserved { .. } => formatter.write_str("nonzero catalog-entry reserved bytes"),
        }
    }
}

impl Error for CatalogEntryDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutIdentity { source } => Some(source),
            _ => None,
        }
    }
}
