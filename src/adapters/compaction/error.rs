//! This boundary module owns typed refusals of filesystem compaction.

use std::error::Error;
use std::fmt;
use std::io;

use super::CompactionRefusal;
use crate::adapters::gc::{GcLivenessObservationError, GcPlanError};
use crate::adapters::{
    CatalogEncodeError, CatalogPublicationError, CatalogRestartError,
    FilesystemRetentionSnapshotError, SegmentPublicationError, SegmentReadError, SegmentWriteError,
};

/// Why filesystem compaction refused.
#[derive(Debug)]
pub enum FilesystemCompactionError {
    /// A read, open, or stage operation outside the publication protocol failed.
    Observe {
        /// The exact failure.
        source: io::Error,
    },
    /// The reopened view refused.
    Snapshot(Box<FilesystemRetentionSnapshotError>),
    /// Re-observing the store refused.
    Liveness(Box<GcLivenessObservationError>),
    /// Re-planning refused where the plan expected a compaction.
    Refused(CompactionRefusal),
    /// The reopened store plans differently: the plan's evidence is stale.
    PlanStale,
    /// The current catalog could not be reloaded.
    Catalog(Box<CatalogRestartError>),
    /// A live record the plan copies is not in the current catalog.
    RecordVanished,
    /// A retained or staged segment refused admission.
    Segment(Box<SegmentReadError>),
    /// Writing the new segment refused.
    Stage(Box<SegmentWriteError>),
    /// Selecting the sealed stage refused.
    Selection(SegmentPublicationError),
    /// Encoding the successor catalog refused.
    Successor(Box<CatalogEncodeError>),
    /// A publication phase refused; the completed phases' effects remain
    /// for [`recover_compaction`](super::recover_compaction).
    Publish(Box<CatalogPublicationError>),
    /// After publication, the reopened store did not show every superseded
    /// segment as a superseded retirement candidate.
    Revalidation(GcPlanError),
    /// After publication, a superseded segment is not a retirement candidate.
    NotSuperseded,
}

impl fmt::Display for FilesystemCompactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Observe { .. } => formatter.write_str("compaction observation failed"),
            Self::Snapshot(_) => formatter.write_str("compaction could not reopen the view"),
            Self::Liveness(_) => formatter.write_str("compaction could not re-observe liveness"),
            Self::Refused(source) => write!(formatter, "compaction refused: {source}"),
            Self::PlanStale => formatter.write_str("the store no longer matches the plan"),
            Self::Catalog(_) => formatter.write_str("the current catalog could not be reloaded"),
            Self::RecordVanished => formatter.write_str("a planned live record is not named"),
            Self::Segment(_) => formatter.write_str("a segment refused admission"),
            Self::Stage(_) => formatter.write_str("the new segment stage refused"),
            Self::Selection(source) => write!(formatter, "stage selection refused: {source}"),
            Self::Successor(_) => formatter.write_str("the successor catalog refused to encode"),
            Self::Publish(source) => write!(formatter, "publication refused: {source}"),
            Self::Revalidation(source) => write!(formatter, "revalidation refused: {source}"),
            Self::NotSuperseded => {
                formatter.write_str("a superseded segment is not a retirement candidate")
            }
        }
    }
}

impl Error for FilesystemCompactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Observe { source } => Some(source),
            Self::Snapshot(source) => Some(source.as_ref()),
            Self::Liveness(source) => Some(source.as_ref()),
            Self::Refused(source) => Some(source),
            Self::Catalog(source) => Some(source.as_ref()),
            Self::Segment(source) => Some(source.as_ref()),
            Self::Stage(source) => Some(source.as_ref()),
            Self::Selection(source) => Some(source),
            Self::Successor(source) => Some(source.as_ref()),
            Self::Publish(source) => Some(source.as_ref()),
            Self::Revalidation(source) => Some(source),
            Self::PlanStale | Self::RecordVanished | Self::NotSuperseded => None,
        }
    }
}
