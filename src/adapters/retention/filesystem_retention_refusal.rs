//! This boundary module owns typed refusals from filesystem current-state verification.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RetentionHeadDecodeError, RetentionManifestDecodeError};
use super::{RetentionRecoveryError, RetentionRecoveryRefusal};
use crate::adapters::{CatalogDecodeError, PublicationHeadDecodeError};
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
    /// This store's catalog `HEAD` did not decode.
    CatalogHeadRefused {
        /// The exact decode refusal.
        source: PublicationHeadDecodeError,
    },
    /// The catalog pool entry this store's `HEAD` selects is absent.
    CatalogAbsent,
    /// The catalog pool entry this store's `HEAD` selects did not decode.
    CatalogRefused {
        /// The exact decode refusal.
        source: Box<CatalogDecodeError>,
    },
    /// The catalog pool entry decodes to a generation or digest other than the
    /// one `HEAD` names.
    CatalogChanged,
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
    /// The `retention` directory carries an entry outside `HEAD`, `roots`,
    /// and `manifests`.
    UnknownRetentionEntry,
    /// A `retention/roots` entry is not a 64-lowercase-hex directory.
    NonNamespaceEntry,
    /// A pool entry is not a regular `<generation>-<digest>` file with the
    /// pool's canonical suffix.
    NoncanonicalPoolEntry {
        /// The pool that carries the entry.
        pool: &'static str,
    },
    /// Admitting the candidate would exceed the namespace or pool ceiling.
    NamespaceCapacity,
    /// The candidate's namespace directory disagreed with the claimed
    /// expectation, at verification or when it was admitted between phases.
    NamespaceExpectationViolated,
    /// A later phase received a root whose namespace is not the one the
    /// attempt admitted.
    AttemptNamespaceDisagreed,
    /// A byte-identical already-committed retry was presented while no
    /// retention head is published, so nothing can have committed it.
    CommittedRetryOverAbsentHead,
    /// A protocol directory named at admission (`retention`, `roots`, or
    /// `manifests`) no longer names the pinned directory that was admitted.
    ProtocolDirectoryReplaced,
    /// Restart recovery refused the retained stages as unrecoverable ambiguity.
    RecoveryRefused {
        /// The exact planning refusal.
        source: RetentionRecoveryRefusal,
    },
    /// A restart recovery step refused; the completed prefix remains.
    RecoveryStepRefused {
        /// The refused step, the completed prefix, and the storage error.
        source: RetentionRecoveryError,
    },
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
            Self::CatalogDisagreed {
                expected_generation,
                ..
            } => write!(
                formatter,
                "closure was verified against catalog generation {} which is not this store's \
                 current catalog",
                expected_generation.get()
            ),
            Self::Superseded {
                current_generation, ..
            } => write!(
                formatter,
                "candidate is superseded: the current head is liveness generation {}",
                current_generation.get()
            ),
            Self::NoncanonicalPoolEntry { pool } => {
                write!(formatter, "retention {pool} carries a noncanonical entry")
            }
            _ => formatter.write_str(self.message()),
        }
    }
}

impl RetentionCurrentStateRefusal {
    /// The fixed description of every variant that renders no field.
    const fn message(&self) -> &'static str {
        match self {
            Self::RetainedStage => "retained retention stage requires recovery before publication",
            Self::HeadAbsentWithArtifacts => {
                "retention head is absent while retention pools hold artifacts; recovery is \
             required"
            }
            Self::ExpectedCurrentOverAbsentHead => {
                "expected a current retention generation but no head is published"
            }
            Self::NonInitialOverAbsentHead => {
                "absent retention head admits only an initial publication with no predecessor"
            }
            Self::HeadRefused { .. } => "current retention head refused admission",
            Self::PreparedHeadRefused { .. } => "prepared retention head refused admission",
            Self::ManifestAbsent => "current retention head names an absent manifest",
            Self::ManifestRefused { .. } => "current retention manifest refused admission",
            Self::ManifestDisagreed => "current retention manifest disagreed with its head",
            Self::HeadPredecessorDisagreed => {
                "current retention head and its manifest name different predecessors"
            }
            Self::CatalogAbsent => "this store's catalog head selects an absent catalog pool entry",
            Self::CatalogRefused { .. } => {
                "this store's selected catalog pool entry refused admission"
            }
            Self::CatalogChanged => {
                "this store's selected catalog pool entry names another generation or digest"
            }
            Self::CatalogHeadRefused { .. } => "this store's catalog head refused admission",
            Self::LivenessExhausted => "current liveness generation cannot advance",
            Self::StaleCommittedRetry => {
                "already-committed retry is stale: another successor is current"
            }
            Self::CommittedSelectionMissing => {
                "committed manifest does not select the candidate namespace"
            }
            Self::CommittedSelectionMismatch => {
                "committed manifest selects a different root for the candidate namespace"
            }
            Self::CommittedNamespaceUnavailable => {
                "committed root namespace directory is unavailable"
            }
            Self::CommittedRootAbsent => "committed root pool entry is absent",
            Self::CommittedRootChanged => "committed root pool entry bytes disagreed",
            Self::PredecessorMismatch => {
                "candidate does not name the current root as its predecessor"
            }
            Self::PredecessorRootAbsent => {
                "predecessor root pool entry is absent or exceeds the format bound"
            }
            Self::PredecessorRootChanged => {
                "predecessor root pool entry does not decode to the manifest's selection"
            }
            Self::UnknownRetentionEntry => "retention namespace carries an unknown entry",
            Self::NonNamespaceEntry => "retention roots carries a non-namespace entry",
            Self::NamespaceCapacity => "retention namespace or pool count would exceed its ceiling",
            Self::NamespaceExpectationViolated => {
                "namespace directory state disagreed with the claimed generation expectation"
            }
            Self::RecordKindOrLength => "retention record kind or length disagreed",
            Self::RecordTrailingBytes => "retention record carried trailing bytes",
            Self::RecordLengthOverflow => "retention record length exceeded the addressable range",
            Self::RecoveryRefused { .. } => {
                "restart recovery refused the retained retention stages"
            }
            Self::RecoveryStepRefused { .. } => "a restart recovery step refused",
            Self::ProtocolDirectoryReplaced => {
                "a retention protocol directory was replaced after admission"
            }
            Self::CommittedRetryOverAbsentHead => {
                "already-committed retry presented while no retention head is published"
            }
            Self::AttemptNamespaceDisagreed => {
                "root handed to a publication phase names a different namespace than admitted"
            }
            Self::CatalogDisagreed { .. } => {
                "closure was verified against a catalog that is not this store's current catalog"
            }
            Self::Superseded { .. } => "candidate is superseded by the current head",
            Self::NoncanonicalPoolEntry { .. } => "retention pool carries a noncanonical entry",
        }
    }
}

impl Error for RetentionCurrentStateRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HeadRefused { source } | Self::PreparedHeadRefused { source } => Some(source),
            Self::ManifestRefused { source } => Some(source),
            Self::CatalogHeadRefused { source } => Some(source),
            Self::CatalogRefused { source } => Some(source.as_ref()),
            Self::RecoveryRefused { source } => Some(source),
            Self::RecoveryStepRefused { source } => Some(source),
            _ => None,
        }
    }
}
