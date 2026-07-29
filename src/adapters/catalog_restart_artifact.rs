//! Restart-loaded physical artifact classifications.

use super::SegmentDigest;

/// Physical artifact whose kind, length, or allocation was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogRestartArtifact {
    /// Fixed publication head.
    Head,
    /// Exact head-selected catalog.
    Catalog,
    /// Exact catalog-selected segment.
    Segment {
        /// Selected physical digest.
        digest: SegmentDigest,
    },
}
