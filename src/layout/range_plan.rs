//! Minimal ordered overlap planning for one admitted layout.

use crate::{BlobLength, ByteRange, ChunkLength, ChunkOffset};

use super::{AdmittedLayout, RangePlanError};

/// The minimal ordered layout-entry interval overlapping one requested range.
///
/// Empty ranges select no entries. This plan identifies logical entries only;
/// it contains no physical location, loaded bytes, or verification claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "a range plan records the exact layout entries that may be loaded"]
pub struct RangePlan {
    requested: ByteRange,
    first_entry: Option<usize>,
    entry_count: usize,
    end_entry: usize,
}

impl RangePlan {
    /// Returns the requested logical byte range.
    #[must_use]
    pub const fn requested(self) -> ByteRange {
        self.requested
    }

    /// Returns the first overlapping layout-entry index, if any.
    #[must_use]
    pub const fn first_entry(self) -> Option<usize> {
        self.first_entry
    }

    /// Returns the number of overlapping layout entries.
    #[must_use]
    pub const fn entry_count(self) -> usize {
        self.entry_count
    }

    pub(crate) const fn end_entry(self) -> usize {
        self.end_entry
    }
}

impl AdmittedLayout {
    /// Plans the minimal ordered entries overlapping `requested`.
    ///
    /// Zero-length ranges at any coordinate through end-of-blob are lawful and
    /// select no entries. This operation performs no I/O and does not allocate.
    ///
    /// # Errors
    ///
    /// Returns [`RangePlanError`] when the requested range exceeds the target
    /// blob or checked layout-entry arithmetic cannot produce a plan.
    pub fn plan_range(&self, requested: ByteRange) -> Result<RangePlan, RangePlanError> {
        ensure_in_bounds(requested, self.target().logical_length())?;
        if requested.is_empty() {
            return Ok(RangePlan {
                requested,
                first_entry: None,
                entry_count: 0,
                end_entry: 0,
            });
        }
        plan_nonempty(self, requested)
    }
}

const fn ensure_in_bounds(
    requested: ByteRange,
    target_length: BlobLength,
) -> Result<(), RangePlanError> {
    if requested.offset().get() > target_length.get() || requested.end().get() > target_length.get()
    {
        return Err(RangePlanError::OutOfBounds {
            requested,
            target_length,
        });
    }
    Ok(())
}

fn plan_nonempty(
    layout: &AdmittedLayout,
    requested: ByteRange,
) -> Result<RangePlan, RangePlanError> {
    let mut first_entry = None;
    let mut end_entry = 0;
    for (index, entry) in layout.entries().iter().copied().enumerate() {
        let entry_end = checked_entry_end(index, entry.offset(), entry.chunk_id().length())?;
        if entry_end <= requested.offset().get() {
            continue;
        }
        if entry.offset().get() >= requested.end().get() {
            break;
        }
        if first_entry.is_none() {
            first_entry = Some(index);
        }
        end_entry = index
            .checked_add(1)
            .ok_or(RangePlanError::EntryIndexOverflow { index })?;
    }
    let first = first_entry.ok_or_else(|| RangePlanError::NoOverlap {
        requested,
        target_length: layout.target().logical_length(),
    })?;
    let entry_count =
        end_entry
            .checked_sub(first)
            .ok_or(RangePlanError::EntryIntervalInverted {
                first,
                end: end_entry,
            })?;
    Ok(RangePlan {
        requested,
        first_entry: Some(first),
        entry_count,
        end_entry,
    })
}

fn checked_entry_end(
    index: usize,
    offset: ChunkOffset,
    length: ChunkLength,
) -> Result<u64, RangePlanError> {
    offset
        .get()
        .checked_add(u64::from(length.get()))
        .ok_or(RangePlanError::EntryEndOverflow {
            index,
            offset,
            length,
        })
}
