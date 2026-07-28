//! Isolated heap-allocation evidence for segment-seal admission.

mod support;

use std::error::Error;

use allocation_counter::{AllocationInfo, measure};
use keep::SegmentSeal;
use support::decode_hex;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");

#[test]
fn segment_seal_decoding_and_verification_allocate_nothing() -> Result<(), Box<dyn Error>> {
    let segment = decode_hex(
        EMPTY_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let prefix = segment
        .get(..64)
        .ok_or("segment fixture lacks its pre-seal bytes")?;
    let seal = segment.get(64..).ok_or("segment fixture lacks its seal")?;

    let mut decoded = None;
    let allocations = measure(|| {
        decoded = Some(SegmentSeal::decode(prefix, seal));
    });
    let _admitted = decoded.ok_or("seal allocation measurement did not run")??;

    assert_eq!(allocations, AllocationInfo::default());
    Ok(())
}
