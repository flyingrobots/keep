//! This module owns refusal of a root without the format marker at every open path.
//!
//! Removing `FORMAT` from an otherwise valid, published, retained store is the
//! sharpest form of a root that is not the single format: every other entry
//! and record still admits, so a refusal can only come from the marker law.

use std::fs;
use std::path::Path;

use keep::{
    CatalogRestartError, DurableStore, DurableStoreError, FilesystemCatalogSnapshot,
    FilesystemNamespaceRefusal, FilesystemPlatformAdmissionError,
    FilesystemRecoveryInventoryReader, FilesystemRetentionSnapshot,
    FilesystemRetentionSnapshotError, FilesystemVersionTwoAdmission,
    FilesystemVersionTwoRecoveryAdmission, ReaderAttemptLimit, RecoveryEntryRole,
    RecoveryInventoryError, RecoveryInventoryLimit, RecoveryNamespace,
    VersionTwoInitializationError, classify_version_two_recovery_names,
};

use super::durable_fixture;
use super::writer_fixture::{TestResult, first_blob, witness};

fn limit() -> TestResult<RecoveryInventoryLimit> {
    Ok(RecoveryInventoryLimit::new(4096)?)
}

#[test]
fn every_root_entry_of_a_published_store_is_a_known_role() -> TestResult<()> {
    let sandbox = durable_fixture::build("format-marker-roles", &[&first_blob()])?;

    let inventory =
        FilesystemRecoveryInventoryReader::open_version_two(sandbox.path())?.read(limit()?)?;
    let manifest = classify_version_two_recovery_names(inventory)?;

    for role in [
        RecoveryEntryRole::WriterLock,
        RecoveryEntryRole::ReaderFence,
        RecoveryEntryRole::FormatMarker,
        RecoveryEntryRole::RetentionDirectory,
        RecoveryEntryRole::GcDirectory,
        RecoveryEntryRole::RecoveryDirectory,
        RecoveryEntryRole::CurrentHead,
    ] {
        assert!(
            manifest.entries().iter().any(|entry| entry.role() == role),
            "root entry {role:?} was not classified as known"
        );
    }
    Ok(())
}

#[test]
fn every_open_path_refuses_a_root_without_the_format_marker() -> TestResult<()> {
    let sandbox = durable_fixture::build("format-marker-missing", &[&first_blob()])?;
    let root = sandbox.path();
    fs::remove_file(root.join("FORMAT"))?;
    let before = witness(root)?;

    require_writer_paths_refuse(root)?;
    require_inventory_refuses(root)?;
    require_content_readers_refuse(root)?;

    assert_eq!(witness(root)?, before);
    Ok(())
}

fn require_writer_paths_refuse(root: &Path) -> TestResult<()> {
    let Err(writer) = FilesystemVersionTwoAdmission::reopen(root) else {
        return Err("writer admission admitted a root without FORMAT".into());
    };
    require_missing_marker(&writer)?;
    let Err(recovery) = FilesystemVersionTwoRecoveryAdmission::reopen(root) else {
        return Err("recovery admission admitted a root without FORMAT".into());
    };
    require_missing_marker(&recovery)?;
    let Err(VersionTwoInitializationError::Refused { .. }) =
        FilesystemVersionTwoAdmission::initialize(root)
    else {
        return Err("initialization adopted a populated root without FORMAT".into());
    };
    Ok(())
}

fn require_inventory_refuses(root: &Path) -> TestResult<()> {
    let Err(RecoveryInventoryError::Io {
        namespace: RecoveryNamespace::Root,
        source,
        ..
    }) = FilesystemRecoveryInventoryReader::open_version_two(root)
    else {
        return Err("recovery inventory admitted a root without FORMAT".into());
    };
    require_missing_marker(
        source
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<FilesystemPlatformAdmissionError>())
            .ok_or("inventory refusal lost its admission source")?,
    )?;
    Ok(())
}

fn require_content_readers_refuse(root: &Path) -> TestResult<()> {
    let policy = durable_fixture::policy()?;
    let Err(CatalogRestartError::Admission { source }) =
        FilesystemCatalogSnapshot::load(root, policy)
    else {
        return Err("catalog restart admitted a root without FORMAT".into());
    };
    require_missing_marker(&source)?;
    let Err(FilesystemRetentionSnapshotError::Admission { source }) =
        FilesystemRetentionSnapshot::load(root, policy, ReaderAttemptLimit::DEFAULT)
    else {
        return Err("the retention reader admitted a root without FORMAT".into());
    };
    require_refusal(&source)?;
    let Err(DurableStoreError::Snapshot(snapshot)) =
        DurableStore::open(root, policy, ReaderAttemptLimit::DEFAULT)?.snapshot()
    else {
        return Err("the durable reader admitted a root without FORMAT".into());
    };
    let FilesystemRetentionSnapshotError::Admission { source } = snapshot.as_ref() else {
        return Err(format!("unexpected durable refusal {snapshot:?}").into());
    };
    require_refusal(source)
}

fn require_missing_marker(error: &FilesystemPlatformAdmissionError) -> TestResult<()> {
    let FilesystemPlatformAdmissionError::Namespace { source } = error else {
        return Err(format!("unexpected admission refusal {error:?}").into());
    };
    require_refusal(source)
}

fn require_refusal(source: &std::io::Error) -> TestResult<()> {
    match source
        .get_ref()
        .and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>())
    {
        Some(FilesystemNamespaceRefusal::MissingFormatMarker) => Ok(()),
        _ => Err(format!("refusal is not MissingFormatMarker: {source:?}").into()),
    }
}
