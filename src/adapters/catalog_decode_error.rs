//! Catalog decoding failures.

use super::CatalogEntryDecodeError;
use crate::{CatalogGenerationError, CatalogLengthError};

/// Failure to decode and integrity-verify one version-1 catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogDecodeError {
    /// The input is too short to contain fixed catalog framing.
    MinimumLength {
        /// Smallest complete catalog.
        minimum: usize,
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
        /// Canonical flags.
        expected: u16,
        /// Observed flags.
        observed: u16,
    },
    /// The fixed header-width field was noncanonical.
    HeaderLength {
        /// Canonical header width.
        expected: u16,
        /// Observed header width.
        observed: u16,
    },
    /// The fixed entry-width field was noncanonical.
    EntryLength {
        /// Canonical entry width.
        expected: u16,
        /// Observed entry width.
        observed: u16,
    },
    /// The generation coordinate was invalid.
    Generation {
        /// Exact generation admission failure.
        source: CatalogGenerationError,
    },
    /// Generation 1 carried a forbidden predecessor digest.
    UnexpectedPredecessor {
        /// Observed generation.
        generation: u64,
        /// Observed nonzero predecessor.
        observed: [u8; 32],
    },
    /// A later generation omitted its required predecessor digest.
    MissingPredecessor {
        /// Observed generation.
        generation: u64,
    },
    /// The declared entry count exceeded the format bound.
    EntryCountOutOfBounds {
        /// Largest supported entry count.
        maximum: u64,
        /// Declared entry count.
        observed: u64,
    },
    /// The declared catalog length was invalid.
    CatalogLength {
        /// Exact catalog-length admission failure.
        source: CatalogLengthError,
    },
    /// Entry count and declared length disagreed.
    EntryCountLengthMismatch {
        /// Declared entry count.
        entry_count: u64,
        /// Length derived by checked format arithmetic.
        expected: u64,
        /// Declared catalog length.
        observed: u64,
    },
    /// The actual input width disagreed with the declared canonical width.
    ObservedLength {
        /// Declared canonical width.
        declared: u64,
        /// Actual input width.
        observed: usize,
    },
    /// Catalog-length arithmetic overflowed.
    LengthArithmetic {
        /// Entry count participating in the calculation.
        entry_count: u64,
    },
    /// A host byte width could not enter the canonical hash frame.
    HashLength {
        /// Host byte width supplied to the frame.
        observed: usize,
    },
    /// The catalog checksum algorithm is unsupported.
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
    /// Version-1 header reserved bytes were nonzero.
    Reserved {
        /// Required all-zero bytes.
        expected: [u8; 46],
        /// Observed reserved bytes.
        observed: [u8; 46],
    },
    /// One fixed-width catalog entry was invalid.
    Entry {
        /// Zero-based canonical entry index.
        index: u64,
        /// Exact entry admission failure.
        source: CatalogEntryDecodeError,
    },
    /// Two entries carried the same logical identity.
    DuplicateIdentity {
        /// First occurrence.
        first_index: u64,
        /// Duplicate occurrence.
        duplicate_index: u64,
    },
    /// Logical identities were not in canonical order.
    IdentityOrder {
        /// Earlier physical entry index.
        previous_index: u64,
        /// Out-of-order physical entry index.
        observed_index: u64,
    },
    /// The stored checksum disagreed with canonical catalog bytes.
    ChecksumMismatch {
        /// Derived checksum.
        expected: [u8; 32],
        /// Stored checksum.
        observed: [u8; 32],
    },
    /// The stored physical digest disagreed with canonical catalog bytes.
    DigestMismatch {
        /// Derived digest.
        expected: [u8; 32],
        /// Stored digest.
        observed: [u8; 32],
    },
}
