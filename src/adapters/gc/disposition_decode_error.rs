//! This boundary module owns typed recovery-disposition decoding failures.

use std::error::Error;
use std::fmt;

/// Which registered enumeration a disposition field belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDispositionField {
    /// The artifact kind at offset 24.
    ArtifactKind,
    /// The decision at offset 26.
    Decision,
    /// The recovery classification at offset 28.
    Classification,
}

/// Failure to decode and admit one recovery-disposition receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDispositionDecodeError {
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
    /// A reserved region was nonzero.
    NonZeroReserved,
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// An enumeration field carried an unregistered code.
    UnregisteredCode {
        /// The field.
        field: RecoveryDispositionField,
        /// The observed code.
        observed: u16,
    },
    /// A generation field was zero.
    ZeroGeneration {
        /// The record offset of the field.
        offset: usize,
    },
    /// Liveness generation zero was paired with a digest other than the
    /// canonical empty retention state.
    EmptyRetentionDigestMismatch {
        /// The observed manifest-digest slot.
        observed: [u8; 32],
    },
}

impl fmt::Display for RecoveryDispositionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "recovery disposition requires {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid recovery disposition magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported recovery disposition version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "recovery disposition record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => write!(
                formatter,
                "unsupported recovery disposition flags {observed:#010x}"
            ),
            Self::NonZeroReserved => {
                formatter.write_str("recovery disposition reserved bytes are nonzero")
            }
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("recovery disposition checksum mismatch")
            }
            Self::UnregisteredCode { field, observed } => write!(
                formatter,
                "recovery disposition {field:?} code {observed} is not registered"
            ),
            Self::ZeroGeneration { offset } => write!(
                formatter,
                "recovery disposition generation at offset {offset} is zero"
            ),
            Self::EmptyRetentionDigestMismatch { .. } => formatter.write_str(
                "recovery disposition names liveness generation zero without the empty \
                 retention-state digest",
            ),
        }
    }
}

impl Error for RecoveryDispositionDecodeError {}
