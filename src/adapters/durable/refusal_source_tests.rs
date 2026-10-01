//! Durable admission preserves the exact selected-root decoding refusal.

use std::error::Error;
use std::fs;

use super::{DurableSnapshot, DurableStoreError};
use crate::FilesystemRetentionSnapshot;
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, open_authority, root_pool_path,
};
use crate::{AdmittedRetentionRoot, ReaderAttemptLimit, RetentionRootDecodeError};

#[test]
fn a_corrupt_retained_root_keeps_its_typed_cause_through_durable_admission()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("durable-root-refusal-source")?;
    let mut bytes = fixture(ROOT_HEX)?;
    let candidate = AdmittedRetentionRoot::decode(&bytes)?;
    let path = root_pool_path(sandbox.path(), &candidate);
    let preparation = initial_preparation(&bytes)?;
    let _receipt = crate::execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    *bytes.last_mut().ok_or("root checksum missing")? ^= 1;
    fs::write(path, bytes)?;
    let error = DurableSnapshot::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .err()
    .ok_or("corrupt retained root admitted")?;
    assert!(matches!(&error, DurableStoreError::RetainedRoot { .. }));
    let mut cause: &(dyn Error + 'static) = &error;
    loop {
        if let Some(refusal) = cause.downcast_ref::<RetentionRootDecodeError>() {
            assert!(matches!(
                refusal,
                RetentionRootDecodeError::ChecksumMismatch { .. }
            ));
            break;
        }
        cause = if let Some(inner) = cause
            .downcast_ref::<std::io::Error>()
            .and_then(std::io::Error::get_ref)
        {
            inner
        } else {
            cause
                .source()
                .ok_or("typed root decoding cause was erased")?
        };
    }
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_non_regular_selected_root_keeps_the_exact_record_refusal() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("durable-root-record-refusal")?;
    let bytes = fixture(ROOT_HEX)?;
    let candidate = AdmittedRetentionRoot::decode(&bytes)?;
    let path = root_pool_path(sandbox.path(), &candidate);
    let preparation = initial_preparation(&bytes)?;
    let _receipt = crate::execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let snapshot = FilesystemRetentionSnapshot::load(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    fs::remove_file(&path)?;
    fs::create_dir(&path)?;
    let error = snapshot
        .retained_root(candidate.root().namespace().digest())
        .err()
        .ok_or("a non-regular root admitted")?;
    let crate::FilesystemRetentionSnapshotError::Root { source } = error else {
        return Err("wrong root refusal boundary".into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<ExactRecordError>()),
        Some(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength))
    ));
    drop(snapshot);
    sandbox.remove()?;
    Ok(())
}
