//! Filesystem retention catalog-binding laws: closure evidence must belong to this store.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, refusal, retention_witness,
};
use super::{RetentionCurrentStateRefusal, RetentionPublicationStorage};
use crate::execute_retention_publication;

/// A generation-one version-one head that names a different catalog digest.
const FOREIGN_CATALOG_HEAD_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-head.hex");

#[test]
fn closure_verified_against_another_catalog_refuses_before_staging() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-foreign-catalog")?;
    fs::write(
        sandbox.path().join("HEAD"),
        fixture(FOREIGN_CATALOG_HEAD_HEX)?,
    )?;
    let before = retention_witness(sandbox.path())?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("closure verified against a foreign catalog was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::CatalogDisagreed { .. })
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn committed_retry_over_a_foreign_catalog_refuses_before_reporting_committed()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) =
        open_authority("filesystem-retention-committed-foreign-catalog")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    fs::write(
        sandbox.path().join("HEAD"),
        fixture(FOREIGN_CATALOG_HEAD_HEX)?,
    )?;
    let before = retention_witness(sandbox.path())?;
    let retry = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &retry)
        .err()
        .ok_or("already-committed retry was reported over a foreign catalog head")?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::CatalogDisagreed { .. })
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
