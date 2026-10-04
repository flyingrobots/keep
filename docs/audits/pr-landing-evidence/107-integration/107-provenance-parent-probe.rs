//! Runtime reproduction against the unfixed PR's original infallible projection API.
mod support;
use std::{error::Error, io::Cursor};
use keep::{BlobId, CanonicalVerificationReceipt, LayoutEntryLimit, ReferenceStore,
    ReferenceStoreCapacity, VerificationDepth, VerificationError, VerificationReceipt,
    VerificationSubject, VerificationView};

fn unrelated_durable_view() -> Result<VerificationView, Box<dyn Error>> {
    let bytes = support::decode_hex(include_str!("../conformance/verification-receipt/v1/durable-corrupt-chunk-refusal.hex").trim())?;
    Ok(CanonicalVerificationReceipt::decode(&bytes)?.receipt().view())
}

#[test]
fn reference_report_must_not_acquire_unrelated_durable_provenance() -> Result<(), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let published = store.stage(&mut Cursor::new(b"reference only"), LayoutEntryLimit::MAXIMUM)?.commit(&mut store)?;
    let report = store.verify(VerificationSubject::Blob(published.target()), VerificationDepth::CompleteBlobIdentity)?;
    let unrelated = unrelated_durable_view()?;
    let projected = VerificationReceipt::from_report(&report, unrelated);
    assert_ne!(projected.view(), unrelated, "reference report must not acquire unrelated durable provenance");
    Ok(())
}

#[test]
fn reference_refusal_must_not_acquire_unrelated_durable_provenance() -> Result<(), Box<dyn Error>> {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let target = BlobId::hash_bytes(b"absent reference blob")?;
    let Err(VerificationError::Refused(refusal)) = store.verify(VerificationSubject::Blob(target), VerificationDepth::CompleteBlobIdentity) else {
        return Err("expected reference absence".into());
    };
    let unrelated = unrelated_durable_view()?;
    let projected = VerificationReceipt::from_refusal(&refusal, unrelated);
    assert_ne!(projected.view(), unrelated, "reference refusal must not acquire unrelated durable provenance");
    Ok(())
}
