//! This boundary module owns typed verification receipt decoding failures.

use std::error::Error;
use std::fmt;

use crate::{BlobIdBinaryParseError, LayoutIdBinaryParseError};

/// One registered field a decoder refuses by name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationReceiptField {
    /// The outcome kind.
    Outcome,
    /// The depth or stage.
    Depth,
    /// The subject kind.
    SubjectKind,
    /// The view kind.
    ViewKind,
    /// The refusal class.
    RefusalClass,
    /// The evidence kind.
    EvidenceKind,
    /// The layout-present flag.
    LayoutPresent,
    /// The supported-minimum depth.
    SupportedMinimum,
    /// The supported-maximum depth.
    SupportedMaximum,
    /// The target-present flag.
    TargetPresent,
    /// The reserved bytes.
    Reserved,
    /// The catalog generation.
    CatalogGeneration,
    /// The liveness generation.
    LivenessGeneration,
}

/// Failure to decode and admit one verification receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationReceiptDecodeError {
    /// The input was not exactly one complete receipt.
    WrongLength {
        /// Required width.
        expected: usize,
        /// Observed width.
        observed: usize,
    },
    /// The magic was not canonical.
    InvalidMagic {
        /// Observed magic bytes.
        observed: [u8; 16],
    },
    /// The version is not supported.
    UnsupportedVersion {
        /// Supported version.
        expected: u16,
        /// Observed version.
        observed: u16,
    },
    /// The record length is not canonical.
    InvalidRecordLength {
        /// Expected length.
        expected: u16,
        /// Observed length.
        observed: u16,
    },
    /// Nonzero flag bits.
    UnsupportedFlags {
        /// Observed flags.
        observed: u32,
    },
    /// The verification contract version is not the one this crate implements.
    UnsupportedContract {
        /// Supported contract version.
        expected: u32,
        /// Observed contract version.
        observed: u32,
    },
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed checksum.
        expected: [u8; 32],
        /// Stored checksum.
        observed: [u8; 32],
    },
    /// A registered enumeration field holds an unregistered code.
    UnregisteredCode {
        /// The field.
        field: VerificationReceiptField,
        /// The observed code.
        observed: u16,
    },
    /// A field that must be zero is not.
    NonZero {
        /// The field.
        field: VerificationReceiptField,
    },
    /// The subject, layout, or target slot does not parse as a `BlobId`.
    BlobId {
        /// The exact parse refusal.
        source: BlobIdBinaryParseError,
    },
    /// The subject or layout slot does not parse as a `LayoutId`.
    LayoutId {
        /// The exact parse refusal.
        source: LayoutIdBinaryParseError,
    },
    /// The fields contradict one of the receipt's semantic laws.
    Semantic {
        /// The law violated.
        law: &'static str,
    },
}

impl fmt::Display for VerificationReceiptDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "verification receipt is {observed} bytes; expected {expected}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("verification receipt magic mismatch"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "verification receipt version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "verification receipt record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(
                    formatter,
                    "unsupported verification receipt flags {observed:#010x}"
                )
            }
            Self::UnsupportedContract { expected, observed } => write!(
                formatter,
                "verification contract {observed}; expected {expected}"
            ),
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("verification receipt checksum mismatch")
            }
            Self::UnregisteredCode { field, observed } => {
                write!(formatter, "unregistered code {observed} in {field:?}")
            }
            Self::NonZero { field } => write!(formatter, "{field:?} must be zero"),
            Self::BlobId { source } => write!(formatter, "receipt blob identity: {source}"),
            Self::LayoutId { source } => write!(formatter, "receipt layout identity: {source}"),
            Self::Semantic { law } => write!(formatter, "verification receipt violates: {law}"),
        }
    }
}

impl Error for VerificationReceiptDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BlobId { source } => Some(source),
            Self::LayoutId { source } => Some(source),
            _ => None,
        }
    }
}
