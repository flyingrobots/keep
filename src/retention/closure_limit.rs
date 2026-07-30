//! This module owns semantic names for bounded closure resources.

use std::fmt;

/// One independently bounded retention closure resource.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RetentionClosureLimit {
    /// Number of logical and physical closure nodes.
    Nodes,
    /// Maximum traversal depth.
    Depth,
    /// Total encoded bytes inspected.
    EncodedBytes,
    /// Total physical bytes inspected.
    PhysicalBytes,
}

impl fmt::Display for RetentionClosureLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Nodes => "closure nodes",
            Self::Depth => "closure depth",
            Self::EncodedBytes => "encoded bytes",
            Self::PhysicalBytes => "physical bytes",
        })
    }
}
