//! Completed migration admits live retention state without taking its ownership.

use std::error::Error;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority,
};
use crate::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot,
    FilesystemStoreMigrationAuthority, ReaderAttemptLimit, SegmentReadPolicy,
    StoreMigrationRecoveryPlan, execute_retention_publication, recover_store_migration,
};

// Size: medium. Oracle: migration completion preserves a successfully published retained root.
// Delete only if migration completion cannot be retried or stronger admission laws subsume it.
#[test]
fn completed_migration_preserves_published_retention_state() -> Result<(), Box<dyn Error>> {
    let (store, mut publisher) = open_authority("completed-migration-retention")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut publisher, &preparation)?;
    drop(publisher);
    let mut migration =
        FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_repository_tasks(
            store.path(),
            SegmentReadPolicy::MAXIMUM,
        )?;
    let intent = migration.observe_intent()?;
    let receipt = recover_store_migration(&mut migration, &intent)?;
    assert_eq!(receipt.plan(), StoreMigrationRecoveryPlan::Complete);
    assert_eq!(receipt.executed_phases().next(), None);
    drop(migration);
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let view =
        FilesystemRetentionSnapshot::load(store.path(), policy, ReaderAttemptLimit::DEFAULT)?;
    let namespace = super::AdmittedRetentionRoot::decode(&root_bytes)?
        .root()
        .namespace()
        .digest();
    let retained = view
        .retained_root(namespace)?
        .ok_or("published root disappeared")?;
    assert_eq!(&*retained, root_bytes.as_slice());
    Ok(())
}
