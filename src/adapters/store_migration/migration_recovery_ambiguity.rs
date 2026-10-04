//! This boundary module owns migration residue that no lawful plan resolves.

use super::{
    StoreFormatMarkerDecodeError, StoreMigrationFixedStage, StoreMigrationIntentDecodeError,
    StoreMigrationReceiptDecodeError,
};

/// A migration effect that can exist only after a durable intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationEffect {
    /// `reader.lock` or a prefix directory.
    Namespace,
    /// `FORMAT` or `FORMAT.next`.
    Marker,
    /// `migration.receipt` or `migration.receipt.next`.
    Receipt,
}

/// Residue that recovery refuses to interpret.
///
/// Recovery never guesses which step produced conflicting evidence; every
/// variant names the exact conflict so a human can resolve it.
#[derive(Debug)]
pub enum StoreMigrationRecoveryAmbiguity {
    /// A later effect exists although no durable intent does.
    EffectBeforeIntent {
        /// The earliest effect found.
        effect: StoreMigrationEffect,
    },
    /// A stage survived an effect that requires its prior removal.
    StageAfterEffect {
        /// The surviving stage.
        stage: StoreMigrationFixedStage,
        /// The later effect.
        effect: StoreMigrationEffect,
    },
    /// A complete-length stage does not decode as a canonical record.
    StageUndecodable {
        /// Preserved stage decoder failure.
        source: super::StoreMigrationStageDecodeError,
        /// The stage.
        stage: StoreMigrationFixedStage,
    },
    /// A stage is longer than its canonical record.
    StageOverlong {
        /// The stage.
        stage: StoreMigrationFixedStage,
        /// Observed byte length.
        observed: usize,
    },
    /// A complete stage and its canonical target, or a complete pre-effect
    /// stage and the expected record, carry different bytes.
    StageDiffers {
        /// The stage.
        stage: StoreMigrationFixedStage,
    },
    /// `migration.intent` does not decode.
    IntentUndecodable {
        /// Preserved decode failure.
        source: StoreMigrationIntentDecodeError,
    },
    /// `migration.intent` binds a different version-1 store or root
    /// (any coordinate but the mount identity).
    IntentDiffers,
    /// `reader.lock` and the directory prefix are not one contiguous prefix.
    NamespaceOutOfOrder {
        /// Index of the first absent slot (0 is `reader.lock`).
        absent: usize,
        /// Index of a later present slot.
        present: usize,
    },
    /// A marker artifact exists before the namespace prefix is complete.
    MarkerBeforeNamespace,
    /// `FORMAT` does not decode as the registered version-2 marker.
    MarkerUndecodable {
        /// Preserved decode failure.
        source: StoreFormatMarkerDecodeError,
    },
    /// A receipt artifact exists before `FORMAT`.
    ReceiptBeforeMarker,
    /// `migration.receipt` does not bind the observed intent and marker.
    ReceiptUndecodable {
        /// Preserved decode failure.
        source: StoreMigrationReceiptDecodeError,
    },
}
