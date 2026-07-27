//! Public bounded flat-layout decoding and admission laws.

pub mod support;

use std::error::Error;

use keep::{AdmittedLayout, LayoutDecodePolicy, LayoutEntryLimit, LayoutId};
use support::{field, layout_record_bytes};

const LAYOUTS: &str = include_str!("../conformance/layout/v1/layouts.tsv");

#[test]
fn every_golden_record_decodes_admits_and_reencodes_canonically() -> Result<(), Box<dyn Error>> {
    for row in LAYOUTS.lines().skip(2) {
        let case = field(row, 0)?;
        let bytes = layout_record_bytes(case)?;
        let expected_id = field(row, 10)?.parse::<LayoutId>()?;
        let policy =
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM).with_expected_id(expected_id);

        let layout = AdmittedLayout::decode_record(&bytes, policy)?;
        let reencoded = layout.encode_record()?;

        assert_eq!(reencoded.bytes(), bytes, "{case}");
        assert_eq!(reencoded.id(), expected_id, "{case}");
    }
    Ok(())
}
