//! This boundary module owns the blocking capabilities migration recovery
//! needs beyond the forward protocol.

use std::io;

use super::{
    CanonicalStoreMigrationIntent, StoreMigrationFixedStage, StoreMigrationResidue,
    StoreMigrationStorage,
};

/// Blocking storage capabilities for recovering one interrupted migration.
///
/// An implementation holds exclusive writer authority over a root that may
/// carry any lawful migration residue. It observes without mutation, adopts
/// the exact stage and canonical handles a resume point needs by reopening
/// them by device and inode identity, and removes only an incomplete
/// pre-effect stage the planner named.
pub trait StoreMigrationRecoveryStorage: StoreMigrationStorage {
    /// Observes every fixed migration name without mutation.
    ///
    /// # Errors
    ///
    /// Returns the exact I/O failure, or a typed refusal for a wrong file
    /// kind, a link, or an unknown entry.
    fn observe_residue(&mut self) -> io::Result<StoreMigrationResidue>;

    /// Reopens, verifies, and retains every exact stage and canonical record
    /// the residue holds, so later phases find the handles the forward
    /// protocol would have retained.
    ///
    /// # Errors
    ///
    /// Returns the exact reopen, identity, or byte verification failure.
    fn adopt_residue(
        &mut self,
        residue: &StoreMigrationResidue,
        intent: &CanonicalStoreMigrationIntent,
    ) -> io::Result<()>;

    /// Removes one incomplete pre-effect stage and synchronizes the root.
    ///
    /// # Errors
    ///
    /// Returns the exact removal or synchronization failure, or a typed
    /// refusal when the stage's canonical target already exists.
    fn discard_stage(&mut self, stage: StoreMigrationFixedStage) -> io::Result<()>;
}
