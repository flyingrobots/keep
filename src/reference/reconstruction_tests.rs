//! Private-state corruption laws for authenticated reconstruction.

use std::error::Error;
use std::io::Cursor;

use crate::{LayoutEntryLimit, ReconstructionError, ReferenceStore, ReferenceStoreCapacity};

#[test]
fn corrupted_stored_chunk_refuses_before_output() -> Result<(), Box<dyn Error>> {
    let source = b"stored bytes must continue to match their chunk identity";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let layout = store
        .layout(published.layout_id())
        .ok_or("published layout is absent")?;
    let identity = layout
        .entries()
        .first()
        .ok_or("published layout has no chunk")?
        .chunk_id();
    let byte = store
        .chunks
        .get_mut(&identity)
        .and_then(|bytes| bytes.first_mut())
        .ok_or("published chunk is absent")?;
    *byte ^= 1;
    let mut output = Vec::new();

    let error = store
        .reconstruct(published.target(), &mut output)
        .err()
        .ok_or("corrupt chunk unexpectedly reconstructed")?;

    assert!(matches!(
        error,
        ReconstructionError::ChunkIdentityMismatch { expected, .. }
            if expected == identity
    ));
    assert!(output.is_empty());
    Ok(())
}

/// Issue #71: every chunk is hashed exactly once per reconstruction. The
/// verification pass hashes; the emission pass fetches by identity.
#[test]
fn every_chunk_is_hashed_exactly_once_per_reconstruction() -> Result<(), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(vec![0_u8; 300_000]);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let entries: Vec<_> = store
        .layout(published.layout_id())
        .ok_or("published layout is absent")?
        .entries()
        .iter()
        .map(|entry| entry.chunk_id())
        .collect();
    assert!(entries.len() >= 2, "the law needs a multi-chunk blob");
    store.observed_chunk_hashes.borrow_mut().clear();
    let mut output = Vec::new();

    let receipt = store.reconstruct(published.target(), &mut output)?;

    assert_eq!(receipt.bytes_written().get(), 300_000);
    assert_eq!(store.observed_chunk_hashes.borrow().as_slice(), entries);
    Ok(())
}
