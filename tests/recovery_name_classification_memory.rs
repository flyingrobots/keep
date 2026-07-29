//! Isolated heap-allocation evidence for recovery name classification.

#[path = "recovery_inventory/inventory_double.rs"]
pub mod inventory_double;

use std::error::Error;

use allocation_counter::measure;
use inventory_double::InventoryDouble;
use keep::{
    RecoveryEntryName, RecoveryInventoryLimit, RecoveryNameClassificationError,
    classify_recovery_names, read_recovery_inventory,
};

#[test]
fn refusal_moves_the_exact_name_without_an_extra_allocation() -> Result<(), Box<dyn Error>> {
    let names = [
        recovery_names(&["writer.lock", "staging", "segments", "catalogs", "unknown"])?,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    let mut storage = InventoryDouble::new([5, 0, 0, 0], names);
    let inventory =
        read_recovery_inventory(&mut storage, RecoveryInventoryLimit::protocol_maximum())?;
    assert_eq!(storage.calls().len(), 8);
    let mut input = Some(inventory);
    let mut result = None;

    let allocations = measure(|| {
        if let Some(inventory) = input.take() {
            result = Some(classify_recovery_names(inventory));
        }
    });
    let error = match result.ok_or("name classification did not run")? {
        Ok(_manifest) => return Err("unknown recovery name was admitted".into()),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        RecoveryNameClassificationError::Unexpected { ref name, .. }
            if name.as_bytes() == b"unknown"
    ));
    assert_eq!(allocations.count_total, 1);
    assert_eq!(allocations.count_max, 1);
    Ok(())
}

fn recovery_names(values: &[&str]) -> Result<Vec<RecoveryEntryName>, Box<dyn Error>> {
    values
        .iter()
        .map(|value| Ok(RecoveryEntryName::new(value.as_bytes().to_vec())?))
        .collect()
}
