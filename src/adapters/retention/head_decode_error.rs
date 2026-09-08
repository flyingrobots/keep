//! This boundary module owns typed retention-head decoding failures.

use crate::{LivenessGenerationError, RetentionHeadError, RetentionManifestLengthError};

/// Failure to decode and admit one version-2 retention head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionHeadDecodeError {
    /// The input was not exactly one complete fixed-width head.
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
    /// The fixed record length field was noncanonical.
    InvalidRecordLength {
        /// Required record length.
        expected: u16,
        /// Observed record length.
        observed: u16,
    },
    /// The record carried unsupported flags.
    UnsupportedFlags {
        /// Observed flag bits.
        observed: u32,
    },
    /// Reserved bytes were nonzero.
    NonZeroReserved {
        /// Observed reserved bytes.
        observed: [u8; 8],
    },
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// Liveness-generation admission failed.
    LivenessGeneration {
        /// Preserved generation failure.
        source: LivenessGenerationError,
    },
    /// Manifest-length admission failed.
    ManifestLength {
        /// Preserved manifest-length failure.
        source: RetentionManifestLengthError,
    },
    /// Final semantic head admission failed.
    Semantic {
        /// Preserved semantic failure.
        source: RetentionHeadError,
    },
}
