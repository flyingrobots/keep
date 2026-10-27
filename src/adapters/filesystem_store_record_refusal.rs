//! This module owns the typed refusal behind store record admission.

use std::error::Error;
use std::fmt;
use std::io;

use super::StoreFormatMarkerDecodeError;

/// Why the `FORMAT` record refused admission.
///
/// Carried as the source of the `InvalidData` error behind
/// [`FilesystemStoreAdmissionError::FormatMarker`](super::FilesystemStoreAdmissionError::FormatMarker),
/// so a caller can recover the decoder's own
/// diagnosis instead of a rendered string.
#[derive(Debug)]
#[non_exhaustive]
pub enum StoreRecordRefusal {
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
    /// `FORMAT` did not decode as a canonical format marker.
    Marker {
        /// The exact decode refusal.
        source: StoreFormatMarkerDecodeError,
    },
}

impl StoreRecordRefusal {
    /// Wraps the refusal as the `InvalidData` error record admission returns.
    pub(super) fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}

impl fmt::Display for StoreRecordRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow { name } => write!(
                formatter,
                "store record {name} has a canonical length beyond the addressable range"
            ),
            Self::KindOrLength { name } => {
                write!(
                    formatter,
                    "store record {name} has the wrong kind or length"
                )
            }
            Self::TrailingBytes { name } => {
                write!(formatter, "store record {name} carried trailing bytes")
            }
            Self::Marker { .. } => formatter.write_str("store record FORMAT refused admission"),
        }
    }
}

impl Error for StoreRecordRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Marker { source } => Some(source),
            Self::LengthOverflow { .. }
            | Self::KindOrLength { .. }
            | Self::TrailingBytes { .. } => None,
        }
    }
}
