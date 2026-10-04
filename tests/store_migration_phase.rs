//! Ordered version-2 store-migration durability phase laws.

use keep::StoreMigrationPhase;

const EXPECTED: [(StoreMigrationPhase, &str); 21] = [
    (
        StoreMigrationPhase::WriteIntentStage,
        "migration-intent stage write",
    ),
    (
        StoreMigrationPhase::SynchronizeIntentStage,
        "migration-intent stage synchronization",
    ),
    (
        StoreMigrationPhase::LinkIntent,
        "migration-intent canonical link",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterIntent,
        "store-root synchronization after intent link",
    ),
    (
        StoreMigrationPhase::RemoveIntentStage,
        "migration-intent stage removal",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterIntentCleanup,
        "store-root synchronization after intent cleanup",
    ),
    (
        StoreMigrationPhase::AdmitReaderFence,
        "persistent reader-fence admission",
    ),
    (
        StoreMigrationPhase::AdmitNamespacePrefix,
        "canonical namespace-prefix admission",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterNamespace,
        "store-root synchronization after namespace admission",
    ),
    (
        StoreMigrationPhase::WriteMarkerStage,
        "format-marker stage write",
    ),
    (
        StoreMigrationPhase::SynchronizeMarkerStage,
        "format-marker stage synchronization",
    ),
    (
        StoreMigrationPhase::LinkMarker,
        "format-marker canonical link",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterMarker,
        "store-root synchronization after marker link",
    ),
    (
        StoreMigrationPhase::RemoveMarkerStage,
        "format-marker stage removal",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup,
        "store-root synchronization after marker cleanup",
    ),
    (
        StoreMigrationPhase::WriteReceiptStage,
        "migration-receipt stage write",
    ),
    (
        StoreMigrationPhase::SynchronizeReceiptStage,
        "migration-receipt stage synchronization",
    ),
    (
        StoreMigrationPhase::LinkReceipt,
        "migration-receipt canonical link",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterReceipt,
        "store-root synchronization after receipt link",
    ),
    (
        StoreMigrationPhase::RemoveReceiptStage,
        "migration-receipt stage removal",
    ),
    (
        StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup,
        "final store-root synchronization",
    ),
];

#[test]
fn migration_phases_are_complete_ordered_and_stably_named() {
    assert_eq!(
        StoreMigrationPhase::ALL,
        EXPECTED.map(|(phase, _name)| phase)
    );
    for (phase, name) in EXPECTED {
        assert_eq!(phase.to_string(), name);
    }
}
