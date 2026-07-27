//! Independent golden-record reconstruction before production cross-checking.

#[path = "layout_oracle/support.rs"]
pub mod oracle_support;
pub mod support;

use std::error::Error;

use keep::{AdmittedLayout, BlobId, LayoutEntryLimit, RegisteredStorageProfile};
use oracle_support::verify_record;
use support::{detect_spans, field, layout_record_bytes, layout_source_bytes};

const LAYOUTS: &str = include_str!("../conformance/layout/v1/layouts.tsv");

#[test]
fn independent_record_oracle_precedes_production_encoder_cross_check() -> Result<(), Box<dyn Error>>
{
    for row in LAYOUTS.lines().skip(2) {
        let case = field(row, 0)?;
        let record = layout_record_bytes(case)?;
        verify_record(case, row, &record)?;

        let source = layout_source_bytes(case, field(row, 3)?)?;
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
