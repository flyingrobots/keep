//! Executable identity contract for deterministic crash injection.

#![cfg(feature = "repository-tasks")]

use xtask::{DurabilityCrashPoint, DurabilityCrashSequence};

use DurabilityCrashSequence::{
    Catalog, Head, Initialization, Migration, RecoveryDiscard, Retention, Segment,
};

const EXPECTED: &[(DurabilityCrashPoint, &str, DurabilityCrashSequence)] = &[
    (
        DurabilityCrashPoint::CreateSegmentStage,
        "KEEP-CRASH-001",
        Segment,
    ),
    (
        DurabilityCrashPoint::WriteSegmentHeader,
        "KEEP-CRASH-002",
        Segment,
    ),
    (
        DurabilityCrashPoint::AppendSegmentRecord,
        "KEEP-CRASH-003",
        Segment,
    ),
    (
        DurabilityCrashPoint::FlushSegmentRecordPrefix,
        "KEEP-CRASH-004",
        Segment,
    ),
    (
        DurabilityCrashPoint::SynchronizeSegmentRecordPrefix,
        "KEEP-CRASH-005",
        Segment,
    ),
    (
        DurabilityCrashPoint::AppendSegmentSeal,
        "KEEP-CRASH-006",
        Segment,
    ),
    (
        DurabilityCrashPoint::FlushSealedSegment,
        "KEEP-CRASH-007",
        Segment,
    ),
    (
        DurabilityCrashPoint::SynchronizeSealedSegment,
        "KEEP-CRASH-008",
        Segment,
    ),
    (DurabilityCrashPoint::LinkSegment, "KEEP-CRASH-009", Segment),
    (
        DurabilityCrashPoint::SynchronizeSegmentPool,
        "KEEP-CRASH-010",
        Segment,
    ),
    (
        DurabilityCrashPoint::RemoveSegmentStage,
        "KEEP-CRASH-011",
        Segment,
    ),
    (
        DurabilityCrashPoint::SynchronizeStagingAfterSegment,
        "KEEP-CRASH-012",
        Segment,
    ),
    (
        DurabilityCrashPoint::CreateCatalogStage,
        "KEEP-CRASH-013",
        Catalog,
    ),
    (
        DurabilityCrashPoint::WriteCatalog,
        "KEEP-CRASH-014",
        Catalog,
    ),
    (
        DurabilityCrashPoint::FlushCatalog,
        "KEEP-CRASH-015",
        Catalog,
    ),
    (
        DurabilityCrashPoint::SynchronizeCatalog,
        "KEEP-CRASH-016",
        Catalog,
    ),
    (DurabilityCrashPoint::LinkCatalog, "KEEP-CRASH-017", Catalog),
    (
        DurabilityCrashPoint::SynchronizeCatalogPool,
        "KEEP-CRASH-018",
        Catalog,
    ),
    (
        DurabilityCrashPoint::RemoveCatalogStage,
        "KEEP-CRASH-019",
        Catalog,
    ),
    (
        DurabilityCrashPoint::SynchronizeStagingAfterCatalog,
        "KEEP-CRASH-020",
        Catalog,
    ),
    (
        DurabilityCrashPoint::CreateHeadStage,
        "KEEP-CRASH-021",
        Head,
    ),
    (DurabilityCrashPoint::WriteHead, "KEEP-CRASH-022", Head),
    (DurabilityCrashPoint::FlushHead, "KEEP-CRASH-023", Head),
    (
        DurabilityCrashPoint::SynchronizeHead,
        "KEEP-CRASH-024",
        Head,
    ),
    (DurabilityCrashPoint::ReplaceHead, "KEEP-CRASH-025", Head),
    (
        DurabilityCrashPoint::SynchronizeRootAfterHead,
        "KEEP-CRASH-026",
        Head,
    ),
    (
        DurabilityCrashPoint::RemoveRecoveryStage,
        "KEEP-CRASH-027",
        RecoveryDiscard,
    ),
    (
        DurabilityCrashPoint::SynchronizeStagingAfterRecovery,
        "KEEP-CRASH-028",
        RecoveryDiscard,
    ),
    (
        DurabilityCrashPoint::RemoveRecoveryHead,
        "KEEP-CRASH-029",
        RecoveryDiscard,
    ),
    (
        DurabilityCrashPoint::SynchronizeRootAfterRecovery,
        "KEEP-CRASH-030",
        RecoveryDiscard,
    ),
    (
        DurabilityCrashPoint::OpenAndLockWriterFile,
        "KEEP-CRASH-031",
        Initialization,
    ),
    (
        DurabilityCrashPoint::CreateStagingDirectory,
        "KEEP-CRASH-032",
        Initialization,
    ),
    (
        DurabilityCrashPoint::CreateSegmentPoolDirectory,
        "KEEP-CRASH-033",
        Initialization,
    ),
    (
        DurabilityCrashPoint::CreateCatalogPoolDirectory,
        "KEEP-CRASH-034",
        Initialization,
    ),
    (
        DurabilityCrashPoint::SynchronizeRootAfterInitialization,
        "KEEP-CRASH-035",
        Initialization,
    ),
    (
        DurabilityCrashPoint::WriteRootStage,
        "KEEP-CRASH-036",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeRootStage,
        "KEEP-CRASH-037",
        Retention,
    ),
    (
        DurabilityCrashPoint::AdmitRootNamespace,
        "KEEP-CRASH-038",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeRootsAfterNamespace,
        "KEEP-CRASH-039",
        Retention,
    ),
    (DurabilityCrashPoint::LinkRoot, "KEEP-CRASH-040", Retention),
    (
        DurabilityCrashPoint::SynchronizeRootNamespace,
        "KEEP-CRASH-041",
        Retention,
    ),
    (
        DurabilityCrashPoint::WriteManifestStage,
        "KEEP-CRASH-042",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeManifestStage,
        "KEEP-CRASH-043",
        Retention,
    ),
    (
        DurabilityCrashPoint::LinkManifest,
        "KEEP-CRASH-044",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeManifestPool,
        "KEEP-CRASH-045",
        Retention,
    ),
    (
        DurabilityCrashPoint::WriteHeadStage,
        "KEEP-CRASH-046",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeHeadStage,
        "KEEP-CRASH-047",
        Retention,
    ),
    (
        DurabilityCrashPoint::ReplaceRetentionHead,
        "KEEP-CRASH-048",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeRetentionNamespace,
        "KEEP-CRASH-049",
        Retention,
    ),
    (
        DurabilityCrashPoint::RemoveRootStage,
        "KEEP-CRASH-050",
        Retention,
    ),
    (
        DurabilityCrashPoint::RemoveManifestStage,
        "KEEP-CRASH-051",
        Retention,
    ),
    (
        DurabilityCrashPoint::SynchronizeRetentionCleanup,
        "KEEP-CRASH-052",
        Retention,
    ),
    (
        DurabilityCrashPoint::MigrationWriteIntentStage,
        "KEEP-CRASH-053",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeIntentStage,
        "KEEP-CRASH-054",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationLinkIntent,
        "KEEP-CRASH-055",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterIntent,
        "KEEP-CRASH-056",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationRemoveIntentStage,
        "KEEP-CRASH-057",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterIntentCleanup,
        "KEEP-CRASH-058",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationAdmitReaderFence,
        "KEEP-CRASH-059",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationAdmitNamespacePrefix,
        "KEEP-CRASH-060",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterNamespace,
        "KEEP-CRASH-061",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationWriteMarkerStage,
        "KEEP-CRASH-062",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeMarkerStage,
        "KEEP-CRASH-063",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationLinkMarker,
        "KEEP-CRASH-064",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterMarker,
        "KEEP-CRASH-065",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationRemoveMarkerStage,
        "KEEP-CRASH-066",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterMarkerCleanup,
        "KEEP-CRASH-067",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationWriteReceiptStage,
        "KEEP-CRASH-068",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeReceiptStage,
        "KEEP-CRASH-069",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationLinkReceipt,
        "KEEP-CRASH-070",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterReceipt,
        "KEEP-CRASH-071",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationRemoveReceiptStage,
        "KEEP-CRASH-072",
        Migration,
    ),
    (
        DurabilityCrashPoint::MigrationSynchronizeRootAfterReceiptCleanup,
        "KEEP-CRASH-073",
        Migration,
    ),
];

