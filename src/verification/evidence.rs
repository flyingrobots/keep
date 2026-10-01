//! This module owns the exact evidence a verification refusal carries.

use crate::{BlobId, ChunkId, LayoutId};

/// What was absent from the admitted view.
///
/// Absence is evidenced only against a complete view; the reference store's
/// in-memory indexes are complete by construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissingEvidence {
    /// No committed layout names the blob.
    Blob(BlobId),
    /// The exact layout is not committed.
    Layout(LayoutId),
    /// A committed or supplied layout names a chunk the view lacks.
    Chunk {
        /// Layout naming the chunk.
        layout: LayoutId,
        /// Zero-based entry index.
        index: usize,
        /// Absent exact chunk identity.
        chunk: ChunkId,
    },
}

/// Which integrity proposition failed, with the expected and observed values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorruptionEvidence {
    /// Stored chunk bytes do not hash to the identity the layout names.
    ChunkIdentity {
        /// Layout naming the chunk.
        layout: LayoutId,
        /// Zero-based entry index.
        index: usize,
        /// Identity named by the layout.
        expected: ChunkId,
        /// Identity calculated from the stored bytes.
        observed: ChunkId,
    },
    /// The committed layout does not produce the identity it is keyed by.
    LayoutIdentity {
        /// Identity the view committed the layout under.
        expected: LayoutId,
        /// Identity the canonical record produces.
        observed: LayoutId,
    },
    /// The authenticated chunks do not reproduce the target blob identity.
    BlobIdentity {
        /// Layout reconstructed.
        layout: LayoutId,
        /// Target named by the layout.
        expected: BlobId,
        /// Identity calculated from every authenticated chunk.
        observed: BlobId,
    },
    /// Replaying the registered storage profile did not reproduce the layout.
    ProfileBoundary {
        /// Layout reconstructed.
        layout: LayoutId,
        /// Zero-based boundary index at which replay diverged.
        index: usize,
    },
}
