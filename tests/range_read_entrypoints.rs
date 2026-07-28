//! Public success laws for every exact range-read entrypoint.

use std::error::Error;
use std::io::Cursor;

use keep::{
    ByteLength, ByteOffset, ByteRange, LayoutDecodePolicy, LayoutEntryLimit, ReferenceStore,
    ReferenceStoreCapacity,
};

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
