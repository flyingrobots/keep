//! Canonical text layout identity decoding failures.

use std::error::Error;
use std::fmt;

/// Failure to parse a canonical text `LayoutId`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutIdTextParseError {
    /// Input exceeds the bounded parser limit.
    InputTooLong {
        /// Maximum accepted encoded byte length.
        maximum: usize,
        /// Observed encoded byte length.
        observed: usize,
    },
    /// One or more required colon-delimited fields are absent or empty.
    MalformedStructure,
    /// Fields follow the complete canonical identity.
    TrailingData,
    /// The `keep` scheme token did not match.
    InvalidScheme,
    /// The `layout` kind token did not match.
    InvalidKind,
    /// The version token is not canonical version syntax.
    MalformedVersion,
    /// The input names a canonical but unsupported identity version.
    UnsupportedVersion {
        /// Version declared by the input.
        observed: u16,
    },
    /// The layout codec token is not implemented.
    UnsupportedCodec,
    /// The input names an algorithm not admitted by version 1.
    UnsupportedAlgorithm,
    /// The plan length is not canonical unsigned decimal.
    NonCanonicalPlanLength,
    /// The canonical decimal plan length exceeds `u64::MAX`.
    PlanLengthOverflow,
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
    /// The digest does not contain exactly 64 hexadecimal characters.
    InvalidDigestLength {
        /// Required digest character count.
        expected: usize,
        /// Observed digest character count.
        observed: usize,
    },
    /// Uppercase hexadecimal appeared in the digest.
    NonCanonicalDigestCase,
    /// A digest character is outside lowercase hexadecimal.
    InvalidDigestAlphabet,
}

impl fmt::Display for LayoutIdTextParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLong { maximum, observed } => write!(
                formatter,
                "layout identity text exceeds {maximum} bytes with {observed} observed"
            ),
            Self::MalformedStructure => formatter.write_str("malformed layout identity structure"),
            Self::TrailingData => formatter.write_str("layout identity contains trailing data"),
            Self::InvalidScheme => formatter.write_str("invalid layout identity scheme"),
            Self::InvalidKind => formatter.write_str("invalid layout identity kind"),
            Self::MalformedVersion => formatter.write_str("malformed layout identity version"),
            Self::UnsupportedVersion { observed } => {
                write!(formatter, "unsupported layout identity version {observed}")
            }
            Self::UnsupportedCodec => formatter.write_str("unsupported layout identity codec"),
            Self::UnsupportedAlgorithm => {
                formatter.write_str("unsupported layout identity algorithm")
            }
            Self::NonCanonicalPlanLength => formatter.write_str("noncanonical layout plan length"),
            Self::PlanLengthOverflow => formatter.write_str("layout plan length exceeds u64"),
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
            Self::InvalidDigestLength { expected, observed } => write!(
                formatter,
                "invalid layout identity digest length: expected {expected}, observed {observed}"
            ),
            Self::NonCanonicalDigestCase => {
                formatter.write_str("noncanonical uppercase layout identity digest")
            }
            Self::InvalidDigestAlphabet => {
                formatter.write_str("invalid layout identity digest alphabet")
            }
        }
    }
}

impl Error for LayoutIdTextParseError {}
