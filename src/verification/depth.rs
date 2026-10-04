//! This module owns the explicit vocabulary of verification requests.

/// Requested verification work, interpreted for a specific subject.
///
/// Operations document their supported depths. No total implication order exists:
/// catalog reachability does not establish every referenced blob's identity,
/// and retention closure does not establish a future snapshot-binding format.
///
/// Depths have equality but no cross-subject implication order. Compare a
/// request with the operation's explicit supported set, never an ordinal.
///
/// ```compile_fail,E0369
/// use keep::VerificationDepth;
/// let _ = VerificationDepth::CatalogReachability
///     >= VerificationDepth::CompleteBlobIdentity;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
