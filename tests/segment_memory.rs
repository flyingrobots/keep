//! Isolated heap-allocation evidence for complete segment admission.

#[path = "segment/format_oracle.rs"]
pub mod format_oracle;
mod support;

use std::error::Error;

use allocation_counter::{AllocationInfo, measure};
use keep::{AdmittedSegment, SegmentReadError, SegmentReadPolicy};
use support::decode_hex;

use format_oracle::seal_segment;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");

#[test]
fn duplicate_index_allocation_scales_exactly_with_admitted_record_count()
-> Result<(), Box<dyn Error>> {
    let empty = segment_bytes(EMPTY_SEGMENT_HEX)?;
    let one = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let prefix = one
        .get(..209)
        .ok_or("segment fixture lacks its pre-seal bytes")?;
    let record = prefix
        .get(64..)
        .ok_or("segment fixture lacks its complete record")?;
    let mut duplicate_prefix = prefix.to_vec();
    duplicate_prefix.extend_from_slice(record);
    let duplicate = seal_segment(&duplicate_prefix, 2)?;

    let empty_allocations = admission_allocations(&empty)?;
    let one_allocations = admission_allocations(&one)?;
    let duplicate_allocations = refusal_allocations(&duplicate)?;
    let doubled_bytes = one_allocations
        .bytes_total
        .checked_mul(2)
        .ok_or("test allocation witness overflow")?;

    assert_eq!(empty_allocations, AllocationInfo::default());
    assert_eq!(one_allocations.count_total, 1);
    assert_eq!(one_allocations.count_max, 1);
    assert_eq!(one_allocations.count_current, 0);
    assert_eq!(duplicate_allocations.count_total, 1);
    assert_eq!(duplicate_allocations.count_max, 1);
    assert_eq!(duplicate_allocations.count_current, 0);
    assert_eq!(duplicate_allocations.bytes_total, doubled_bytes);
    Ok(())
}

fn admission_allocations(encoded: &[u8]) -> Result<AllocationInfo, Box<dyn Error>> {
    let mut result = None;
    let allocations = measure(|| {
        result = Some(AdmittedSegment::decode(encoded, SegmentReadPolicy::MAXIMUM));
    });
    let _admitted = result.ok_or("segment allocation measurement did not run")??;
    Ok(allocations)
}

fn refusal_allocations(encoded: &[u8]) -> Result<AllocationInfo, Box<dyn Error>> {
    let mut result = None;
    let allocations = measure(|| {
        result = Some(AdmittedSegment::decode(encoded, SegmentReadPolicy::MAXIMUM));
    });
    let error = match result.ok_or("segment refusal allocation measurement did not run")? {
        Ok(_admitted) => return Err("duplicate segment was admitted".into()),
        Err(error) => error,
    };
    if !matches!(error, SegmentReadError::DuplicateRecordIdentity { .. }) {
        return Err(format!("unexpected duplicate refusal: {error}").into());
    }
    Ok(allocations)
}

fn segment_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )
    .map_err(Into::into)
}
