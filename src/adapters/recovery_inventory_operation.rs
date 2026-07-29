//! This module owns recovery-inventory storage operation identity.

use std::fmt;

/// Exact storage operation attempted during recovery inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryInventoryOperation {
    /// Count entries without retaining their names.
    Count,
    /// Read validated raw entry names after count admission.
    ReadNames,
}

impl fmt::Display for RecoveryInventoryOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Count => "entry count",
            Self::ReadNames => "entry-name read",
        })
    }
}
