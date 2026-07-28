//! Human-readable complete immutable-segment admission failures.

use std::error::Error;
use std::fmt;

use super::SegmentReadError;

impl fmt::Display for SegmentReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(result) = display_outer(self, formatter) {
            return result;
        }
        if let Some(result) = display_record(self, formatter) {
            return result;
        }
        display_terminal(self, formatter).unwrap_or(Err(fmt::Error))
    }
}

impl Error for SegmentReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Header { source } => Some(source),
            Self::Seal { source } => Some(source),
            Self::IdentityIndexAllocation { source, .. } => Some(source),
            Self::RecordHeader { source, .. } => Some(source),
            Self::RecordDecode { source, .. } => Some(source),
            Self::RecordAdmission { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn display_outer(
    error: &SegmentReadError,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    let result = match error {
        SegmentReadError::WrongLength { minimum, observed } => write!(
            formatter,
            "complete segment requires at least {minimum} bytes, observed {observed}"
        ),
        SegmentReadError::Header { source } => {
            write!(formatter, "invalid complete segment header: {source}")
        }
        SegmentReadError::Seal { source } => {
            write!(formatter, "invalid complete segment seal: {source}")
        }
        SegmentReadError::RecordCountLimit { maximum, observed } => write!(
            formatter,
            "segment record count must not exceed configured limit {maximum}, observed {observed}"
        ),
        SegmentReadError::RecordCountHostWidth { observed } => write!(
            formatter,
            "segment record count {observed} cannot be represented on this host"
        ),
        SegmentReadError::IdentityIndexAllocation { record_count, .. } => write!(
            formatter,
            "could not reserve duplicate-detection index for {record_count} segment records"
        ),
        _ => return None,
    };
    Some(result)
}

fn display_record(
    error: &SegmentReadError,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    let result = match error {
        SegmentReadError::RecordHeaderTruncated {
            record_index,
            offset,
            required,
            observed,
        } => write!(
            formatter,
            "segment record {record_index} at offset {offset} requires {required} header bytes, \
             observed {observed}"
        ),
        SegmentReadError::RecordHeader {
            record_index,
            offset,
            source,
        } => write!(
            formatter,
            "invalid segment record {record_index} header at offset {offset}: {source}"
        ),
        SegmentReadError::RecordLengthHostWidth {
            record_index,
            observed,
        } => write!(
            formatter,
            "segment record {record_index} length {observed} cannot be represented on this host"
        ),
        SegmentReadError::RecordTruncated {
            record_index,
            offset,
            expected,
            observed,
        } => write!(
            formatter,
            "segment record {record_index} at offset {offset} requires {expected} bytes, \
             observed {observed}"
        ),
        SegmentReadError::RecordDecode {
            record_index,
            offset,
            source,
        } => write!(
            formatter,
            "invalid segment record {record_index} at offset {offset}: {source}"
        ),
        SegmentReadError::RecordAdmission {
            record_index,
            offset,
            source,
        } => write!(
            formatter,
            "untrusted segment record {record_index} at offset {offset}: {source}"
        ),
        _ => return None,
    };
    Some(result)
}

fn display_terminal(
    error: &SegmentReadError,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    let result = match error {
        SegmentReadError::OffsetArithmetic {
            record_index,
            offset,
            record_length,
        } => write!(
            formatter,
            "segment record {record_index} offset overflow: {offset} + {record_length}"
        ),
        SegmentReadError::RecordIndexArithmetic { record_index } => write!(
            formatter,
            "segment record index overflow after position {record_index}"
        ),
        SegmentReadError::RecordCountArithmetic {
            record_index,
            remaining,
        } => write!(
            formatter,
            "segment remaining-record count underflow at position {record_index} from {remaining}"
        ),
        SegmentReadError::TrailingRecordBytes { offset, observed } => write!(
            formatter,
            "segment has {observed} unexpected record bytes beginning at offset {offset}"
        ),
        SegmentReadError::DuplicateRecordIdentity {
            identity,
            first_index,
            duplicate_index,
            first_offset,
            duplicate_offset,
        } => write!(
            formatter,
            "segment record {duplicate_index} at offset {duplicate_offset} duplicates identity \
             {identity:?} from record {first_index} at offset {first_offset}"
        ),
        _ => return None,
    };
    Some(result)
}
