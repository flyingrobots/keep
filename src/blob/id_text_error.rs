//! Canonical text identity decoding failures.

use std::error::Error;
use std::fmt;

/// Failure to parse a canonical text `BlobId`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobIdTextParseError {
    /// Input exceeds the maximum possible canonical version-1 length.
    InputTooLong {
        /// Maximum accepted encoded byte length.
        maximum: usize,
        /// Observed encoded byte length.
        observed: usize,
    },
    /// One or more required colon-delimited fields are absent or empty.
    MissingField,
    /// Fields follow the complete canonical identity.
    TrailingData,
    /// The `keep` scheme token did not match.
    InvalidScheme,
    /// The `blob` kind token did not match.
    InvalidKind,
    /// The version token is not canonical version syntax.
    MalformedVersion,
    /// The input names a canonical but unsupported identity version.
    UnsupportedVersion {
        /// Version declared by the input.
        observed: u16,
    },
    /// The input names an algorithm not admitted by version 1.
    UnsupportedAlgorithm,
    /// The logical length is not canonical unsigned decimal.
    NonCanonicalLength,
    /// The canonical decimal logical length exceeds `u64::MAX`.
    LengthOverflow,
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

impl fmt::Display for BlobIdTextParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLong { maximum, observed } => write!(
                formatter,
                "blob identity text exceeds {maximum} bytes with {observed} bytes observed"
            ),
            Self::MissingField => {
                formatter.write_str("blob identity text is missing a required field")
            }
            Self::TrailingData => formatter.write_str("blob identity text contains trailing data"),
            Self::InvalidScheme => formatter.write_str("invalid blob identity text scheme"),
            Self::InvalidKind => formatter.write_str("invalid blob identity text kind"),
            Self::MalformedVersion => formatter.write_str("malformed blob identity version"),
            Self::UnsupportedVersion { observed } => {
                write!(formatter, "unsupported blob identity version {observed}")
            }
            Self::UnsupportedAlgorithm => {
                formatter.write_str("unsupported blob identity algorithm")
            }
            Self::NonCanonicalLength => {
                formatter.write_str("noncanonical blob identity logical length")
            }
            Self::LengthOverflow => formatter.write_str("blob identity logical length exceeds u64"),
            Self::InvalidDigestLength { expected, observed } => write!(
                formatter,
                "invalid blob identity digest length: expected {expected}, observed {observed}"
            ),
            Self::NonCanonicalDigestCase => {
                formatter.write_str("noncanonical uppercase blob identity digest")
            }
            Self::InvalidDigestAlphabet => {
                formatter.write_str("invalid blob identity digest alphabet")
            }
        }
    }
}

impl Error for BlobIdTextParseError {}
