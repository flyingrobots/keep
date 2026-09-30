//! This boundary module owns the pure compaction planner: which named
//! segments are retained, compacted, or omitted, and what the successor
//! copies.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::CompactionObservation;
use crate::CatalogGeneration;
use crate::adapters::gc::{GcLivenessCoordinates, GcRetentionState};
use crate::adapters::{SegmentDigest, SegmentRecordIdentity};

/// Fixed record framing: 112-byte header plus 32-byte checksum.
const RECORD_FRAMING: u64 = 144;
/// Segment header plus seal.
const SEGMENT_FRAMING: u64 = 192;
/// The version-1 segment ceilings the successor's new segment must respect.
const MAXIMUM_RECORD_COUNT: u64 = 1_048_576;
const MAXIMUM_SEGMENT_LENGTH: u64 = 1_073_741_824;

/// What the successor does with one segment the current catalog names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactionSegmentDisposition {
    /// Every record is live: the successor names the segment unchanged.
    Retained,
    /// Live and unreachable records share it: the live records are copied
    /// into the new segment and the unreachable ones omitted.
    Compacted {
        /// The live records to copy, in canonical identity order.
        live: Vec<SegmentRecordIdentity>,
        /// How many named records the successor omits.
        unreachable: u64,
    },
    /// No record is live: the successor omits the whole segment.
    Omitted {
        /// How many named records the successor omits.
        unreachable: u64,
    },
}

/// One deterministic compaction plan over one observation.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionPlan {
    coordinates: GcLivenessCoordinates,
    successor_generation: CatalogGeneration,
    segments: BTreeMap<SegmentDigest, CompactionSegmentDisposition>,
    copied_records: u64,
    copied_bytes: u64,
    reclaimable_bytes: u64,
}

impl CompactionPlan {
    /// The coordinates the plan was computed under.
    #[must_use]
    pub const fn coordinates(&self) -> GcLivenessCoordinates {
        self.coordinates
    }

    /// The generation the successor catalog will carry.
    pub const fn successor_generation(&self) -> CatalogGeneration {
        self.successor_generation
    }

    /// Every named segment's disposition, in digest order.
    #[must_use]
    pub const fn segments(&self) -> &BTreeMap<SegmentDigest, CompactionSegmentDisposition> {
        &self.segments
    }

    /// Segments the successor names unchanged.
    pub fn retained(&self) -> impl Iterator<Item = SegmentDigest> + '_ {
        self.segments
            .iter()
            .filter(|(_, disposition)| {
                matches!(disposition, CompactionSegmentDisposition::Retained)
            })
            .map(|(digest, _)| *digest)
    }

    /// Segments the successor no longer names, in digest order.
    pub fn superseded(&self) -> impl Iterator<Item = SegmentDigest> + '_ {
        self.segments
            .iter()
            .filter(|(_, disposition)| {
                !matches!(disposition, CompactionSegmentDisposition::Retained)
            })
            .map(|(digest, _)| *digest)
    }

    /// Every record the new segment copies, in canonical identity order.
    pub fn copied(&self) -> impl Iterator<Item = SegmentRecordIdentity> + '_ {
        self.segments
            .values()
            .flat_map(|disposition| match disposition {
                CompactionSegmentDisposition::Compacted { live, .. } => live.as_slice(),
                _ => &[],
            })
            .copied()
    }

    /// How many records the new segment copies; zero when the successor only
    /// omits whole segments.
    #[must_use]
    pub const fn copied_records(&self) -> u64 {
        self.copied_records
    }

    /// The exact record bytes the new segment will hold.
    #[must_use]
    pub const fn copied_bytes(&self) -> u64 {
        self.copied_bytes
    }

    /// The pool bytes GC may reclaim once the successor is durable: the
    /// complete length of every superseded segment.
    #[must_use]
    pub const fn reclaimable_bytes(&self) -> u64 {
        self.reclaimable_bytes
    }
}

/// Why an observation yields no compaction plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactionRefusal {
    /// No retention head is published: nothing is retained, so compaction
    /// would omit every record; an operator publishes retention first.
    RetentionNotPublished,
    /// Every named record is live: nothing to compact is not a plan.
    NothingToCompact,
    /// A retained closure reaches a record the catalog does not name.
    LiveRecordUnnamed {
        /// The record.
        identity: SegmentRecordIdentity,
    },
    /// The catalog names a segment absent from the pool.
    NamedSegmentMissing {
        /// The segment.
        segment: SegmentDigest,
    },
    /// The copied records exceed one segment's ceilings.
    CopyExceedsSegmentCeiling {
        /// Records to copy.
        records: u64,
        /// Bytes to copy.
        bytes: u64,
    },
    /// The successor generation would overflow.
    SuccessorGenerationOverflow,
}

