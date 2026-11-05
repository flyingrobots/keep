//! This module owns public direct-initialization laws under the production profile.

use std::fs;
use std::io::ErrorKind;

use keep::{
    FilesystemCatalogPublisher, FilesystemNamespaceRefusal, FilesystemRetentionSnapshot,
    FilesystemRetentionSnapshotError, FilesystemStoreAdmission, FilesystemStoreAdmissionError,
    ReaderAttemptLimit, RetainedCatalogPublicationError, RetentionCurrentStateRefusal,
    RetentionEvidencePhase, StoreInitializationError, StoreInitializationPrefix,
    StoreInitializationRefusal,
};

use super::durable_sandbox::TestDirectory;
use super::populated_store;
#[cfg(target_os = "linux")]
use super::unsupported_directory::UnsupportedDirectory;
use super::writer_fixture::{TestResult, first_blob, witness};

#[test]
fn an_initialized_store_has_no_catalog_and_admits_writers_but_not_readers() -> TestResult<()> {
    let sandbox = TestDirectory::create("single-format-unpublished")?;
    let root = sandbox.path();

    let initialized = FilesystemStoreAdmission::initialize(root)?;

    assert_eq!(initialized.found(), StoreInitializationPrefix::Empty);
    drop(initialized);
    assert!(!root.join("HEAD").exists());
    drop(FilesystemStoreAdmission::reopen(
        root,
        keep::StoreExpectation::NoExpectation,
    )?);
    let reader = FilesystemRetentionSnapshot::load(
        root,
        populated_store::policy()?,
        ReaderAttemptLimit::DEFAULT,
        keep::StoreExpectation::NoExpectation,
    );
    assert!(matches!(
        reader,
        Err(FilesystemRetentionSnapshotError::Unpublished)
    ));
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn initialization_refuses_an_unsupported_platform_before_any_mutation() -> TestResult<()> {
    let directory = UnsupportedDirectory::create()?;

    let refused = FilesystemStoreAdmission::initialize(directory.path());

    let Err(StoreInitializationError::Platform { source }) = refused else {
        return Err("initialization admitted an unsupported platform".into());
    };
    assert_eq!(source.kind(), ErrorKind::Unsupported);
    assert_eq!(fs::read_dir(directory.path())?.count(), 0);
    directory.remove()?;
    Ok(())
}

// devfs is always mounted, and admission refuses it before any write could be tried.
#[cfg(target_os = "macos")]
#[test]
fn initialization_refuses_devfs_at_platform_admission() -> TestResult<()> {
    let refused = FilesystemStoreAdmission::initialize(std::path::Path::new("/dev"));

    let Err(StoreInitializationError::Platform { source }) = refused else {
        return Err("initialization admitted an unsupported platform".into());
    };
    assert_eq!(source.kind(), ErrorKind::Unsupported);
    Ok(())
}

#[test]
fn a_second_initialization_of_a_committed_store_refuses_unchanged() -> TestResult<()> {
    let sandbox = TestDirectory::create("single-format-reinitialize")?;
    let root = sandbox.path();
    let (_created, writer) = populated_store::create_holding_authority(
        root,
        populated_store::StoreAdmission::PLATFORM,
        &[&first_blob()],
        populated_store::FirstCatalogRecords::Complete,
    )?;
    drop(writer);
    let before = witness(root)?;

    let refused = FilesystemStoreAdmission::initialize(root);

    assert!(matches!(
        refused,
        Err(StoreInitializationError::Refused {
            refusal: StoreInitializationRefusal::AlreadyInitialized
        })
    ));
    assert_eq!(witness(root)?, before);
    Ok(())
}

#[test]
fn an_uncommitted_root_is_refused_by_writers_and_readers() -> TestResult<()> {
    let sandbox = TestDirectory::create("single-format-uncommitted")?;
    let root = sandbox.path();
    drop(FilesystemStoreAdmission::initialize(root)?);
    fs::hard_link(root.join("FORMAT"), root.join("FORMAT.next"))?;

    let Err(FilesystemStoreAdmissionError::Namespace { source }) =
        FilesystemStoreAdmission::reopen(root, keep::StoreExpectation::NoExpectation)
    else {
        return Err("an uncommitted root admitted a writer".into());
    };

    assert_eq!(
        source
            .get_ref()
            .and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>()),
        Some(&FilesystemNamespaceRefusal::InitializationIncomplete)
    );
    assert!(matches!(
        FilesystemRetentionSnapshot::load(
            root,
            populated_store::policy()?,
            ReaderAttemptLimit::DEFAULT,
            keep::StoreExpectation::NoExpectation
        ),
        Err(FilesystemRetentionSnapshotError::Admission { .. })
    ));
    assert_eq!(
        FilesystemStoreAdmission::initialize(root)?.found(),
        StoreInitializationPrefix::MarkerLinked
    );
    Ok(())
}

/// The first publication has no predecessor, so it must prove that no
/// retention state exists instead of assuming it: a retained root without a
/// retention head contradicts the absence and refuses before any effect.
#[test]
fn the_first_publication_refuses_retention_artifacts_without_a_head() -> TestResult<()> {
    let sandbox = TestDirectory::create("single-format-first-contradicted")?;
    let root = sandbox.path();
    let admission = FilesystemStoreAdmission::initialize(root)?.into_admission();
    let namespace = root.join("retention/roots").join("a".repeat(64));
    fs::create_dir(&namespace)?;
    let before = witness(root)?;
    let mut publisher = FilesystemCatalogPublisher::open(admission, populated_store::policy()?)?;

    let refused = populated_store::publish_first(root, &mut publisher, &[&first_blob()]);
    drop(publisher);

    let error = refused
        .err()
        .ok_or("a contradicted first publication published")?;
    let Some(RetainedCatalogPublicationError::RetentionStorage {
        phase: RetentionEvidencePhase::AdmitHead,
        source,
    }) = error.downcast_ref::<RetainedCatalogPublicationError>()
    else {
        return Err(format!("unexpected first-publication refusal: {error:?}").into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|cause| cause.downcast_ref::<RetentionCurrentStateRefusal>()),
        Some(RetentionCurrentStateRefusal::HeadAbsentWithArtifacts)
    ));
    let mut after = witness(root)?;
    after.retain(|(path, _)| path != "staging/current.seg");
    assert_eq!(
        after, before,
        "a refused first publication mutated the store"
    );
    Ok(())
}
