//! This boundary module owns digest-named retention namespace admission outcomes.

/// Result of exact retention root-namespace admission.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionNamespaceAdmission {
    /// The exact digest-named directory already existed and was admitted.
    Existing,
    /// The exact digest-named directory was created and admitted.
    Created,
}
