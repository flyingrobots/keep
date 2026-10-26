//! Generic laws every `ContentReads` view satisfies, run by each backend's
//! own tests against its own fixtures.
//!
//! Refusals are asserted through each backend's typed oracle rather than as
//! bare errors: the port fixes the law, the backend names the exact variant.

#![expect(
    clippy::redundant_pub_crate,
    reason = "the durable adapter runs the generic laws through this crate-private surface"
)]

use std::error::Error;

use super::ContentReads;
use crate::{BlobId, ByteLength, ByteOffset, ByteRange, LayoutId};

/// How one backend names the refusals the generic laws require.
pub(crate) struct RefusalOracle<C: ContentReads> {
    /// Whether a range read refused because the range ends past the blob.
    pub(crate) past_end: fn(&C::RangeError) -> bool,
    /// Whether a reconstruction refused because the blob is absent.
    pub(crate) absent: fn(&C::ReconstructionError) -> bool,
}

/// The view reconstructs `target` to exactly `expected`, through both the
/// automatic and the exact-layout entry points, with equal receipts.
pub(crate) fn reconstructs_exactly<C: ContentReads>(
    view: &C,
    target: BlobId,
    layout_id: LayoutId,
    expected: &[u8],
) -> Result<(), Box<dyn Error>> {
    assert!(view.contains_blob(target)?);
    let mut output = Vec::new();
    let receipt = view.reconstruct(target, &mut output)?;
    assert_eq!(output.as_slice(), expected);
    let mut exact = Vec::new();
    let exact_receipt = view.reconstruct_layout(layout_id, &mut exact)?;
    assert_eq!(exact.as_slice(), expected);
    assert_eq!(exact_receipt, receipt);
    Ok(())
}

/// Every range the view is asked for emits exactly that slice, and a range
/// past the end refuses as such with nothing written.
pub(crate) fn ranges_exactly<C: ContentReads>(
    view: &C,
    oracle: &RefusalOracle<C>,
    target: BlobId,
    expected: &[u8],
    ranges: &[(u64, u64)],
) -> Result<(), Box<dyn Error>> {
    for (offset, length) in ranges {
        let requested = ByteRange::new(ByteOffset::new(*offset), ByteLength::new(*length))?;
        let mut output = Vec::new();
        let _receipt = view.read_range(target, requested, &mut output)?;
        let start = usize::try_from(*offset)?;
        let end = usize::try_from(offset.checked_add(*length).ok_or("range end overflow")?)?;
        assert_eq!(Some(output.as_slice()), expected.get(start..end));
    }
    let past = ByteRange::new(
        ByteOffset::new(u64::try_from(expected.len())?),
        ByteLength::new(1),
    )?;
    let mut output = Vec::new();
    let refusal = view
        .read_range(target, past, &mut output)
        .err()
        .ok_or("a range past the end returned a receipt")?;
    assert!((oracle.past_end)(&refusal), "unexpected refusal: {refusal}");
    assert!(output.is_empty());
    Ok(())
}

/// An absent blob is reported absent and refused as absent with nothing
/// written.
pub(crate) fn absence_refuses<C: ContentReads>(
    view: &C,
    oracle: &RefusalOracle<C>,
    absent: BlobId,
) -> Result<(), Box<dyn Error>> {
    assert!(!view.contains_blob(absent)?);
    let mut output = Vec::new();
    let refusal = view
        .reconstruct(absent, &mut output)
        .err()
        .ok_or("an absent blob returned a receipt")?;
    assert!((oracle.absent)(&refusal), "unexpected refusal: {refusal}");
    assert!(output.is_empty());
    Ok(())
}
