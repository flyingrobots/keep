//! This module owns typed refusal of cross-version writer and recovery entry.

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, FilesystemCatalogPublisher, FilesystemNamespaceRefusal,
    FilesystemPlatformAdmission, FilesystemPlatformAdmissionError,
    FilesystemRecoveryInventoryReader, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageDiscardOpenError,
    FilesystemRecoveryStageDiscarder, FilesystemVersionTwoAdmission,
    FilesystemVersionTwoRecoveryAdmission, RecoveryEntryRole, RecoveryInventoryError,
    RecoveryInventoryLimit, RecoveryNameClassificationError, RecoveryNamespace,
    RecoveryRequiredEntry, SegmentPublicationError, SegmentRecordLimit, StagedSegment,
    classify_recovery_names, classify_version_two_recovery_names,
};

use super::durable_fixture;
use super::durable_sandbox::TestDirectory;
use super::writer_fixture::{TestResult, first_blob, open_publisher, segment_policy};

fn limit() -> TestResult<RecoveryInventoryLimit> {
    Ok(RecoveryInventoryLimit::new(4096)?)
}

#[test]
fn version_one_entry_points_refuse_a_version_two_root() -> TestResult<()> {
    let sandbox = durable_fixture::build("v2-versions-refuse-v1", &[&first_blob()])?;
    let root = sandbox.path();
    let policy = durable_fixture::policy()?;

    assert!(matches!(
        FilesystemPlatformAdmission::reopen(root),
        Err(FilesystemPlatformAdmissionError::Namespace { .. })
    ));
    assert!(matches!(
        FilesystemRecoveryInventoryReader::open(root),
        Err(RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Root,
            ..
        })
    ));
    assert!(matches!(
        FilesystemRecoveryStageDiscarder::open(root),
        Err(FilesystemRecoveryStageDiscardOpenError::Namespace { .. })
    ));
    assert!(matches!(
        FilesystemRecoveryNextHeadFinalizer::open(root, policy),
        Err(FilesystemRecoveryNextHeadFinalizationOpenError::Namespace { .. })
    ));

    let inventory = FilesystemRecoveryInventoryReader::open_version_two(root)?.read(limit()?)?;
    assert!(matches!(
        classify_recovery_names(inventory.clone()),
        Err(RecoveryNameClassificationError::Unexpected {
            namespace: RecoveryNamespace::Root,
            ..
        })
    ));
    let manifest = classify_version_two_recovery_names(inventory)?;
    for role in [
        RecoveryEntryRole::ReaderFence,
        RecoveryEntryRole::FormatMarker,
        RecoveryEntryRole::RetentionDirectory,
        RecoveryEntryRole::GcDirectory,
        RecoveryEntryRole::RecoveryDirectory,
        RecoveryEntryRole::CurrentHead,
    ] {
        assert!(
            manifest.entries().iter().any(|entry| entry.role() == role),
            "version-two root entry {role:?} was not classified as known"
        );
    }
    Ok(())
}

#[test]
fn version_two_entry_points_refuse_a_version_one_root() -> TestResult<()> {
    let sandbox = TestDirectory::create("v2-versions-refuse-v2")?;
    let root = sandbox.path();
    drop(FilesystemPlatformAdmission::initialize(root)?);
    let missing = |error: &FilesystemPlatformAdmissionError| {
        matches!(
            error,
            FilesystemPlatformAdmissionError::Namespace { source }
                if source.get_ref().and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>())
                    == Some(&FilesystemNamespaceRefusal::MissingFormatMarker)
        )
    };

    let Err(writer) = FilesystemVersionTwoAdmission::reopen(root) else {
        return Err("version-two writer admission admitted a version-one root".into());
    };
    assert!(missing(&writer), "unexpected refusal: {writer:?}");
    let Err(recovery) = FilesystemVersionTwoRecoveryAdmission::reopen(root) else {
        return Err("version-two recovery admission admitted a version-one root".into());
    };
    assert!(missing(&recovery), "unexpected refusal: {recovery:?}");
    let Err(RecoveryInventoryError::Io {
        namespace: RecoveryNamespace::Root,
        source,
        ..
    }) = FilesystemRecoveryInventoryReader::open_version_two(root)
    else {
        return Err("version-two inventory admitted a version-one root".into());
    };
    let admission = source
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<FilesystemPlatformAdmissionError>())
        .ok_or("inventory refusal lost its admission source")?;
    assert!(missing(admission), "unexpected refusal: {admission:?}");

    let inventory = FilesystemRecoveryInventoryReader::open(root)?.read(limit()?)?;
    assert!(matches!(
        classify_version_two_recovery_names(inventory),
        Err(RecoveryNameClassificationError::Missing {
            required: RecoveryRequiredEntry::ReaderFence
        })
    ));
    Ok(())
}

#[test]
fn a_version_one_segment_stage_cannot_be_selected_by_the_version_two_writer() -> TestResult<()> {
    let version_one = TestDirectory::create("v2-versions-stage-v1")?;
    let admission = FilesystemPlatformAdmission::initialize(version_one.path())?;
    let legacy = FilesystemCatalogPublisher::open(admission, durable_fixture::policy()?)?;
    let sealed = StagedSegment::begin(legacy.create_segment_stage()?, SegmentRecordLimit::MAXIMUM)?
        .append(AdmittedSegmentRecord::for_chunk(b"version-one stage")?)?
        .seal()?;
    let bytes = std::fs::read(version_one.path().join("staging/current.seg"))?;
    let segment = AdmittedSegment::decode(&bytes, segment_policy())?;

    let version_two = durable_fixture::build("v2-versions-stage-v2", &[&first_blob()])?;
    let publisher = open_publisher(version_two.path())?;
    assert!(matches!(
        publisher.select_segment(sealed, &segment),
        Err(SegmentPublicationError::PublisherAuthority)
    ));
    Ok(())
}
