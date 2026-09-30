//! This boundary module owns the immutable liveness snapshot GC plans from.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::{GcLivenessCoordinates, GcRetainedClosure, VerificationEvidenceDigest};
use crate::RetentionNamespaceDigest;
use crate::adapters::SegmentDigest;

/// One immutable liveness snapshot plus one bounded physical inventory.
///
/// The snapshot is assembled by an observer that has already admitted every
/// byte it names; the planner reads it and nothing else. Every collection is
/// canonically ordered so planning is deterministic.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcLivenessSnapshot {
    coordinates: GcLivenessCoordinates,
    inventory: BTreeMap<SegmentDigest, u64>,
    named: BTreeSet<SegmentDigest>,
    retained: Vec<GcRetainedClosure>,
    superseded: BTreeSet<SegmentDigest>,
    disposed: BTreeSet<SegmentDigest>,
    evidence: BTreeMap<SegmentDigest, VerificationEvidenceDigest>,
    disposition_checksums: BTreeSet<[u8; 32]>,
}

/// A snapshot that could not be assembled without contradiction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcLivenessSnapshotError {
    /// One segment was inventoried twice.
    DuplicateInventory {
        /// The repeated segment.
        segment: SegmentDigest,
    },
    /// One namespace was retained twice.
    DuplicateNamespace {
        /// The repeated namespace.
        namespace: RetentionNamespaceDigest,
    },
}

impl fmt::Display for GcLivenessSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateInventory { .. } => {
                formatter.write_str("segment inventoried twice in one liveness snapshot")
            }
            Self::DuplicateNamespace { .. } => {
                formatter.write_str("namespace retained twice in one liveness snapshot")
            }
        }
    }
}

impl std::error::Error for GcLivenessSnapshotError {}

impl GcLivenessSnapshot {
    /// Starts an empty snapshot under `coordinates`.
    pub const fn new(coordinates: GcLivenessCoordinates) -> Self {
        Self {
            coordinates,
            inventory: BTreeMap::new(),
            named: BTreeSet::new(),
            retained: Vec::new(),
            superseded: BTreeSet::new(),
            disposed: BTreeSet::new(),
            evidence: BTreeMap::new(),
            disposition_checksums: BTreeSet::new(),
        }
    }

    /// Records one physical segment present in the pool with its length.
    ///
    /// # Errors
    ///
    /// Returns [`GcLivenessSnapshotError::DuplicateInventory`] when the
    /// segment was already inventoried.
    pub fn inventory_segment(
        &mut self,
        segment: SegmentDigest,
        length: u64,
    ) -> Result<(), GcLivenessSnapshotError> {
        if self.inventory.insert(segment, length).is_some() {
            return Err(GcLivenessSnapshotError::DuplicateInventory { segment });
        }
        Ok(())
    }

    /// Records that the current catalog names at least one record in `segment`.
    pub fn name_segment(&mut self, segment: SegmentDigest) {
        self.named.insert(segment);
    }

    /// Records one retained root's verified closure.
    ///
    /// # Errors
    ///
    /// Returns [`GcLivenessSnapshotError::DuplicateNamespace`] when the
    /// namespace already has a retained closure.
    pub fn retain(&mut self, closure: GcRetainedClosure) -> Result<(), GcLivenessSnapshotError> {
        let namespace = closure.namespace();
        if self
            .retained
            .iter()
            .any(|retained| retained.namespace() == namespace)
        {
            return Err(GcLivenessSnapshotError::DuplicateNamespace { namespace });
        }
        self.retained.push(closure);
        Ok(())
    }

    /// Records that a predecessor catalog in the pool's chain named `segment`
    /// and the current catalog no longer does: the segment was superseded by
    /// a durably published catalog successor. `evidence` is the digest of
    /// that predecessor catalog, the durable record that released it.
    pub fn supersede_segment(
        &mut self,
        segment: SegmentDigest,
        evidence: VerificationEvidenceDigest,
    ) {
        self.superseded.insert(segment);
        self.evidence.entry(segment).or_insert(evidence);
    }

    /// Records that a durable `RecoveryDispositionReceipt` retired `segment`.
    /// `evidence` is that receipt's checksum, the durable record that
    /// released it; the checksum also enters the admitted disposition set.
    pub fn dispose_segment(
        &mut self,
        segment: SegmentDigest,
        evidence: VerificationEvidenceDigest,
    ) {
        self.disposed.insert(segment);
        self.evidence.entry(segment).or_insert(evidence);
        self.disposition_checksums.insert(*evidence.as_bytes());
    }

    /// Returns the digest of the durable record that released `segment`, if
    /// any.
    #[must_use]
    pub fn release_evidence(&self, segment: SegmentDigest) -> Option<VerificationEvidenceDigest> {
        self.evidence.get(&segment).copied()
    }

    /// Returns the checksums of every exact disposition receipt admitted, in
    /// canonical order.
    #[must_use]
    pub const fn disposition_checksums(&self) -> &BTreeSet<[u8; 32]> {
        &self.disposition_checksums
    }

    /// Returns the coordinates the snapshot binds.
    #[must_use]
    pub const fn coordinates(&self) -> GcLivenessCoordinates {
        self.coordinates
    }

    /// Returns every inventoried segment with its length.
    #[must_use]
    pub const fn inventory(&self) -> &BTreeMap<SegmentDigest, u64> {
        &self.inventory
    }

    /// Returns every segment the current catalog names.
    #[must_use]
    pub const fn named(&self) -> &BTreeSet<SegmentDigest> {
        &self.named
    }

    /// Returns every retained closure in retention order.
    pub fn retained(&self) -> &[GcRetainedClosure] {
        &self.retained
    }

    /// Returns every superseded segment.
    #[must_use]
    pub const fn superseded(&self) -> &BTreeSet<SegmentDigest> {
        &self.superseded
    }

    /// Returns every segment with a durable retirement disposition.
    #[must_use]
    pub const fn disposed(&self) -> &BTreeSet<SegmentDigest> {
        &self.disposed
    }
}
