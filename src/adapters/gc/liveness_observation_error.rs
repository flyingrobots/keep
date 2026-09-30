//! This boundary module owns GC liveness observation refusals.

use std::error::Error;
use std::fmt;
use std::io;

use super::GcLivenessSnapshotError;
use crate::adapters::{CatalogDecodeError, CatalogRestartError, SegmentDigest, SegmentReadError};
use crate::{CatalogGeneration, RetentionClosureVerificationError, RetentionNamespaceDigest};

/// Failure to assemble one liveness snapshot from a fenced store view.
///
/// Every variant is a refusal: nothing is planned from a store whose evidence
/// could not be admitted in full.
#[derive(Debug)]
pub enum GcLivenessObservationError {
    /// The fenced catalog could not be re-admitted.
    Catalog {
        /// The exact restart refusal.
        source: CatalogRestartError,
    },
    /// The catalog's record-to-segment projection could not be decoded.
    CatalogEntries {
        /// The exact decode refusal.
        source: CatalogDecodeError,
    },
    /// The manifest names a root the roots pool does not hold.
    RetainedRootAbsent {
        /// The namespace whose root is missing.
        namespace: RetentionNamespaceDigest,
    },
    /// A retained root could not be read or decoded.
    RetainedRoot {
        /// The namespace.
        namespace: RetentionNamespaceDigest,
        /// The exact snapshot or decode refusal.
        source: Box<dyn Error + Send + Sync>,
    },
    /// A retained root's closure no longer verifies against the catalog.
    Closure {
        /// The namespace.
        namespace: RetentionNamespaceDigest,
        /// The exact closure refusal.
        source: Box<RetentionClosureVerificationError>,
    },
    /// A closure member resolved through the catalog has no catalog entry.
    ClosureMemberUnindexed {
        /// The namespace.
        namespace: RetentionNamespaceDigest,
    },
    /// The segment or catalog pool refused an I/O action.
    Pool {
        /// The action.
        action: &'static str,
        /// The original error.
        source: io::Error,
    },
    /// A pool entry is not named by a lowercase segment digest.
    PoolEntryName,
    /// A pool entry is not a regular file.
    PoolEntryKind {
        /// The entry's declared segment.
        segment: SegmentDigest,
    },
    /// Reading the pool would exceed the byte limit.
    PoolByteLimit {
        /// The limit.
        limit: u64,
    },
    /// A pool segment did not admit.
    SegmentAdmission {
        /// The segment named by the entry.
        segment: SegmentDigest,
        /// The exact segment refusal.
        source: Box<SegmentReadError>,
    },
    /// A pool segment admits but hashes to a different digest than its name.
    SegmentDigestMismatch {
        /// The name's digest.
        expected: SegmentDigest,
        /// The bytes' digest.
        observed: SegmentDigest,
    },
    /// A predecessor catalog in the chain could not be read or decoded.
    PredecessorCatalog {
        /// The predecessor generation.
        generation: CatalogGeneration,
        /// The exact refusal.
        source: Box<dyn Error + Send + Sync>,
    },
    /// A predecessor catalog does not carry the generation and digest the
    /// chain names.
    PredecessorCatalogMismatch {
        /// The expected generation.
        generation: CatalogGeneration,
    },
    /// A `recovery/dispositions` entry is not named by a lowercase artifact
    /// digest with the `.receipt` suffix, or its name is not its own identity.
    DispositionEntryName,
    /// A disposition receipt did not decode.
    Disposition {
        /// The exact decode refusal.
        source: crate::adapters::RecoveryDispositionDecodeError,
    },
    /// The snapshot could not be assembled without contradiction.
    Snapshot {
        /// The exact contradiction.
        source: GcLivenessSnapshotError,
    },
}

impl GcLivenessObservationError {
    pub(super) const fn pool(action: &'static str, source: io::Error) -> Self {
        Self::Pool { action, source }
    }
}

impl fmt::Display for GcLivenessObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog { .. } => formatter.write_str("fenced catalog could not be re-admitted"),
            Self::CatalogEntries { .. } => {
                formatter.write_str("catalog record-to-segment projection could not be decoded")
            }
            Self::RetainedRootAbsent { .. } => {
                formatter.write_str("manifest names a root the roots pool does not hold")
            }
            Self::RetainedRoot { .. } => formatter.write_str("retained root refused"),
            Self::Closure { .. } => {
                formatter.write_str("retained closure no longer verifies against the catalog")
            }
            Self::ClosureMemberUnindexed { .. } => {
                formatter.write_str("closure member has no catalog entry")
            }
            Self::Pool { action, .. } => write!(formatter, "segment pool refused to {action}"),
            Self::PoolEntryName => {
                formatter.write_str("segment pool entry is not named by a lowercase digest")
            }
            Self::PoolEntryKind { .. } => {
                formatter.write_str("segment pool entry is not a regular file")
            }
            Self::PoolByteLimit { limit } => {
                write!(
                    formatter,
                    "segment pool exceeds the {limit}-byte read limit"
                )
            }
            Self::SegmentAdmission { .. } => formatter.write_str("pool segment did not admit"),
            Self::SegmentDigestMismatch { .. } => {
                formatter.write_str("pool segment hashes to a digest other than its name")
            }
            Self::PredecessorCatalog { generation, .. } => write!(
                formatter,
                "predecessor catalog generation {} refused",
                generation.get()
            ),
            Self::PredecessorCatalogMismatch { generation } => write!(
                formatter,
                "predecessor catalog generation {} carries other coordinates",
                generation.get()
            ),
            Self::DispositionEntryName => {
                formatter.write_str("disposition entry is not named by its artifact digest")
            }
            Self::Disposition { .. } => formatter.write_str("disposition receipt did not decode"),
            Self::Snapshot { .. } => formatter.write_str("liveness snapshot contradicts itself"),
        }
    }
}

impl Error for GcLivenessObservationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Catalog { source } => Some(source),
            Self::CatalogEntries { source } => Some(source),
            Self::RetainedRoot { source, .. } | Self::PredecessorCatalog { source, .. } => {
                Some(source.as_ref())
            }
            Self::Closure { source, .. } => Some(source.as_ref()),
            Self::Pool { source, .. } => Some(source),
            Self::SegmentAdmission { source, .. } => Some(source.as_ref()),
            Self::Snapshot { source } => Some(source),
            Self::Disposition { source } => Some(source),
            Self::RetainedRootAbsent { .. }
            | Self::DispositionEntryName
            | Self::ClosureMemberUnindexed { .. }
            | Self::PoolEntryName
            | Self::PoolEntryKind { .. }
            | Self::PoolByteLimit { .. }
            | Self::SegmentDigestMismatch { .. }
            | Self::PredecessorCatalogMismatch { .. } => None,
        }
    }
}
