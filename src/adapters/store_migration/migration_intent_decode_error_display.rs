//! This boundary module owns migration-intent error formatting and sources.

use std::error::Error;
use std::fmt;

use super::StoreMigrationIntentDecodeError;

impl fmt::Display for StoreMigrationIntentDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "migration intent requires {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid migration-intent magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported migration-intent version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "migration-intent record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(
                    formatter,
                    "unsupported migration-intent flags {observed:#010x}"
                )
            }
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("migration-intent checksum mismatch")
            }
            Self::InvalidCatalogGeneration { observed, .. } => {
                write!(formatter, "invalid migration catalog generation {observed}")
            }
            Self::InvalidCatalogLength { observed, .. } => {
                write!(formatter, "invalid migration catalog length {observed}")
            }
            Self::NonZeroInitialPredecessor { .. } => {
                formatter.write_str("initial migration catalog forbids a predecessor")
            }
            Self::MissingSuccessorPredecessor { generation } => write!(
                formatter,
                "migration catalog generation {generation} requires a predecessor"
            ),
            Self::DefinitionDigestMismatch { .. } => {
                formatter.write_str("migration target definition digest mismatch")
            }
            Self::StoreIdentifierMismatch { .. } => {
                formatter.write_str("migration store identifier mismatch")
            }
        }
    }
}

impl Error for StoreMigrationIntentDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCatalogGeneration { source, .. } => Some(source),
            Self::InvalidCatalogLength { source, .. } => Some(source),
            _ => None,
        }
    }
}
