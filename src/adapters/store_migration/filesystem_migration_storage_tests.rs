//! Filesystem migration storage laws.

use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::filesystem_migration_test_fixture::open_authority;
use super::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    CanonicalStoreFormatMarker, StoreMigrationStorage, execute_store_migration,
};

#[test]
fn complete_migration_preserves_v1_bytes_and_publishes_exact_v2_prefix()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-migration-complete")?;
    let before = version_one_witness(sandbox.path())?;
    let intent = authority.observe_intent()?;

    let receipt = execute_store_migration(&mut authority, &intent)?;

    assert_eq!(version_one_witness(sandbox.path())?, before);
    admit_published_records(sandbox.path(), &intent, receipt.encoded())?;
    assert_complete_namespace(sandbox.path())?;
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn existing_intent_stage_is_never_truncated() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-migration-exclusive-stage")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    let stage = sandbox.path().join("migration.intent.next");
    fs::write(&stage, b"retained partial evidence")?;

    let error = StoreMigrationStorage::write_intent_stage(&mut authority, &intent)
        .err()
        .ok_or("existing intent stage was unexpectedly replaced")?;

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&stage)?, b"retained partial evidence");
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn byte_equal_substituted_canonical_intent_is_refused() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-migration-substituted-target")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    StoreMigrationStorage::write_intent_stage(&mut authority, &intent)?;
    StoreMigrationStorage::synchronize_intent_stage(&mut authority)?;
    fs::write(sandbox.path().join("migration.intent"), intent.encoded())?;

    let error = StoreMigrationStorage::link_intent(&mut authority, &intent)
        .err()
        .ok_or("substituted canonical intent was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn out_of_order_namespace_refuses_before_creating_its_predecessor() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-migration-prefix-order")?;
    let intent = authority.observe_intent()?;
    publish_intent_and_reader(&mut authority, &intent)?;
    fs::create_dir(sandbox.path().join("gc"))?;

    let error = StoreMigrationStorage::admit_namespace_prefix(&mut authority)
        .err()
        .ok_or("out-of-order gc namespace was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(!sandbox.path().join("retention").exists());
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn canonical_intent_drift_refuses_before_marker_stage_creation() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-migration-intent-drift")?;
    let intent = authority.observe_intent()?;
    publish_intent_and_reader(&mut authority, &intent)?;
    StoreMigrationStorage::admit_namespace_prefix(&mut authority)?;
    StoreMigrationStorage::synchronize_root_after_namespace(&mut authority)?;
    fs::write(
        sandbox.path().join("migration.intent"),
        vec![0_u8; intent.encoded().len()],
    )?;
    let marker = CanonicalStoreFormatMarker::version_two();

    let error = StoreMigrationStorage::write_marker_stage(&mut authority, &marker)
        .err()
        .ok_or("changed canonical intent unexpectedly authorized marker publication")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(!sandbox.path().join("FORMAT.next").exists());
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

fn publish_intent_and_reader(
    authority: &mut super::FilesystemStoreMigrationAuthority,
    intent: &super::CanonicalStoreMigrationIntent,
) -> io::Result<()> {
    StoreMigrationStorage::verify_current(authority, intent)?;
    StoreMigrationStorage::write_intent_stage(authority, intent)?;
    StoreMigrationStorage::synchronize_intent_stage(authority)?;
    StoreMigrationStorage::link_intent(authority, intent)?;
    StoreMigrationStorage::synchronize_root_after_intent(authority)?;
    StoreMigrationStorage::remove_intent_stage(authority)?;
    StoreMigrationStorage::synchronize_root_after_intent_cleanup(authority)?;
    StoreMigrationStorage::admit_reader_fence(authority)
}

fn admit_published_records(
    root: &Path,
    expected_intent: &super::CanonicalStoreMigrationIntent,
    expected_receipt: &[u8],
) -> Result<(), Box<dyn Error>> {
    let intent_bytes = fs::read(root.join("migration.intent"))?;
    let marker_bytes = fs::read(root.join("FORMAT"))?;
    let receipt_bytes = fs::read(root.join("migration.receipt"))?;
    let intent = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let marker = AdmittedStoreFormatMarker::decode(&marker_bytes)?;
    let receipt = AdmittedStoreMigrationReceipt::decode(&receipt_bytes, &intent, &marker)?;
    assert_eq!(intent.encoded(), expected_intent.encoded());
    assert_eq!(receipt.encoded(), expected_receipt);
    Ok(())
}

fn assert_complete_namespace(root: &Path) -> Result<(), Box<dyn Error>> {
    let expected = BTreeSet::from([
        OsString::from("FORMAT"),
        OsString::from("HEAD"),
        OsString::from("catalogs"),
        OsString::from("gc"),
        OsString::from("migration.intent"),
        OsString::from("migration.receipt"),
        OsString::from("reader.lock"),
        OsString::from("recovery"),
        OsString::from("retention"),
        OsString::from("segments"),
        OsString::from("staging"),
        OsString::from("writer.lock"),
    ]);
    assert_eq!(directory_names(root)?, expected);
    assert_eq!(fs::metadata(root.join("reader.lock"))?.len(), 0);
    assert_eq!(
        directory_names(&root.join("retention"))?,
        BTreeSet::from([OsString::from("manifests"), OsString::from("roots")])
    );
    assert_eq!(
        directory_names(&root.join("recovery"))?,
        BTreeSet::from([OsString::from("dispositions")])
    );
    for relative in [
        "gc",
        "retention/roots",
        "retention/manifests",
        "recovery/dispositions",
    ] {
        assert!(directory_names(&root.join(relative))?.is_empty());
    }
    for stage in [
        "migration.intent.next",
        "FORMAT.next",
        "migration.receipt.next",
    ] {
        assert!(!root.join(stage).exists());
    }
    Ok(())
}

fn version_one_witness(root: &Path) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut witness = Vec::new();
    witness.push((PathBuf::from("HEAD"), fs::read(root.join("HEAD"))?));
    for pool in ["segments", "catalogs"] {
        for entry in fs::read_dir(root.join(pool))? {
            let path = PathBuf::from(pool).join(entry?.file_name());
            witness.push((path.clone(), fs::read(root.join(path))?));
        }
    }
    witness.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(witness)
}

fn directory_names(path: &Path) -> io::Result<BTreeSet<OsString>> {
    fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect()
}

#[test]
fn a_retained_version_one_stage_refuses_migration_before_any_intent() -> Result<(), Box<dyn Error>>
{
    let (sandbox, authority) = open_authority("filesystem-migration-retained-stage")?;
    fs::write(
        sandbox.path().join("staging").join("current.seg"),
        b"interrupted version-one publication",
    )?;
    let before = version_one_witness(sandbox.path())?;

    let error = authority
        .observe_intent()
        .err()
        .ok_or("a store with a retained version-one stage was admitted for migration")?;

    assert!(matches!(
        error,
        crate::adapters::FilesystemMigrationAuthorityError::Namespace { .. }
    ));
    assert_eq!(version_one_witness(sandbox.path())?, before);
    assert!(!sandbox.path().join("migration.intent").exists());
    assert!(!sandbox.path().join("migration.intent.next").exists());
    Ok(())
}
