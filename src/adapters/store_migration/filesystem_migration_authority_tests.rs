//! Writer-locked filesystem migration authority laws.

use std::error::Error;
use std::fs;

use super::FilesystemMigrationAuthorityError;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{AdmittedSegment, physical_pool_name};

use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};

const EMPTY_SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/empty-segment.hex");
const INVENTORY_DIGEST: &str = "40bf5d49c34847ac9cf46a256f343cee80cd980d1405d2dd02ceff8f58d674f9";

#[test]
fn exact_published_v1_authority_constructs_and_revalidates_one_intent() -> Result<(), Box<dyn Error>>
{
    let (sandbox, authority) = open_authority("migration-authority-current")?;
    let intent = authority.observe_intent()?;
    authority.verify_current(&intent)?;

    assert_eq!(intent.catalog_generation().get(), 1);
    assert_eq!(
        intent.inventory_digest().as_bytes().as_slice(),
        decode_hex(INVENTORY_DIGEST)?
    );
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn immutable_pool_drift_refuses_the_retained_intent() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("migration-authority-inventory-drift")?;
    let intent = authority.observe_intent()?;
    let bytes = decode_hex(EMPTY_SEGMENT_HEX.trim())?;
    let segment = AdmittedSegment::decode(&bytes, maximum_policy())?;
    fs::write(
        sandbox
            .path()
            .join("segments")
            .join(physical_pool_name::segment(segment.digest())),
        bytes,
    )?;

    let error = authority
        .verify_current(&intent)
        .err()
        .ok_or("changed inventory unexpectedly retained authority")?;
    assert!(matches!(
        error,
        FilesystemMigrationAuthorityError::IntentChanged { expected, observed }
            if expected == intent.digest() && observed != expected
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn version_two_namespace_evidence_refuses_before_mutation() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("migration-authority-v2-evidence")?;
    let intent = authority.observe_intent()?;
    fs::write(sandbox.path().join("FORMAT"), [])?;

    let error = authority
        .verify_current(&intent)
        .err()
        .ok_or("version-two evidence unexpectedly retained authority")?;
    assert!(matches!(
        error,
        FilesystemMigrationAuthorityError::Namespace { source }
            if source.kind() == std::io::ErrorKind::InvalidData
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
