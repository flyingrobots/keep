//! Public exact byte-range read and refusal laws.

pub mod support;

use std::error::Error;
use std::io::{self, Cursor};

use keep::{
    BlobId, ByteLength, ByteOffset, ByteRange, LayoutDecodeError, LayoutDecodePolicy,
    LayoutEntryLimit, RangePlanError, RangeReadError, ReferenceStore, ReferenceStoreCapacity,
};
use support::PartitionWriter;

#[test]
fn public_range_reads_return_exact_boundary_slices() -> Result<(), Box<dyn Error>> {
    let (store, source, target, layout_id, first_end) = published_fixture()?;
    let total = u64::try_from(source.len())?;
    let final_offset = total
        .checked_sub(1)
        .ok_or_else(|| io::Error::other("fixture source is empty"))?;
    let cross_offset = first_end
        .checked_sub(1)
        .ok_or_else(|| io::Error::other("fixture first chunk is empty"))?;
    let cases = [
        ByteRange::new(ByteOffset::ZERO, ByteLength::new(1))?,
        ByteRange::new(ByteOffset::new(final_offset), ByteLength::new(1))?,
        ByteRange::new(ByteOffset::new(cross_offset), ByteLength::new(2))?,
        ByteRange::new(ByteOffset::ZERO, ByteLength::new(total))?,
    ];

    for requested in cases {
        let mut output = Vec::new();
        let receipt = store.read_range(target, requested, &mut output)?;

        assert_eq!(output, expected_slice(&source, requested)?);
        assert_eq!(receipt.target(), target);
        assert_eq!(receipt.layout_id(), layout_id);
        assert_eq!(receipt.requested(), requested);
        assert_eq!(receipt.bytes_written(), requested.length());
    }
    Ok(())
}

#[test]
fn zero_length_reads_through_eof_write_nothing() -> Result<(), Box<dyn Error>> {
    let (store, source, target, _, _) = published_fixture()?;
    let total = u64::try_from(source.len())?;
    for offset in [0, 1, total] {
        let requested = ByteRange::new(ByteOffset::new(offset), ByteLength::ZERO)?;
        let mut output = Vec::new();
        let receipt = store.read_range(target, requested, &mut output)?;

        assert!(output.is_empty());
        assert_eq!(receipt.requested(), requested);
        assert!(receipt.bytes_written().is_empty());
    }

    let mut empty_store = ReferenceStore::new(ReferenceStoreCapacity::new(0));
    let mut empty_source = Cursor::new([]);
    let empty = empty_store
        .stage(&mut empty_source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut empty_store)?;
    let requested = ByteRange::new(ByteOffset::ZERO, ByteLength::ZERO)?;
    let mut output = Vec::new();
    let receipt = empty_store.read_range(empty.target(), requested, &mut output)?;
    assert!(output.is_empty());
    assert!(receipt.bytes_written().is_empty());
    Ok(())
}

#[test]
fn range_reads_complete_short_and_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let (store, source, _, layout_id, first_end) = published_fixture()?;
    let requested = ByteRange::new(
        ByteOffset::new(
            first_end
                .checked_sub(3)
                .ok_or_else(|| io::Error::other("fixture first chunk is too short"))?,
        ),
        ByteLength::new(9),
    )?;
    let widths = [1, 2, 5];
    let mut output = PartitionWriter::new(&widths)?;

    let receipt = store.read_layout_range(layout_id, requested, &mut output)?;

    assert_eq!(output.bytes(), expected_slice(&source, requested)?);
    assert_eq!(receipt.bytes_written(), ByteLength::new(9));
    Ok(())
}

#[test]
fn absent_and_out_of_bounds_ranges_refuse_before_output() -> Result<(), Box<dyn Error>> {
    let (store, source, target, _, _) = published_fixture()?;
    let total = u64::try_from(source.len())?;
    let absent = BlobId::hash_bytes(b"absent range target")?;
    let one_byte = ByteRange::new(ByteOffset::ZERO, ByteLength::new(1))?;
    let mut output = Vec::new();

    assert!(matches!(
        store.read_range(absent, one_byte, &mut output),
        Err(RangeReadError::BlobMissing { requested }) if requested == absent
    ));
    assert!(output.is_empty());

    let past_end = ByteRange::new(ByteOffset::new(total), ByteLength::new(1))?;
    assert!(matches!(
        store.read_range(target, past_end, &mut output),
        Err(RangeReadError::RangePlan(RangePlanError::OutOfBounds {
            requested,
            target_length
        })) if requested == past_end && target_length == target.logical_length()
    ));
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn malformed_layout_records_refuse_before_range_output() -> Result<(), Box<dyn Error>> {
    let source = patterned_source()?;
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(2_000_000));
    let mut reader = Cursor::new(&source);
    let staged = store.stage(&mut reader, LayoutEntryLimit::MAXIMUM)?;
    let mut encoded = staged.layout().encode_record()?.bytes().to_vec();
    let checksum_tail = encoded
        .last_mut()
        .ok_or_else(|| io::Error::other("fixture layout record is empty"))?;
    *checksum_tail ^= 1;
    let requested = ByteRange::new(ByteOffset::ZERO, ByteLength::new(1))?;
    let policy = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);
    let mut output = Vec::new();

    let error = store
        .read_record_range(&encoded, policy, requested, &mut output)
        .err()
        .ok_or_else(|| io::Error::other("malformed layout unexpectedly produced a range"))?;

    assert!(matches!(
        error,
        RangeReadError::LayoutDecode(LayoutDecodeError::ChecksumMismatch { .. })
    ));
    assert!(output.is_empty());
    Ok(())
}

type PublishedFixture = (ReferenceStore, Vec<u8>, BlobId, keep::LayoutId, u64);

fn published_fixture() -> Result<PublishedFixture, Box<dyn Error>> {
    let source = patterned_source()?;
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(2_000_000));
    let mut reader = Cursor::new(&source);
    let staged = store.stage(&mut reader, LayoutEntryLimit::MAXIMUM)?;
    let first_end = u64::from(
        staged
            .layout()
            .entries()
            .first()
            .ok_or_else(|| io::Error::other("fixture layout is empty"))?
            .chunk_id()
            .length()
            .get(),
    );
    let published = staged.commit(&mut store)?;
    Ok((
        store,
        source,
        published.target(),
        published.layout_id(),
        first_end,
    ))
}

fn patterned_source() -> Result<Vec<u8>, Box<dyn Error>> {
    let maximum = usize::try_from(keep::FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    let length = maximum
        .checked_mul(3)
        .ok_or_else(|| io::Error::other("fixture length overflow"))?;
    let mut source = Vec::new();
    source.try_reserve_exact(length)?;
    for index in 0..length {
        let value = index
            .checked_rem(251)
            .ok_or_else(|| io::Error::other("fixture divisor is zero"))?;
        source.push(u8::try_from(value)?);
    }
    Ok(source)
}

fn expected_slice(source: &[u8], requested: ByteRange) -> Result<&[u8], Box<dyn Error>> {
    let start = usize::try_from(requested.offset().get())?;
    let end = usize::try_from(requested.end().get())?;
    source
        .get(start..end)
        .ok_or_else(|| io::Error::other("requested fixture range is out of bounds").into())
}
