//! This boundary module owns GC retirement intent error formatting.

use std::error::Error;
use std::fmt;

use super::{GcRetirementIntentDecodeError, GcRetirementIntentEncodeError};

impl fmt::Display for GcRetirementIntentDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { expected, observed } => write!(
                formatter,
                "GC intent requires {expected} bytes, observed {observed}"
            ),
            Self::TrailingData { expected, observed } => write!(
                formatter,
                "GC intent carries {observed} bytes after its exact {expected}"
            ),
            Self::InvalidMagic { .. } => formatter.write_str("invalid GC intent magic"),
            Self::UnsupportedVersion { expected, observed } => write!(
                formatter,
                "unsupported GC intent version {observed}; expected {expected}"
            ),
            Self::InvalidHeaderLength { expected, observed } => write!(
                formatter,
                "GC intent header length {observed}; expected {expected}"
            ),
            Self::UnsupportedFlags { observed } => {
                write!(formatter, "unsupported GC intent flags {observed:#010x}")
            }
            Self::DeclaredLengthMismatch { expected, observed } => write!(
                formatter,
                "GC intent declares {observed} bytes; canonical length is {expected}"
            ),
            Self::LengthOverflow => formatter.write_str("GC intent length arithmetic overflowed"),
            Self::InvalidCandidateWidth { expected, observed } => write!(
                formatter,
                "GC intent candidate width {observed}; expected {expected}"
            ),
            Self::NonZeroReserved { field } => {
                write!(formatter, "GC intent reserved {field} bytes are nonzero")
            }
            Self::CandidateCountExceeded { maximum, observed } => write!(
                formatter,
                "GC intent declares {observed} candidates; maximum {maximum}"
            ),
            Self::Generation { .. } => formatter.write_str("GC intent generation refused"),
            Self::LivenessGeneration { .. } => {
                formatter.write_str("GC intent liveness generation refused")
            }
            Self::CatalogGeneration { .. } => {
                formatter.write_str("GC intent catalog generation refused")
            }
            Self::Profile { .. } => formatter.write_str("GC intent realization profile refused"),
            Self::Allocation { .. } => formatter.write_str("GC intent allocation refused"),
            Self::CandidateSetDigestMismatch { .. } => {
                formatter.write_str("GC intent candidate-set digest mismatch")
            }
            Self::IntentDigestMismatch { .. } => formatter.write_str("GC intent digest mismatch"),
            Self::ChecksumMismatch { .. } => formatter.write_str("GC intent checksum mismatch"),
            Self::Semantic { .. } => formatter.write_str("GC intent semantic admission refused"),
        }
    }
}

impl Error for GcRetirementIntentDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Generation { source } => Some(source),
            Self::LivenessGeneration { source } => Some(source),
            Self::CatalogGeneration { source } => Some(source),
            Self::Profile { source } => Some(source),
            Self::Allocation { source } => Some(source),
            Self::Semantic { source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for GcRetirementIntentEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => formatter.write_str("GC intent length arithmetic overflowed"),
            Self::Allocation { .. } => formatter.write_str("GC intent allocation refused"),
        }
    }
}

impl Error for GcRetirementIntentEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source } => Some(source),
            Self::LengthOverflow => None,
        }
    }
}
