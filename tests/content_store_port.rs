//! The content-store port from outside the crate: code written once
//! against `ContentStaging` and `ContentReads` runs on the reference
//! backend; receipts carry the identities they claim; the byte and entry
//! limits refuse before anything becomes visible; an expected identity
//! that does not match refuses with both identities.

use std::error::Error;
use std::io::Cursor;

use keep::{
    BlobHasher, BlobId, ByteLength, ByteOffset, ByteRange, CommitReceipt, ContentReads,
    ContentStaging, IngestionError, LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity,
    StagedByteLimit, StagedContent, StagingLimits,
};

const CAPACITY: usize = 4 * 1024 * 1024;

fn content(seed: u64, length: usize) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).unwrap_or_default()
        })
        .collect()
}

/// The committed identity and the bytes read back through the port.
type RoundTrip = Result<(BlobId, Vec<u8>), Box<dyn Error>>;

/// Backend-neutral: stage, commit, and read back through the port alone.
fn round_trip<S>(store: &mut S, bytes: &[u8]) -> RoundTrip
where
    S: ContentStaging + ContentReads,
{
    let staged = store.stage(
        &mut Cursor::new(bytes),
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    let expected_target = staged.target();
    let expected_layout = staged.layout_id();
    let receipt = staged.commit()?;
    // The receipt is `Copy`, so reading its identities ends the store borrow.
    let (target, layout_id) = (receipt.target(), receipt.layout_id());
    assert_eq!(target, expected_target);
    assert_eq!(layout_id, expected_layout);
    let mut output = Vec::new();
    store
        .reconstruct_layout(layout_id, &mut output)
        .map_err(|error| error.to_string())?;
    Ok((target, output))
}

#[test]
fn the_reference_backend_round_trips_through_the_port() -> Result<(), Box<dyn Error>> {
    let bytes = content(0x1234_5678_9abc_def1, 200 * 1024);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let (target, output) = round_trip(&mut store, &bytes)?;
    assert_eq!(output, bytes);
    assert!(ContentReads::contains_blob(&store, target)?);
    let requested = ByteRange::new(ByteOffset::new(70_000), ByteLength::new(3_000))?;
    let mut range = Vec::new();
    let _receipt = ContentReads::read_range(&store, target, requested, &mut range)?;
    assert_eq!(Some(range.as_slice()), bytes.get(70_000..73_000));
    Ok(())
}

#[test]
fn staging_refuses_the_byte_limit_before_anything_is_visible() -> Result<(), Box<dyn Error>> {
    let bytes = content(0xfeed_beef_dead_c0de, 96 * 1024);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let limits = StagingLimits::new(LayoutEntryLimit::MAXIMUM, StagedByteLimit::new(64 * 1024));
    let refusal = ContentStaging::stage(&mut store, &mut Cursor::new(&bytes), limits);
    assert!(matches!(
        refusal,
        Err(IngestionError::ByteLimitExceeded { limit, .. }) if limit.get() == 64 * 1024
    ));
    let (target, _) = round_trip(&mut store, &bytes)?;
    assert!(ContentReads::contains_blob(&store, target)?);
    Ok(())
}

#[test]
fn staging_refuses_the_entry_limit_through_the_port() -> Result<(), Box<dyn Error>> {
    let bytes = content(0x0bad_cafe_f00d_face, 512 * 1024);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let limits = StagingLimits::entries(LayoutEntryLimit::new(1)?);
    let refusal = ContentStaging::stage(&mut store, &mut Cursor::new(&bytes), limits);
    assert!(refusal.is_err());
    Ok(())
}

#[test]
fn an_expected_identity_that_does_not_match_refuses_with_both() -> Result<(), Box<dyn Error>> {
    let bytes = content(0x7777_1111_2222_3333, 8 * 1024);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let wrong = BlobHasher::new().finish();
    let mut hasher = BlobHasher::new();
    hasher.update(&bytes)?;
    let right = hasher.finish();
    let refusal = ContentStaging::stage_expected(
        &mut store,
        &mut Cursor::new(&bytes),
        wrong,
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    );
    assert!(matches!(
        refusal,
        Err(IngestionError::BlobIdentityMismatch { expected, observed })
            if expected == wrong && observed == right
    ));
    let staged = ContentStaging::stage_expected(
        &mut store,
        &mut Cursor::new(&bytes),
        right,
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    assert_eq!(StagedContent::target(&staged), right);
    Ok(())
}
