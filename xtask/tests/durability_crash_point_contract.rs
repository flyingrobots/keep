//! Executable identity contract for deterministic crash injection.

#![cfg(feature = "repository-tasks")]

use xtask::{DurabilityCrashPoint, DurabilityCrashSequence};

use DurabilityCrashSequence::{
    Catalog, Head, Initialization, RecoveryDiscard, Retention, Segment, VersionTwoCatalog,
    VersionTwoInitialization,
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
        DurabilityCrashPoint::VersionTwoLinkSegment,
        "KEEP-CRASH-074",
        VersionTwoCatalog,
    ),
    (
        DurabilityCrashPoint::VersionTwoCreateCatalogStage,
        "KEEP-CRASH-075",
        VersionTwoCatalog,
    ),
    (
        DurabilityCrashPoint::VersionTwoSynchronizeHead,
        "KEEP-CRASH-076",
        VersionTwoCatalog,
    ),
    (
        DurabilityCrashPoint::VersionTwoReplaceHead,
        "KEEP-CRASH-077",
        VersionTwoCatalog,
    ),
    (
        DurabilityCrashPoint::InitializeOpenAndLockWriterFile,
        "KEEP-CRASH-081",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeWriteMarkerStage,
        "KEEP-CRASH-082",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeSynchronizeMarkerStage,
        "KEEP-CRASH-083",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeSynchronizeRootAfterMarkerStage,
        "KEEP-CRASH-084",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeAdmitReaderFence,
        "KEEP-CRASH-085",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeAdmitDirectory,
        "KEEP-CRASH-086",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeLinkMarker,
        "KEEP-CRASH-087",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeSynchronizeRootAfterMarker,
        "KEEP-CRASH-088",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeRemoveMarkerStage,
        "KEEP-CRASH-089",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeSynchronizeRootAfterCleanup,
        "KEEP-CRASH-090",
        VersionTwoInitialization,
    ),
    (
        DurabilityCrashPoint::InitializeDiscardIncompleteMarkerStage,
        "KEEP-CRASH-091",
        VersionTwoInitialization,
    ),
];

#[test]
fn crash_boundaries_have_one_contiguous_stable_vocabulary() {
    let actual =
        DurabilityCrashPoint::ALL.map(|point| (point, point.identifier(), point.sequence()));

    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn only_record_append_and_initialization_directories_select_an_occurrence() {
    let occurrence_counted: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.occurrence_counted())
        .collect();

    assert_eq!(
        occurrence_counted,
        [
            DurabilityCrashPoint::AppendSegmentRecord,
            DurabilityCrashPoint::InitializeAdmitDirectory
        ]
    );
}

#[test]
fn initialization_directory_admission_runs_one_during_case_per_directory() {
    for point in DurabilityCrashPoint::ALL {
        let expected = if point == DurabilityCrashPoint::InitializeAdmitDirectory {
            DurabilityCrashPoint::INITIALIZATION_DIRECTORIES
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
    assert_eq!(
        DurabilityCrashPoint::INITIALIZATION_DIRECTORIES,
        u32::try_from(keep::VersionTwoInitializationDirectory::ALL.len()).unwrap_or(u32::MAX)
    );
}

#[test]
fn retired_crash_identifiers_are_never_reassigned() {
    for ordinal in (53..=73).chain(78..=80) {
        let identifier = format!("KEEP-CRASH-{ordinal:03}");
        assert_eq!(
            DurabilityCrashPoint::from_identifier(&identifier),
            None,
            "{identifier} is retired or reserved and must not name a boundary"
        );
    }
}

#[test]
fn initialization_boundaries_take_consecutive_new_identifiers() {
    let initialization: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.sequence() == VersionTwoInitialization)
        .map(DurabilityCrashPoint::identifier)
        .collect();
    let expected: Vec<_> = (81..=91)
        .map(|ordinal| format!("KEEP-CRASH-{ordinal:03}"))
        .collect();
    assert_eq!(initialization, expected);
}

// Subject: repository-tool runtime, not Keep recovery acceptance evidence.
// Size: small; oracle: the exact sequence names documented in README.md.
// Delete if this CLI is removed or a stronger command-boundary test subsumes it.
#[test]
fn crash_matrix_cli_admits_its_stable_sequence_names() {
    for (name, expected) in [
        ("segment", Segment),
        ("catalog", Catalog),
        ("head", Head),
        ("recovery-discard", RecoveryDiscard),
        ("initialization", Initialization),
        ("retention", Retention),
        ("version-two-catalog", VersionTwoCatalog),
        ("version-two-initialization", VersionTwoInitialization),
    ] {
        assert_eq!(
            DurabilityCrashSequence::from_identifier(name),
            Some(expected),
            "CLI sequence {name} must select {expected:?}"
        );
    }
}

// Subject: repository-tool runtime. Size: small; oracle: exact CLI vocabulary.
// Delete if this CLI is removed or a stronger command-boundary test subsumes it.
#[test]
fn crash_matrix_cli_refuses_names_outside_its_exact_vocabulary() {
    for name in ["", "unknown", "Retention", "retentions", "retention "] {
        assert_eq!(
            DurabilityCrashSequence::from_identifier(name),
            None,
            "unsupported CLI sequence {name:?} must refuse"
        );
    }
}
