//! This boundary module owns store-migration intent decoding failures.

use crate::{CatalogGenerationError, CatalogLengthError};

/// Failure to decode and admit one version-2 migration intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationIntentDecodeError {
    /// The input was not exactly one complete intent.
    WrongLength {
        /// Required fixed width.
        expected: usize,
        /// Observed input width.
        observed: usize,
    },
    /// The fixed record magic was not canonical.
    InvalidMagic {
        /// Observed 16 magic bytes.
        observed: [u8; 16],
    },
    /// The format version is unsupported.
    UnsupportedVersion {
        /// Supported version.
        expected: u16,
        /// Observed version.
        observed: u16,
    },
    /// The record-length field was noncanonical.
    InvalidRecordLength {
        /// Required record length.
        expected: u16,
        /// Observed record length.
        observed: u16,
    },
    /// The intent carried unsupported flags.
    UnsupportedFlags {
        /// Observed flag bits.
        observed: u32,
    },
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// The catalog generation was not positive.
    InvalidCatalogGeneration {
        /// Observed generation.
        observed: u64,
        /// Precise generation refusal.
        source: CatalogGenerationError,
    },
    /// The catalog length was outside the canonical version-1 grammar.
    InvalidCatalogLength {
        /// Observed length.
        observed: u64,
        /// Precise catalog-length refusal.
        source: CatalogLengthError,
    },
    /// Generation 1 carried a forbidden predecessor.
    NonZeroInitialPredecessor {
        /// Observed nonzero predecessor digest.
        observed: [u8; 32],
    },
    /// A later generation omitted its required predecessor.
    MissingSuccessorPredecessor {
        /// Observed later generation.
        generation: u64,
    },
    /// The target definition was not the registered version-2 definition.
    DefinitionDigestMismatch {
        /// Registered version-2 definition digest.
        expected: [u8; 32],
        /// Observed target definition digest.
        observed: [u8; 32],
    },
    /// The stored identifier did not match the deterministic derivation.
    StoreIdentifierMismatch {
        /// Identifier derived from the admitted semantic fields.
        expected: [u8; 32],
        /// Identifier stored in the record.
        observed: [u8; 32],
    },
}
