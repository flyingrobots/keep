//! Public exact byte-range coordinate and planning laws.

pub(crate) mod support;

use std::error::Error;
use std::io;

use keep::{
    AdmittedLayout, BlobId, ByteLength, ByteOffset, ByteRange, ByteRangeError, FastCdc,
    LayoutEntryLimit, RangePlanError, RegisteredStorageProfile,
};
use support::detect_spans;

#[test]
fn range_construction_refuses_only_end_overflow() {
    let range = ByteRange::new(ByteOffset::new(7), ByteLength::new(5));
    assert!(matches!(
        range,
        Ok(admitted)
            if admitted.offset() == ByteOffset::new(7)
                && admitted.length() == ByteLength::new(5)
                && admitted.end() == ByteOffset::new(12)
                && !admitted.is_empty()
    ));

    let empty = ByteRange::new(ByteOffset::new(u64::MAX), ByteLength::ZERO);
    assert!(matches!(empty, Ok(admitted) if admitted.is_empty()));

    let overflow = ByteRange::new(ByteOffset::new(u64::MAX), ByteLength::new(1));
    assert!(matches!(
        overflow,
        Err(ByteRangeError::EndOverflow { offset, length })
            if offset == ByteOffset::new(u64::MAX) && length == ByteLength::new(1)
    ));
}

#[test]
fn empty_ranges_select_no_chunks_at_every_lawful_coordinate() -> Result<(), Box<dyn Error>> {
    let layout = multi_chunk_layout()?;
    let blob_length = layout.target().logical_length().get();
    let final_byte = blob_length
        .checked_sub(1)
        .ok_or_else(|| io::Error::other("fixture blob is empty"))?;
    for offset in [0, 1, final_byte, blob_length] {
        let requested = ByteRange::new(ByteOffset::new(offset), ByteLength::ZERO)?;
        let plan = layout.plan_range(requested)?;

        assert_eq!(plan.requested(), requested);
        assert_eq!(plan.first_entry(), None);
        assert_eq!(plan.entry_count(), 0);
    }
    Ok(())
}

#[test]
fn planning_selects_the_minimal_ordered_overlap() -> Result<(), Box<dyn Error>> {
    let layout = multi_chunk_layout()?;
    let entries = layout.entries();
    let first_end = u64::from(
        entries
            .first()
            .ok_or_else(|| io::Error::other("fixture layout is empty"))?
            .chunk_id()
            .length()
            .get(),
    );
    let total = layout.target().logical_length().get();
    let last_index = entries
        .len()
        .checked_sub(1)
        .ok_or_else(|| io::Error::other("fixture layout is empty"))?;

    let first = layout.plan_range(ByteRange::new(ByteOffset::ZERO, ByteLength::new(1))?)?;
    assert_eq!(first.first_entry(), Some(0));
    assert_eq!(first.entry_count(), 1);

    let cross = layout.plan_range(ByteRange::new(
        ByteOffset::new(
            first_end
                .checked_sub(1)
                .ok_or_else(|| io::Error::other("first chunk is empty"))?,
        ),
        ByteLength::new(2),
    )?)?;
    assert_eq!(cross.first_entry(), Some(0));
    assert_eq!(cross.entry_count(), 2);

    let last = layout.plan_range(ByteRange::new(
        ByteOffset::new(
            total
                .checked_sub(1)
                .ok_or_else(|| io::Error::other("fixture blob is empty"))?,
        ),
        ByteLength::new(1),
    )?)?;
    assert_eq!(last.first_entry(), Some(last_index));
    assert_eq!(last.entry_count(), 1);

    let full = layout.plan_range(ByteRange::new(ByteOffset::ZERO, ByteLength::new(total))?)?;
    assert_eq!(full.first_entry(), Some(0));
    assert_eq!(full.entry_count(), entries.len());
    Ok(())
}

#[test]
fn planning_refuses_every_coordinate_past_the_target() -> Result<(), Box<dyn Error>> {
    let layout = multi_chunk_layout()?;
    let blob_length = layout.target().logical_length();
    let past_end = blob_length
        .get()
        .checked_add(1)
        .ok_or_else(|| io::Error::other("fixture length cannot advance"))?;
    let starts_past_end = ByteRange::new(ByteOffset::new(past_end), ByteLength::ZERO)?;
    let ends_past_end = ByteRange::new(ByteOffset::new(blob_length.get()), ByteLength::new(1))?;

    assert!(matches!(
        layout.plan_range(starts_past_end),
        Err(RangePlanError::OutOfBounds {
            requested,
            target_length
        }) if requested == starts_past_end && target_length == blob_length
    ));
    assert!(matches!(
        layout.plan_range(ends_past_end),
        Err(RangePlanError::OutOfBounds {
            requested,
            target_length
        }) if requested == ends_past_end && target_length == blob_length
    ));
    Ok(())
}

fn multi_chunk_layout() -> Result<AdmittedLayout, Box<dyn Error>> {
    let maximum = usize::try_from(FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    let source_length = maximum
        .checked_mul(3)
        .ok_or_else(|| io::Error::other("fixture length overflow"))?;
    let source = vec![0_u8; source_length];
    let target = BlobId::hash_bytes(&source)?;
    Ok(AdmittedLayout::from_spans(
        target,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        detect_spans(&source)?,
        LayoutEntryLimit::MAXIMUM,
    )?)
}
