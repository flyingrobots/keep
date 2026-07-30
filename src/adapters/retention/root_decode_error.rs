//! This boundary module owns typed retention root decoding failures.

use std::collections::TryReserveError;

use crate::{
    BlobIdBinaryParseError, LayoutIdBinaryParseError, RetentionClosureLimitError,
    RetentionNamespaceError, RetentionProfileAdmissionError, RetentionRootError,
    RootGenerationError,
};

/// Failure to decode and admit one version-2 retention root.
#[derive(Debug)]
pub enum RetentionRootDecodeError {
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
    /// The fixed anchor width was not canonical.
    InvalidAnchorWidth {
        /// Required anchor width.
        expected: u16,
        /// Observed anchor width.
        observed: u16,
    },
    /// A reserved field was nonzero.
    NonZeroReserved {
        /// Protocol field name.
        field: &'static str,
    },
    /// Root generation admission failed.
    Generation {
        /// Preserved generation failure.
        source: RootGenerationError,
    },
    /// Namespace admission failed.
    Namespace {
        /// Preserved namespace failure.
        source: RetentionNamespaceError,
    },
    /// The declared anchor count exceeded the fixed bound.
    AnchorCountExceeded {
        /// Fixed maximum count.
        maximum: u32,
        /// Observed count.
        observed: u32,
    },
    /// Realization-profile admission failed.
    Profile {
        /// Preserved profile failure.
        source: RetentionProfileAdmissionError,
    },
    /// Closure-limit admission failed.
    ClosureLimit {
        /// Preserved limit failure.
        source: RetentionClosureLimitError,
    },
    /// One anchor contained a malformed `BlobId`.
    BlobId {
        /// Zero-based anchor index.
        index: u32,
        /// Preserved coordinate failure.
        source: BlobIdBinaryParseError,
    },
    /// One anchor contained a malformed `LayoutId`.
    LayoutId {
        /// Zero-based anchor index.
        index: u32,
        /// Preserved coordinate failure.
        source: LayoutIdBinaryParseError,
    },
    /// Canonical anchor ordering was violated.
    NonCanonicalAnchorOrder {
        /// Zero-based index of the observed anchor.
        index: u32,
    },
    /// Anchor allocation was refused.
    Allocation {
        /// Preserved allocation failure.
        source: TryReserveError,
    },
    /// The anchor-set digest did not match the exact body.
    AnchorSetDigestMismatch {
        /// Computed canonical digest.
        expected: [u8; 32],
        /// Digest stored in the header.
        observed: [u8; 32],
    },
    /// The root digest did not match the exact header and body.
    RootDigestMismatch {
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
    /// Final semantic root admission failed.
    Semantic {
        /// Preserved semantic failure.
        source: RetentionRootError,
    },
}
