//! This module owns a validated observation of the migration namespace prefix.

use super::{StoreMigrationRecoveryAmbiguity, StoreMigrationResidue};

const NAMES: [&str; 7] = [
    "reader.lock",
    "retention",
    "retention/roots",
    "retention/manifests",
    "gc",
    "recovery",
    "recovery/dispositions",
];

/// The exact contiguous namespace prefix observed before recovery writes.
///
/// This is observation evidence, not a durability claim or storage identity.
/// Only recovery constructs it after validating the residue's ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreMigrationNamespacePrefix {
    extent: usize,
}

impl StoreMigrationNamespacePrefix {
    pub(super) fn observe(
        residue: &StoreMigrationResidue,
    ) -> Result<Self, StoreMigrationRecoveryAmbiguity> {
        residue
            .namespace_extent()
            .map(|extent| Self { extent })
            .map_err(
                |(absent, present)| StoreMigrationRecoveryAmbiguity::NamespaceOutOfOrder {
                    absent,
                    present,
                },
            )
    }

    /// Returns the number of observed names, including `reader.lock`.
    pub const fn len(self) -> usize {
        self.extent
    }

    /// Returns whether no namespace effect was observed.
    pub const fn is_empty(self) -> bool {
        self.extent == 0
    }

    /// Iterates the exact observed protocol names in admission order.
    ///
    /// Iteration allocates nothing and performs no I/O.
    pub fn names(self) -> impl Iterator<Item = &'static str> {
        NAMES.into_iter().take(self.extent)
    }
}
