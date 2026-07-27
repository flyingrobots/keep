//! Public flat-layout admission and canonical encoding laws.

pub mod support;

use std::error::Error;

use keep::{
    AdmittedLayout, BlobId, LayoutEntryLimit, LayoutRecordLength, LayoutValidationError,
    RegisteredStorageProfile,
};
use support::{detect_spans, field, layout_record_bytes, layout_source_bytes, require_error};

const LAYOUTS: &str = include_str!("../conformance/layout/v1/layouts.tsv");

#[test]
fn record_length_bounds_remain_domain_typed() {
    let minimum: LayoutRecordLength = LayoutRecordLength::MINIMUM;
    let maximum: LayoutRecordLength = LayoutRecordLength::MAXIMUM;

    assert_eq!(minimum.get(), 176);
    assert_eq!(maximum.get(), 46_137_520);
}

#[test]
fn every_semantic_golden_layout_encodes_to_the_frozen_record() -> Result<(), Box<dyn Error>> {
    for line in LAYOUTS.lines().skip(2) {
        let case = field(line, 0)?;
        let source = layout_source_bytes(case, field(line, 3)?)?;
        let target = BlobId::hash_bytes(&source)?;
        let spans = detect_spans(&source)?;
        let layout = AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            spans,
            LayoutEntryLimit::MAXIMUM,
        )?;
        let encoded = layout.encode_record()?;
        let expected = layout_record_bytes(case)?;

        assert_eq!(encoded.bytes(), expected, "{case}");
        assert_eq!(encoded.id().to_string(), field(line, 10)?, "{case}");
        assert_eq!(layout.target(), target, "{case}");
        assert_eq!(
            layout.profile(),
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            "{case}"
        );
        assert_eq!(
            layout.entries().len().to_string(),
            field(line, 7)?,
            "{case}"
        );
    }
    Ok(())
}

#[test]
fn admission_refuses_entry_counts_above_the_caller_cap() -> Result<(), Box<dyn Error>> {
    let source = [0_u8];
    let target = BlobId::hash_bytes(&source)?;
    let error = require_error(
        AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            detect_spans(&source)?,
            LayoutEntryLimit::new(0)?,
        ),
        "one entry did not exceed a zero-entry cap",
    )?;

    assert!(matches!(
        error,
        LayoutValidationError::EntryLimitExceeded {
            maximum: 0,
            observed: 1
        }
    ));
    Ok(())
}

#[test]
fn admission_refuses_empty_and_nonempty_cardinality_inversions() -> Result<(), Box<dyn Error>> {
    let source = [0_u8];
    let empty_target = BlobId::hash_bytes(&[])?;
    let nonempty_target = BlobId::hash_bytes(&source)?;

    let empty_error = require_error(
        AdmittedLayout::from_spans(
            empty_target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            detect_spans(&source)?,
            LayoutEntryLimit::MAXIMUM,
        ),
        "an empty target admitted an entry",
    )?;
    assert!(matches!(
        empty_error,
        LayoutValidationError::EmptyBlobHasEntries { observed: 1 }
    ));

    let nonempty_error = require_error(
        AdmittedLayout::from_spans(
            nonempty_target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            Vec::new(),
            LayoutEntryLimit::MAXIMUM,
        ),
        "a nonempty target admitted no entries",
    )?;
    assert!(matches!(
        nonempty_error,
        LayoutValidationError::NonemptyBlobHasNoEntries
    ));
    Ok(())
}

#[test]
fn admission_refuses_an_entry_aggregate_that_misses_the_target() -> Result<(), Box<dyn Error>> {
    let target = BlobId::hash_bytes(&[0_u8, 0_u8])?;
    let error = require_error(
        AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            detect_spans(&[0_u8])?,
            LayoutEntryLimit::MAXIMUM,
        ),
        "an entry aggregate admitted the wrong target length",
    )?;

    assert!(matches!(
        error,
        LayoutValidationError::AggregateLengthMismatch { observed: 1, .. }
    ));
    Ok(())
}
