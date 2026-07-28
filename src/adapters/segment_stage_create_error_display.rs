//! Human-readable exclusive segment-stage creation failures.

use std::error::Error;
use std::fmt;

use super::SegmentStageCreateError;

impl fmt::Display for SegmentStageCreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create { source } => {
                write!(
                    formatter,
                    "could not exclusively create segment stage: {source}"
                )
            }
        }
    }
}

impl Error for SegmentStageCreateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Create { source } => Some(source),
        }
    }
}
