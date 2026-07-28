//! Public success laws for every exact range-read entrypoint.

pub(crate) mod support;

use std::error::Error;
use std::io::Cursor;

use keep::{
    AdmittedLayout, BlobId, ByteLength, ByteOffset, ByteRange, LayoutDecodePolicy,
    LayoutEntryLimit, RangeReadError, ReferenceStore, ReferenceStoreCapacity,
    RegisteredStorageProfile,
};
use support::detect_spans;

#[test]
fn admitted_and_canonical_record_ranges_return_the_same_exact_bytes() -> Result<(), Box<dyn Error>>
{
    let source = b"all public range entrypoints preserve exact bytes";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let staged = store.stage(&mut reader, LayoutEntryLimit::MAXIMUM)?;
    let layout = staged.layout().clone();
    let record = layout.encode_record()?.bytes().to_vec();
    let published = staged.commit(&mut store)?;
    let requested = ByteRange::new(ByteOffset::new(4), ByteLength::new(12))?;
    let expected = b"public range";

    let mut admitted_output = Vec::new();
    let admitted = store.read_admitted_layout_range(&layout, requested, &mut admitted_output)?;
    assert_eq!(admitted_output, expected);
    assert_eq!(admitted.layout_id(), published.layout_id());

    let mut record_output = Vec::new();
    let decoded = store.read_record_range(
        &record,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
        requested,
        &mut record_output,
    )?;
    assert_eq!(record_output, expected);
    assert_eq!(decoded.layout_id(), published.layout_id());
    assert_eq!(decoded, admitted);
    Ok(())
}

#[test]
fn caller_supplied_ranges_require_a_committed_target_layout_binding() -> Result<(), Box<dyn Error>>
{
    let source = b"stored bytes cannot be relabeled by an admitted layout";
    let claimed = vec![b'x'; source.len()];
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let _published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let forged = AdmittedLayout::from_spans(
        BlobId::hash_bytes(&claimed)?,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        detect_spans(source)?,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let record = forged.encode_record()?;
    let forged_layout_id = record.id();
    let requested = ByteRange::new(ByteOffset::ZERO, ByteLength::new(5))?;
    let mut output = Vec::new();

    assert!(matches!(
        store.read_admitted_layout_range(&forged, requested, &mut output),
        Err(RangeReadError::LayoutMissing { requested })
            if requested == forged_layout_id
    ));
    assert!(output.is_empty());

    assert!(matches!(
        store.read_record_range(
            record.bytes(),
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
            requested,
            &mut output,
        ),
        Err(RangeReadError::LayoutMissing { requested })
            if requested == forged_layout_id
    ));
    assert!(output.is_empty());
    Ok(())
}
