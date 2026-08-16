//! Public output-boundary refusal laws for exact byte-range reads.

pub(crate) mod support;

use std::error::Error;
use std::io::{Cursor, ErrorKind};

use keep::{
    ByteLength, ByteOffset, ByteRange, LayoutEntryLimit, RangeReadError, ReferenceStore,
    ReferenceStoreCapacity,
};
use support::{FailingWriter, LyingWriter, PrefixThenFailWriter, ZeroWriter};

#[test]
fn broken_range_writers_preserve_exact_failure_boundaries() -> Result<(), Box<dyn Error>> {
    let source = b"authenticated range before the writer is called";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(11))?;

    let mut zero = ZeroWriter;
    assert!(matches!(
        store.read_range(published.target(), requested, &mut zero),
        Err(RangeReadError::WriteZero {
            layout,
            bytes_written
        }) if layout == published.layout_id() && bytes_written.is_empty()
    ));

    let maximum = usize::try_from(requested.length().get())?;
    let observed = maximum
        .checked_add(1)
        .ok_or("fixture write count overflow")?;
    let mut lying = LyingWriter;
    assert!(matches!(
        store.read_range(published.target(), requested, &mut lying),
        Err(RangeReadError::InvalidWriteCount {
            layout,
            maximum: actual_maximum,
            observed: actual_observed,
            bytes_written
        }) if layout == published.layout_id()
            && actual_maximum == maximum
            && actual_observed == observed
            && bytes_written.is_empty()
    ));

    let mut failing = FailingWriter;
    let error = store
        .read_range(published.target(), requested, &mut failing)
        .err()
        .ok_or("failing writer unexpectedly accepted a range")?;
    assert!(matches!(
        error,
        RangeReadError::Write {
            layout,
            bytes_written,
            ref source
        } if layout == published.layout_id()
            && bytes_written.is_empty()
            && source.kind() == ErrorKind::PermissionDenied
    ));
    assert!(Error::source(&error).is_some());
    Ok(())
}

#[test]
fn range_failure_reports_the_prefix_already_accepted() -> Result<(), Box<dyn Error>> {
    let source = b"an accepted range prefix remains untrusted without a receipt";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut reader = Cursor::new(source);
    let published = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let requested = ByteRange::new(ByteOffset::new(3), ByteLength::new(13))?;
    let accepted_length = 5_usize;
    let mut output = PrefixThenFailWriter::new(accepted_length)?;

    let error = store
        .read_range(published.target(), requested, &mut output)
        .err()
        .ok_or("prefix-failing writer unexpectedly accepted a range")?;

    assert!(matches!(
        error,
        RangeReadError::Write {
            layout,
            bytes_written,
            ref source
        } if layout == published.layout_id()
            && bytes_written.get() == u64::try_from(accepted_length)?
            && source.kind() == ErrorKind::PermissionDenied
    ));
    let requested_start = usize::try_from(requested.offset().get())?;
    let accepted_end = requested_start
        .checked_add(accepted_length)
        .ok_or("accepted range overflow")?;
    assert_eq!(
        output.bytes(),
        source
            .get(requested_start..accepted_end)
            .ok_or("accepted range")?
    );
    assert!(Error::source(&error).is_some());
    Ok(())
}
