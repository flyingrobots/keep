//! This boundary module owns typed GC retirement receipt decoding failures.

use std::error::Error;
use std::fmt;

use super::ReaderLockCoordinate;

/// Failure to decode and admit one GC retirement receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcRetirementReceiptDecodeError {
    /// The input was not exactly one complete fixed-width receipt.
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
    /// The receipt carried unsupported flags.
    UnsupportedFlags {
        /// Observed flag bits.
        observed: u32,
    },
    /// The reserved bytes were nonzero.
    NonZeroReserved,
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// The receipt named a generation other than its intent's.
    GenerationMismatch {
        /// Intent generation.
        expected: u64,
        /// Receipt generation.
        observed: u64,
    },
    /// The receipt did not bind the supplied admitted intent.
    IntentDigestMismatch {
        /// Supplied intent digest.
        expected: [u8; 32],
        /// Receipt intent digest.
        observed: [u8; 32],
    },
    /// The retired candidate set was not the intent's candidate set.
    RetiredSetDigestMismatch {
        /// Intent candidate-set digest.
        expected: [u8; 32],
        /// Receipt retired-set digest.
        observed: [u8; 32],
    },
    /// The revalidated liveness generation disagreed with the intent.
    LivenessGenerationMismatch {
        /// Intent liveness generation.
        expected: u64,
        /// Receipt liveness generation.
        observed: u64,
    },
    /// The revalidated retention-manifest digest disagreed with the intent.
    ManifestDigestMismatch {
        /// Intent manifest digest.
        expected: [u8; 32],
        /// Receipt manifest digest.
        observed: [u8; 32],
    },
    /// The revalidated catalog generation disagreed with the intent.
    CatalogGenerationMismatch {
        /// Intent catalog generation.
        expected: u64,
        /// Receipt catalog generation.
        observed: u64,
    },
    /// The revalidated catalog digest disagreed with the intent.
    CatalogDigestMismatch {
        /// Intent catalog digest.
        expected: [u8; 32],
        /// Receipt catalog digest.
        observed: [u8; 32],
    },
    /// One `reader.lock` coordinate disagreed with the intent.
    ReaderLockMismatch {
        /// The coordinate that disagreed.
        coordinate: ReaderLockCoordinate,
        /// Intent value.
        expected: u64,
        /// Receipt value.
        observed: u64,
    },
    /// The synchronization count was not one per candidate.
    SynchronizationCountMismatch {
        /// Intent-derived count.
        expected: u64,
        /// Receipt count.
        observed: u64,
    },
}

impl fmt::Display for GcRetirementReceiptDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "GC receipt requires {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid GC receipt magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported GC receipt version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "GC receipt record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(formatter, "unsupported GC receipt flags {observed:#010x}")
            }
            Self::NonZeroReserved => formatter.write_str("GC receipt reserved bytes are nonzero"),
            Self::ChecksumMismatch { .. } => formatter.write_str("GC receipt checksum mismatch"),
            Self::GenerationMismatch { expected, observed } => write!(
                formatter,
                "GC receipt generation {observed} does not complete intent generation {expected}"
            ),
            Self::IntentDigestMismatch { .. } => {
                formatter.write_str("GC receipt intent digest mismatch")
            }
            Self::RetiredSetDigestMismatch { .. } => {
                formatter.write_str("GC receipt retired candidate-set digest mismatch")
            }
            Self::LivenessGenerationMismatch { expected, observed } => write!(
                formatter,
                "GC receipt liveness generation {observed}; intent bound {expected}"
            ),
            Self::ManifestDigestMismatch { .. } => {
                formatter.write_str("GC receipt retention-manifest digest mismatch")
            }
            Self::CatalogGenerationMismatch { expected, observed } => write!(
                formatter,
                "GC receipt catalog generation {observed}; intent bound {expected}"
            ),
            Self::CatalogDigestMismatch { .. } => {
                formatter.write_str("GC receipt catalog digest mismatch")
            }
            Self::ReaderLockMismatch { coordinate, .. } => {
                write!(formatter, "GC receipt reader-lock {coordinate:?} mismatch")
            }
            Self::SynchronizationCountMismatch { expected, observed } => write!(
                formatter,
                "GC receipt synchronization count {observed}; intent derives {expected}"
            ),
        }
    }
}

impl Error for GcRetirementReceiptDecodeError {}
