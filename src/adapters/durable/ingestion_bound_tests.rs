//! A commit bounds stage admission by the length the writer actually sealed.

use std::error::Error;
use std::fs::OpenOptions;
use std::io::Cursor;

use super::{DurableIngestionError, DurableWriter};
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store,
};
use crate::{FilesystemVersionTwoAdmission, LayoutEntryLimit, StagingLimits};

#[test]
fn a_grown_sealed_stage_refuses_before_payload_allocation_or_publication()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("durable-stage-length-bound")?;
    let admission =
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?;
    let mut writer = DurableWriter::open(admission, sandbox.path(), catalog_policy()?)?;
    let head = std::fs::read(sandbox.path().join("HEAD"))?;
    let staged = writer.stage(
        &mut Cursor::new(b"bounded stage"),
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    let path = sandbox.path().join("staging/current.seg");
    let file = OpenOptions::new().write(true).open(&path)?;
    let changed_length = file
        .metadata()?
        .len()
        .checked_add(1)
        .ok_or("stage length overflow")?;
    file.set_len(changed_length)?;
    drop(file);
    let error = staged.commit().err().ok_or("grown stage committed")?;
    let DurableIngestionError::ReadStage { source } = error else {
        return Err("stage length was not refused at the read boundary".into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<ExactRecordError>()),
        Some(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength))
    ));
    assert_eq!(std::fs::read(sandbox.path().join("HEAD"))?, head);
    assert_eq!(std::fs::metadata(path)?.len(), changed_length);
    drop(writer);
    sandbox.remove()?;
    Ok(())
}
