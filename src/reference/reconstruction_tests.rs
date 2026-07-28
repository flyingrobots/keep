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
