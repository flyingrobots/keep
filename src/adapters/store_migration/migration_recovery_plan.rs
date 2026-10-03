//! This boundary module owns the lawful responses to migration residue.

use super::StoreMigrationPhase;

/// One fixed-name migration stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationFixedStage {
    /// `migration.intent.next`.
    Intent,
    /// `FORMAT.next`.
    Marker,
    /// `migration.receipt.next`.
    Receipt,
}

/// The one lawful response to an observed migration residue.
///
/// A plan is a statement about the residue; executing it is the recovery
/// storage's job. Every resume point is the earliest phase whose effect the
/// residue cannot prove, so re-running from it is idempotent.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationRecoveryPlan {
    /// No migration artifact exists: the exact version-1 store admits.
    VersionOne,
    /// Remove one incomplete pre-effect stage, then resume at `resume`.
    DiscardStage {
        /// The incomplete stage to remove.
        stage: StoreMigrationFixedStage,
        /// The forward phase to resume at after removal.
        resume: StoreMigrationPhase,
    },
    /// Resume the forward protocol at `resume`.
    Resume {
        /// The earliest phase the residue cannot prove complete.
        resume: StoreMigrationPhase,
    },
    /// The exact receipt is canonical and no stage remains.
    Complete,
}