#[test]
fn crash_boundaries_have_one_contiguous_stable_vocabulary() {
    let actual =
        DurabilityCrashPoint::ALL.map(|point| (point, point.identifier(), point.sequence()));

    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn only_record_append_and_namespace_prefix_select_an_occurrence() {
    let occurrence_counted: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.occurrence_counted())
        .collect();

    assert_eq!(
        occurrence_counted,
        [
            DurabilityCrashPoint::AppendSegmentRecord,
            DurabilityCrashPoint::MigrationAdmitNamespacePrefix
        ]
    );
}

#[test]
fn the_namespace_prefix_runs_one_during_case_per_directory() {
    for point in DurabilityCrashPoint::ALL {
        let expected = if point == DurabilityCrashPoint::MigrationAdmitNamespacePrefix {
            DurabilityCrashPoint::NAMESPACE_PREFIX_DIRECTORIES
        } else {
            1
        };
        assert_eq!(
            point.during_occurrences(),
            expected,
            "{}",
            point.identifier()
        );
    }
    assert_eq!(DurabilityCrashPoint::NAMESPACE_PREFIX_DIRECTORIES, 6);
}

#[test]
fn migration_boundaries_follow_the_twenty_one_phases_in_order() {
    let migration: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.sequence() == Migration)
        .collect();

    assert_eq!(migration, DurabilityCrashPoint::MIGRATION);
    assert_eq!(migration.len(), keep::StoreMigrationPhase::ALL.len());
    assert_eq!(
        DurabilityCrashPoint::MIGRATION.map(DurabilityCrashPoint::identifier),
        std::array::from_fn::<_, 21, _>(|index| {
            let ordinal = 53 + index;
            let identifier = format!("KEEP-CRASH-{ordinal:03}");
            DurabilityCrashPoint::from_identifier(&identifier)
                .map_or("missing", DurabilityCrashPoint::identifier)
        })
    );
}

#[test]
fn sequences_round_trip_their_command_line_identifiers() {
    for sequence in DurabilityCrashSequence::ALL {
        assert_eq!(
            DurabilityCrashSequence::from_identifier(sequence.identifier()),
            Some(sequence)
        );
    }
    assert_eq!(
        DurabilityCrashSequence::from_identifier("publication"),
        None
    );
}
