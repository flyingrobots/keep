//! Verification laws that need to tamper with the reference view's bytes.
//! Size: Small. Oracle: tampering must refuse at chunk identity before deeper claims.
//! Delete only if reference verification is removed or stronger coverage subsumes it.

use std::io::Cursor;

use crate::{
    CorruptionEvidence, LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity,
    ReferenceVerificationEvidence, ReferenceVerificationSource, VerificationDepth,
    VerificationError, VerificationRefusal, VerificationSource, VerificationSubject,
};

#[test]
fn tampered_stored_chunk_is_corrupt_at_the_chunk_identity_stage()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut source = Cursor::new(b"bytes the store will silently change");
    let published = store
        .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let layout = store
        .layout(published.layout_id())
        .ok_or("published layout is absent")?;
    let entry = layout
        .entries()
        .first()
        .copied()
        .ok_or("published layout names no chunk")?;
    let stored = store
        .chunks
        .get_mut(&entry.chunk_id())
        .ok_or("published chunk is absent")?;
    let first = stored.first_mut().ok_or("published chunk is empty")?;
    *first ^= 1;

    for depth in [
        VerificationDepth::ChunkIdentity,
        VerificationDepth::CompleteBlobIdentity,
    ] {
        let outcome = store.verify(
            VerificationSubject::Blob {
                identity: published.target(),
            },
            depth,
        );
        assert!(
            matches!(
                &outcome,
                Err(VerificationError::Refused {
                    refusal: VerificationRefusal::Corrupt { .. }, source: Some(source),
                }) if matches!(source.as_ref(),
                    VerificationSource::Reference(ReferenceVerificationSource::Refusal(context))
                    if context.stage() == VerificationDepth::ChunkIdentity && matches!(context.evidence(),
                        ReferenceVerificationEvidence::Corrupt(CorruptionEvidence::ChunkIdentity { index: 0, .. }))
                )
            ),
            "depth {depth:?} did not report the chunk-identity contradiction: {outcome:?}"
        );
    }
    Ok(())
}
