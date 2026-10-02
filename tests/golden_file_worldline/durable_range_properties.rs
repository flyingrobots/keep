//! Bounded generated durable range laws with an independent source-slice oracle.
//!
//! Size: medium; owned ext4 stores and fixed deterministic input domains.
//! No random seed: coordinates are enumerated in checked-in order. Short ranges
//! run in increasing length, so the first failure is minimal in that domain.
//! Delete when range reads disappear or stronger source-slice evidence subsumes these laws.

use std::error::Error;

use super::durable_fixture::{build, policy};
use super::identity_corpus::find_case;
use keep::{
    BlobId, ByteLength, ByteOffset, ByteRange, DurableSnapshot, DurableStore, ReaderAttemptLimit,
};

#[test]
fn every_short_durable_range_equals_its_source_slice() -> Result<(), Box<dyn Error>> {
    let source = (0_u8..64).collect::<Vec<_>>();
    let sandbox = build("durable-all-short-ranges", &[&source])?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let target = BlobId::hash_bytes(&source)?;
    let size = u64::try_from(source.len())?;
    for length in 0..=size {
        for start in 0..=size
            .checked_sub(length)
            .ok_or("range length exceeds source")?
        {
            let range = ByteRange::new(ByteOffset::new(start), ByteLength::new(length))?;
            assert_slice(&snapshot, target, &source, range)?;
        }
    }
    Ok(())
}

#[test]
fn multichunk_durable_boundary_ranges_equal_the_frozen_worldline_source()
-> Result<(), Box<dyn Error>> {
    let case = find_case("large-ramp")?;
    let source = case.bytes()?;
    let sandbox = build("durable-multichunk-boundaries", &[&source])?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let size = u64::try_from(source.len())?;
    let positions = [
        0,
        1,
        65_535,
        65_536,
        65_537,
        262_143,
        262_144,
        size.checked_sub(1).ok_or("large corpus empty")?,
        size,
    ];
    for (index, start) in positions.iter().copied().enumerate() {
        for end in positions
            .get(index..)
            .ok_or("position suffix absent")?
            .iter()
            .copied()
        {
            let length = end.checked_sub(start).ok_or("inverted range")?;
            let range = ByteRange::new(ByteOffset::new(start), ByteLength::new(length))?;
            assert_slice(&snapshot, case.expected_id()?, &source, range)?;
        }
    }
    Ok(())
}

fn assert_slice(
    snapshot: &DurableSnapshot,
    target: BlobId,
    source: &[u8],
    requested: ByteRange,
) -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();
    let receipt = snapshot.read_range(target, requested, &mut output)?;
    let expected = source
        .get(usize::try_from(requested.offset().get())?..usize::try_from(requested.end().get())?)
        .ok_or("range outside source")?;
    assert_eq!(output, expected, "source-slice oracle for {requested:?}");
    assert_eq!(receipt.receipt().requested(), requested);
    assert_eq!(receipt.receipt().bytes_written(), requested.length());
    Ok(())
}

#[test]
fn generated_multichunk_durable_ranges_equal_the_reference_domain() -> Result<(), Box<dyn Error>> {
    // This is the same finite input domain as range_read_properties.rs, with
    // independent source slices as the oracle rather than another read engine.
    let mut source = Vec::new();
    for index in 0_usize..786_432 {
        let value = index
            .checked_mul(17)
            .and_then(|scaled| scaled.checked_add(29))
            .and_then(|shifted| shifted.checked_rem(251))
            .ok_or("source arithmetic overflow")?;
        source.push(u8::try_from(value)?);
    }
    let sandbox = build("durable-reference-range-domain", &[&source])?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let target = BlobId::hash_bytes(&source)?;
    let coordinate_count = u64::try_from(source.len())?
        .checked_add(1)
        .ok_or("coordinate overflow")?;
    for case in 0_u64..128 {
        let first = range_coordinate(case, 104_729, 17, coordinate_count)?;
        let second = range_coordinate(case, 130_363, 101, coordinate_count)?;
        let start = first.min(second);
        let length = first
            .max(second)
            .checked_sub(start)
            .ok_or("inverted range")?;
        let requested = ByteRange::new(ByteOffset::new(start), ByteLength::new(length))?;
        assert_slice(&snapshot, target, &source, requested)?;
    }
    Ok(())
}

fn range_coordinate(
    case: u64,
    multiplier: u64,
    increment: u64,
    count: u64,
) -> Result<u64, Box<dyn Error>> {
    case.checked_mul(multiplier)
        .and_then(|product| product.checked_add(increment))
        .and_then(|expanded| expanded.checked_rem(count))
        .ok_or_else(|| "coordinate arithmetic refused".into())
}

#[test]
fn a_durable_range_needs_no_nonoverlapping_chunk_records() -> Result<(), Box<dyn Error>> {
    let source = [
        vec![0_u8; 262_144],
        vec![1_u8; 262_144],
        vec![2_u8; 262_144],
    ]
    .concat();
    let identified = super::durable_fixture::identify(&source)?;
    let selected = identified.spans.get(1).ok_or("interior chunk absent")?;
    let sandbox = super::durable_fixture::build_selected_chunk("durable-only-overlap", &source, 1)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT).snapshot()?;
    let start = selected
        .offset()
        .get()
        .checked_add(1)
        .ok_or("offset overflow")?;
    let requested = ByteRange::new(ByteOffset::new(start), ByteLength::new(1))?;
    let mut output = Vec::new();
    let receipt = snapshot.read_layout_range(identified.record.id(), requested, &mut output)?;
    assert_eq!(
        output,
        source
            .get(
                usize::try_from(start)?
                    ..usize::try_from(start.checked_add(1).ok_or("end overflow")?)?
            )
            .ok_or("slice absent")?
    );
    assert_eq!(receipt.receipt().requested(), requested);
    assert_eq!(receipt.receipt().bytes_written(), requested.length());
    let mut whole_output = vec![0xAB];
    let failure = snapshot
        .reconstruct_layout(identified.record.id(), &mut whole_output)
        .err()
        .ok_or("missing nonoverlapping chunk reconstructed")?;
    let first = identified.spans.first().ok_or("first chunk absent")?.id();
    assert!(
        matches!(&failure, keep::DurableReadError::Reconstruction(error)
        if matches!(error.as_ref(), keep::ReconstructionError::ChunkMissing { layout, index: 0, requested }
            if *layout == identified.record.id() && *requested == first)),
        "whole reconstruction must expose missing evidence outside the successful range: {failure:?}"
    );
    assert_eq!(whole_output, [0xAB]);
    Ok(())
}
