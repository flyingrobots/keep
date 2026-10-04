//! This module owns the coordinates naming a verification subject.

use crate::segment_digest::SegmentDigest;
use crate::{
    BlobId, CatalogDigest, CatalogGeneration, ChunkId, LayoutId, RetentionNamespaceDigest,
    RetentionRootDigest, RootGeneration,
};

/// Subject to which a verification claim or refusal applies.
///
/// Coordinates contain no paths or content bytes. A catalog coordinate is a
/// physical view selection, not a logical blob identity or retention promise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationSubject {
    /// Supplied segment bytes that have not established a physical identity.
    SegmentInput,
    /// Supplied layout bytes before canonical identity admission.
    LayoutInput,
    /// Supplied retention-root bytes before namespace and identity admission.
    RetentionRootInput,
    /// The catalog selected by a store's publication head, before admission.
    PublishedCatalog,
    /// A published catalog/retention observation before its subject admits.
    PublishedView,
    /// The published root requested by namespace, before its coordinates admit.
    RetentionNamespace {
        /// Requested opaque namespace digest.
        namespace: RetentionNamespaceDigest,
    },

    /// A logical blob requested independently of its physical realization.
    Blob {
        /// Complete logical byte identity.
        identity: BlobId,
    },
    /// An exact canonical retention root, without publication authority.
    RetentionRoot {
        /// Namespace authenticated by the root bytes.
        namespace: RetentionNamespaceDigest,
        /// Root generation authenticated by those same bytes.
        generation: RootGeneration,
        /// Exact canonical root digest.
        digest: RetentionRootDigest,
    },

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
