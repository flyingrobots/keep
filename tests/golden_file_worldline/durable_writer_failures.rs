//! Exact durable output capability failures.
//!
//! Size: medium. Oracle: the Write contract and exact accepted-byte accounting.
//! Delete when durable reads disappear or stronger output laws subsume these cases.

use super::durable_fixture::{build, identify, policy};
use crate::support::{FailingWriter, LyingWriter};
use keep::{
    ByteLength, ByteOffset, ByteRange, DurableReadError, DurableStore, RangeReadError,
    ReaderAttemptLimit, ReconstructionError,
};
use std::error::Error;
use std::io::ErrorKind;

#[test]
fn durable_reconstruction_preserves_immediate_output_failure() -> Result<(), Box<dyn Error>> {
    let bytes = b"exact output capability failure";
    let sandbox = build("durable-reconstruction-failing", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let failure = snapshot
        .reconstruct(identified.target, &mut FailingWriter)
        .err()
        .ok_or("broken writer received a successful receipt")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(error)
        if matches!(error.as_ref(), ReconstructionError::Write { layout, bytes_written, source }
            if *layout == identified.record.id() && bytes_written.is_empty()
                && source.kind() == ErrorKind::PermissionDenied)),
        "exact output failure: {failure:?}"
    );
    Ok(())
}

#[test]
fn durable_reconstruction_rejects_impossible_write_counts() -> Result<(), Box<dyn Error>> {
    let bytes = b"exact output capability failure";
    let sandbox = build("durable-reconstruction-lying", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let failure = snapshot
        .reconstruct(identified.target, &mut LyingWriter)
        .err()
        .ok_or("broken writer received a successful receipt")?;
    let expected_maximum = bytes.len();
    let expected_observed = expected_maximum.checked_add(1).ok_or("count overflow")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(error)
        if matches!(error.as_ref(), ReconstructionError::InvalidWriteCount { layout, maximum, observed, bytes_written }
            if *layout == identified.record.id() && bytes_written.is_empty()
                && *maximum == expected_maximum && *observed == expected_observed)),
        "exact output failure: {failure:?}"
    );
    Ok(())
}

#[test]
fn durable_range_preserves_immediate_output_failure() -> Result<(), Box<dyn Error>> {
    let bytes = b"exact output capability failure";
    let sandbox = build("durable-range-failing", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(11))?;
    let failure = snapshot
        .read_range(identified.target, requested, &mut FailingWriter)
        .err()
        .ok_or("broken writer received a successful receipt")?;
    assert!(
        matches!(&failure, DurableReadError::RangeRead(error)
        if matches!(error.as_ref(), RangeReadError::Write { layout, bytes_written, source }
            if *layout == identified.record.id() && bytes_written.is_empty()
                && source.kind() == ErrorKind::PermissionDenied)),
        "exact output failure: {failure:?}"
    );
    Ok(())
}

#[test]
fn durable_range_rejects_impossible_write_counts() -> Result<(), Box<dyn Error>> {
    let bytes = b"exact output capability failure";
    let sandbox = build("durable-range-lying", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(11))?;
    let failure = snapshot
        .read_range(identified.target, requested, &mut LyingWriter)
        .err()
        .ok_or("broken writer received a successful receipt")?;
    let expected_maximum = 11_usize;
    let expected_observed = expected_maximum.checked_add(1).ok_or("count overflow")?;
    assert!(
        matches!(&failure, DurableReadError::RangeRead(error)
        if matches!(error.as_ref(), RangeReadError::InvalidWriteCount { layout, maximum, observed, bytes_written }
            if *layout == identified.record.id() && bytes_written.is_empty()
                && *maximum == expected_maximum && *observed == expected_observed)),
        "exact output failure: {failure:?}"
    );
    Ok(())
}
