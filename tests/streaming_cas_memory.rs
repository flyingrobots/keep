//! Isolated heap-allocation evidence for reference-store staging and reconstruction.

use std::error::Error;
use std::io::{Cursor, sink};

use allocation_counter::{AllocationInfo, measure};
use keep::{
    ByteLength, ByteOffset, ByteRange, FastCdc, IngestionError, LayoutEntryLimit, ReferenceStore,
    ReferenceStoreCapacity,
};

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

#[test]
fn committed_range_reads_allocate_no_adapter_owned_heap_memory() -> Result<(), Box<dyn Error>> {
    let source = vec![0_u8; 300_000];
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let requested = ByteRange::new(ByteOffset::new(260_000), ByteLength::new(10_000))?;
    let mut writer = sink();
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.read_layout_range(published.layout_id(), requested, &mut writer));
    });

    let receipt = result.ok_or("allocation measurement did not run the range read")??;
    assert_eq!(receipt.requested(), requested);
    assert_eq!(receipt.bytes_written(), requested.length());
    assert_eq!(observed, AllocationInfo::default());
    Ok(())
}

const CEILING: usize = 200_000;

/// Layout metadata a staging call may hold per emitted chunk: one span, one
/// admitted entry, one encoded record entry, and map-node slack.
const METADATA_ALLOWANCE_PER_CHUNK: usize = 1_024;

#[test]
fn staging_ceiling_refuses_before_pending_bytes_exceed_capacity() -> Result<(), Box<dyn Error>> {
    let source = deterministic_bytes(1_000_000);
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(CEILING));
    let mut reader = Cursor::new(&source);
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.stage(&mut reader, LayoutEntryLimit::MAXIMUM));
    });

    let error = result
        .ok_or("allocation measurement did not run staging")?
        .err()
        .ok_or("a source five times the capacity unexpectedly staged")?;
    let maximum_chunk = usize::try_from(FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    assert!(matches!(
        error,
        IngestionError::CapacityExceeded { capacity, attempted }
            if capacity == CEILING
                && attempted > CEILING
                && attempted <= CEILING.saturating_add(maximum_chunk)
    ));
    let ceiling = CEILING
        .saturating_add(ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES)
        .saturating_add(metadata_allowance(source.len()));
    assert!(
        observed.bytes_max <= u64::try_from(ceiling)?,
        "staging peaked at {} bytes above the {ceiling}-byte ceiling",
        observed.bytes_max
    );
    assert_eq!(observed.bytes_current, 0, "a refusal retained heap memory");
    Ok(())
}

#[test]
fn deduplicated_staging_allocates_only_bounded_scratch() -> Result<(), Box<dyn Error>> {
    let source = deterministic_bytes(300_000);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(&source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let mut reader = Cursor::new(&source);
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.stage(&mut reader, LayoutEntryLimit::MAXIMUM));
    });

    let staged = result.ok_or("allocation measurement did not run staging")??;
    assert_eq!(staged.target(), published.target());
    assert_eq!(staged.pending_chunk_count(), 0);
    assert_eq!(staged.pending_materialized_bytes(), 0);
    let floor = ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES
        .saturating_add(metadata_allowance(source.len()));
    assert!(
        observed.bytes_max <= u64::try_from(floor)?,
        "deduplicated staging peaked at {} bytes above the {floor}-byte floor",
        observed.bytes_max
    );
    Ok(())
}

fn metadata_allowance(source_length: usize) -> usize {
    let minimum_chunk = usize::try_from(FastCdc::MINIMUM_CHUNK_LENGTH.get()).unwrap_or(1);
    source_length
        .div_ceil(minimum_chunk)
        .saturating_add(1)
        .saturating_mul(METADATA_ALLOWANCE_PER_CHUNK)
}

fn deterministic_bytes(length: usize) -> Vec<u8> {
    let mut state = 0x0123_4567_89ab_cdef_u64;
    let mut bytes = Vec::with_capacity(length);
    for _ in 0..length {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        let [byte, ..] = state.to_le_bytes();
        bytes.push(byte);
    }
    bytes
}
