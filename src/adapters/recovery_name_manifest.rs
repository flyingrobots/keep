//! This module owns canonical recovery name manifests.

use super::{RecoveryEntryName, RecoveryEntryRole, RecoveryNamespace};

/// One namespace entry paired with its canonical semantic role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryNamedEntry {
    namespace: RecoveryNamespace,
    name: RecoveryEntryName,
    role: RecoveryEntryRole,
}

impl RecoveryNamedEntry {
    pub(super) const fn new(
        namespace: RecoveryNamespace,
        name: RecoveryEntryName,
        role: RecoveryEntryRole,
    ) -> Self {
        Self {
            namespace,
            name,
            role,
        }
    }

    /// Returns the owning namespace.
    #[must_use]
    pub const fn namespace(&self) -> RecoveryNamespace {
        self.namespace
    }

    /// Returns the exact raw entry name.
    #[must_use]
    pub const fn name(&self) -> &RecoveryEntryName {
        &self.name
    }

    /// Returns the canonical semantic role.
    #[must_use]
    pub const fn role(&self) -> RecoveryEntryRole {
        self.role
    }
}

/// One complete canonical namespace manifest awaiting content classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryNameManifest {
    entries: Vec<RecoveryNamedEntry>,
}

impl RecoveryNameManifest {
    pub(super) const fn new(entries: Vec<RecoveryNamedEntry>) -> Self {
        Self { entries }
    }

    /// Returns entries in inventory namespace-and-name order.
    #[must_use]
    pub fn entries(&self) -> &[RecoveryNamedEntry] {
        &self.entries
    }
}
