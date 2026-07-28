//! Isolated heap-allocation evidence for committed CAS reconstruction.

use std::error::Error;
use std::io::{Cursor, sink};

use allocation_counter::{AllocationInfo, measure};
use keep::{LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity};

#[test]
fn committed_reconstruction_allocates_no_adapter_owned_heap_memory() -> Result<(), Box<dyn Error>> {
    let source = vec![0_u8; 300_000];
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let mut writer = sink();
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.reconstruct(published.target(), &mut writer));
    });

    let receipt = result.ok_or("allocation measurement did not run reconstruction")??;
    assert_eq!(receipt.target(), published.target());
    assert_eq!(observed, AllocationInfo::default());
    Ok(())
}
