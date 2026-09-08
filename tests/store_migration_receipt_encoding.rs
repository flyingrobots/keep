//! Canonical version-2 store-migration receipt encoding laws.

#[path = "store_migration_receipt/fixture.rs"]
mod fixture;
mod support;

use std::error::Error;

use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
};

#[test]
fn canonical_completion_receipt_reproduces_every_frozen_field() -> Result<(), Box<dyn Error>> {
    let expected = fixture::receipt_bytes()?;
    let intent_bytes = fixture::intent_bytes()?;
    let admitted_intent = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let intent = CanonicalStoreMigrationIntent::from_admitted(&admitted_intent);
    let marker = CanonicalStoreFormatMarker::version_two();
    assert_eq!(marker.encoded(), fixture::marker_bytes()?);

    let canonical = CanonicalStoreMigrationReceipt::from_canonical(&intent, &marker);
    assert_eq!(canonical.encoded(), expected);

    let admitted_marker = AdmittedStoreFormatMarker::decode(marker.encoded())?;
    let admitted = AdmittedStoreMigrationReceipt::decode(
        canonical.encoded(),
        &admitted_intent,
        &admitted_marker,
    )?;
    assert_eq!(
        admitted.initial_retention_state_digest().as_bytes(),
        &fixture::INITIAL_RETENTION_DIGEST
    );
    assert_eq!(
        admitted.initial_gc_state_digest().as_bytes(),
        &fixture::INITIAL_GC_DIGEST
    );
    assert_eq!(
        admitted.empty_disposition_set_digest().as_bytes(),
        &fixture::DISPOSITION_DIGEST
    );
    assert_eq!(admitted.synchronization_mask().bits(), 0x03ff);
    Ok(())
}
