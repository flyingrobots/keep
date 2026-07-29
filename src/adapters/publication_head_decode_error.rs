//! Publication-head decoding failures.

use crate::{CatalogGenerationError, CatalogLengthError};

/// Failure to decode and checksum-verify a version-1 publication head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationHeadDecodeError {
    /// The input is not exactly one complete head.
    WrongLength {
        /// Required fixed width.
        expected: usize,
        /// Observed input width.
        observed: usize,
    },
    /// The fixed format magic did not match.
    InvalidMagic {
        /// Bounded magic observed in the input.
        observed: [u8; 16],
    },
    /// The format version is unsupported.
    UnsupportedVersion {
        /// Version implemented by this decoder.
        expected: u16,
        /// Version declared by the input.
        observed: u16,
    },
    /// Version-1 flags were nonzero.
    Flags {
        /// Canonical flag field.
        expected: u16,
        /// Observed flag field.
        observed: u16,
    },
    /// The fixed head length field was noncanonical.
    HeadLength {
        /// Canonical length field.
        expected: u16,
        /// Observed length field.
        observed: u16,
    },
    /// The head checksum algorithm is unsupported.
    ChecksumAlgorithm {
        /// Algorithm implemented by this decoder.
        expected: u8,
        /// Algorithm declared by the input.
        observed: u8,
    },
    /// The catalog digest algorithm is unsupported.
    DigestAlgorithm {
        /// Algorithm implemented by this decoder.
        expected: u8,
        /// Algorithm declared by the input.
        observed: u8,
    },
    /// The generation coordinate was invalid.
    Generation {
        /// Exact generation admission failure.
        source: CatalogGenerationError,
    },
    /// The catalog length coordinate was invalid.
    CatalogLength {
        /// Exact catalog-length admission failure.
        source: CatalogLengthError,
    },
    /// Version-1 reserved bytes were nonzero.
    Reserved {
        /// Required all-zero bytes.
        expected: [u8; 24],
        /// Observed reserved bytes.
        observed: [u8; 24],
    },
    /// The stored checksum disagreed with the canonical framed hash.
    ChecksumMismatch {
        /// Checksum derived from the covered bytes.
        expected: [u8; 32],
        /// Checksum stored in the head.
        observed: [u8; 32],
    },
}