impl fmt::Display for CompactionRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetentionNotPublished => {
                formatter.write_str("no retention head is published; nothing is retained")
            }
            Self::NothingToCompact => formatter.write_str("every named record is live"),
            Self::LiveRecordUnnamed { .. } => {
                formatter.write_str("a retained closure reaches an unnamed record")
            }
            Self::NamedSegmentMissing { .. } => {
                formatter.write_str("the catalog names a segment absent from the pool")
            }
            Self::CopyExceedsSegmentCeiling { records, bytes } => write!(
                formatter,
                "{records} records ({bytes} bytes) exceed one segment"
            ),
            Self::SuccessorGenerationOverflow => {
                formatter.write_str("the successor generation overflows")
            }
        }
    }
}

impl std::error::Error for CompactionRefusal {}

/// Plans one compaction over `observation`. Pure and deterministic.
///
/// # Errors
///
/// Returns [`CompactionRefusal`] when nothing lawful can be planned.
pub fn plan_compaction(
    observation: &CompactionObservation,
) -> Result<CompactionPlan, CompactionRefusal> {
    let coordinates = observation.coordinates();
    if coordinates.retention() == GcRetentionState::Empty {
        return Err(CompactionRefusal::RetentionNotPublished);
    }
    if let Some(identity) = observation
        .live()
        .iter()
        .find(|identity| !observation.named().contains_key(*identity))
    {
        return Err(CompactionRefusal::LiveRecordUnnamed {
            identity: *identity,
        });
    }
    let mut per_segment: BTreeMap<SegmentDigest, (Vec<SegmentRecordIdentity>, u64)> =
        BTreeMap::new();
    for (identity, segment) in observation.named() {
        let entry = per_segment.entry(*segment).or_default();
        if observation.live().contains(identity) {
            entry.0.push(*identity);
        } else {
            entry.1 = entry.1.saturating_add(1);
        }
    }
    let mut segments = BTreeMap::new();
    let (mut copied_records, mut copied_bytes, mut reclaimable_bytes) = (0_u64, 0_u64, 0_u64);
    for (segment, (live, unreachable)) in per_segment {
        let length = *observation
            .inventory()
            .get(&segment)
            .ok_or(CompactionRefusal::NamedSegmentMissing { segment })?;
        let disposition = if unreachable == 0 {
            CompactionSegmentDisposition::Retained
        } else {
            reclaimable_bytes = reclaimable_bytes.saturating_add(length);
            if live.is_empty() {
                CompactionSegmentDisposition::Omitted { unreachable }
            } else {
                for identity in &live {
                    copied_records = copied_records.saturating_add(1);
                    copied_bytes = copied_bytes
                        .saturating_add(identity.payload_length().saturating_add(RECORD_FRAMING));
                }
                CompactionSegmentDisposition::Compacted { live, unreachable }
            }
        };
        segments.insert(segment, disposition);
    }
    if segments
        .values()
        .all(|disposition| matches!(disposition, CompactionSegmentDisposition::Retained))
    {
        return Err(CompactionRefusal::NothingToCompact);
    }
    if copied_records > MAXIMUM_RECORD_COUNT
        || copied_bytes.saturating_add(SEGMENT_FRAMING) > MAXIMUM_SEGMENT_LENGTH
    {
        return Err(CompactionRefusal::CopyExceedsSegmentCeiling {
            records: copied_records,
            bytes: copied_bytes,
        });
    }
    let successor_generation = coordinates
        .catalog_generation()
        .successor()
        .map_err(|_source| CompactionRefusal::SuccessorGenerationOverflow)?;
    Ok(CompactionPlan {
        coordinates,
        successor_generation,
        segments,
        copied_records,
        copied_bytes,
        reclaimable_bytes,
    })
}

/// A set view of the plan's superseded segments, for revalidation.
pub(super) fn superseded_set(plan: &CompactionPlan) -> BTreeSet<SegmentDigest> {
    plan.superseded().collect()
}
