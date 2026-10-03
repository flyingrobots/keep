//! This module owns the coordinates naming a verification subject.

use crate::segment_digest::SegmentDigest;
use crate::{CatalogDigest, CatalogGeneration, ChunkId, LayoutId};

/// Subject to which a verification claim or refusal applies.
///
/// Coordinates contain no paths or content bytes. A catalog coordinate is a
/// physical view selection, not a logical blob identity or retention promise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationSubject {
    /// One exact immutable segment, without a publication or retention claim.
    Segment {
        /// Verified physical segment digest.
        digest: SegmentDigest,
    },
    /// Exact chunk bytes admitted from a segment record.
    Chunk {
        /// Logical chunk identity verified during record admission.
        identity: ChunkId,
    },
    /// One admitted canonical layout, without a claim about its chunk closure.
    Layout {
        /// Canonical layout identity verified during record admission.
        identity: LayoutId,
    },
    /// One exact immutable catalog selected by a publication head.
    Catalog {
        /// Generation selected by the admitted head.
        generation: CatalogGeneration,
        /// Digest selected by that same admitted head.
        digest: CatalogDigest,
    },
}
