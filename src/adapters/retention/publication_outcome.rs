//! This boundary module owns retention publication outcome vocabulary.

/// Durable outcome of one authority-revalidated retention publication.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionPublicationOutcome {
    /// The complete successor became durable and visible.
    Published,
    /// The exact candidate and global manifest were already current.
    AlreadyCommitted,
}
