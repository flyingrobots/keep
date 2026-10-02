//! This module owns semantic refusals of exact retention records.

use std::error::Error;
use std::fmt;

/// Why a retention record cannot be used for a storage transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetentionRecordRefusal {
    /// Automatic disposition of incomplete stages is not supported.
    IncompleteDispositionRequired,
    /// The expected byte length exceeds the filesystem's range.
    LengthOverflow,
    /// The entry is not a regular file of the expected length.
    KindOrLength,
    /// The entry's kind, length, or identity differs from the retained record.
    KindLengthOrIdentity,
    /// The record differs from the admitted bytes.
    Bytes,
    /// The record has bytes beyond its expected end.
    TrailingBytes,
    /// A removed entry remains visible.
    RemainedVisible,
}

impl fmt::Display for RetentionRecordRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IncompleteDispositionRequired => {
                "incomplete retention stage requires explicit disposition"
            }
            Self::LengthOverflow => "retention record length exceeded the filesystem's range",
            Self::KindOrLength => "retention record kind or length disagreed",
            Self::KindLengthOrIdentity => "retention record kind, length, or identity disagreed",
            Self::Bytes => "retention record bytes disagreed",
            Self::TrailingBytes => "retention record carried trailing bytes",
            Self::RemainedVisible => "removed retention record remained visible",
        })
    }
}

impl Error for RetentionRecordRefusal {}
