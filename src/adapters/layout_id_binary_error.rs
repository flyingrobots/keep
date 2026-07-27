//! Canonical binary layout identity decoding failures.

use std::error::Error;
use std::fmt;

/// Failure to parse a canonical binary `LayoutId`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutIdBinaryParseError {
    /// The coordinate does not have the exact fixed version-1 width.
    WrongLength {
        /// Required canonical coordinate length.
        expected: usize,
        /// Observed input length.
        observed: usize,
    },
    /// The fixed identity-domain magic did not match.
    InvalidMagic {
        /// Bounded 16-byte magic observed in the input.
        observed: [u8; 16],
    },
    /// The coordinate names an identity version Keep does not implement.
    UnsupportedVersion {
        /// Version implemented by this decoder.
        expected: u16,
        /// Version declared by the input.
        observed: u16,
    },
    /// The coordinate names a layout codec Keep does not implement.
    UnsupportedCodec {
        /// Codec implemented by this decoder.
        expected: u16,
        /// Codec declared by the input.
        observed: u16,
    },
    /// The plan length is outside the version-1 wire bound.
    PlanLengthOutOfBounds {
        /// Smallest canonical record length.
        minimum: u64,
        /// Largest canonical record length.
        maximum: u64,
        /// Length declared by the coordinate.
        observed: u64,
    },
    /// The plan length cannot describe a fixed-width entry sequence.
    PlanLengthNotCongruent {
        /// Length declared by the coordinate.
        observed: u64,
    },
}

impl fmt::Display for LayoutIdBinaryParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "wrong layout identity binary length: expected {expected}, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid layout identity magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported layout identity version {observed}; version {expected} is required"
            ),
            Self::UnsupportedCodec { expected, observed } => write!(
                formatter,
                "unsupported layout codec {observed}; codec {expected} is required"
            ),
            Self::PlanLengthOutOfBounds {
                minimum,
                maximum,
                observed,
            } => write!(
                formatter,
                "layout plan length {observed} is outside {minimum}..={maximum}"
            ),
            Self::PlanLengthNotCongruent { observed } => {
                write!(formatter, "layout plan length {observed} is not congruent")
            }
        }
    }
}

impl Error for LayoutIdBinaryParseError {}
