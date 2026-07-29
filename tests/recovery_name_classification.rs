//! Deterministic recovery namespace classification laws.

#[path = "recovery_name_classification/grammar_laws.rs"]
mod grammar_laws;
#[path = "recovery_inventory/inventory_double.rs"]
pub mod inventory_double;
#[path = "recovery_name_classification/refusal_laws.rs"]
mod refusal_laws;

use std::error::Error;

use inventory_double::InventoryDouble;
use keep::{
    RecoveryEntryName, RecoveryEntryRole, RecoveryInventory, RecoveryInventoryLimit,
    classify_recovery_names, read_recovery_inventory,
};

#[test]
fn canonical_names_produce_exact_typed_roles() -> Result<(), Box<dyn Error>> {
    let segment_name = format!("{}.seg", "00".repeat(32));
    let catalog_name = format!("{:016x}-{}.cat", 1_u64, "11".repeat(32));
    let inventory = inventory([
        names(&["writer.lock", "staging", "segments", "catalogs", "HEAD"])?,
        names(&["current.seg"])?,
        vec![name(segment_name.as_bytes())?],
        vec![name(catalog_name.as_bytes())?],
    ])?;

    let manifest = classify_recovery_names(inventory)?;

    assert!(
        manifest
            .entries()
            .iter()
            .any(|entry| matches!(entry.role(), RecoveryEntryRole::CurrentHead))
    );
    assert!(
        manifest
            .entries()
            .iter()
            .any(|entry| matches!(entry.role(), RecoveryEntryRole::SegmentStage))
    );
    assert!(manifest.entries().iter().any(|entry| matches!(
        entry.role(),
        RecoveryEntryRole::ImmutableSegment { digest }
            if digest.as_bytes() == &[0_u8; 32]
    )));
    assert!(manifest.entries().iter().any(|entry| matches!(
        entry.role(),
        RecoveryEntryRole::ImmutableCatalog { generation, digest }
            if generation.get() == 1 && digest.as_bytes() == &[0x11_u8; 32]
    )));
    Ok(())
}

pub(crate) fn inventory(
    names: [Vec<RecoveryEntryName>; 4],
) -> Result<RecoveryInventory, Box<dyn Error>> {
    let counts = names.each_ref().map(|entries| {
        u64::try_from(entries.len()).map_err(|_| "test inventory count does not fit u64")
    });
    let [root, staging, segments, catalogs] = counts;
    let mut storage = InventoryDouble::new([root?, staging?, segments?, catalogs?], names);
    let inventory =
        read_recovery_inventory(&mut storage, RecoveryInventoryLimit::protocol_maximum())?;
    assert_eq!(storage.calls().len(), 8);
    Ok(inventory)
}

pub(crate) fn initialized_root() -> Result<Vec<RecoveryEntryName>, Box<dyn Error>> {
    names(&["writer.lock", "staging", "segments", "catalogs"])
}

pub(crate) fn names(values: &[&str]) -> Result<Vec<RecoveryEntryName>, Box<dyn Error>> {
    values.iter().map(|value| name(value.as_bytes())).collect()
}

pub(crate) fn name(bytes: &[u8]) -> Result<RecoveryEntryName, Box<dyn Error>> {
    Ok(RecoveryEntryName::new(bytes.to_vec())?)
}
