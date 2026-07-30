//! This boundary module owns migration-receipt error formatting.

use std::error::Error;
use std::fmt;

use super::StoreMigrationReceiptDecodeError;

impl fmt::Display for StoreMigrationReceiptDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "migration receipt requires {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid migration-receipt magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported migration-receipt version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "migration-receipt record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(
                    formatter,
                    "unsupported migration-receipt flags {observed:#010x}"
                )
            }
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("migration-receipt checksum mismatch")
            }
            Self::IntentDigestMismatch { .. } => {
                formatter.write_str("migration-receipt intent digest mismatch")
            }
            Self::StoreIdentifierMismatch { .. } => {
                formatter.write_str("migration-receipt store identifier mismatch")
            }
            Self::FormatMarkerDigestMismatch { .. } => {
                formatter.write_str("migration-receipt format-marker digest mismatch")
            }
            Self::InitialRetentionStateDigestMismatch { .. } => {
                formatter.write_str("migration-receipt initial retention-state digest mismatch")
            }
            Self::InitialGcStateDigestMismatch { .. } => {
                formatter.write_str("migration-receipt initial GC-state digest mismatch")
            }
            Self::EmptyDispositionSetDigestMismatch { .. } => {
                formatter.write_str("migration-receipt empty disposition-set digest mismatch")
            }
            Self::UnsupportedSynchronizationBits { observed, .. } => write!(
                formatter,
                "migration-receipt synchronization mask has unknown bits: {observed:#018x}"
            ),
            Self::IncompleteSynchronizationMask { observed, .. } => write!(
                formatter,
                "migration-receipt synchronization mask is incomplete: {observed:#018x}"
            ),
        }
    }
}

impl Error for StoreMigrationReceiptDecodeError {}
