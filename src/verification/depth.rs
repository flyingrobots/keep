//! This module owns the ordered verification depths.

/// One explicit depth to which a verification establishes its subject.
///
/// Depths are ordered: establishing a deeper depth requires every shallower
/// depth the view supports. A report names the one depth it established and
/// can never be read as a deeper one. Not every view supports every depth;
/// a view refuses a depth it cannot establish instead of degrading to a
/// shallower one.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VerificationDepth {
    /// Every durable record the subject depends on has canonical framing.
    Framing,
    /// Every durable record the subject depends on has a matching checksum.
    Checksum,
    /// Every chunk the subject names is present and hashes to its `ChunkId`.
    ChunkIdentity,
    /// The subject's layout produces its canonical `LayoutId`.
    LayoutIdentity,
    /// The complete reconstructed sequence hashes to the target `BlobId` and
    /// replays the registered storage profile.
    CompleteBlobIdentity,
    /// One admitted catalog generation names every record the subject needs.
    CatalogReachability,
    /// One retained root's closure reaches the subject under one fenced view.
    RetentionClosure,
}

impl VerificationDepth {
    /// Every depth in ascending order.
    pub const ALL: [Self; 7] = [
        Self::Framing,
        Self::Checksum,
        Self::ChunkIdentity,
        Self::LayoutIdentity,
        Self::CompleteBlobIdentity,
        Self::CatalogReachability,
        Self::RetentionClosure,
    ];
}
