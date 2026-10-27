//! Store reopen laws: writer authority requires a committed namespace and the canonical marker.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, initialized_store, open_authority, refusal,
};
use super::{
    FilesystemRetentionPublicationAuthority, RetentionCurrentStateRefusal,
    RetentionPublicationStorage,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{
    FilesystemNamespaceRefusal, FilesystemStoreAdmission, FilesystemStoreAdmissionError,
    StoreRecordRefusal,
};
use crate::execute_retention_publication;

#[test]
fn reopen_refuses_a_corrupt_format_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-corrupt-marker")?;
    let marker = sandbox.path().join("FORMAT");
    let mut bytes = fs::read(&marker)?;
    *bytes
        .get_mut(40)
        .ok_or("format marker shorter than 41 bytes")? ^= 0x01;
    fs::write(&marker, &bytes)?;

    let error = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())
        .err()
        .ok_or("corrupt format marker was unexpectedly admitted")?;

    let FilesystemStoreAdmissionError::FormatMarker { source } = error else {
        return Err("corrupt format marker refused outside record admission".into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|refusal| refusal.downcast_ref::<StoreRecordRefusal>()),
        Some(StoreRecordRefusal::Marker { .. })
    ));
    Ok(())
}

#[test]
fn reopen_refuses_an_oversized_format_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-oversized-marker")?;
    let marker = sandbox.path().join("FORMAT");
    let mut bytes = fs::read(&marker)?;
    bytes.push(0);
    fs::write(&marker, &bytes)?;

    let error = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())
        .err()
        .ok_or("oversized format marker was unexpectedly admitted")?;

    assert!(matches!(
        error,
        FilesystemStoreAdmissionError::FormatMarker { .. }
    ));
    Ok(())
}

#[test]
fn reopen_admits_a_canonical_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-exact")?;

    let admission =
        FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?;

    drop(admission);
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn production_reopen_refuses_an_aliased_protocol_directory() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-aliased-gc")?;
    let alias_target = sandbox.path().join("elsewhere");
    fs::create_dir(&alias_target)?;
    fs::remove_dir(sandbox.path().join("gc"))?;
    std::os::unix::fs::symlink(&alias_target, sandbox.path().join("gc"))?;

    let error = FilesystemStoreAdmission::reopen(sandbox.path())
        .err()
        .ok_or("aliased gc protocol directory was unexpectedly admitted")?;

    assert!(matches!(
        error,
        FilesystemStoreAdmissionError::Platform { .. }
    ));
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn production_reopen_admits_an_exact_initialized_store() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-production-exact")?;

    let admission = FilesystemStoreAdmission::reopen(sandbox.path())?;

    drop(admission);
    Ok(())
}

fn refuses_namespace(
    name: &str,
    mutate: impl FnOnce(&std::path::Path) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store(name)?;
    mutate(sandbox.path())?;

    let error = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())
        .err()
        .ok_or_else(|| format!("{name}: store root was unexpectedly admitted"))?;

    assert!(
        matches!(error, FilesystemStoreAdmissionError::Namespace { .. }),
        "{name}: expected a Namespace refusal, got {error:?}"
    );
    Ok(())
}

#[test]
fn reopen_refuses_a_missing_dispositions_directory() -> Result<(), Box<dyn Error>> {
    refuses_namespace("store-admission-no-dispositions", |root| {
        fs::remove_dir(root.join("recovery").join("dispositions"))?;
        Ok(())
    })
}

#[test]
fn reopen_refuses_a_stray_gc_entry() -> Result<(), Box<dyn Error>> {
    refuses_namespace("store-admission-gc-junk", |root| {
        fs::write(root.join("gc").join("junk"), b"")?;
        Ok(())
    })
}

#[test]
fn reopen_refuses_a_stray_recovery_entry() -> Result<(), Box<dyn Error>> {
    refuses_namespace("store-admission-recovery-junk", |root| {
        fs::write(root.join("recovery").join("junk"), b"")?;
        Ok(())
    })
}

#[test]
fn reopen_refuses_a_missing_retention_pool() -> Result<(), Box<dyn Error>> {
    refuses_namespace("store-admission-no-roots", |root| {
        fs::remove_dir(root.join("retention").join("roots"))?;
        Ok(())
    })
}

#[test]
fn a_protocol_directory_replaced_after_reopen_is_neither_opened_nor_published_into()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut first) = open_authority("store-admission-replaced-retention")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut first, &preparation)?;
    drop(first);
    let admission =
        FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?;
    fs::rename(
        sandbox.path().join("retention"),
        sandbox.path().join("retention.moved"),
    )?;
    fs::create_dir_all(sandbox.path().join("retention").join("roots"))?;
    fs::create_dir(sandbox.path().join("retention").join("manifests"))?;

    let mut authority = FilesystemRetentionPublicationAuthority::open(admission)?;
    let observed = authority
        .observe_current()?
        .ok_or("the admitted retention directory lost its published head")?;
    let error = authority
        .verify_current(&preparation)
        .err()
        .ok_or("publication proceeded although retention was replaced after admission")?;

    assert_eq!(
        observed.head().generation(),
        preparation.liveness_generation()
    );
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::ProtocolDirectoryReplaced)
    ));
    Ok(())
}

#[test]
fn an_initialized_store_without_a_published_catalog_reopens() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("store-admission-unpublished")?;
    drop(FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(sandbox.path())?);
    assert!(!sandbox.path().join("HEAD").exists());

    let admission =
        FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?;

    drop(admission);
    Ok(())
}

#[test]
fn reopen_refuses_a_root_whose_marker_stage_remains() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-marker-stage")?;
    fs::hard_link(
        sandbox.path().join("FORMAT"),
        sandbox.path().join("FORMAT.next"),
    )?;

    let error = reopen_namespace_refusal(sandbox.path())?;

    assert_eq!(error, FilesystemNamespaceRefusal::InitializationIncomplete);
    Ok(())
}

#[test]
fn reopen_refuses_a_root_without_a_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("store-admission-no-marker")?;
    fs::remove_file(sandbox.path().join("FORMAT"))?;

    let error = reopen_namespace_refusal(sandbox.path())?;

    assert_eq!(error, FilesystemNamespaceRefusal::MissingFormatMarker);
    Ok(())
}

fn reopen_namespace_refusal(
    root: &std::path::Path,
) -> Result<FilesystemNamespaceRefusal, Box<dyn Error>> {
    let error = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(root)
        .err()
        .ok_or("an uncommitted root was unexpectedly admitted")?;
    let FilesystemStoreAdmissionError::Namespace { source } = error else {
        return Err(format!("expected a namespace refusal, got {error:?}").into());
    };
    source
        .get_ref()
        .and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>())
        .copied()
        .ok_or_else(|| "namespace refusal carried no typed cause".into())
}
