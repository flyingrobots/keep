//! This module owns typed retention-closure resource dimensions.

use std::fmt;

/// Resource dimension enforced during retention-closure verification.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetentionClosureCounter {
    /// Unique first-scheduled catalog record identities.
    Nodes,
    /// Maximum catalog-record edge depth.
    Depth,
    /// Unique structured layout payload bytes decoded.
    EncodedBytes,
    /// Complete record bytes charged to reconstruction work.
    PhysicalBytes,
}

impl fmt::Display for RetentionClosureCounter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Nodes => "closure nodes",
            Self::Depth => "closure depth",
            Self::EncodedBytes => "encoded closure bytes",
            Self::PhysicalBytes => "physical closure bytes",
        })
    }
}
