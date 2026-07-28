//! Isolated heap-allocation evidence for chunk segment records.

mod support;

use std::error::Error;

use allocation_counter::{AllocationInfo, measure};
use keep::{AdmittedSegmentRecord, ChecksummedSegmentRecord, LayoutEntryLimit};
use support::decode_hex;

const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");

#[test]
fn chunk_record_decoding_admission_and_preparation_allocate_nothing() -> Result<(), Box<dyn Error>>
{
    let segment = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let record = segment
        .get(64..209)
        .ok_or("segment fixture lacks its chunk record")?;

    let mut decoded = None;
    let decode_allocations = measure(|| {
        decoded = Some(ChecksummedSegmentRecord::decode(record));
    });
    let checksummed = decoded.ok_or("decode allocation measurement did not run")??;

    let mut admitted = None;
    let admission_allocations = measure(|| {
        admitted = Some(checksummed.admit(LayoutEntryLimit::MAXIMUM));
    });
    let admitted = admitted.ok_or("admission allocation measurement did not run")??;

    let mut prepared = None;
    let preparation_allocations = measure(|| {
        prepared = Some(AdmittedSegmentRecord::for_chunk(admitted.payload()));
    });
    let _prepared = prepared.ok_or("preparation allocation measurement did not run")??;

    assert_eq!(decode_allocations, AllocationInfo::default());
    assert_eq!(admission_allocations, AllocationInfo::default());
    assert_eq!(preparation_allocations, AllocationInfo::default());
    Ok(())
}
