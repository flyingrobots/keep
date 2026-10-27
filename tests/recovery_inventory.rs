//! Bounded, deterministic recovery-inventory laws.

#[path = "recovery_inventory/inventory_double.rs"]
pub mod inventory_double;
#[path = "recovery_inventory/refusal_laws.rs"]
mod refusal_laws;
#[cfg(not(target_os = "linux"))]
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;

use std::error::Error;
#[cfg(not(target_os = "linux"))]
use std::fs;

use inventory_double::{InventoryCall, InventoryDouble};
#[cfg(not(target_os = "linux"))]
use keep::{FilesystemRecoveryInventoryReader, RecoveryInventoryError, RecoveryInventoryOperation};
use keep::{RecoveryEntryName, RecoveryInventoryLimit, RecoveryNamespace, read_recovery_inventory};
#[cfg(not(target_os = "linux"))]
use sandbox::TestDirectory;

#[test]
fn every_count_precedes_name_retention_in_fixed_namespace_order() -> Result<(), Box<dyn Error>> {
    let names = [
        vec![name(b"writer.lock")?, name(b"HEAD")?],
        vec![name(b"current.seg")?],
        vec![],
        vec![name(b"catalog-2")?],
    ];
    let mut storage = InventoryDouble::new([2, 1, 0, 1], names);

    let inventory =
        read_recovery_inventory(&mut storage, RecoveryInventoryLimit::protocol_maximum())?;

    assert_eq!(
        storage.calls(),
        &[
            InventoryCall::Count(RecoveryNamespace::Root, 2_097_152),
            InventoryCall::Count(RecoveryNamespace::Staging, 2_097_150),
            InventoryCall::Count(RecoveryNamespace::Segments, 2_097_149),
            InventoryCall::Count(RecoveryNamespace::Catalogs, 2_097_149),
            InventoryCall::Read(RecoveryNamespace::Root, 2),
            InventoryCall::Read(RecoveryNamespace::Staging, 1),
            InventoryCall::Read(RecoveryNamespace::Segments, 0),
            InventoryCall::Read(RecoveryNamespace::Catalogs, 1),
        ]
    );
    assert_eq!(
        inventory
            .entries()
            .iter()
            .map(|entry| (entry.namespace(), entry.name().as_bytes()))
            .collect::<Vec<_>>(),
        vec![
            (RecoveryNamespace::Root, b"HEAD".as_slice()),
            (RecoveryNamespace::Root, b"writer.lock".as_slice()),
            (RecoveryNamespace::Staging, b"current.seg".as_slice()),
            (RecoveryNamespace::Catalogs, b"catalog-2".as_slice()),
        ]
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[test]
fn unsupported_platform_refuses_before_filesystem_inventory_mutation() -> Result<(), Box<dyn Error>>
{
    let sandbox = TestDirectory::create("recovery-inventory-unsupported")?;
    let Err(error) = FilesystemRecoveryInventoryReader::open(sandbox.path()) else {
        return Err("filesystem inventory admitted an unsupported platform".into());
    };

    assert!(matches!(
        error,
        RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Root,
            operation: RecoveryInventoryOperation::OpenNamespace,
            ref source,
        } if source.kind() == std::io::ErrorKind::Unsupported
    ));
    assert_eq!(fs::read_dir(sandbox.path())?.count(), 0);
    sandbox.remove()?;
    Ok(())
}

pub(crate) fn name(bytes: &[u8]) -> Result<RecoveryEntryName, Box<dyn Error>> {
    Ok(RecoveryEntryName::new(bytes.to_vec())?)
}
