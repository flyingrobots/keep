//! This boundary module owns retention root decode diagnostics and sources.

use std::{error::Error, fmt};

use super::RetentionRootDecodeError;
impl fmt::Display for RetentionRootDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { expected, observed } => {
                write!(
                    formatter,
                    "retention root has {observed} bytes; expected {expected}"
                )
            }
            Self::TrailingData { expected, observed } => write!(
                formatter,
                "retention root has trailing data: expected {expected} bytes, observed {observed}"
            ),
            Self::InvalidMagic { observed } => {
                write!(formatter, "invalid retention root magic {observed:02x?}")
            }
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported retention root version {observed}; expected {expected}"
            ),
            Self::InvalidHeaderLength { expected, observed } => write!(
                formatter,
                "retention root header length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(
                    formatter,
                    "unsupported retention root flags {observed:#010x}"
                )
            }
            Self::DeclaredLengthMismatch { expected, observed } => write!(
                formatter,
                "retention root declares {observed} bytes; canonical fields require {expected}"
            ),
            Self::LengthOverflow => formatter.write_str("retention root length overflow"),
            Self::InvalidAnchorWidth { expected, observed } => write!(
                formatter,
                "retention root anchor width {observed}; expected {expected}"
            ),
            Self::NonZeroReserved { field } => {
                write!(
                    formatter,
                    "retention root {field} reserved bytes are nonzero"
                )
            }
            Self::Generation { source } => write!(formatter, "invalid root generation: {source}"),
            Self::Namespace { source } => write!(formatter, "invalid root namespace: {source}"),
            Self::AnchorCountExceeded { maximum, observed } => write!(
                formatter,
                "retention root declares {observed} anchors; maximum is {maximum}"
            ),
            Self::Profile { source } => write!(formatter, "invalid root profile: {source}"),
            Self::ClosureLimit { source } => {
                write!(formatter, "invalid root closure limit: {source}")
            }
            Self::BlobId { index, source } => {
                write!(
                    formatter,
                    "invalid BlobId in retention anchor {index}: {source}"
                )
            }
            Self::LayoutId { index, source } => {
                write!(
                    formatter,
                    "invalid LayoutId in retention anchor {index}: {source}"
                )
            }
            Self::NonCanonicalAnchorOrder { index, .. } => write!(
                formatter,
                "retention anchor {index} is not greater than its predecessor"
            ),
            Self::Allocation { .. } => {
                formatter.write_str("retention root anchor allocation failed")
            }
            Self::AnchorSetDigestMismatch { .. } => {
                formatter.write_str("retention root anchor-set digest mismatch")
            }
            Self::RootDigestMismatch { .. } => {
                formatter.write_str("retention root digest mismatch")
            }
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("retention root checksum mismatch")
            }
            Self::Semantic { source } => write!(formatter, "invalid semantic root: {source}"),
        }
    }
}

impl Error for RetentionRootDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Generation { source } => Some(source),
            Self::Namespace { source } => Some(source),
            Self::Profile { source } => Some(source),
            Self::ClosureLimit { source } => Some(source),
            Self::BlobId { source, .. } => Some(source),
            Self::LayoutId { source, .. } => Some(source),
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
            | Self::InvalidAnchorWidth { .. }
            | Self::NonZeroReserved { .. }
            | Self::AnchorCountExceeded { .. }
            | Self::NonCanonicalAnchorOrder { .. }
            | Self::AnchorSetDigestMismatch { .. }
            | Self::RootDigestMismatch { .. }
            | Self::ChecksumMismatch { .. } => None,
        }
    }
}
