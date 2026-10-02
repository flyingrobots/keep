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
