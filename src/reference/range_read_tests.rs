//! Private-state lookup and corruption laws for exact range reads.

use std::error::Error;
use std::io::{self, Cursor};

use crate::{
    ByteLength, ByteOffset, ByteRange, ChunkId, LayoutEntry, LayoutEntryLimit, LayoutId,
    RangeReadError, ReferenceStore, ReferenceStoreCapacity,
};

#[test]
fn an_interior_range_loads_only_its_single_overlapping_chunk() -> Result<(), Box<dyn Error>> {
    let (mut store, source, layout_id, index, entry) = range_fixture()?;
    let selected = entry.chunk_id();
    assert!(store.chunks.len() > 1);
    store
        .chunks
        .retain(|identity, _bytes| *identity == selected);
    assert_eq!(store.chunks.len(), 1);
    store.observed_chunk_reads.borrow_mut().clear();
    let requested = one_byte_inside(entry)?;
    let mut output = Vec::new();

    let receipt = store.read_layout_range(layout_id, requested, &mut output)?;

    assert_eq!(output, source_slice(&source, requested)?);
    assert_eq!(receipt.bytes_written(), ByteLength::new(1));
    assert_eq!(
        store.observed_chunk_reads.borrow().as_slice(),
        [selected, selected]
    );
    assert!(index > 0);
    Ok(())
}

#[test]
fn a_missing_selected_chunk_refuses_before_output() -> Result<(), Box<dyn Error>> {
    let (mut store, _, layout_id, index, entry) = range_fixture()?;
    let selected = entry.chunk_id();
    assert!(store.chunks.remove(&selected).is_some());
    let requested = one_byte_inside(entry)?;
    let mut output = Vec::new();

    let error = store
        .read_layout_range(layout_id, requested, &mut output)
        .err()
        .ok_or_else(|| io::Error::other("missing selected chunk unexpectedly read"))?;

    assert!(matches!(
        error,
        RangeReadError::ChunkMissing {
            layout,
            index: observed_index,
            requested: observed_chunk
        } if layout == layout_id && observed_index == index && observed_chunk == selected
    ));
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn a_corrupt_selected_chunk_refuses_before_output() -> Result<(), Box<dyn Error>> {
    let (mut store, _, layout_id, index, entry) = range_fixture()?;
    let selected = entry.chunk_id();
    let stored = store
        .chunks
        .get(&selected)
        .ok_or_else(|| io::Error::other("selected chunk is absent"))?;
    let mut corrupted = stored.to_vec();
    let first = corrupted
        .first_mut()
        .ok_or_else(|| io::Error::other("selected chunk is empty"))?;
    *first ^= 0xff;
    let observed = ChunkId::hash_bytes(&corrupted)?;
    store.chunks.insert(selected, corrupted.into_boxed_slice());
    let requested = one_byte_inside(entry)?;
    let mut output = Vec::new();

    let error = store
        .read_layout_range(layout_id, requested, &mut output)
        .err()
        .ok_or_else(|| io::Error::other("corrupt selected chunk unexpectedly read"))?;

    assert!(matches!(
        error,
        RangeReadError::ChunkIdentityMismatch {
            layout,
            index: observed_index,
            expected,
            observed: actual
        } if layout == layout_id
            && observed_index == index
            && expected == selected
            && actual == observed
    ));
    assert!(output.is_empty());
    Ok(())
}

type RangeFixture = (ReferenceStore, Vec<u8>, LayoutId, usize, LayoutEntry);

fn range_fixture() -> Result<RangeFixture, Box<dyn Error>> {
    let maximum = usize::try_from(crate::FastCdc::MAXIMUM_CHUNK_LENGTH.get())?;
    let mut source = Vec::new();
    source.try_reserve_exact(
        maximum
            .checked_mul(3)
            .ok_or_else(|| io::Error::other("fixture length overflow"))?,
    )?;
    source.resize(maximum, 0);
    source.resize(
        maximum
            .checked_mul(2)
            .ok_or_else(|| io::Error::other("fixture length overflow"))?,
        1,
    );
    source.resize(
        maximum
            .checked_mul(3)
            .ok_or_else(|| io::Error::other("fixture length overflow"))?,
        2,
    );
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(2_000_000));
    let mut reader = Cursor::new(&source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let layout = store
        .layout(published.layout_id())
        .ok_or_else(|| io::Error::other("published layout is absent"))?;
    let entry_count = layout.entries().len();
    let (index, entry) = layout
        .entries()
        .iter()
        .copied()
        .enumerate()
        .find(|(index, _entry)| {
            *index > 0 && index.checked_add(1).is_some_and(|next| next < entry_count)
        })
        .ok_or_else(|| io::Error::other("fixture has no interior chunk"))?;
    Ok((store, source, published.layout_id(), index, entry))
}

fn one_byte_inside(entry: LayoutEntry) -> Result<ByteRange, Box<dyn Error>> {
    let offset = entry
        .offset()
        .get()
        .checked_add(1)
        .ok_or_else(|| io::Error::other("fixture range offset overflow"))?;
    Ok(ByteRange::new(ByteOffset::new(offset), ByteLength::new(1))?)
}

fn source_slice(source: &[u8], requested: ByteRange) -> Result<&[u8], Box<dyn Error>> {
    let start = usize::try_from(requested.offset().get())?;
    let end = usize::try_from(requested.end().get())?;
    source
        .get(start..end)
        .ok_or_else(|| io::Error::other("fixture range is unavailable").into())
}
