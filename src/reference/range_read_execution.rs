//! Allocation-free authenticated execution of one admitted range plan.

use std::io::Write;

use crate::{
    AdmittedLayout, ByteLength, ByteRange, ChunkId, LayoutEntry, LayoutId, RangePlan,
    ReferenceStore,
};

use super::chunk_verification::verified_chunk;
use super::output_write::write_all;
use super::range_read_error_mapping::{range_chunk_error, range_output_error};
use super::{RangeReadError, RangeReadReceipt};

pub(super) fn read_admitted<W>(
    store: &ReferenceStore,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    requested: ByteRange,
    output: &mut W,
) -> Result<RangeReadReceipt, RangeReadError>
where
    W: Write + ?Sized,
{
    let plan = layout
        .plan_range(requested)
        .map_err(RangeReadError::RangePlan)?;
    verify_selected(store, layout_id, layout, plan)?;
    let written = emit_selected(store, layout_id, layout, plan, output)?;
    if written != requested.length() {
        return Err(RangeReadError::WrittenLengthMismatch {
            layout: layout_id,
            expected: requested.length(),
            observed: written,
        });
    }
    Ok(RangeReadReceipt::new(
        layout.target(),
        layout_id,
        requested,
        written,
    ))
}

fn verify_selected(
    store: &ReferenceStore,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    plan: RangePlan,
) -> Result<(), RangeReadError> {
    let (first, entries) = selected_entries(layout, plan)?;
    for (index, entry) in (first..plan.end_entry()).zip(entries.iter().copied()) {
        let _bytes = verified_chunk(store, layout_id, index, entry).map_err(range_chunk_error)?;
    }
    Ok(())
}

fn emit_selected<W>(
    store: &ReferenceStore,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    plan: RangePlan,
    output: &mut W,
) -> Result<ByteLength, RangeReadError>
where
    W: Write + ?Sized,
{
    let (first, entries) = selected_entries(layout, plan)?;
    let mut written = 0_u64;
    for (index, entry) in (first..plan.end_entry()).zip(entries.iter().copied()) {
        let bytes = verified_chunk(store, layout_id, index, entry).map_err(range_chunk_error)?;
        let selected = selected_chunk_slice(layout_id, index, entry, plan.requested(), bytes)?;
        write_all(output, selected, &mut written)
            .map_err(|error| range_output_error(layout_id, error))?;
    }
    Ok(ByteLength::new(written))
}

fn selected_entries(
    layout: &AdmittedLayout,
    plan: RangePlan,
) -> Result<(usize, &[LayoutEntry]), RangeReadError> {
    let first = plan.first_entry().map_or(0, std::convert::identity);
    let end = plan.end_entry();
    let entries =
        layout
            .entries()
            .get(first..end)
            .ok_or_else(|| RangeReadError::PlanEntriesUnavailable {
                first,
                end,
                available: layout.entries().len(),
            })?;
    Ok((first, entries))
}

fn selected_chunk_slice(
    layout: LayoutId,
    index: usize,
    entry: LayoutEntry,
    requested: ByteRange,
    bytes: &[u8],
) -> Result<&[u8], RangeReadError> {
    let chunk = entry.chunk_id();
    let entry_start = entry.offset().get();
    let entry_end = entry_start
        .checked_add(u64::from(chunk.length().get()))
        .ok_or_else(|| slice_unavailable(layout, index, requested, chunk))?;
    let absolute_start = requested.offset().get().max(entry_start);
    let absolute_end = requested.end().get().min(entry_end);
    let relative_start = absolute_start
        .checked_sub(entry_start)
        .ok_or_else(|| slice_unavailable(layout, index, requested, chunk))?;
    let relative_end = absolute_end
        .checked_sub(entry_start)
        .ok_or_else(|| slice_unavailable(layout, index, requested, chunk))?;
    let start = usize::try_from(relative_start)
        .map_err(|_source| slice_unavailable(layout, index, requested, chunk))?;
    let end = usize::try_from(relative_end)
        .map_err(|_source| slice_unavailable(layout, index, requested, chunk))?;
    bytes
        .get(start..end)
        .ok_or_else(|| slice_unavailable(layout, index, requested, chunk))
}

const fn slice_unavailable(
    layout: LayoutId,
    index: usize,
    requested: ByteRange,
    chunk: ChunkId,
) -> RangeReadError {
    RangeReadError::ChunkSliceUnavailable {
        layout,
        index,
        requested,
        chunk,
    }
}
