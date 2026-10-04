//! This module owns validated immutable-pool recovery coordinates.

use super::{RecoveryStageCompletionPool, SegmentDigest};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Exact immutable-pool coordinate derived from a complete stage.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageCompletionTarget {
    /// One fully admitted immutable segment.
    Segment {
        /// Verified physical segment digest.
        digest: SegmentDigest,
    },
    /// One framing-, checksum-, digest-, and entry-verified catalog.
    Catalog {
        /// Positive catalog generation.
        generation: CatalogGeneration,
        /// Exact canonical catalog byte length.
        length: CatalogLength,
        /// Verified physical catalog digest.
        digest: CatalogDigest,
    },
}

impl RecoveryStageCompletionTarget {
    /// Returns the immutable pool selected by this coordinate.
    pub const fn pool(self) -> RecoveryStageCompletionPool {
        match self {
            Self::Segment { .. } => RecoveryStageCompletionPool::Segments,
            Self::Catalog { .. } => RecoveryStageCompletionPool::Catalogs,
        }
    }
}
