//! Public recovery catalog- and next-head-stage classification laws.

#[path = "recovery_publication_stage_classification/catalog_laws.rs"]
mod catalog_laws;
#[path = "recovery_publication_stage_classification/next_head_laws.rs"]
mod next_head_laws;
mod support;

use std::error::Error;

use support::decode_hex;

const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_HEADER_LENGTH: usize = 128;
const HEAD_LENGTH: usize = 128;

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("publication fixture must end in one LF")?,
    )
    .map_err(Into::into)
}
