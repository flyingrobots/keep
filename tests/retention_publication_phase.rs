//! Retention publication phase vocabulary laws.

use keep::RetentionPublicationPhase;

#[test]
fn publication_phases_are_complete_ordered_and_stably_named() {
    let expected = [
        (
            RetentionPublicationPhase::WriteRootStage,
            "root-stage write",
        ),
        (
            RetentionPublicationPhase::SynchronizeRootStage,
            "root-stage synchronization",
        ),
        (
            RetentionPublicationPhase::AdmitRootNamespace,
            "root-namespace admission",
        ),
        (
            RetentionPublicationPhase::SynchronizeRootsAfterNamespace,
            "post-namespace roots synchronization",
        ),
        (RetentionPublicationPhase::LinkRoot, "immutable root link"),
        (
            RetentionPublicationPhase::SynchronizeRootNamespace,
            "root-namespace synchronization",
        ),
        (
            RetentionPublicationPhase::WriteManifestStage,
            "manifest-stage write",
        ),
        (
            RetentionPublicationPhase::SynchronizeManifestStage,
            "manifest-stage synchronization",
        ),
        (
            RetentionPublicationPhase::LinkManifest,
            "immutable manifest link",
        ),
        (
            RetentionPublicationPhase::SynchronizeManifestPool,
            "manifest-pool synchronization",
        ),
        (
            RetentionPublicationPhase::WriteHeadStage,
            "retention-head-stage write",
        ),
        (
            RetentionPublicationPhase::SynchronizeHeadStage,
            "retention-head-stage synchronization",
        ),
        (
            RetentionPublicationPhase::ReplaceHead,
            "retention-head replacement",
        ),
        (
            RetentionPublicationPhase::SynchronizeRetentionNamespace,
            "retention-namespace synchronization",
        ),
        (
            RetentionPublicationPhase::RemoveRootStage,
            "retained root-stage removal",
        ),
        (
            RetentionPublicationPhase::RemoveManifestStage,
            "retained manifest-stage removal",
        ),
        (
            RetentionPublicationPhase::SynchronizeCleanup,
            "retention cleanup synchronization",
        ),
    ];

    assert_eq!(
        RetentionPublicationPhase::ALL,
        expected.map(|entry| entry.0)
    );
    assert_eq!(
        RetentionPublicationPhase::ALL.map(|phase| phase.to_string()),
        expected.map(|entry| entry.1.to_owned())
    );
}
