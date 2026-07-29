//! Bounded, deterministic recovery-inventory laws.

#[path = "recovery_inventory/inventory_double.rs"]
pub mod inventory_double;
#[path = "recovery_inventory/refusal_laws.rs"]
mod refusal_laws;

use std::error::Error;

use inventory_double::{InventoryCall, InventoryDouble};
use keep::{RecoveryEntryName, RecoveryInventoryLimit, RecoveryNamespace, read_recovery_inventory};

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
            InventoryCall::Count(RecoveryNamespace::Root),
            InventoryCall::Count(RecoveryNamespace::Staging),
            InventoryCall::Count(RecoveryNamespace::Segments),
            InventoryCall::Count(RecoveryNamespace::Catalogs),
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

pub(crate) fn name(bytes: &[u8]) -> Result<RecoveryEntryName, Box<dyn Error>> {
    Ok(RecoveryEntryName::new(bytes.to_vec())?)
}
