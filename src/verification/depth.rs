//! This module owns the explicit vocabulary of verification requests.

/// Requested verification work, interpreted for a specific subject.
///
/// Operations document their supported depths. Ordering alone is not a proof:
/// catalog reachability does not establish every referenced blob's identity,
/// and retention closure does not establish a future snapshot-binding format.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VerificationDepth {
    /// Canonical record framing and structural bounds.
    Framing,
    /// Checksums covering the subject's encoded representation.
    Checksum,
    /// Logical identities of the subject's chunk bytes.
    ChunkIdentity,
    /// Canonical layout identity and layout structure.
    LayoutIdentity,
    /// Complete logical blob identity and registered profile replay.
    CompleteBlobIdentity,
    /// Exact binding of catalog entries to admitted physical records.
    CatalogReachability,
    /// Complete logical closure of retained roots in one catalog view.
    RetentionClosure,
    /// Snapshot-format binding, unsupported until that protocol exists.
    SnapshotBinding,
}
