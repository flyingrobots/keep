//! This module owns caller-supplied retention generation expectations.

use super::RootGeneration;

/// Expected current state of one retention namespace.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionGenerationExpectation {
    /// The namespace must not yet have a published root.
    Absent,
    /// The namespace must have exactly this current root generation.
    Current(RootGeneration),
}
