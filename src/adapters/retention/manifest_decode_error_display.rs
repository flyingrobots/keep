//! This boundary module owns retention manifest decode diagnostics and sources.

use std::{error::Error, fmt};

use super::RetentionManifestDecodeError;

impl fmt::Display for RetentionManifestDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { expected, observed } => write!(
                formatter,
                "retention manifest has {observed} bytes; expected {expected}"
            ),
            Self::TrailingData { expected, observed } => write!(
                formatter,
                "retention manifest has trailing data: expected {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { observed } => {
                write!(
                    formatter,
                    "invalid retention manifest magic {observed:02x?}"
                )
            }
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported retention manifest version {observed}; expected {expected}"
            ),
            Self::InvalidHeaderLength { expected, observed } => write!(
                formatter,
                "retention manifest header length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(
                    formatter,
                    "unsupported retention manifest flags {observed:#010x}"
                )
            }
            Self::DeclaredLengthMismatch { expected, observed } => write!(
                formatter,
                "retention manifest declares {observed} bytes; canonical fields require {expected}"
            ),
            Self::LengthOverflow => formatter.write_str("retention manifest length overflow"),
            Self::InvalidEntryWidth { expected, observed } => write!(
                formatter,
                "retention manifest entry width {observed}; expected {expected}"
            ),
            Self::NonZeroReserved { field } => write!(
                formatter,
                "retention manifest {field} reserved bytes are nonzero"
            ),
            Self::LivenessGeneration { source } => {
                write!(formatter, "invalid liveness generation: {source}")
            }
            Self::EntryCountExceeded { maximum, observed } => write!(
                formatter,
                "retention manifest declares {observed} entries; maximum is {maximum}"
            ),
            Self::RootGeneration { index, source } => write!(
                formatter,
                "invalid root generation in retention entry {index}: {source}"
            ),
            Self::NonCanonicalEntryOrder { index } => write!(
                formatter,
                "retention manifest entry {index} is not greater than its predecessor"
            ),
            Self::Allocation { .. } => {
                formatter.write_str("retention manifest entry allocation failed")
            }
            Self::EntrySetDigestMismatch { .. } => {
                formatter.write_str("retention manifest entry-set digest mismatch")
            }
            Self::ManifestDigestMismatch { .. } => {
                formatter.write_str("retention manifest digest mismatch")
            }
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("retention manifest checksum mismatch")
            }
            Self::Semantic { source } => {
                write!(formatter, "invalid semantic retention manifest: {source}")
            }
        }
    }
}

impl Error for RetentionManifestDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LivenessGeneration { source } => Some(source),
            Self::RootGeneration { source, .. } => Some(source),
            Self::Allocation { source } => Some(source),
            Self::Semantic { source } => Some(source),
            Self::Truncated { .. }
            | Self::TrailingData { .. }
            | Self::InvalidMagic { .. }
            | Self::UnsupportedVersion { .. }
            | Self::InvalidHeaderLength { .. }
            | Self::UnsupportedFlags { .. }
            | Self::DeclaredLengthMismatch { .. }
            | Self::LengthOverflow
            | Self::InvalidEntryWidth { .. }
            | Self::NonZeroReserved { .. }
            | Self::EntryCountExceeded { .. }
            | Self::NonCanonicalEntryOrder { .. }
            | Self::EntrySetDigestMismatch { .. }
            | Self::ManifestDigestMismatch { .. }
            | Self::ChecksumMismatch { .. } => None,
        }
    }
}
