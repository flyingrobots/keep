//! Public bounded flat-layout decoding and admission laws.

pub mod support;

use std::error::Error;

use keep::{AdmittedLayout, LayoutDecodePolicy, LayoutEntryLimit, LayoutId};
use support::{decode_hex, invalid_corpus};

const LAYOUTS: &str = include_str!("../conformance/layout/v1/layouts.tsv");

#[test]
fn every_golden_record_decodes_admits_and_reencodes_canonically() -> Result<(), Box<dyn Error>> {
    for row in LAYOUTS.lines().skip(2) {
        let case = field(row, 0)?;
        let bytes = decode_hex(record_fixture(case)?)?;
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

fn record_fixture(case: &str) -> Result<&'static str, Box<dyn Error>> {
    match case {
        "empty" => Ok(include_str!("../conformance/layout/v1/empty.layout.hex").trim_end()),
        "one-zero" => Ok(include_str!("../conformance/layout/v1/one-zero.layout.hex").trim_end()),
        "max-plus-one-zeros" => {
            Ok(include_str!("../conformance/layout/v1/max-plus-one-zeros.layout.hex").trim_end())
        }
        "zeros-long" => {
            Ok(include_str!("../conformance/layout/v1/zeros-long.layout.hex").trim_end())
        }
        _ => Err(Box::new(invalid_corpus("unknown record fixture"))),
    }
}

fn field(row: &str, index: usize) -> Result<&str, Box<dyn Error>> {
    row.split('\t')
        .nth(index)
        .ok_or_else(|| Box::<dyn Error>::from(invalid_corpus("TSV row is missing a field")))
}
