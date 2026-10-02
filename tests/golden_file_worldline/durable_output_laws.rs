//! Public durable output failure and short-write laws.
//!
//! Size: medium; controlled filesystem and writer capabilities, no network or sleeps.
//! Oracle: caller-supplied bytes, specified accepted-prefix accounting and writer refusal.
//! Delete when these APIs disappear or stronger boundary evidence subsumes these laws.

use std::error::Error;
use std::io::ErrorKind;

use super::durable_fixture::{build, identify, policy};
use crate::support::{PartitionWriter, PrefixThenFailWriter, ZeroWriter};
use keep::{
    ByteLength, ByteOffset, ByteRange, DurableReadError, DurableStore, RangeReadError,
    ReaderAttemptLimit, ReconstructionError,
};

#[test]
fn durable_reconstruction_completes_short_and_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let bytes = b"authenticated durable bytes through short writes";
    let sandbox = build("durable-short-writes", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let mut output = PartitionWriter::new(&[1, 7, 3])?;
    let receipt = snapshot.reconstruct(identified.target, &mut output)?;
    assert_eq!(output.bytes(), bytes);
    assert_eq!(
        receipt.receipt().bytes_written().get(),
        u64::try_from(bytes.len())?
    );
    Ok(())
}

#[test]
fn durable_reconstruction_preserves_a_failed_writers_accepted_prefix() -> Result<(), Box<dyn Error>>
{
    let bytes = b"a failed output has no successful receipt";
    let sandbox = build("durable-output-prefix", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let mut output = PrefixThenFailWriter::new(5)?;
    let failure = snapshot
        .reconstruct(identified.target, &mut output)
        .err()
        .ok_or("failed writer received a receipt")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(source)
        if matches!(source.as_ref(), ReconstructionError::Write { layout, bytes_written, source }
            if *layout == identified.record.id() && bytes_written.get() == 5
                && source.kind() == ErrorKind::PermissionDenied)),
        "exact prefix and cause: {failure:?}"
    );
    assert_eq!(output.bytes(), bytes.get(..5).ok_or("prefix absent")?);
    Ok(())
}

#[test]
fn durable_range_preserves_a_failed_writers_accepted_prefix() -> Result<(), Box<dyn Error>> {
    let bytes = b"a range failure preserves its exact output prefix";
    let sandbox = build("durable-range-prefix", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(13))?;
    let mut output = PrefixThenFailWriter::new(5)?;
    let failure = snapshot
        .read_range(identified.target, requested, &mut output)
        .err()
        .ok_or("failed range writer received a receipt")?;
    assert!(
        matches!(&failure, DurableReadError::RangeRead(source)
        if matches!(source.as_ref(), RangeReadError::Write { layout, bytes_written, source }
            if *layout == identified.record.id() && bytes_written.get() == 5
                && source.kind() == ErrorKind::PermissionDenied)),
        "exact prefix and cause: {failure:?}"
    );
    assert_eq!(output.bytes(), bytes.get(3..8).ok_or("prefix absent")?);
    Ok(())
}

#[test]
fn a_zero_progress_durable_writer_refuses_at_the_output_boundary() -> Result<(), Box<dyn Error>> {
    let bytes = b"nonempty output must make progress";
    let sandbox = build("durable-output-zero", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let failure = snapshot
        .reconstruct(identified.target, &mut ZeroWriter)
        .err()
        .ok_or("zero writer succeeded")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(source)
        if matches!(source.as_ref(), ReconstructionError::WriteZero { layout, bytes_written }
            if *layout == identified.record.id() && bytes_written.is_empty())),
        "exact zero-progress refusal: {failure:?}"
    );
    Ok(())
}

#[test]
fn durable_ranges_complete_short_and_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let bytes = b"authenticated durable range through short writes";
    let sandbox = build("durable-range-short", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(13))?;
    let mut output = PartitionWriter::new(&[1, 7, 3])?;
    let receipt = snapshot.read_range(identified.target, requested, &mut output)?;
    assert_eq!(output.bytes(), bytes.get(3..16).ok_or("range absent")?);
    assert_eq!(receipt.receipt().bytes_written(), requested.length());
    Ok(())
}

#[test]
fn a_zero_progress_durable_range_writer_refuses_before_acceptance() -> Result<(), Box<dyn Error>> {
    let bytes = b"nonempty range must make progress";
    let sandbox = build("durable-range-zero", &[bytes])?;
    let identified = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(13))?;
    let failure = snapshot
        .read_range(identified.target, requested, &mut ZeroWriter)
        .err()
        .ok_or("zero range writer succeeded")?;
    assert!(
        matches!(&failure, DurableReadError::RangeRead(source)
        if matches!(source.as_ref(), RangeReadError::WriteZero { layout, bytes_written }
            if *layout == identified.record.id() && bytes_written.is_empty())),
        "exact zero-progress range refusal: {failure:?}"
    );
    Ok(())
}
