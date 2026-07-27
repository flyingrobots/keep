//! Canonical storage-profile identity text decoding failures.

use std::error::Error;
use std::fmt;

/// Failure to parse a canonical text `StorageProfileId`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageProfileIdParseError {
    /// Input exceeds the only possible canonical version-1 length.
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
    /// The `storage-profile` kind token did not match.
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

impl fmt::Display for StorageProfileIdParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLong { maximum, observed } => write!(
                formatter,
                "storage-profile identity text exceeds {maximum} bytes with {observed} observed"
            ),
            Self::MalformedStructure => {
                formatter.write_str("malformed storage-profile identity structure")
            }
            Self::TrailingData => {
                formatter.write_str("storage-profile identity contains trailing data")
            }
            Self::InvalidScheme => formatter.write_str("invalid storage-profile identity scheme"),
            Self::InvalidKind => formatter.write_str("invalid storage-profile identity kind"),
            Self::MalformedVersion => formatter.write_str("malformed storage-profile version"),
            Self::UnsupportedVersion { observed } => {
                write!(formatter, "unsupported storage-profile version {observed}")
            }
            Self::UnsupportedAlgorithm => {
                formatter.write_str("unsupported storage-profile identity algorithm")
            }
            Self::InvalidDigestLength { expected, observed } => write!(
                formatter,
                "invalid storage-profile digest length: expected {expected}, observed {observed}"
            ),
            Self::NonCanonicalDigestCase => {
                formatter.write_str("noncanonical uppercase storage-profile digest")
            }
            Self::InvalidDigestAlphabet => {
                formatter.write_str("invalid storage-profile digest alphabet")
            }
        }
    }
}

impl Error for StorageProfileIdParseError {}
