//! Deterministic bounded duplicate-identity detection for segment admission.

use super::segment_record_cursor::SegmentRecordCursor;
use super::{SegmentReadError, SegmentReadPolicy, SegmentRecordHeader, SegmentRecordIdentity};

#[derive(Clone, Copy)]
struct IdentityCoordinate {
    identity: SegmentRecordIdentity,
    record_index: u32,
    offset: u64,
}

pub(super) fn validate(
    records: &[u8],
    record_count: u32,
    policy: SegmentReadPolicy,
) -> Result<(), SegmentReadError> {
    let declared_capacity = usize::try_from(record_count).map_err(|_source| {
        SegmentReadError::RecordCountHostWidth {
            observed: record_count,
        }
    })?;
    let physical_capacity = records
        .chunks_exact(SegmentRecordHeader::ENCODED_LENGTH)
        .len();
    let capacity = declared_capacity.min(physical_capacity);
    let mut coordinates = Vec::new();
    coordinates.try_reserve_exact(capacity).map_err(|source| {
        SegmentReadError::IdentityIndexAllocation {
            record_count,
            source,
        }
    })?;
    let mut cursor = SegmentRecordCursor::new(records, record_count, policy);
    while let Some(located) = cursor.next_record()? {
        coordinates.push(IdentityCoordinate {
            identity: located.record.identity(),
            record_index: located.record_index,
            offset: located.offset,
        });
    }
    cursor.finish()?;
    coordinates.sort_unstable_by_key(|coordinate| (coordinate.identity, coordinate.record_index));
    refuse_duplicate(&coordinates)
}

fn refuse_duplicate(coordinates: &[IdentityCoordinate]) -> Result<(), SegmentReadError> {
    for adjacent in coordinates.windows(2) {
        let [first, duplicate] = adjacent else {
            continue;
        };
        if first.identity == duplicate.identity {
            return Err(SegmentReadError::DuplicateRecordIdentity {
                identity: first.identity,
                first_index: first.record_index,
                duplicate_index: duplicate.record_index,
                first_offset: first.offset,
                duplicate_offset: duplicate.offset,
            });
        }
    }
    Ok(())
}
