//! This boundary module owns typed refusals from filesystem current-state verification.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RetentionHeadDecodeError, RetentionManifestDecodeError};
use crate::{CatalogGeneration, LivenessGeneration, RetentionManifestDigest};

/// Exact reason filesystem current-state verification refused a transition.
///
/// Every variant is carried as the source of the `io::Error` that
/// [`RetentionPublicationStorage::verify_current`][verify] returns, so callers
/// can distinguish a lawful stale state that should be replanned from
/// corruption or ambiguity that must route through recovery.
///
/// [verify]: super::RetentionPublicationStorage::verify_current
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionCurrentStateRefusal {
    /// A retained `root.next`, `manifest.next`, or `head.next` exists.
    RetainedStage,
    /// `retention/HEAD` is absent while a pool holds artifacts.
    HeadAbsentWithArtifacts,
    /// `retention/HEAD` is absent but a current generation was expected.
    ExpectedCurrentOverAbsentHead,
    /// `retention/HEAD` is absent but the prepared head is not an initial head.
    NonInitialOverAbsentHead,
    /// The published head refused admission.
    HeadRefused {
        /// The exact decode refusal.
        source: RetentionHeadDecodeError,
    },
    /// The prepared successor head refused admission.
    PreparedHeadRefused {
        /// The exact decode refusal.
        source: RetentionHeadDecodeError,
    },
    /// The head names a manifest that is absent from the pool.
    ManifestAbsent,
    /// The head-selected manifest refused admission.
    ManifestRefused {
        /// The exact decode refusal.
        source: RetentionManifestDecodeError,
    },
    /// The head-selected manifest disagrees with the head's digest or generation.
    ManifestDisagreed,
    /// The head's predecessor digest disagrees with its manifest's predecessor.
    HeadPredecessorDisagreed,
    /// The store's catalog head is not the catalog the closure was verified against.
    CatalogDisagreed {
        /// The catalog generation the closure was verified against.
        expected_generation: CatalogGeneration,
        /// The catalog generation the store's head names, if it decoded.
        observed_generation: Option<CatalogGeneration>,
    },
    /// The store's catalog head refused admission.
    CatalogHeadRefused,
    /// The current liveness generation has no successor.
    LivenessExhausted,
    /// A byte-identical retry found that another successor is current.
    StaleCommittedRetry,
    /// The prepared successor does not name the current head as its predecessor.
    Superseded {
        /// The generation the current head names.
        current_generation: LivenessGeneration,
        /// The manifest digest the current head names.
        current_digest: RetentionManifestDigest,
    },
    /// The committed manifest carries no entry for the candidate namespace.
    CommittedSelectionMissing,
    /// The committed manifest selects a different root for the candidate namespace.
    CommittedSelectionMismatch,
    /// The committed namespace directory cannot be opened.
    CommittedNamespaceUnavailable,
    /// The committed root pool entry is absent.
    CommittedRootAbsent,
    /// The committed root pool entry holds different bytes.
    CommittedRootChanged,
    /// The candidate does not name the manifest's current root as its predecessor.
    PredecessorMismatch,
    /// The predecessor root pool entry is absent or exceeds the format bound.
    PredecessorRootAbsent,
    /// The predecessor root pool entry does not decode to the manifest's selection.
    PredecessorRootChanged,
    /// A record's kind or length disagreed with its declaration.
    RecordKindOrLength,
    /// A record carried bytes beyond its declared length.
    RecordTrailingBytes,
    /// A declared record length exceeded the platform's addressable range.
    RecordLengthOverflow,
}

impl RetentionCurrentStateRefusal {
    /// Wraps the refusal as the `InvalidData` error the storage port returns.
    pub(super) fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}

impl fmt::Display for RetentionCurrentStateRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetainedStage => {
                formatter.write_str("retained retention stage requires recovery before publication")
            }
            Self::HeadAbsentWithArtifacts => formatter.write_str(
                "retention head is absent while retention pools hold artifacts; recovery is \
                 required",
            ),
            Self::ExpectedCurrentOverAbsentHead => formatter
                .write_str("expected a current retention generation but no head is published"),
            Self::NonInitialOverAbsentHead => formatter.write_str(
                "absent retention head admits only an initial publication with no predecessor",
            ),
            Self::HeadRefused { .. } => {
                formatter.write_str("current retention head refused admission")
            }
            Self::PreparedHeadRefused { .. } => {
                formatter.write_str("prepared retention head refused admission")
            }
            Self::ManifestAbsent => {
                formatter.write_str("current retention head names an absent manifest")
            }
            Self::ManifestRefused { .. } => {
                formatter.write_str("current retention manifest refused admission")
            }
            Self::ManifestDisagreed => {
                formatter.write_str("current retention manifest disagreed with its head")
            }
            Self::HeadPredecessorDisagreed => formatter
                .write_str("current retention head and its manifest name different predecessors"),
            Self::CatalogDisagreed {
                expected_generation,
                ..
            } => write!(
                formatter,
                "closure was verified against catalog generation {} which is not this store's \
                 current catalog",
                expected_generation.get()
            ),
            Self::CatalogHeadRefused => {
                formatter.write_str("this store's catalog head refused admission")
            }
            Self::LivenessExhausted => {
                formatter.write_str("current liveness generation cannot advance")
            }
            Self::StaleCommittedRetry => formatter
                .write_str("already-committed retry is stale: another successor is current"),
            Self::Superseded {
                current_generation, ..
            } => write!(
                formatter,
                "candidate is superseded: the current head is liveness generation {}",
                current_generation.get()
            ),
            Self::CommittedSelectionMissing => {
                formatter.write_str("committed manifest does not select the candidate namespace")
            }
            Self::CommittedSelectionMismatch => formatter.write_str(
                "committed manifest selects a different root for the candidate namespace",
            ),
            Self::CommittedNamespaceUnavailable => {
                formatter.write_str("committed root namespace directory is unavailable")
            }
            Self::CommittedRootAbsent => formatter.write_str("committed root pool entry is absent"),
            Self::CommittedRootChanged => {
                formatter.write_str("committed root pool entry bytes disagreed")
            }
            Self::PredecessorMismatch => {
                formatter.write_str("candidate does not name the current root as its predecessor")
            }
            Self::PredecessorRootAbsent => formatter
                .write_str("predecessor root pool entry is absent or exceeds the format bound"),
            Self::PredecessorRootChanged => formatter.write_str(
                "predecessor root pool entry does not decode to the manifest's selection",
            ),
            Self::RecordKindOrLength => {
                formatter.write_str("retention record kind or length disagreed")
            }
            Self::RecordTrailingBytes => {
                formatter.write_str("retention record carried trailing bytes")
            }
            Self::RecordLengthOverflow => {
                formatter.write_str("retention record length exceeded the addressable range")
            }
        }
    }
}

impl Error for RetentionCurrentStateRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HeadRefused { source } | Self::PreparedHeadRefused { source } => Some(source),
            Self::ManifestRefused { source } => Some(source),
            _ => None,
        }
    }
}
