//! Canonical binary identity decoding failures.

use std::error::Error;
use std::fmt;

/// Failure to parse a canonical binary `BlobId`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobIdBinaryParseError {
    /// The input ended before the fixed identity frame was complete.
    Truncated {
        /// Required canonical frame length.
        expected: usize,
        /// Observed input length.
        observed: usize,
    },
    /// Bytes followed an otherwise complete fixed identity frame.
    TrailingData {
        /// Required canonical frame length.
        expected: usize,
        /// Observed input length.
        observed: usize,
    },
    /// The fixed identity-domain magic did not match.
    InvalidMagic {
        /// Bounded 16-byte magic observed in the input.
        observed: [u8; 16],
    },
    /// The frame names an identity version Keep does not implement.
    UnsupportedVersion {
        /// Version implemented by this decoder.
        expected: u16,
        /// Version declared by the input.
        observed: u16,
    },
    /// The frame names a hash algorithm invalid for version 1.
    UnsupportedAlgorithm {
        /// Algorithm required by version 1.
        expected: u8,
        /// Algorithm declared by the input.
        observed: u8,
    },
}

impl fmt::Display for BlobIdBinaryParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { expected, observed } => write!(
                formatter,
                "truncated blob identity frame: expected {expected} bytes, observed {observed}"
            ),
            Self::TrailingData { expected, observed } => write!(
                formatter,
                "trailing blob identity data: expected {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid blob identity binary magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported blob identity version {observed}; version {expected} is required"
            ),
            Self::UnsupportedAlgorithm { expected, observed } => write!(
                formatter,
                "unsupported blob identity algorithm {observed}; algorithm {expected} is required"
            ),
        }
    }
}

impl Error for BlobIdBinaryParseError {}
