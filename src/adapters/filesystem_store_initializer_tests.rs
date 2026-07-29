//! Concrete crash-safe filesystem initialization laws.

use std::error::Error;
use std::fs;

use super::filesystem_test_sandbox::TestDirectory;
use super::{FilesystemPlatformAdmission, StoreInitializationError, StoreInitializationPhase};

const LOCK_NAME: &str = "writer.lock";
const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";

#[test]
fn empty_namespace_is_admitted_only_with_the_complete_root_shape() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("store-initialization-empty")?;

    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;

    assert!(sandbox.path().join(LOCK_NAME).is_file());
    assert!(sandbox.path().join(STAGING_NAME).is_dir());
    assert!(sandbox.path().join(SEGMENTS_NAME).is_dir());
    assert!(sandbox.path().join(CATALOGS_NAME).is_dir());
    drop(admission);
    sandbox.remove()?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn linux_initializer_synchronizes_an_opath_root() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("store-initialization-opath-sync")?;

    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;

    drop(admission);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn partial_canonical_namespace_is_completed_without_replacing_evidence()
-> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("store-initialization-partial")?;
    let retained = b"retained lock evidence";
    fs::write(sandbox.path().join(LOCK_NAME), retained)?;
    fs::create_dir(sandbox.path().join(STAGING_NAME))?;

    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;

    assert_eq!(fs::read(sandbox.path().join(LOCK_NAME))?, retained);
    assert!(sandbox.path().join(STAGING_NAME).is_dir());
    assert!(sandbox.path().join(SEGMENTS_NAME).is_dir());
    assert!(sandbox.path().join(CATALOGS_NAME).is_dir());
    drop(admission);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn unknown_namespace_refuses_before_writer_file_creation() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("store-initialization-unknown")?;
    fs::write(sandbox.path().join("unknown"), [])?;

    let Err(error) = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())
    else {
        return Err("initializer admitted an unknown namespace entry".into());
    };

    assert!(matches!(
        error,
        StoreInitializationError::Io {
            phase: StoreInitializationPhase::AdmitPlatform,
            ref source,
        } if source.kind() == std::io::ErrorKind::InvalidData
    ));
    assert!(!sandbox.path().join(LOCK_NAME).exists());
    sandbox.remove()?;
    Ok(())
}

#[test]
fn retained_initializer_authority_excludes_a_second_initializer() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("store-initialization-exclusion")?;
    let first = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;

    let Err(error) = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())
    else {
        return Err("second initializer acquired live writer authority".into());
    };

    assert!(matches!(
        error,
        StoreInitializationError::Io {
            phase: StoreInitializationPhase::OpenAndLockWriterFile,
            ref source,
        } if matches!(
            source
                .get_ref()
                .and_then(|nested| nested.downcast_ref::<super::WriterLockAcquireError>()),
            Some(super::WriterLockAcquireError::Busy)
        )
    ));
    drop(first);
    let successor = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;
    drop(successor);
    sandbox.remove()?;
    Ok(())
}
