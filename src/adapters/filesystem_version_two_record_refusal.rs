//! This module owns the typed refusal behind version-two record admission.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    StoreFormatMarkerDecodeError, StoreMigrationIntentDecodeError, StoreMigrationReceiptDecodeError,
};

/// Why `FORMAT`, `migration.intent`, or `migration.receipt` refused admission.
///
/// Carried as the source of the `InvalidData` error behind
/// [`FilesystemPlatformAdmissionError::MigrationRecord`](super::FilesystemPlatformAdmissionError::MigrationRecord),
/// so a caller can tell which record refused and recover the decoder's own
/// diagnosis instead of a rendered string.
#[derive(Debug)]
#[non_exhaustive]
pub enum VersionTwoRecordRefusal {
    /// The record's canonical length exceeds the platform's addressable range.
    LengthOverflow {
        /// The record's fixed name.
        name: &'static str,
    },
    /// The record is not a regular file of its canonical length.
    KindOrLength {
        /// The record's fixed name.
        name: &'static str,
    },
    /// The record carried bytes beyond its canonical length.
    TrailingBytes {
        /// The record's fixed name.
        name: &'static str,
    },
    /// `FORMAT` did not decode as a canonical version-two marker.
    Marker {
        /// The exact decode refusal.
        source: StoreFormatMarkerDecodeError,
    },
    /// `migration.intent` did not decode as a canonical intent.
    Intent {
        /// The exact decode refusal.
        source: StoreMigrationIntentDecodeError,
    },
    /// `migration.receipt` did not admit against the decoded intent and marker.
    Receipt {
        /// The exact admission refusal.
        source: StoreMigrationReceiptDecodeError,
    },
}

impl VersionTwoRecordRefusal {
    /// Wraps the refusal as the `InvalidData` error record admission returns.
    pub(super) fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}

impl fmt::Display for VersionTwoRecordRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow { name } => write!(
                formatter,
                "version-two record {name} has a canonical length beyond the addressable range"
            ),
            Self::KindOrLength { name } => {
                write!(
                    formatter,
                    "version-two record {name} has the wrong kind or length"
                )
            }
            Self::TrailingBytes { name } => {
                write!(
                    formatter,
                    "version-two record {name} carried trailing bytes"
                )
            }
            Self::Marker { .. } => {
                formatter.write_str("version-two record FORMAT refused admission")
            }
            Self::Intent { .. } => {
                formatter.write_str("version-two record migration.intent refused admission")
            }
            Self::Receipt { .. } => {
                formatter.write_str("version-two record migration.receipt refused admission")
            }
        }
    }
}

impl Error for VersionTwoRecordRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Marker { source } => Some(source),
            Self::Intent { source } => Some(source),
            Self::Receipt { source } => Some(source),
            Self::LengthOverflow { .. }
            | Self::KindOrLength { .. }
            | Self::TrailingBytes { .. } => None,
        }
    }
}
