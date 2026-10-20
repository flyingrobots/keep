//! This module owns semantic refusals while classifying compaction residue.

use std::error::Error;
use std::fmt;

use crate::{CatalogDigest, CatalogGeneration, SegmentRecordIdentity};

/// Why compaction residue cannot be proved safe to discard or finalize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompactionRecoveryRefusal {
    /// The current catalog does not name a record needed to reproduce the stage.
    RecordNotNamed {
        /// The staged logical record identity.
        identity: SegmentRecordIdentity,
    },
    /// The catalog-selected record differs from the staged record.
    RecordMismatch {
        /// The identity under which the differing records were found.
        identity: SegmentRecordIdentity,
    },
    /// An allegedly unpublished stage holds a record already named by the catalog.
    RecordAlreadyNamed {
        /// The already-published logical record identity.
        identity: SegmentRecordIdentity,
    },
    /// A staged catalog does not bind the current head as its exact predecessor.
    SuccessorMismatch {
        /// The required successor generation.
        expected_generation: CatalogGeneration,
        /// The stage's generation.
        observed_generation: CatalogGeneration,
        /// The required predecessor digest.
        expected_predecessor: CatalogDigest,
        /// The stage's predecessor, absent only for an initial catalog.
        observed_predecessor: Option<CatalogDigest>,
    },
    /// The assessed stage is no longer present.
    StageAbsent,
    /// The stage's current bytes differ from the assessed bytes.
    StageChanged,
}

impl fmt::Display for CompactionRecoveryRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "compaction recovery evidence refused: {self:?}")
    }
}

impl Error for CompactionRecoveryRefusal {}
