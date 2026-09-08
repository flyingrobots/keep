//! This boundary module owns exact store-migration durability phases.

use std::fmt;

/// Storage transition attempted by version-2 store migration.
///
/// [`Self::ALL`] corresponds in order to `KEEP-CRASH-053` through
/// `KEEP-CRASH-073`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationPhase {
    /// Write the complete canonical `migration.intent.next`.
    WriteIntentStage,
    /// Synchronize `migration.intent.next`.
    SynchronizeIntentStage,
    /// Link the synchronized intent stage to `migration.intent`.
    LinkIntent,
    /// Synchronize the store root after the intent link.
    SynchronizeRootAfterIntent,
    /// Remove the retained `migration.intent.next`.
    RemoveIntentStage,
    /// Synchronize the store root after intent-stage cleanup.
    SynchronizeRootAfterIntentCleanup,
    /// Create or exactly admit the persistent reader fence.
    AdmitReaderFence,
    /// Create or exactly admit the canonical version-2 directory prefix.
    AdmitNamespacePrefix,
    /// Synchronize the store root after namespace admission.
    SynchronizeRootAfterNamespace,
    /// Write the complete canonical `FORMAT.next`.
    WriteMarkerStage,
    /// Synchronize `FORMAT.next`.
    SynchronizeMarkerStage,
    /// Link the synchronized marker stage to `FORMAT`.
    LinkMarker,
    /// Synchronize the store root after the marker link.
    SynchronizeRootAfterMarker,
    /// Remove the retained `FORMAT.next`.
    RemoveMarkerStage,
    /// Synchronize the store root after marker-stage cleanup.
    SynchronizeRootAfterMarkerCleanup,
    /// Write the complete canonical `migration.receipt.next`.
    WriteReceiptStage,
    /// Synchronize `migration.receipt.next`.
    SynchronizeReceiptStage,
    /// Link the synchronized receipt stage to `migration.receipt`.
    LinkReceipt,
    /// Synchronize the store root after the receipt link.
    SynchronizeRootAfterReceipt,
    /// Remove the retained `migration.receipt.next`.
    RemoveReceiptStage,
    /// Synchronize the store root after receipt-stage cleanup.
    SynchronizeRootAfterReceiptCleanup,
}

impl StoreMigrationPhase {
    /// Every migration phase in normative crash-boundary order.
    pub const ALL: [Self; 21] = [
        Self::WriteIntentStage,
        Self::SynchronizeIntentStage,
        Self::LinkIntent,
        Self::SynchronizeRootAfterIntent,
        Self::RemoveIntentStage,
        Self::SynchronizeRootAfterIntentCleanup,
        Self::AdmitReaderFence,
        Self::AdmitNamespacePrefix,
        Self::SynchronizeRootAfterNamespace,
        Self::WriteMarkerStage,
        Self::SynchronizeMarkerStage,
        Self::LinkMarker,
        Self::SynchronizeRootAfterMarker,
        Self::RemoveMarkerStage,
        Self::SynchronizeRootAfterMarkerCleanup,
        Self::WriteReceiptStage,
        Self::SynchronizeReceiptStage,
        Self::LinkReceipt,
        Self::SynchronizeRootAfterReceipt,
        Self::RemoveReceiptStage,
        Self::SynchronizeRootAfterReceiptCleanup,
    ];
}

impl fmt::Display for StoreMigrationPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WriteIntentStage => "migration-intent stage write",
            Self::SynchronizeIntentStage => "migration-intent stage synchronization",
            Self::LinkIntent => "migration-intent canonical link",
            Self::SynchronizeRootAfterIntent => "store-root synchronization after intent link",
            Self::RemoveIntentStage => "migration-intent stage removal",
            Self::SynchronizeRootAfterIntentCleanup => {
                "store-root synchronization after intent cleanup"
            }
            Self::AdmitReaderFence => "persistent reader-fence admission",
            Self::AdmitNamespacePrefix => "canonical namespace-prefix admission",
            Self::SynchronizeRootAfterNamespace => {
                "store-root synchronization after namespace admission"
            }
            Self::WriteMarkerStage => "format-marker stage write",
            Self::SynchronizeMarkerStage => "format-marker stage synchronization",
            Self::LinkMarker => "format-marker canonical link",
            Self::SynchronizeRootAfterMarker => "store-root synchronization after marker link",
            Self::RemoveMarkerStage => "format-marker stage removal",
            Self::SynchronizeRootAfterMarkerCleanup => {
                "store-root synchronization after marker cleanup"
            }
            Self::WriteReceiptStage => "migration-receipt stage write",
            Self::SynchronizeReceiptStage => "migration-receipt stage synchronization",
            Self::LinkReceipt => "migration-receipt canonical link",
            Self::SynchronizeRootAfterReceipt => "store-root synchronization after receipt link",
            Self::RemoveReceiptStage => "migration-receipt stage removal",
            Self::SynchronizeRootAfterReceiptCleanup => "final store-root synchronization",
        })
    }
}
