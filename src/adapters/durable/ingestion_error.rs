//! This boundary module owns typed durable-ingestion failures.

use std::error::Error;
use std::fmt;
use std::io;

use crate::adapters::{
    CatalogEncodeError, CatalogPublicationError, CatalogRestartError, SegmentPublicationError,
    SegmentReadError, SegmentRecordAdmissionError, SegmentWriteError,
};
use crate::{CatalogGeneration, CatalogGenerationError, ChunkId, IngestionError, LayoutId};

/// Why a durable staging or commit returned no receipt.
///
/// Every variant names the exact boundary. A refusal during staging leaves
/// nothing visible; a refusal during commit leaves the completed phases'
/// residue for the version-one recovery protocol.
#[derive(Debug)]
pub enum DurableIngestionError {
    /// The publication directories could not be pinned.
    Open {
        /// The exact filesystem refusal.
        source: io::Error,
    },
    /// The pinned catalog could not be loaded or re-admitted.
    Catalog(Box<CatalogRestartError>),
    /// The streaming core refused the source, its identity, or its limits.
    Ingestion(IngestionError),
    /// A `staging/current.seg` from an earlier interrupted staging is
    /// retained; recover the store before staging again.
    StageRetained,
    /// The segment stage refused a write, including the record-count and
    /// segment-length ceilings.
    Stage(Box<SegmentWriteError>),
    /// A chunk or layout record could not be admitted for the segment.
    Record(SegmentRecordAdmissionError),
    /// The pinned catalog holds a record under the chunk's identity whose
    /// bytes differ from the source's chunk.
    ChunkRepresentation {
        /// The identity in dispute.
        identity: ChunkId,
    },
    /// The pinned catalog holds a layout record under the layout's identity
    /// whose bytes differ from the canonical record.
    LayoutRepresentation {
        /// The identity in dispute.
        identity: LayoutId,
    },
    /// The catalog generation the staging was verified against is no
    /// longer the current one.
    CatalogMoved {
        /// The generation the staging was verified against.
        staged: CatalogGeneration,
        /// The generation observed at commit.
        observed: CatalogGeneration,
    },
    /// The sealed stage could not be read back for admission.
    ReadStage {
        /// The exact filesystem refusal.
        source: io::Error,
    },
    /// A segment did not admit under the read policy.
    Segment(Box<SegmentReadError>),
    /// The sealed stage did not bind to its admitted bytes.
    Selection(SegmentPublicationError),
    /// The successor generation is not representable.
    Generation(CatalogGenerationError),
    /// The successor catalog could not be encoded.
    Successor(Box<CatalogEncodeError>),
    /// The catalog protocol refused publication.
    Publish(Box<CatalogPublicationError>),
}

impl From<IngestionError> for DurableIngestionError {
    fn from(source: IngestionError) -> Self {
        Self::Ingestion(source)
    }
}

impl fmt::Display for DurableIngestionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { .. } => formatter.write_str("publication directories could not be pinned"),
            Self::Catalog(_) => formatter.write_str("the pinned catalog refused"),
            Self::Ingestion(source) => write!(formatter, "ingestion: {source}"),
            Self::StageRetained => {
                formatter.write_str("a retained staging/current.seg needs recovery first")
            }
            Self::Stage(source) => write!(formatter, "segment stage: {source}"),
            Self::Record(source) => write!(formatter, "record admission: {source}"),
            Self::ChunkRepresentation { identity } => {
                write!(
                    formatter,
                    "catalog chunk {identity:?} differs from the source's bytes"
                )
            }
            Self::LayoutRepresentation { identity } => write!(
                formatter,
                "catalog layout {identity:?} differs from the canonical record"
            ),
            Self::CatalogMoved { staged, observed } => write!(
                formatter,
                "staged against catalog generation {} but generation {} is current",
                staged.get(),
                observed.get()
            ),
            Self::ReadStage { .. } => formatter.write_str("the sealed stage could not be read"),
            Self::Segment(source) => write!(formatter, "segment admission: {source}"),
            Self::Selection(source) => write!(formatter, "stage selection: {source}"),
            Self::Generation(source) => write!(formatter, "successor generation: {source}"),
            Self::Successor(source) => write!(formatter, "successor catalog: {source}"),
            Self::Publish(source) => write!(formatter, "publication: {source}"),
        }
    }
}

impl Error for DurableIngestionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { source } | Self::ReadStage { source } => Some(source),
            Self::Catalog(source) => Some(source.as_ref()),
            Self::Ingestion(source) => Some(source),
            Self::Stage(source) => Some(source.as_ref()),
            Self::Record(source) => Some(source),
            Self::Segment(source) => Some(source.as_ref()),
            Self::Selection(source) => Some(source),
            Self::Generation(source) => Some(source),
            Self::Successor(source) => Some(source.as_ref()),
            Self::Publish(source) => Some(source.as_ref()),
            Self::StageRetained
            | Self::ChunkRepresentation { .. }
            | Self::LayoutRepresentation { .. }
            | Self::CatalogMoved { .. } => None,
        }
    }
}
