//! This module owns exact semantic reasons for truncated-stage discard.

use super::RecoverySegmentTruncation;

/// Exact truncation that makes a fixed stage eligible for explicit discard.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageDiscardReason {
    /// Exact incomplete segment boundary.
    Segment(RecoverySegmentTruncation),
    /// Incomplete fixed catalog header.
    CatalogHeader {
        /// Required fixed-header byte count.
        required: usize,
        /// Supplied byte count.
        observed: usize,
    },
    /// Admitted catalog header whose declared body is incomplete.
    CatalogBody {
        /// Declared canonical catalog byte count.
        expected: u64,
        /// Supplied byte count.
        observed: usize,
    },
    /// Incomplete fixed-width next publication head.
    NextHead {
        /// Required publication-head byte count.
        required: usize,
        /// Supplied byte count.
        observed: usize,
    },
}
