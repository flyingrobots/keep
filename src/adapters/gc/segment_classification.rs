//! This boundary module owns the physical GC classification vocabulary.

use std::fmt;

/// Why an unreachable segment may enter a retirement plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcUnreachableEvidence {
    /// A predecessor catalog in the pool's chain named the segment and the
    /// durably published current catalog omits it.
    Superseded,
    /// A durable `RecoveryDispositionReceipt` retired the segment.
    Disposed,
}

/// The one classification a plan assigns to each inventoried segment.
///
/// The order of the variants is the order the planner decides them in:
/// catalog naming first, then retained reachability, then the evidence that
/// releases an unnamed segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcSegmentClassification {
    /// The current catalog names the segment and at least one retained
    /// closure reaches a record in it.
    Live {
        /// How many retained roots reach the segment.
        retained_roots: u32,
    },
    /// The current catalog names the segment but no retained closure reaches
    /// it. It is not a candidate: only a catalog successor published through
    /// compaction can release a named segment.
    NamedUnreachable,
    /// No catalog in the pool's chain names the segment and no disposition
    /// receipt retires it: a verified orphan of an interrupted publication,
    /// protected until an explicit finalize-or-retire disposition.
    RecoveryProtected,
    /// The segment is unreachable and evidenced as collectible.
    Unreachable(GcUnreachableEvidence),
}

impl GcSegmentClassification {
    /// Returns the stable identifier used by the golden plan ledger.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Live { .. } => "live",
            Self::NamedUnreachable => "named-unreachable",
            Self::RecoveryProtected => "recovery-protected",
            Self::Unreachable(GcUnreachableEvidence::Superseded) => "unreachable-superseded",
            Self::Unreachable(GcUnreachableEvidence::Disposed) => "unreachable-disposed",
        }
    }

    /// Reports whether the segment may enter a retirement plan.
    #[must_use]
    pub const fn is_candidate(self) -> bool {
        matches!(self, Self::Unreachable(_))
    }
}

impl fmt::Display for GcSegmentClassification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.identifier())
    }
}
