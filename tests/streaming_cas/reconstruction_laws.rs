//! Exact authenticated reconstruction laws.

use std::error::Error;
use std::io::Cursor;

use keep::{
    AdmittedLayout, BlobId, LayoutEntryLimit, ReconstructionError, ReferenceStore,
    ReferenceStoreCapacity, RegisteredStorageProfile,
};

use crate::support::{PartitionWriter, detect_spans};

#[test]
fn committed_blob_reconstructs_exactly_through_short_writes() -> Result<(), Box<dyn Error>> {
    let source = vec![0_u8; 300_000];
    let capacity = ReferenceStoreCapacity::new(1_048_576);
    let mut store = ReferenceStore::new(capacity);
    let mut reader = Cursor::new(&source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let widths = [1, 7, 4_093, 8_192];
    let mut writer = PartitionWriter::new(&widths);

    let receipt = store.reconstruct(published.target(), &mut writer)?;

    assert_eq!(writer.bytes(), source);
    assert_eq!(receipt.target(), published.target());
    assert_eq!(receipt.layout_id(), published.layout_id());
    assert_eq!(receipt.bytes_written().get(), 300_000);
    Ok(())
}

#[test]
fn whole_blob_mismatch_refuses_before_writing_any_bytes() -> Result<(), Box<dyn Error>> {
    let source = vec![0_u8; 300_000];
    let wrong_bytes = vec![1_u8; source.len()];
    let wrong_target = BlobId::hash_bytes(&wrong_bytes)?;
    let observed_target = BlobId::hash_bytes(&source)?;
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(&source);
    let _published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let mismatched = AdmittedLayout::from_spans(
        wrong_target,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        detect_spans(&source)?,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let mut output = Vec::new();

    let error = store
        .reconstruct_admitted_layout(&mismatched, &mut output)
        .err()
        .ok_or("mismatched target unexpectedly reconstructed")?;

    assert!(matches!(
        error,
        ReconstructionError::BlobIdentityMismatch {
            expected,
            observed,
            ..
        } if expected == wrong_target && observed == observed_target
    ));
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn missing_chunk_refuses_before_writing_any_bytes() -> Result<(), Box<dyn Error>> {
    let source = b"layout exists but its exact chunk does not";
    let target = BlobId::hash_bytes(source)?;
    let layout = AdmittedLayout::from_spans(
        target,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        detect_spans(source)?,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let expected_chunk = layout
        .entries()
        .first()
        .ok_or("missing expected layout entry")?
        .chunk_id();
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut output = Vec::new();

    let error = store
        .reconstruct_admitted_layout(&layout, &mut output)
        .err()
        .ok_or("absent chunk unexpectedly reconstructed")?;

    assert!(matches!(
        error,
        ReconstructionError::ChunkMissing {
            index: 0,
            requested,
            ..
        } if requested == expected_chunk
    ));
    assert!(output.is_empty());
    Ok(())
}
