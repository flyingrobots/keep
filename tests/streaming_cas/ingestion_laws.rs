//! Streaming ingestion, staging, and publication laws.

use std::error::Error;
use std::io::{self, Cursor, ErrorKind, Read};

use keep::{
    BlobId, IngestionError, LayoutEntryLimit, LayoutValidationError, PublishError,
    ReconstructionError, ReferenceStore, ReferenceStoreCapacity,
};

use crate::support::{FailingReader, LyingReader, PartitionReader};

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

#[test]
fn entry_limit_refuses_during_streaming_before_a_later_source_failure() -> Result<(), Box<dyn Error>>
{
    let source = vec![0_u8; 524_288];
    let mut reader = BytesThenFailure {
        bytes: Cursor::new(source),
    };
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let limit = LayoutEntryLimit::new(1)?;

    let error = store
        .stage(&mut reader, limit)
        .err()
        .ok_or("two chunks unexpectedly staged under a one-entry limit")?;

    assert!(matches!(
        error,
        IngestionError::Layout(LayoutValidationError::EntryLimitExceeded {
            maximum: 1,
            observed: 2
        })
    ));
    Ok(())
}

#[test]
fn intervening_capacity_refusal_publishes_no_partial_state() -> Result<(), Box<dyn Error>> {
    let first_bytes = b"first staged value";
    let second_bytes = b"other staged value";
    assert_eq!(first_bytes.len(), second_bytes.len());
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(first_bytes.len()));
    let mut first_source = Cursor::new(first_bytes);
    let first = store.stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?;
    let mut second_source = Cursor::new(second_bytes);
    let second = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;
    let second_target = second.target();
    let first_published = first.commit(&mut store)?;

    let error = second
        .commit(&mut store)
        .err()
        .ok_or("intervening capacity use unexpectedly admitted both blobs")?;

    assert!(matches!(
        error,
        PublishError::CapacityExceeded {
            capacity,
            attempted
        } if capacity == first_bytes.len()
            && attempted == first_bytes.len().checked_add(second_bytes.len())
                .ok_or("fixture length overflow")?
    ));
    let mut first_output = Vec::new();
    store.reconstruct(first_published.target(), &mut first_output)?;
    assert_eq!(first_output, first_bytes);
    assert!(matches!(
        store.reconstruct(second_target, &mut Vec::new()),
        Err(ReconstructionError::BlobMissing { requested })
            if requested == second_target
    ));
    Ok(())
}

#[test]
fn cross_store_publication_refuses_chunks_not_owned_by_staged_work() -> Result<(), Box<dyn Error>> {
    let source = b"deduplicated bytes belong to the store that supplied them";
    let capacity = ReferenceStoreCapacity::new(1_048_576);
    let mut origin = ReferenceStore::new(capacity);
    let mut initial_source = Cursor::new(source);
    origin
        .stage(&mut initial_source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut origin)?;
    let mut repeated_source = Cursor::new(source);
    let staged = origin.stage(&mut repeated_source, LayoutEntryLimit::MAXIMUM)?;
    assert_eq!(staged.pending_chunk_count(), 0);
    let target = staged.target();
    let layout = staged.layout_id();
    let missing = staged
        .layout()
        .entries()
        .first()
        .ok_or("fixture layout has no chunk")?
        .chunk_id();
    let mut destination = ReferenceStore::new(capacity);

    let error = staged
        .commit(&mut destination)
        .err()
        .ok_or("cross-store commit published a layout without its chunks")?;

    assert!(matches!(
        error,
        PublishError::StagedChunkMissing {
            layout: observed_layout,
            chunk
        } if observed_layout == layout && chunk == missing
    ));
    assert!(!destination.contains_blob(target));
    assert!(matches!(
        destination.reconstruct(target, &mut Vec::new()),
        Err(ReconstructionError::BlobMissing { requested }) if requested == target
    ));
    Ok(())
}

struct BytesThenFailure {
    bytes: Cursor<Vec<u8>>,
}

impl Read for BytesThenFailure {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let observed = self.bytes.read(buffer)?;
        if observed == 0 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "failure after complete test bytes",
            ));
        }
        Ok(observed)
    }
}
