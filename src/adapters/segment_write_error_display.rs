//! Human-readable staged-segment writing and sealing failures.

use std::error::Error;
use std::fmt;

use super::SegmentWriteError;

impl fmt::Display for SegmentWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecordCountLimit { .. }
            | Self::RecordCountArithmetic { .. }
            | Self::DuplicateRecordIdentity { .. }
            | Self::IdentityIndexAllocation { .. }
            | Self::SegmentLengthArithmetic { .. }
            | Self::SegmentLengthLimit { .. } => display_policy(self, formatter),
            Self::InvalidWriteCount { .. }
            | Self::WriteZero { .. }
            | Self::WriteLengthArithmetic { .. }
            | Self::Write { .. } => display_write(self, formatter),
            Self::Flush { .. } | Self::Synchronize { .. } | Self::Seal { .. } => {
                display_durability(self, formatter)
            }
        }
    }
}

impl Error for SegmentWriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IdentityIndexAllocation { source, .. } => Some(source),
            Self::Write { source, .. }
            | Self::Flush { source, .. }
            | Self::Synchronize { source, .. } => Some(source),
            Self::Seal { source } => Some(source),
            _ => None,
        }
    }
}

fn display_policy(error: &SegmentWriteError, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        SegmentWriteError::RecordCountLimit { maximum, observed } => write!(
            formatter,
            "staged segment record count must not exceed {maximum}, attempted {observed}"
        ),
        SegmentWriteError::RecordCountArithmetic { observed } => {
            write!(
                formatter,
                "staged segment record count overflow after {observed}"
            )
        }
        SegmentWriteError::DuplicateRecordIdentity { identity } => {
            write!(formatter, "staged segment already contains {identity:?}")
        }
        SegmentWriteError::IdentityIndexAllocation { identity, .. } => {
            write!(
                formatter,
                "could not index staged segment identity {identity:?}"
            )
        }
        SegmentWriteError::SegmentLengthArithmetic {
            bytes_before_record,
            record_length,
        } => write!(
            formatter,
            "staged segment length overflow: {bytes_before_record} + record {record_length}"
        ),
        SegmentWriteError::SegmentLengthLimit { maximum, observed } => write!(
            formatter,
            "complete staged segment must not exceed {maximum} bytes, attempted {observed}"
        ),
        _ => Err(fmt::Error),
    }
}

fn display_write(error: &SegmentWriteError, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        SegmentWriteError::InvalidWriteCount {
            phase,
            maximum,
            observed,
            bytes_written,
        } => write!(
            formatter,
            "{phase:?} write reported {observed} bytes from at most {maximum} after \
             {bytes_written} staged bytes"
        ),
        SegmentWriteError::WriteZero {
            phase,
            bytes_written,
        } => write!(
            formatter,
            "{phase:?} write made no progress after {bytes_written} staged bytes"
        ),
        SegmentWriteError::WriteLengthArithmetic {
            phase,
            bytes_written,
            incoming,
        } => write!(
            formatter,
            "{phase:?} write offset overflow after {bytes_written} bytes plus {incoming}"
        ),
        SegmentWriteError::Write {
            phase,
            bytes_written,
            source,
        } => write!(
            formatter,
            "{phase:?} write failed after {bytes_written} staged bytes: {source}"
        ),
        _ => Err(fmt::Error),
    }
}

fn display_durability(
    error: &SegmentWriteError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        SegmentWriteError::Flush { phase, source } => {
            write!(formatter, "failed to flush {phase:?}: {source}")
        }
        SegmentWriteError::Synchronize { phase, source } => {
            write!(formatter, "failed to synchronize {phase:?}: {source}")
        }
        SegmentWriteError::Seal { source } => {
            write!(
                formatter,
                "failed to construct staged segment seal: {source}"
            )
        }
        _ => Err(fmt::Error),
    }
}
