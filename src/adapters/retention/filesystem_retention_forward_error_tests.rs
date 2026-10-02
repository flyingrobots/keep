//! This module owns typed stage refusal through the forward I/O boundary.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority, retention_witness,
};
use super::{RetentionPublicationStorage, RetentionRecordRefusal, RetentionStorageError};

// Size: medium. Oracle: forward stage refusal preserves typed cause and retained bytes.
// Delete when forward publication is removed or a stronger public-boundary law subsumes this case.
#[test]
fn forward_stage_refusal_preserves_its_typed_record_cause() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("forward-stage-typed-refusal")?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 2)?;
    let path = sandbox.path().join("retention/root.next");
    let mut bytes = fs::read(&path)?;
    *bytes.last_mut().ok_or("empty root stage")? ^= 1;
    fs::write(path, bytes)?;
    let before = retention_witness(sandbox.path())?;

    let error = RetentionPublicationStorage::synchronize_root_stage(&mut authority)
        .err()
        .ok_or("changed forward stage was synchronized successfully")?;

    assert!(
        matches!(
            error
                .get_ref()
                .and_then(|source| source.downcast_ref::<RetentionStorageError>()),
            Some(RetentionStorageError::Refused {
                source: RetentionRecordRefusal::Bytes
            })
        ),
        "forward I/O failure must retain the typed byte refusal: {error:?}"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "forward refusal must preserve retained evidence"
    );
    Ok(())
}
