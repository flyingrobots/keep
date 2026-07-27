//! Malformed-layout and output-boundary refusal laws.

use std::error::Error;
use std::io::{Cursor, ErrorKind};

use keep::{
    IngestionError, LayoutDecodeError, LayoutDecodePolicy, LayoutEntryLimit, ReconstructionError,
    ReferenceStore, ReferenceStoreCapacity,
};

use crate::support::{FailingWriter, LyingWriter, ZeroWriter};

#[test]
fn malformed_layout_refuses_before_chunk_lookup_or_output() -> Result<(), Box<dyn Error>> {
    let source = b"canonical layout before checksum corruption";
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let staged = store.stage(&mut reader, LayoutEntryLimit::MAXIMUM)?;
    let mut encoded = staged.layout().encode_record()?.bytes().to_vec();
    let checksum_tail = encoded
        .last_mut()
        .ok_or("canonical layout unexpectedly empty")?;
    *checksum_tail ^= 1;
    let mut output = Vec::new();
    let policy = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);

    let error = store
        .reconstruct_record(&encoded, policy, &mut output)
        .err()
        .ok_or("corrupt layout unexpectedly reconstructed")?;

    assert!(matches!(
        error,
        ReconstructionError::LayoutDecode(LayoutDecodeError::ChecksumMismatch { .. })
    ));
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn broken_writers_preserve_exact_failure_boundaries() -> Result<(), Box<dyn Error>> {
    let source = b"authenticated before the writer is called";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;

    let mut zero = ZeroWriter;
    assert!(matches!(
        store.reconstruct(published.target(), &mut zero),
        Err(ReconstructionError::WriteZero { bytes_written, .. })
            if bytes_written.is_empty()
    ));

    let mut lying = LyingWriter;
    assert!(matches!(
        store.reconstruct(published.target(), &mut lying),
        Err(ReconstructionError::InvalidWriteCount {
            maximum,
            observed,
            bytes_written,
            ..
        }) if maximum == source.len()
            && observed == source.len().checked_add(1).ok_or("fixture overflow")?
            && bytes_written.is_empty()
    ));

    let mut failing = FailingWriter;
    let error = store
        .reconstruct(published.target(), &mut failing)
        .err()
        .ok_or("failing writer unexpectedly reconstructed")?;
    assert!(matches!(
        error,
        ReconstructionError::Write {
            bytes_written,
            ref source,
            ..
        } if bytes_written.is_empty() && source.kind() == ErrorKind::PermissionDenied
    ));
    assert!(Error::source(&error).is_some());
    Ok(())
}

#[test]
fn failed_ingestion_remains_distinct_from_reconstruction_refusal() {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(0));
    let mut reader = Cursor::new([1_u8]);
    assert!(matches!(
        store.stage(&mut reader, LayoutEntryLimit::MAXIMUM),
        Err(IngestionError::CapacityExceeded {
            capacity: 0,
            attempted: 1
        })
    ));
}
