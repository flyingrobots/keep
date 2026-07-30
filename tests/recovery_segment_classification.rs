//! Public recovery segment-stage classification laws.

#[path = "recovery_segment_classification/refusal_laws.rs"]
mod refusal_laws;
#[path = "recovery_segment_classification/state_laws.rs"]
mod state_laws;
mod support;

use std::error::Error;

use keep::{LayoutEntryLimit, SegmentReadPolicy, SegmentRecordLimit};
use support::decode_hex;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const HEADER_LENGTH: usize = 64;
const RECORD_END: usize = 209;
const SEAL_LENGTH: usize = 128;

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn segment_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )
    .map_err(Into::into)
}
