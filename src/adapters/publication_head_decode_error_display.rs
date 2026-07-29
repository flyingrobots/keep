//! Human-readable publication-head decoding diagnostics.

use std::error::Error;
use std::fmt;

use super::PublicationHeadDecodeError;

impl fmt::Display for PublicationHeadDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => write!(
                formatter,
                "wrong publication-head length: expected {expected}, observed {observed}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid publication-head magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported publication-head version {observed}; version {expected} is required"
            ),
            Self::Flags { expected, observed } => write!(
                formatter,
                "noncanonical publication-head flags: expected {expected}, observed {observed}"
            ),
            Self::HeadLength { expected, observed } => write!(
                formatter,
                "wrong publication-head length field: expected {expected}, observed {observed}"
            ),
            Self::ChecksumAlgorithm { expected, observed } => write!(
                formatter,
                "unsupported head checksum algorithm {observed}; algorithm {expected} is required"
            ),
            Self::DigestAlgorithm { expected, observed } => write!(
                formatter,
                "unsupported catalog digest algorithm {observed}; algorithm {expected} is required"
            ),
            Self::Generation { source } => write!(formatter, "invalid head generation: {source}"),
            Self::CatalogLength { source } => {
                write!(formatter, "invalid head catalog length: {source}")
            }
            Self::Reserved { .. } => formatter.write_str("nonzero publication-head reserved bytes"),
            Self::ChecksumMismatch { .. } => {
                formatter.write_str("publication-head checksum mismatch")
            }
        }
    }
}

impl Error for PublicationHeadDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Generation { source } => Some(source),
            Self::CatalogLength { source } => Some(source),
            _ => None,
        }
    }
}
