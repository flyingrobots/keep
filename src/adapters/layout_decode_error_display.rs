//! Diagnostics and source chains for layout decoding failures.

use std::error::Error;
use std::fmt;

use super::LayoutDecodeError;

enum DisplayGroup {
    Header,
    Record,
    Admission,
}

impl fmt::Display for LayoutDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match display_group(self) {
            DisplayGroup::Header => format_header(self, formatter),
            DisplayGroup::Record => format_record(self, formatter),
            DisplayGroup::Admission => format_admission(self, formatter),
        }
    }
}

const fn display_group(error: &LayoutDecodeError) -> DisplayGroup {
    match error {
        LayoutDecodeError::TruncatedHeader { .. }
        | LayoutDecodeError::InvalidMagic { .. }
        | LayoutDecodeError::UnsupportedFormatVersion { .. }
        | LayoutDecodeError::UnsupportedCodec { .. }
        | LayoutDecodeError::UnknownFlags { .. }
        | LayoutDecodeError::WrongHeaderLength { .. }
        | LayoutDecodeError::WrongEntryLength { .. }
        | LayoutDecodeError::UnsupportedChecksumAlgorithm { .. }
        | LayoutDecodeError::UnsupportedChunkHashAlgorithm { .. }
        | LayoutDecodeError::UnsupportedChunkIdentityVersion { .. }
        | LayoutDecodeError::NonzeroReserved { .. } => DisplayGroup::Header,
        LayoutDecodeError::EntryCountLimitExceeded { .. }
        | LayoutDecodeError::ConfiguredEntryLimitExceeded { .. }
        | LayoutDecodeError::RecordLengthLimitExceeded { .. }
        | LayoutDecodeError::RecordLengthArithmetic { .. }
        | LayoutDecodeError::RecordLengthMismatch { .. }
        | LayoutDecodeError::EntryCountLengthMismatch { .. }
        | LayoutDecodeError::TruncatedRecord { .. }
        | LayoutDecodeError::TrailingData { .. }
        | LayoutDecodeError::HostRecordLengthOutOfRange { .. }
        | LayoutDecodeError::ChecksumMismatch { .. } => DisplayGroup::Record,
        LayoutDecodeError::BlobId { .. }
        | LayoutDecodeError::UnsupportedStorageProfileVersion { .. }
        | LayoutDecodeError::UnsupportedStorageProfileAlgorithm { .. }
        | LayoutDecodeError::StorageProfile { .. }
        | LayoutDecodeError::EntryCountHostWidth { .. }
        | LayoutDecodeError::Allocation { .. }
        | LayoutDecodeError::ZeroChunkLength { .. }
        | LayoutDecodeError::Validation { .. }
        | LayoutDecodeError::LayoutIdentity { .. } => DisplayGroup::Admission,
    }
}

fn format_header(error: &LayoutDecodeError, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        LayoutDecodeError::TruncatedHeader { expected, observed } => {
            length_error("layout header", expected, observed, formatter)
        }
        LayoutDecodeError::InvalidMagic { .. } => {
            formatter.write_str("invalid layout record magic")
        }
        LayoutDecodeError::UnsupportedFormatVersion { expected, observed } => {
            unsupported("layout format version", expected, observed, formatter)
        }
        LayoutDecodeError::UnsupportedCodec { expected, observed } => {
            unsupported("layout codec", expected, observed, formatter)
        }
        LayoutDecodeError::UnknownFlags { expected, observed } => {
            unknown_flags(*expected, *observed, formatter)
        }
        LayoutDecodeError::WrongHeaderLength { expected, observed } => {
            wrong_value("layout header length", expected, observed, formatter)
        }
        LayoutDecodeError::WrongEntryLength { expected, observed } => {
            wrong_value("layout entry length", expected, observed, formatter)
        }
        LayoutDecodeError::UnsupportedChecksumAlgorithm { expected, observed } => {
            unsupported("layout checksum algorithm", expected, observed, formatter)
        }
        LayoutDecodeError::UnsupportedChunkHashAlgorithm { expected, observed } => {
            unsupported("chunk hash algorithm", expected, observed, formatter)
        }
        LayoutDecodeError::UnsupportedChunkIdentityVersion { expected, observed } => {
            unsupported("chunk identity version", expected, observed, formatter)
        }
        LayoutDecodeError::NonzeroReserved {
            offset, observed, ..
        } => reserved_byte(*offset, *observed, formatter),
        _ => Err(fmt::Error),
    }
}

