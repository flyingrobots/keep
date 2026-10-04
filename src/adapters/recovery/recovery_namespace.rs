//! This module owns recovery-inventory namespace identity.

use std::fmt;

/// One protocol-owned directory scanned during recovery inventory.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RecoveryNamespace {
    /// The store root.
    Root,
    /// The fixed-name staging directory.
    Staging,
    /// The immutable segment pool.
    Segments,
    /// The immutable catalog pool.
    Catalogs,
}

impl fmt::Display for RecoveryNamespace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Root => "store root",
            Self::Staging => "staging",
            Self::Segments => "segments",
            Self::Catalogs => "catalogs",
        })
    }
}
