//! This boundary module owns typed retention manifest decoding failures.

use std::collections::TryReserveError;

use crate::{LivenessGenerationError, RetentionManifestError, RootGenerationError};

/// Failure to decode and admit one version-2 retention manifest.
#[derive(Debug)]
pub enum RetentionManifestDecodeError {
    /// The byte string ended before its required exact length.
    Truncated {
        /// Required byte length.
        expected: usize,
        /// Observed byte length.
        observed: usize,
    },
    /// Bytes followed the required exact record.
    TrailingData {
        /// Required byte length.
        expected: usize,
        /// Observed byte length.
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
    /// The fixed header width was not canonical.
    InvalidHeaderLength {
        /// Required header width.
        expected: u16,
        /// Observed width.
        observed: u16,
    },
    /// The record carried unsupported flags.
    UnsupportedFlags {
        /// Observed flag bits.
        observed: u32,
    },
    /// The declared total length disagreed with canonical field arithmetic.
    DeclaredLengthMismatch {
        /// Canonical computed length.
        expected: u64,
        /// Declared length.
        observed: u64,
    },
    /// Checked record-length arithmetic overflowed.
    LengthOverflow,
    /// The fixed entry width was not canonical.
    InvalidEntryWidth {
        /// Required entry width.
        expected: u16,
        /// Observed entry width.
        observed: u16,
    },
    /// A reserved field was nonzero.
    NonZeroReserved {
        /// Protocol field name.
        field: &'static str,
    },
    /// Liveness-generation admission failed.
    LivenessGeneration {
        /// Preserved generation failure.
        source: LivenessGenerationError,
    },
    /// The declared entry count exceeded the fixed bound.
    EntryCountExceeded {
        /// Fixed maximum count.
        maximum: u32,
        /// Observed count.
        observed: u32,
    },
    /// One entry contained an invalid root generation.
    RootGeneration {
        /// Zero-based entry index.
        index: u32,
        /// Preserved generation failure.
        source: RootGenerationError,
    },
    /// Canonical namespace ordering was violated.
    NonCanonicalEntryOrder {
        /// Zero-based index of the observed entry.
        index: u32,
    },
    /// Entry allocation was refused.
    Allocation {
        /// Preserved allocation failure.
        source: TryReserveError,
    },
    /// The entry-set digest did not match the exact body.
    EntrySetDigestMismatch {
        /// Computed canonical digest.
        expected: [u8; 32],
        /// Digest stored in the header.
        observed: [u8; 32],
    },
    /// The manifest digest did not match the exact header and body.
    ManifestDigestMismatch {
        /// Computed canonical digest.
        expected: [u8; 32],
        /// Digest stored in the record.
        observed: [u8; 32],
    },
    /// The checksum did not match the complete digest-bearing prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// Final semantic manifest admission failed.
    Semantic {
        /// Preserved semantic failure.
        source: RetentionManifestError,
    },
}