fn format_record(error: &LayoutDecodeError, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        LayoutDecodeError::EntryCountLimitExceeded { maximum, observed } => {
            entry_limit(*maximum, *observed, "protocol", formatter)
        }
        LayoutDecodeError::ConfiguredEntryLimitExceeded { maximum, observed } => {
            entry_limit(*maximum, *observed, "configured", formatter)
        }
        LayoutDecodeError::RecordLengthLimitExceeded { maximum, observed } => {
            record_limit(*maximum, *observed, formatter)
        }
        LayoutDecodeError::RecordLengthArithmetic { entry_count } => {
            record_arithmetic(*entry_count, formatter)
        }
        LayoutDecodeError::RecordLengthMismatch { expected, observed } => {
            wrong_value("layout record length", expected, observed, formatter)
        }
        LayoutDecodeError::EntryCountLengthMismatch {
            entry_count,
            expected,
            observed,
        } => entry_count_length(*entry_count, *expected, *observed, formatter),
        LayoutDecodeError::TruncatedRecord { expected, observed } => {
            length_error("layout record", expected, observed, formatter)
        }
        LayoutDecodeError::TrailingData { expected, observed } => {
            trailing_data(*expected, *observed, formatter)
        }
        LayoutDecodeError::HostRecordLengthOutOfRange { observed, .. } => {
            host_width("layout record length", observed, formatter)
        }
        LayoutDecodeError::ChecksumMismatch { .. } => {
            formatter.write_str("layout checksum mismatch")
        }
        _ => Err(fmt::Error),
    }
}

fn format_admission(error: &LayoutDecodeError, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        LayoutDecodeError::BlobId { source } => {
            write!(formatter, "invalid layout target: {source}")
        }
        LayoutDecodeError::UnsupportedStorageProfileVersion { expected, observed } => {
            unsupported("storage-profile version", expected, observed, formatter)
        }
        LayoutDecodeError::UnsupportedStorageProfileAlgorithm { expected, observed } => {
            unsupported("storage-profile algorithm", expected, observed, formatter)
        }
        LayoutDecodeError::StorageProfile { source } => {
            write!(
                formatter,
                "layout storage profile was not admitted: {source}"
            )
        }
        LayoutDecodeError::EntryCountHostWidth { observed, .. } => {
            host_width("layout entry count", observed, formatter)
        }
        LayoutDecodeError::Allocation { requested, .. } => {
            write!(formatter, "allocation of {requested} layout entries failed")
        }
        LayoutDecodeError::ZeroChunkLength { index } => {
            write!(formatter, "layout entry {index} has zero chunk length")
        }
        LayoutDecodeError::Validation { source } => write!(formatter, "invalid layout: {source}"),
        LayoutDecodeError::LayoutIdentity { source } => {
            write!(formatter, "layout identity mismatch: {source}")
        }
        _ => Err(fmt::Error),
    }
}

fn unsupported<T: fmt::Display>(
    field: &str,
    expected: T,
    observed: T,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(
        formatter,
        "unsupported {field} {observed}; expected {expected}"
    )
}

fn unknown_flags(expected: u32, observed: u32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        formatter,
        "unsupported layout flags {observed:#010x}; expected {expected}"
    )
}

fn reserved_byte(offset: usize, observed: u8, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        formatter,
        "layout reserved byte at {offset} is {observed}; expected 0"
    )
}

fn record_arithmetic(entry_count: u32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        formatter,
        "layout record length arithmetic failed for {entry_count} entries"
    )
}

fn entry_count_length(
    entry_count: u32,
    expected: u64,
    observed: u64,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(
        formatter,
        "layout entry count {entry_count} calculates length {expected}, observed {observed}"
    )
}

fn trailing_data(
    expected: u64,
    observed: usize,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(
        formatter,
        "layout record has trailing data: expected {expected} bytes, observed {observed}"
    )
}

fn wrong_value<T: fmt::Display>(
    field: &str,
    expected: T,
    observed: T,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(formatter, "{field} {observed}; expected {expected}")
}

fn exceeds<T: fmt::Display>(
    field: &str,
    maximum: T,
    observed: T,
    bound: &str,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(
        formatter,
        "{field} {observed} exceeds {bound} maximum {maximum}"
    )
}

fn entry_limit(
    maximum: u32,
    observed: u32,
    bound: &str,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    exceeds("layout entry count", maximum, observed, bound, formatter)
}

fn record_limit(maximum: u64, observed: u64, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    exceeds(
        "layout record length",
        maximum,
        observed,
        "protocol",
        formatter,
    )
}

fn host_width<T: fmt::Display>(
    field: &str,
    observed: T,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(formatter, "{field} {observed} exceeds host width")
}

fn length_error<T: fmt::Display, U: fmt::Display>(
    field: &str,
    expected: T,
    observed: U,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(
        formatter,
        "{field} needs {expected} bytes, observed {observed}"
    )
}

impl Error for LayoutDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HostRecordLengthOutOfRange { source, .. }
            | Self::EntryCountHostWidth { source, .. } => Some(source),
            Self::Allocation { source, .. } => Some(source),
            Self::BlobId { source } => Some(source),
            Self::StorageProfile { source } => Some(source),
            Self::Validation { source } => Some(source),
            Self::LayoutIdentity { source } => Some(source),
            _ => None,
        }
    }
}
