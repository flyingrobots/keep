//! This boundary module owns retention-head decode diagnostics and sources.

use std::{error::Error, fmt};

use super::RetentionHeadDecodeError;

impl fmt::Display for RetentionHeadDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestLengthPrefixImpossible { observed } => write!(
                formatter,
                "retention head size fields have no canonical completion at {observed} bytes"
            ),
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "retention head has {observed} bytes; expected {expected}"
            ),
            Self::PrefixByteMismatch {
                offset,
                expected,
                observed,
            } => write!(
                formatter,
                "retention stage byte {offset} is {observed:#04x}; expected {expected:#04x}"
            ),
            Self::InvalidMagic { observed } => {
                write!(formatter, "invalid retention head magic {observed:02x?}")
            }
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported retention head version {observed}; expected {expected}"
            ),
            Self::InvalidRecordLength { expected, observed } => write!(
                formatter,
                "retention head record length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(
                    formatter,
                    "unsupported retention head flags {observed:#010x}"
                )
            }
            Self::NonZeroReserved { observed } => {
                write!(
                    formatter,
                    "retention head reserved bytes are nonzero: {observed:02x?}"
                )
            }
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("retention head checksum mismatch")
            }
            Self::LivenessGeneration { source } => {
                write!(formatter, "invalid retention-head generation: {source}")
            }
            Self::ManifestLength { source } => {
                write!(formatter, "invalid retention manifest length: {source}")
            }
            Self::Semantic { source } => {
                write!(formatter, "invalid semantic retention head: {source}")
            }
        }
    }
}

impl Error for RetentionHeadDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LivenessGeneration { source } => Some(source),
            Self::ManifestLength { source } => Some(source),
            Self::Semantic { source } => Some(source),
            Self::WrongLength { .. }
            | Self::PrefixByteMismatch { .. }
            | Self::ManifestLengthPrefixImpossible { .. }
            | Self::InvalidMagic { .. }
            | Self::UnsupportedVersion { .. }
            | Self::InvalidRecordLength { .. }
            | Self::UnsupportedFlags { .. }
            | Self::NonZeroReserved { .. }
            | Self::ChecksumMismatch { .. } => None,
        }
    }
}
