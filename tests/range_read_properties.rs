//! Generated public laws for exact byte-range reads.

use std::error::Error;
use std::io::{self, Cursor};

use keep::{
    ByteLength, ByteOffset, ByteRange, LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity,
};

#[test]
fn every_short_valid_range_equals_the_reference_slice() -> Result<(), Box<dyn Error>> {
    let source: Vec<u8> = (0_u8..64).collect();
    let (store, target) = publish(&source)?;
    let length = u64::try_from(source.len())?;

    for start in 0..=length {
        for end in start..=length {
            assert_range_equals_slice(&store, target, &source, start, end)?;
        }
    }
    Ok(())
}

#[test]
fn every_generated_multichunk_range_equals_the_reference_slice() -> Result<(), Box<dyn Error>> {
    let source = multichunk_source()?;
    let (store, target) = publish(&source)?;
    let coordinate_count = u64::try_from(source.len())?
        .checked_add(1)
        .ok_or_else(|| io::Error::other("fixture coordinate count overflow"))?;

    for case in 0_u64..128 {
        let first = generated_coordinate(case, 104_729, 17, coordinate_count)?;
        let second = generated_coordinate(case, 130_363, 101, coordinate_count)?;
        let start = first.min(second);
        let end = first.max(second);
        assert_range_equals_slice(&store, target, &source, start, end)?;
    }
    Ok(())
}

fn publish(source: &[u8]) -> Result<(ReferenceStore, keep::BlobId), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(2_000_000));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    Ok((store, published.target()))
}

fn assert_range_equals_slice(
    store: &ReferenceStore,
    target: keep::BlobId,
    source: &[u8],
    start: u64,
    end: u64,
) -> Result<(), Box<dyn Error>> {
    let length = end
        .checked_sub(start)
        .ok_or_else(|| io::Error::other("generated range is inverted"))?;
    let requested = ByteRange::new(ByteOffset::new(start), ByteLength::new(length))?;
    let mut output = Vec::new();

    let receipt = store.read_range(target, requested, &mut output)?;

    let start_index = usize::try_from(start)?;
    let end_index = usize::try_from(end)?;
    let expected = source
        .get(start_index..end_index)
        .ok_or_else(|| io::Error::other("generated range exceeds its fixture"))?;
    assert_eq!(output, expected);
    assert_eq!(receipt.requested(), requested);
    assert_eq!(receipt.bytes_written(), requested.length());
    Ok(())
}

fn generated_coordinate(
    case: u64,
    multiplier: u64,
    increment: u64,
    coordinate_count: u64,
) -> Result<u64, Box<dyn Error>> {
    let product = u128::from(case)
        .checked_mul(u128::from(multiplier))
        .ok_or_else(|| io::Error::other("fixture product overflow"))?;
    let expanded = product
        .checked_add(u128::from(increment))
        .ok_or_else(|| io::Error::other("fixture coordinate overflow"))?;
    let reduced = expanded
        .checked_rem(u128::from(coordinate_count))
        .ok_or_else(|| io::Error::other("fixture coordinate modulus is zero"))?;
    Ok(u64::try_from(reduced)?)
}

fn multichunk_source() -> Result<Vec<u8>, Box<dyn Error>> {
    let maximum = usize::try_from(keep::FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    let length = maximum
        .checked_mul(3)
        .ok_or_else(|| io::Error::other("fixture length overflow"))?;
    let mut source = Vec::new();
    source.try_reserve_exact(length)?;
    for index in 0..length {
        let value = index
            .checked_mul(17)
            .and_then(|scaled| scaled.checked_add(29))
            .and_then(|shifted| shifted.checked_rem(251))
            .ok_or_else(|| io::Error::other("fixture byte arithmetic failed"))?;
        source.push(u8::try_from(value)?);
    }
    Ok(source)
}
