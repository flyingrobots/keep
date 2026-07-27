//! Diagnostics and source chains for layout decoding failures.

use std::error::Error;
use std::fmt;

use super::LayoutDecodeError;

impl fmt::Display for LayoutDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { expected, observed } => {
                length_error("layout header", expected, observed, formatter)
            }
            Self::InvalidMagic { .. } => formatter.write_str("invalid layout record magic"),
            Self::UnsupportedFormatVersion { expected, observed } => {
                unsupported("layout format version", expected, observed, formatter)
            }
            Self::UnsupportedCodec { expected, observed } => {
                unsupported("layout codec", expected, observed, formatter)
            }
            Self::UnknownFlags { expected, observed } => {
                unknown_flags(*expected, *observed, formatter)
            }
            Self::WrongHeaderLength { expected, observed } => {
                wrong_value("layout header length", expected, observed, formatter)
            }
            Self::WrongEntryLength { expected, observed } => {
                wrong_value("layout entry length", expected, observed, formatter)
            }
            Self::UnsupportedChecksumAlgorithm { expected, observed } => {
                unsupported("layout checksum algorithm", expected, observed, formatter)
            }
            Self::UnsupportedChunkHashAlgorithm { expected, observed } => {
                unsupported("chunk hash algorithm", expected, observed, formatter)
            }
            Self::UnsupportedChunkIdentityVersion { expected, observed } => {
                unsupported("chunk identity version", expected, observed, formatter)
            }
            Self::NonzeroReserved {
                offset, observed, ..
            } => reserved_byte(*offset, *observed, formatter),
            Self::EntryCountLimitExceeded { maximum, observed } => {
                entry_limit(*maximum, *observed, "protocol", formatter)
            }
            Self::ConfiguredEntryLimitExceeded { maximum, observed } => {
                entry_limit(*maximum, *observed, "configured", formatter)
            }
            Self::RecordLengthLimitExceeded { maximum, observed } => {
                record_limit(*maximum, *observed, formatter)
            }
            Self::RecordLengthArithmetic { entry_count } => {
                record_arithmetic(*entry_count, formatter)
            }
            Self::RecordLengthMismatch { expected, observed } => {
                wrong_value("layout record length", expected, observed, formatter)
            }
            Self::EntryCountLengthMismatch {
                entry_count,
                expected,
                observed,
            } => entry_count_length(*entry_count, *expected, *observed, formatter),
            Self::TruncatedRecord { expected, observed } => {
                length_error("layout record", expected, observed, formatter)
            }
            Self::TrailingData { expected, observed } => {
                trailing_data(*expected, *observed, formatter)
            }
            Self::HostRecordLengthOutOfRange { observed, .. } => {
                host_width("layout record length", observed, formatter)
            }
            Self::ChecksumMismatch { .. } => formatter.write_str("layout checksum mismatch"),
            Self::BlobId { source } => write!(formatter, "invalid layout target: {source}"),
            Self::UnsupportedStorageProfileVersion { expected, observed } => {
                unsupported("storage-profile version", expected, observed, formatter)
            }
            Self::UnsupportedStorageProfileAlgorithm { expected, observed } => {
                unsupported("storage-profile algorithm", expected, observed, formatter)
            }
            Self::StorageProfile { source } => {
                write!(
                    formatter,
                    "layout storage profile was not admitted: {source}"
                )
            }
            Self::EntryCountHostWidth { observed, .. } => {
                host_width("layout entry count", observed, formatter)
            }
            Self::Allocation { requested, .. } => {
                write!(formatter, "allocation of {requested} layout entries failed")
            }
            Self::ZeroChunkLength { index } => {
                write!(formatter, "layout entry {index} has zero chunk length")
            }
            Self::Validation { source } => write!(formatter, "invalid layout: {source}"),
            Self::LayoutIdentity { source } => {
                write!(formatter, "layout identity mismatch: {source}")
            }
        }
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
