//! This boundary module owns retention transition disposition vocabulary.

/// Storage-independent result of one admitted retention transition comparison.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionTransitionDisposition {
    /// The candidate is an exact successor that still requires publication.
    Publish,
    /// The byte-identical candidate is already the selected current root.
    AlreadyCommitted,
}
