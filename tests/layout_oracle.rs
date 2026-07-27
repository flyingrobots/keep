//! Independent golden-record reconstruction before production cross-checking.

#[path = "layout_oracle/support.rs"]
pub mod oracle_support;
pub mod support;

use std::error::Error;

use keep::{AdmittedLayout, BlobId, FastCdc, LayoutEntryLimit, RegisteredStorageProfile};
use oracle_support::verify_record;
use support::{decode_hex, invalid_corpus};

const LAYOUTS: &str = include_str!("../conformance/layout/v1/layouts.tsv");

#[test]
fn independent_record_oracle_precedes_production_encoder_cross_check() -> Result<(), Box<dyn Error>>
{
    for row in LAYOUTS.lines().skip(2) {
        let case = field(row, 0)?;
        let record = decode_hex(record_fixture(case)?)?;
        verify_record(case, row, &record)?;

        let source = source_bytes(case, field(row, 3)?)?;
        let target = BlobId::hash_bytes(&source)?;
        let layout = AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            detect_spans(&source)?,
            LayoutEntryLimit::MAXIMUM,
        )?;
        assert_eq!(layout.encode_record()?.bytes(), record, "{case}");
    }
    Ok(())
}

fn detect_spans(bytes: &[u8]) -> Result<Vec<keep::ChunkSpan>, keep::ChunkingError> {
    let mut spans = Vec::new();
    let mut detector = FastCdc::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok(spans)
}

fn source_bytes(case: &str, count: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let length = count.parse::<usize>()?;
    match case {
        "empty" | "one-zero" | "max-plus-one-zeros" | "zeros-long" => Ok(vec![0_u8; length]),
        _ => Err(Box::new(invalid_corpus("unknown source recipe"))),
    }
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
