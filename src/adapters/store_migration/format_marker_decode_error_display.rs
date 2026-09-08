//! This boundary module owns store-format marker decode diagnostics.

use std::{error::Error, fmt};

use super::StoreFormatMarkerDecodeError;

impl fmt::Display for StoreFormatMarkerDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "store-format marker has {observed} bytes; expected {expected}"
            ),
            Self::InvalidMagic { observed } => {
                write!(
                    formatter,
                    "invalid store-format marker magic {observed:02x?}"
                )
            }
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported store-format marker version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "store-format marker record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => write!(
                formatter,
                "unsupported store-format marker flags {observed:#010x}"
            ),
            Self::NonZeroReserved { observed } => write!(
                formatter,
                "store-format marker reserved field is nonzero: {observed:#010x}"
            ),
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("store-format marker checksum mismatch")
            }
            Self::DefinitionDigestMismatch { .. } => {
                formatter.write_str("store-format definition digest mismatch")
            }
            Self::InvalidMaximumNamespaceCount { expected, observed } => write!(
                formatter,
                "store-format maximum namespace count {observed}; expected {expected}"
            ),
        }
    }
}

impl Error for StoreFormatMarkerDecodeError {}
