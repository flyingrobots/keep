//! This module owns immutable pools selected by recovery completion.

use std::fmt;

/// Immutable artifact pool selected by a complete recovery stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageCompletionPool {
    /// Immutable sealed-segment pool.
    Segments,
    /// Immutable checksummed-catalog pool.
    Catalogs,
}

impl fmt::Display for RecoveryStageCompletionPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Segments => formatter.write_str("segments"),
            Self::Catalogs => formatter.write_str("catalogs"),
        }
    }
}
