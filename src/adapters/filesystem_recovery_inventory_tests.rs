//! Capability-relative filesystem recovery-inventory laws.

use std::error::Error;
#[cfg(target_os = "linux")]
use std::ffi::OsString;
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStringExt;

use super::filesystem_test_sandbox::TestDirectory;
use super::{
    FilesystemRecoveryInventoryReader, RecoveryInventoryError, RecoveryInventoryLimit,
    RecoveryInventoryOperation, RecoveryNamespace,
};

const LOCK_NAME: &str = "writer.lock";
const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";

#[test]
fn filesystem_inventory_returns_deterministic_name_order() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_namespace("recovery-inventory-order")?;
    let name = b"segment-z";
    fs::write(
        sandbox.path().join(SEGMENTS_NAME).join("segment-z"),
        b"retained evidence",
    )?;
    let mut reader = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())?;

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
    let mut reader = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())?;

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
    let mut reader = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())?;
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
    fs::write(sandbox.path().join(LOCK_NAME), [])?;
    fs::create_dir(sandbox.path().join("target"))?;
    symlink("target", sandbox.path().join(STAGING_NAME))?;
    fs::create_dir(sandbox.path().join(SEGMENTS_NAME))?;
    fs::create_dir(sandbox.path().join(CATALOGS_NAME))?;

    let Err(error) = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())
    else {
        return Err("filesystem inventory followed a staging-directory link".into());
    };
    assert!(matches!(
        error,
        RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Staging,
            operation: RecoveryInventoryOperation::OpenNamespace,
            ..
        }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn replaced_protocol_directory_refuses_the_pinned_inventory() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_namespace("recovery-inventory-replacement")?;
    let mut reader = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())?;
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

fn initialized_namespace(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    fs::write(sandbox.path().join(LOCK_NAME), [])?;
    fs::create_dir(sandbox.path().join(STAGING_NAME))?;
    fs::create_dir(sandbox.path().join(SEGMENTS_NAME))?;
    fs::create_dir(sandbox.path().join(CATALOGS_NAME))?;
    Ok(sandbox)
}
