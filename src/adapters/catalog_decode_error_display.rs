//! Human-readable catalog decoding diagnostics.

use std::error::Error;
use std::fmt;

use super::CatalogDecodeError;

impl fmt::Display for CatalogDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinimumLength { minimum, observed } => {
                display_minimum_length(formatter, *minimum, *observed)
            }
            Self::InvalidMagic { .. } => formatter.write_str("invalid catalog magic"),
            Self::UnsupportedVersion { expected, observed } => {
                write!(
                    formatter,
                    "unsupported catalog version {observed}; version {expected} is required"
                )
            }
            Self::Flags { expected, observed } => {
                write!(
                    formatter,
                    "noncanonical catalog flags: expected {expected}, observed {observed}"
                )
            }
            Self::HeaderLength { expected, observed } => {
                write!(
                    formatter,
                    "wrong catalog header length: expected {expected}, observed {observed}"
                )
            }
            Self::EntryLength { expected, observed } => {
                write!(
                    formatter,
                    "wrong catalog entry length: expected {expected}, observed {observed}"
                )
            }
            Self::Generation { source } => {
                write!(formatter, "invalid catalog generation: {source}")
            }
            Self::UnexpectedPredecessor { generation, .. } => {
                write!(
                    formatter,
                    "catalog generation {generation} forbids a predecessor"
                )
            }
            Self::MissingPredecessor { generation } => {
                write!(
                    formatter,
                    "catalog generation {generation} requires a predecessor"
                )
            }
            Self::EntryCountOutOfBounds { maximum, observed } => {
                write!(
                    formatter,
                    "catalog entry count {observed} exceeds {maximum}"
                )
            }
            Self::CatalogLength { source } => write!(formatter, "invalid catalog length: {source}"),
            Self::EntryCountLengthMismatch {
                entry_count,
                expected,
                observed,
            } => write!(
                formatter,
                "catalog count {entry_count} requires length {expected}, observed {observed}"
            ),
            Self::ObservedLength { declared, observed } => {
                write!(
                    formatter,
                    "catalog declares {declared} bytes, observed {observed}"
                )
            }
            Self::LengthArithmetic { entry_count } => {
                write!(
                    formatter,
                    "catalog length arithmetic failed for {entry_count} entries"
                )
            }
            Self::HashLength { observed } => display_hash_length(formatter, *observed),
            Self::ChecksumAlgorithm { expected, observed } => {
                display_algorithm(formatter, "checksum", *expected, *observed)
            }
            Self::DigestAlgorithm { expected, observed } => {
                display_algorithm(formatter, "digest", *expected, *observed)
            }
            Self::Reserved { .. } => formatter.write_str("nonzero catalog header reserved bytes"),
            Self::Entry { index, source } => {
                write!(formatter, "invalid catalog entry {index}: {source}")
            }
            Self::DuplicateIdentity {
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "catalog identity at {duplicate_index} duplicates entry {first_index}"
            ),
            Self::IdentityOrder {
                previous_index,
                observed_index,
            } => write!(
                formatter,
                "catalog identity at {observed_index} precedes entry {previous_index}"
            ),
            Self::ChecksumMismatch { .. } => formatter.write_str("catalog checksum mismatch"),
            Self::DigestMismatch { .. } => formatter.write_str("catalog digest mismatch"),
        }
    }
}

fn display_minimum_length(
    formatter: &mut fmt::Formatter<'_>,
    minimum: usize,
    observed: usize,
) -> fmt::Result {
    write!(
        formatter,
        "catalog requires at least {minimum} bytes, observed {observed}"
    )
}

fn display_hash_length(formatter: &mut fmt::Formatter<'_>, observed: usize) -> fmt::Result {
    write!(
        formatter,
        "catalog hash length cannot represent {observed} bytes"
    )
}

fn display_algorithm(
    formatter: &mut fmt::Formatter<'_>,
    field: &str,
    expected: u8,
    observed: u8,
) -> fmt::Result {
    write!(
        formatter,
        "unsupported catalog {field} algorithm {observed}; algorithm {expected} is required"
    )
}

impl Error for CatalogDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Generation { source } => Some(source),
            Self::CatalogLength { source } => Some(source),
            Self::Entry { source, .. } => Some(source),
            _ => None,
        }
    }
}
