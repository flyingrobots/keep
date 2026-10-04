//! Isolated heap-allocation evidence for reference-store staging and reconstruction.

use std::error::Error;
use std::io::{self, Cursor, Read, repeat, sink};

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
                && attempted <= CEILING.checked_add(maximum_chunk).ok_or("bound overflow")?
    ));
    let metadata = metadata_allowance(source.len())?;
    let ceiling = CEILING
        .checked_add(ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES)
        .and_then(|bytes| bytes.checked_add(metadata))
        .ok_or("measurement bound overflow")?;
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
    let maximum = ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES
        .checked_add(metadata_allowance(source.len())?)
        .ok_or("measurement bound overflow")?;
    assert!(
        observed.bytes_max <= u64::try_from(maximum)?,
        "deduplicated staging peaked at {} bytes above the {maximum}-byte limit",
        observed.bytes_max
    );
    assert!(observed.bytes_max >= u64::from(FastCdc::MAXIMUM_CHUNK_LENGTH.get()));
    Ok(())
}

#[test]
fn a_stream_larger_than_capacity_retains_only_unique_payloads() -> Result<(), Box<dyn Error>> {
    let chunk_length = usize::try_from(FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    let source_length = chunk_length.checked_mul(16).ok_or("fixture overflow")?;
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(chunk_length));
    let mut reader = repeat(0).take(u64::try_from(source_length)?);
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.stage(&mut reader, LayoutEntryLimit::MAXIMUM));
    });

    let staged = result.ok_or("measurement did not run staging")??;
    assert_eq!(
        staged.target().logical_length().get(),
        u64::try_from(source_length)?
    );
    assert_eq!(staged.pending_chunk_count(), 1);
    assert_eq!(staged.pending_materialized_bytes(), chunk_length);
    assert!(!store.contains_blob(staged.target()));
    let metadata = metadata_allowance(source_length)?;
    let maximum = ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES
        .checked_add(chunk_length)
        .and_then(|bytes| bytes.checked_add(metadata))
        .ok_or("measurement bound overflow")?;
    assert!(observed.bytes_max <= u64::try_from(maximum)?);
    Ok(())
}

#[test]
fn a_multi_gib_source_refuses_before_materializing_a_prefix() -> Result<(), Box<dyn Error>> {
    const SOURCE_LENGTH: u64 = 4_294_967_296;
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(CEILING));
    let mut reader = repeat(0).take(SOURCE_LENGTH);
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.stage(&mut reader, LayoutEntryLimit::MAXIMUM));
    });

    let error = result
        .ok_or("measurement did not run staging")?
        .err()
        .ok_or("an oversized first chunk unexpectedly staged")?;
    let chunk_length = usize::try_from(FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    assert!(matches!(
        error,
        IngestionError::CapacityExceeded { capacity, attempted }
            if capacity == CEILING && attempted == chunk_length
    ));
    let consumed = SOURCE_LENGTH
        .checked_sub(reader.limit())
        .ok_or("fixture accounting")?;
    assert_eq!(consumed, u64::try_from(chunk_length)?);
    let maximum = ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES
        .checked_add(METADATA_ALLOWANCE_PER_CHUNK)
        .ok_or("measurement bound overflow")?;
    assert!(observed.bytes_max <= u64::try_from(maximum)?);
    assert_eq!(observed.bytes_current, 0);
    Ok(())
}

#[test]
fn source_failure_releases_staged_payload_and_preserves_committed_content()
-> Result<(), Box<dyn Error>> {
    let original = b"already committed bytes";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let published = store
        .stage(&mut Cursor::new(original), LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let refused_target = keep::BlobId::hash_bytes(&vec![0_u8; 300_000])?;
    // Preserve the source error without counting its caller-owned allocation
    // as staging memory retained after refusal.
    let failure = io::Error::new(
        io::ErrorKind::PermissionDenied,
        "source failed after staged chunk",
    );
    let mut reader = repeat(0).take(300_000).chain(FailedSource {
        failure: Some(failure),
    });
    let mut result = None;

    let observed = measure(|| {
        result = Some(store.stage(&mut reader, LayoutEntryLimit::MAXIMUM));
    });

    let error = result
        .ok_or("measurement did not run staging")?
        .err()
        .ok_or("failed source unexpectedly staged")?;
    let IngestionError::Read { source } = error else {
        return Err("source failure reached the wrong boundary".into());
    };
    assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(source.to_string(), "source failed after staged chunk");
    assert_eq!(reader.get_ref().0.limit(), 0);
    assert!(
        observed.bytes_max
            >= u64::from(FastCdc::MAXIMUM_CHUNK_LENGTH.get())
                .checked_mul(2)
                .ok_or("fixture bound overflow")?
    );
    assert_eq!(observed.bytes_current, 0, "source refusal retained heap");
    assert!(!store.contains_blob(refused_target));
    let mut output = Vec::new();
    let receipt = store.reconstruct(published.target(), &mut output)?;
    assert_eq!(receipt.target(), published.target());
    assert_eq!(output, original);
    Ok(())
}

struct FailedSource {
    failure: Option<io::Error>,
}

impl Read for FailedSource {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        self.failure
            .take()
            .map_or_else(|| Err(io::ErrorKind::BrokenPipe.into()), Err)
    }
}

fn metadata_allowance(source_length: usize) -> Result<usize, Box<dyn Error>> {
    let minimum_chunk = usize::try_from(FastCdc::MINIMUM_CHUNK_LENGTH.get())?;
    source_length
        .div_ceil(minimum_chunk)
        .checked_add(1)
        .and_then(|entries| entries.checked_mul(METADATA_ALLOWANCE_PER_CHUNK))
        .ok_or_else(|| "measurement metadata bound overflow".into())
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
