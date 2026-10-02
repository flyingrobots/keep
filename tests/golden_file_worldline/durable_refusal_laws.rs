//! Evidenced logical absence through a completely admitted durable catalog.
//!
//! Size: medium; production publication and migration in an owned ext4 namespace.
//! Oracle: a published layout names the exact missing chunk; no output is permitted.
//! Delete when durable exact-layout reads are removed or stronger refusal evidence subsumes these laws.

use std::error::Error;

use super::durable_fixture::{build, build_missing_chunk, identify, policy};
use keep::{
    ByteLength, ByteOffset, ByteRange, DurableReadError, DurableStore, RangePlanError,
    RangeReadError, ReaderAttemptLimit, ReconstructionError,
};

#[test]
fn a_durable_layout_naming_an_absent_chunk_refuses_before_reconstruction_output()
-> Result<(), Box<dyn Error>> {
    let bytes = b"layout evidence exists but its chunk does not";
    let sandbox = build_missing_chunk("durable-missing-logical-chunk", bytes)?;
    let expected = identify(bytes)?;
    let chunk = expected
        .spans
        .first()
        .ok_or("nonempty input has no chunk")?
        .id();
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .reconstruct_layout(expected.record.id(), &mut output)
        .err()
        .ok_or("missing chunk reconstructed")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(source)
        if matches!(source.as_ref(), ReconstructionError::ChunkMissing { layout, index, requested }
            if *layout == expected.record.id() && *index == 0 && *requested == chunk)),
        "missing logical member must be evidenced with exact coordinates: {failure:?}"
    );
    assert_eq!(
        output,
        [0xAB],
        "refusal must preserve the caller's existing output"
    );
    Ok(())
}

#[test]
fn a_durable_range_naming_an_absent_chunk_refuses_before_output() -> Result<(), Box<dyn Error>> {
    let bytes = b"range layout evidence without its selected chunk";
    let sandbox = build_missing_chunk("durable-range-missing-chunk", bytes)?;
    let expected = identify(bytes)?;
    let chunk = expected
        .spans
        .first()
        .ok_or("nonempty input has no chunk")?
        .id();
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(1), ByteLength::new(3))?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .read_layout_range(expected.record.id(), requested, &mut output)
        .err()
        .ok_or("missing chunk range succeeded")?;
    assert!(
        matches!(&failure, DurableReadError::RangeRead(source)
        if matches!(source.as_ref(), RangeReadError::ChunkMissing { layout, index, requested }
            if *layout == expected.record.id() && *index == 0 && *requested == chunk)),
        "missing logical range member must preserve exact coordinates: {failure:?}"
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

#[test]
fn an_out_of_bounds_durable_range_refuses_the_exact_requested_coordinates()
-> Result<(), Box<dyn Error>> {
    let bytes = b"one";
    let sandbox = build("durable-range-outside", &[bytes])?;
    let expected = identify(bytes)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(2), ByteLength::new(2))?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .read_range(expected.target, requested, &mut output)
        .err()
        .ok_or("out of bounds range succeeded")?;
    assert!(
        matches!(&failure, DurableReadError::RangeRead(source)
        if matches!(source.as_ref(), RangeReadError::RangePlan(RangePlanError::OutOfBounds { requested: actual, target_length })
            if *actual == requested && *target_length == expected.target.logical_length())),
        "exact range bounds: {failure:?}"
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

#[test]
fn an_absent_durable_blob_refuses_range_output_with_its_exact_identity()
-> Result<(), Box<dyn Error>> {
    let sandbox = build("durable-absent-blob", &[b"present"])?;
    let absent = identify(b"absent")?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(0), ByteLength::new(1))?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .read_range(absent.target, requested, &mut output)
        .err()
        .ok_or("absent blob produced a range")?;
    assert!(
        matches!(failure, DurableReadError::BlobMissing { requested } if requested == absent.target)
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

#[test]
fn an_absent_durable_layout_refuses_before_reconstruction_output() -> Result<(), Box<dyn Error>> {
    let sandbox = build("durable-absent-layout", &[b"present"])?;
    let absent = identify(b"absent")?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .reconstruct_layout(absent.record.id(), &mut output)
        .err()
        .ok_or("absent layout reconstructed")?;
    assert!(
        matches!(failure, DurableReadError::LayoutMissing { requested } if requested == absent.record.id())
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}
