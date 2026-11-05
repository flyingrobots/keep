//! Capability-relative filesystem recovery-inventory laws.

use std::error::Error;
#[cfg(target_os = "linux")]
use std::ffi::OsString;
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStringExt;

use super::filesystem_test_sandbox::TestDirectory;
use super::{
    FilesystemEntryKind, FilesystemNamespaceRefusal, FilesystemRecoveryInventoryReader,
    FilesystemStoreAdmission, FilesystemStoreAdmissionError, RecoveryInventoryError,
    RecoveryInventoryLimit, RecoveryInventoryOperation, RecoveryNamespace,
};

const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";

#[test]
fn filesystem_inventory_returns_deterministic_name_order() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_namespace("recovery-inventory-order")?;
    let name = b"segment-z";
    fs::write(
        sandbox.path().join(SEGMENTS_NAME).join("segment-z"),
        b"retained evidence",
    )?;
    let mut reader = open_reader(&sandbox)?;

    let inventory = reader.read(RecoveryInventoryLimit::protocol_maximum())?;

    assert!(inventory.entries().windows(2).all(|pair| {
        let [left, right] = pair else {
            return false;
        };
        left < right
    }));
    assert!(inventory.entries().iter().any(|entry| {
        entry.namespace() == RecoveryNamespace::Segments && entry.name().as_bytes() == name
    }));
    assert_eq!(
        fs::read(sandbox.path().join(SEGMENTS_NAME).join("segment-z"))?,
        b"retained evidence"
    );
    drop(reader);
    sandbox.remove()?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn filesystem_inventory_preserves_raw_linux_name_bytes() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_namespace("recovery-inventory-raw")?;
    let raw_name = vec![b's', b'e', b'g', 0x80];
    fs::write(
        sandbox
            .path()
            .join(SEGMENTS_NAME)
            .join(OsString::from_vec(raw_name.clone())),
        [],
    )?;
    let mut reader = open_reader(&sandbox)?;

    let inventory = reader.read(RecoveryInventoryLimit::protocol_maximum())?;

    assert!(inventory.entries().iter().any(|entry| {
        entry.namespace() == RecoveryNamespace::Segments && entry.name().as_bytes() == raw_name
    }));
    drop(reader);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn filesystem_count_stops_at_the_first_global_excess_entry() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_namespace("recovery-inventory-limit")?;
    fs::write(sandbox.path().join(SEGMENTS_NAME).join("one"), [])?;
    let mut reader = open_reader(&sandbox)?;
    let Err(error) = reader.read(RecoveryInventoryLimit::new(4)?) else {
        return Err("filesystem inventory exceeded the global entry ceiling".into());
    };

    assert!(matches!(
        error,
        RecoveryInventoryError::EntryLimit {
            maximum: 4,
            observed_at_least: 5,
        }
    ));
    drop(reader);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn filesystem_inventory_never_follows_protocol_directory_links() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let sandbox = TestDirectory::create("recovery-inventory-symlink")?;
    let store = sandbox.path().join("store");
    fs::create_dir(&store)?;
    drop(FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(&store)?);
    fs::create_dir(sandbox.path().join("target"))?;
    fs::remove_dir(store.join(STAGING_NAME))?;
    symlink("../target", store.join(STAGING_NAME))?;

    let Err(error) = FilesystemRecoveryInventoryReader::open_unchecked_for_repository_tasks(&store)
    else {
        return Err("filesystem inventory followed a staging-directory link".into());
    };
    let RecoveryInventoryError::Io {
        namespace: RecoveryNamespace::Root,
        operation: RecoveryInventoryOperation::OpenNamespace,
        source,
    } = error
    else {
        return Err(format!("unexpected inventory refusal {error:?}").into());
    };
    assert_eq!(
        namespace_refusal(&source),
        Some(FilesystemNamespaceRefusal::WrongKind {
            expected: FilesystemEntryKind::Directory,
            observed: FilesystemEntryKind::Symlink,
        })
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn replaced_protocol_directory_refuses_the_pinned_inventory() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_namespace("recovery-inventory-replacement")?;
    let mut reader = open_reader(&sandbox)?;
    fs::rename(
        sandbox.path().join(STAGING_NAME),
        sandbox.path().join("displaced-staging"),
    )?;
    fs::create_dir(sandbox.path().join(STAGING_NAME))?;

    let Err(error) = reader.read(RecoveryInventoryLimit::protocol_maximum()) else {
        return Err("filesystem inventory admitted a replaced staging directory".into());
    };
    assert!(matches!(
        error,
        RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Staging,
            operation: RecoveryInventoryOperation::VerifyNamespace,
            ref source,
        } if source.kind() == std::io::ErrorKind::InvalidData
    ));
    drop(reader);
    sandbox.remove()?;
    Ok(())
}

/// A store committed by direct initialization.
fn initialized_namespace(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    drop(FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(sandbox.path())?);
    Ok(sandbox)
}

fn open_reader(
    sandbox: &TestDirectory,
) -> Result<FilesystemRecoveryInventoryReader, RecoveryInventoryError> {
    FilesystemRecoveryInventoryReader::open_unchecked_for_repository_tasks(sandbox.path())
}

/// The namespace refusal one root admission failure carries, if any.
fn namespace_refusal(source: &std::io::Error) -> Option<FilesystemNamespaceRefusal> {
    let admission = source
        .get_ref()?
        .downcast_ref::<FilesystemStoreAdmissionError>()?;
    let FilesystemStoreAdmissionError::Namespace { source } = admission else {
        return None;
    };
    source
        .get_ref()?
        .downcast_ref::<FilesystemNamespaceRefusal>()
        .copied()
}
