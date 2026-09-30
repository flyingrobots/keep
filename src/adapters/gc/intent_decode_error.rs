//! This boundary module owns typed GC retirement intent decoding and
//! encoding failures.

use std::collections::TryReserveError;

use super::GcRetirementIntentError;
use crate::{
    CatalogGenerationError, GcGenerationError, LivenessGenerationError,
    RetentionProfileAdmissionError,
};

/// Failure to decode and admit one GC retirement intent.
#[derive(Debug)]
pub enum GcRetirementIntentDecodeError {
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
    /// The fixed candidate width was not canonical.
    InvalidCandidateWidth {
        /// Required candidate width.
        expected: u16,
        /// Observed candidate width.
        observed: u16,
    },
    /// A reserved field was nonzero.
    NonZeroReserved {
        /// Protocol field name.
        field: &'static str,
    },
    /// The declared candidate count exceeded the fixed bound.
    CandidateCountExceeded {
        /// Fixed maximum count.
        maximum: u32,
        /// Observed count.
        observed: u32,
    },
    /// Garbage-collection generation admission failed.
    Generation {
        /// Preserved generation failure.
        source: GcGenerationError,
    },
    /// Liveness-generation admission failed.
    LivenessGeneration {
        /// Preserved generation failure.
        source: LivenessGenerationError,
    },
    /// Catalog-generation admission failed.
    CatalogGeneration {
        /// Preserved generation failure.
        source: CatalogGenerationError,
    },
    /// Realization-profile admission failed.
    Profile {
        /// Preserved profile failure.
        source: RetentionProfileAdmissionError,
    },
    /// Candidate allocation was refused.
    Allocation {
        /// Preserved allocation failure.
        source: TryReserveError,
    },
    /// The candidate-set digest did not match the exact body.
    CandidateSetDigestMismatch {
        /// Computed canonical digest.
        expected: [u8; 32],
        /// Digest stored in the header.
        observed: [u8; 32],
    },
    /// The intent digest did not match the exact header and body.
    IntentDigestMismatch {
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
    /// Final semantic intent admission failed.
    Semantic {
        /// Preserved semantic failure.
        source: GcRetirementIntentError,
    },
}

/// Failure to encode one GC retirement intent.
#[derive(Debug)]
pub enum GcRetirementIntentEncodeError {
    /// Checked record-length arithmetic overflowed.
    LengthOverflow,
    /// Bounded output allocation was refused.
    Allocation {
        /// Preserved allocation failure.
        source: TryReserveError,
    },
}
