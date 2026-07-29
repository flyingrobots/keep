//! Storage decision after current catalog state verification.

/// Writer-locked decision for a fully preflighted catalog candidate.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPublicationReadiness {
    /// The expected predecessor is current, so publication may proceed.
    Ready,
    /// The complete proposed generation is already current.
    AlreadyPublished,
}
