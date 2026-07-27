//! Public bounded streaming CAS laws.

pub mod support;

use std::error::Error;
use std::io::{Cursor, ErrorKind};

use keep::{BlobId, IngestionError, LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity};
use support::{FailingReader, LyingReader, PartitionReader};

#[test]
fn staged_content_is_invisible_until_explicit_commit() -> Result<(), Box<dyn Error>> {
    let source = b"staged bytes are not published bytes";
    let capacity = ReferenceStoreCapacity::new(1_048_576);
    let mut store = ReferenceStore::new(capacity);
    let mut reader = Cursor::new(source);

    let staged = store.stage(&mut reader, LayoutEntryLimit::MAXIMUM)?;
    let target = staged.target();

    assert!(!store.contains_blob(target));
    assert_eq!(staged.layout().target(), target);

    let published = staged.commit(&mut store)?;

    assert_eq!(published.target(), target);
    assert!(store.contains_blob(target));
    Ok(())
}

#[test]
fn identical_chunks_are_deduplicated_without_a_retention_claim() -> Result<(), Box<dyn Error>> {
    let source = b"the same exact bytes";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut first_reader = Cursor::new(source);
    let first = store.stage(&mut first_reader, LayoutEntryLimit::MAXIMUM)?;

    assert_eq!(first.pending_chunk_count(), 1);
    assert_eq!(first.pending_materialized_bytes(), source.len());
    let first_receipt = first.commit(&mut store)?;

    let mut second_reader = Cursor::new(source);
    let second = store.stage(&mut second_reader, LayoutEntryLimit::MAXIMUM)?;

    assert_eq!(second.pending_chunk_count(), 0);
    assert_eq!(second.pending_materialized_bytes(), 0);
    let second_receipt = second.commit(&mut store)?;
    assert_eq!(second_receipt, first_receipt);
    Ok(())
}

#[test]
fn capacity_refusal_publishes_nothing() -> Result<(), Box<dyn Error>> {
    let source = b"one byte beyond the configured capacity";
    let target = BlobId::hash_bytes(source)?;
    let capacity = source.len().checked_sub(1).ok_or("empty source")?;
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(capacity));
    let mut reader = Cursor::new(source);

    let error = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)
        .err()
        .ok_or("capacity overflow unexpectedly staged")?;

    assert!(matches!(
        error,
        IngestionError::CapacityExceeded {
            capacity: observed_capacity,
            attempted
        } if observed_capacity == capacity && attempted == source.len()
    ));
    assert!(!store.contains_blob(target));
    Ok(())
}

#[test]
fn short_and_interrupted_reads_preserve_blob_and_layout_identity() -> Result<(), Box<dyn Error>> {
    let source = vec![0_u8; 300_000];
    let capacity = ReferenceStoreCapacity::new(1_048_576);
    let store = ReferenceStore::new(capacity);
    let mut contiguous = Cursor::new(&source);
    let expected = store.stage(&mut contiguous, LayoutEntryLimit::MAXIMUM)?;
    let widths = [1, 7, 4_093, 8_192];
    let mut partitioned = PartitionReader::new(&source, &widths);

    let observed = store.stage(&mut partitioned, LayoutEntryLimit::MAXIMUM)?;

    assert_eq!(observed.target(), expected.target());
    assert_eq!(observed.layout_id(), expected.layout_id());
    assert_eq!(observed.layout(), expected.layout());
    Ok(())
}

#[test]
fn source_failures_preserve_io_boundaries_and_broken_read_counts() {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut failing = FailingReader;
    let failed = store.stage(&mut failing, LayoutEntryLimit::MAXIMUM);
    assert!(matches!(
        failed,
        Err(IngestionError::Read { source }) if source.kind() == ErrorKind::PermissionDenied
    ));

    let mut lying = LyingReader;
    assert!(matches!(
        store.stage(&mut lying, LayoutEntryLimit::MAXIMUM),
        Err(IngestionError::InvalidReadCount {
            maximum: 8_192,
            observed: 8_193
        })
    ));
}
