//! This module owns semantic refusals of exact retention records and of the
//! adapter-retained publication or recovery state those records move through.

use std::error::Error;
use std::fmt;

use super::RetentionFixedStage;

/// Why a retention record cannot be used for a storage transition.
///
/// Carried either as [`RetentionStorageError::Refused`][refused] or as the
/// `InvalidData` payload of a storage port's `io::Error`.
///
/// [refused]: super::RetentionStorageError::Refused
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
    /// A publication phase ran without an attempt admitted by current-state
    /// verification.
    PublicationAttemptAbsent,
    /// A publication phase needs the root namespace the attempt has not admitted.
    NamespaceNotAdmitted,
    /// A publication phase needs a pool coordinate the attempt has not retained.
    PoolCoordinateNotRetained {
        /// The record whose pool coordinate is missing.
        stage: RetentionFixedStage,
    },
    /// A publication phase needs a stage handle the attempt has not retained.
    StageNotRetained {
        /// The stage whose handle is missing.
        stage: RetentionFixedStage,
    },
    /// A recovery step ran without an admitted recovery context.
    RecoveryAbsent,
    /// A recovery step needs a complete stage the context does not hold.
    RecoveryStageNotComplete {
        /// The stage the step required.
        stage: RetentionFixedStage,
    },
    /// A complete recovered root stage carries no namespace coordinate.
    RecoveredRootNamespaceAbsent,
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
            Self::PublicationAttemptAbsent => "no admitted retention publication attempt",
            Self::NamespaceNotAdmitted => "retention root namespace was not admitted",
            Self::PoolCoordinateNotRetained { .. } => "retention pool coordinate was not retained",
            Self::StageNotRetained { .. } => "retention stage was not retained",
            Self::RecoveryAbsent => "no retention recovery is in progress",
            Self::RecoveryStageNotComplete { .. } => "recovery step expected a complete stage",
            Self::RecoveredRootNamespaceAbsent => "recovered root stage has no namespace",
        })
    }
}

impl Error for RetentionRecordRefusal {}
