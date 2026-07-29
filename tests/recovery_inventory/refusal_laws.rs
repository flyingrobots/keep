//! Exact bounded-inventory refusal laws.

use std::error::Error;
use std::io;

use keep::{
    RecoveryEntryNameError, RecoveryInventoryError, RecoveryInventoryLimit,
    RecoveryInventoryLimitError, RecoveryInventoryOperation, RecoveryInventoryStorage,
    RecoveryNamespace, read_recovery_inventory,
};

use super::inventory_double::{InventoryCall, InventoryDouble};
use super::name;

#[test]
fn configured_limit_refuses_before_any_name_is_retained() -> Result<(), Box<dyn Error>> {
    let mut storage = InventoryDouble::new([2, 2, 0, 0], empty_names());
    let limit = RecoveryInventoryLimit::new(3)?;
    let Err(error) = read_recovery_inventory(&mut storage, limit) else {
        return Err("inventory exceeded its configured entry limit".into());
    };

    assert!(matches!(
        error,
        RecoveryInventoryError::EntryLimit {
            maximum: 3,
            observed_at_least: 4,
        }
    ));
    assert_eq!(
        storage.calls(),
        &[
            InventoryCall::Count(RecoveryNamespace::Root, 3),
            InventoryCall::Count(RecoveryNamespace::Staging, 1),
        ]
    );
    Ok(())
}

#[test]
fn changed_entry_count_is_an_exact_refusal() -> Result<(), Box<dyn Error>> {
    let mut names = empty_names();
    names[0] = vec![name(b"HEAD")?, name(b"writer.lock")?];
    let mut storage = InventoryDouble::new([1, 0, 0, 0], names);
    let Err(error) =
        read_recovery_inventory(&mut storage, RecoveryInventoryLimit::protocol_maximum())
    else {
        return Err("inventory admitted a count that changed before reading".into());
    };

    assert!(matches!(
        error,
        RecoveryInventoryError::Changed {
            namespace: RecoveryNamespace::Root,
            counted: 1,
            observed: 2,
        }
    ));
    Ok(())
}

#[test]
fn duplicate_names_are_refused_after_deterministic_sorting() -> Result<(), Box<dyn Error>> {
    let mut names = empty_names();
    names[2] = vec![name(b"segment-a")?, name(b"segment-a")?];
    let mut storage = InventoryDouble::new([0, 0, 2, 0], names);
    let Err(error) =
        read_recovery_inventory(&mut storage, RecoveryInventoryLimit::protocol_maximum())
    else {
        return Err("inventory admitted a duplicate namespace entry".into());
    };

    assert!(matches!(
        error,
        RecoveryInventoryError::Duplicate {
            namespace: RecoveryNamespace::Segments,
            ref name,
        } if name.as_bytes() == b"segment-a"
    ));
    Ok(())
}

#[test]
fn limits_and_entry_names_refuse_out_of_contract_values() {
    assert!(matches!(
        RecoveryInventoryLimit::new(2_097_153),
        Err(RecoveryInventoryLimitError::AboveProtocolMaximum {
            requested: 2_097_153,
            maximum: 2_097_152,
        })
    ));
    assert!(matches!(
        keep::RecoveryEntryName::new(Vec::new()),
        Err(RecoveryEntryNameError::Empty)
    ));
    assert!(matches!(
        keep::RecoveryEntryName::new(b"../HEAD".to_vec()),
        Err(RecoveryEntryNameError::PathSeparator)
    ));
    assert!(matches!(
        keep::RecoveryEntryName::new(vec![b'H', 0, b'D']),
        Err(RecoveryEntryNameError::Nul)
    ));
}

#[test]
fn storage_refusal_preserves_namespace_operation_and_source() -> Result<(), Box<dyn Error>> {
    let mut storage = CountFailure;
    let Err(error) =
        read_recovery_inventory(&mut storage, RecoveryInventoryLimit::protocol_maximum())
    else {
        return Err("inventory discarded a storage count failure".into());
    };

    assert!(matches!(
        error,
        RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Root,
            operation: RecoveryInventoryOperation::Count,
            ref source,
        } if source.kind() == io::ErrorKind::PermissionDenied
    ));
    Ok(())
}

fn empty_names() -> [Vec<keep::RecoveryEntryName>; 4] {
    std::array::from_fn(|_| Vec::new())
}

struct CountFailure;

impl RecoveryInventoryStorage for CountFailure {
    fn count_entries(&mut self, _namespace: RecoveryNamespace, _remaining: u64) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "injected count refusal",
        ))
    }

    fn read_entry_names(
        &mut self,
        _namespace: RecoveryNamespace,
        _expected_count: u64,
    ) -> io::Result<Vec<keep::RecoveryEntryName>> {
        Err(io::Error::other(
            "name reads are unreachable after count refusal",
        ))
    }
}
