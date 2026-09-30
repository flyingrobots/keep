//! Public verification-report laws over the reference view.

#[path = "layout_mutations/support.rs"]
mod layout_mutation_support;
mod support;

use std::error::Error;
use std::io::Cursor;

use keep::{
    AdmittedLayout, BlobHasher, BlobId, ChunkSpan, CorruptionEvidence, FastCdc, LayoutDecodePolicy,
    LayoutEntryLimit, MissingEvidence, PublishedBlob, ReferenceStore, ReferenceStoreCapacity,
    RegisteredStorageProfile, VerificationDepth, VerificationError, VerificationRefusal,
    VerificationReport, VerificationSubject,
};

use layout_mutation_support::mutation_cases;

/// A blob identity together with the spans the registered profile cuts it into.
type Identified = (BlobId, Vec<ChunkSpan>);

const SOURCE: &[u8] = b"exact bytes whose every depth is reported, never inflated";

#[test]
fn a_report_establishes_exactly_the_requested_depth() -> Result<(), Box<dyn Error>> {
    let (store, published) = published_store(SOURCE)?;
    for depth in [
        VerificationDepth::ChunkIdentity,
        VerificationDepth::LayoutIdentity,
        VerificationDepth::CompleteBlobIdentity,
    ] {
        for subject in [
            VerificationSubject::Blob(published.target()),
            VerificationSubject::Layout(published.layout_id()),
        ] {
            let report = store.verify(subject, depth)?;
            assert_eq!(report.depth(), depth, "{subject:?} at {depth:?}");
            assert_eq!(report.subject(), subject);
            assert_eq!(report.layout(), published.layout_id());
            assert_eq!(report.target(), published.target());
            assert_eq!(report.chunks_verified(), 1);
        }
    }
    assert!(
        VerificationDepth::ALL
            .windows(2)
            .all(|pair| matches!(pair, [shallower, deeper] if shallower < deeper))
    );
    Ok(())
}

#[test]
fn unsupported_depths_refuse_with_the_supported_range() -> Result<(), Box<dyn Error>> {
    let (store, published) = published_store(SOURCE)?;
    let subject = VerificationSubject::Blob(published.target());
    for depth in [
        VerificationDepth::Framing,
        VerificationDepth::Checksum,
        VerificationDepth::CatalogReachability,
        VerificationDepth::RetentionClosure,
    ] {
        let refusal = refusal_of(store.verify(subject, depth))?;
        assert!(
            matches!(
                refusal,
                VerificationRefusal::Unsupported {
                    requested,
                    supported_minimum: VerificationDepth::ChunkIdentity,
                    supported_maximum: VerificationDepth::CompleteBlobIdentity,
                    ..
                } if requested == depth
            ),
            "{depth:?} refused with {refusal:?}"
        );
    }
    Ok(())
}

#[test]
fn absent_subjects_are_missing_evidence_against_the_complete_view() -> Result<(), Box<dyn Error>> {
    let (store, _published) = published_store(SOURCE)?;
    let mut other = Cursor::new(b"staged but never committed");
    let staged = store.stage(&mut other, LayoutEntryLimit::MAXIMUM)?;
    let absent_blob = staged.target();
    let absent_layout = staged.layout_id();
    drop(staged);

    let refusal = refusal_of(store.verify(
        VerificationSubject::Blob(absent_blob),
        VerificationDepth::ChunkIdentity,
    ))?;
    assert!(matches!(
        refusal,
        VerificationRefusal::Missing {
            stage: VerificationDepth::ChunkIdentity,
            evidence: MissingEvidence::Blob(blob),
            ..
        } if blob == absent_blob
    ));

    let refusal = refusal_of(store.verify(
        VerificationSubject::Layout(absent_layout),
        VerificationDepth::CompleteBlobIdentity,
    ))?;
    assert!(matches!(
        refusal,
        VerificationRefusal::Missing {
            evidence: MissingEvidence::Layout(layout),
            ..
        } if layout == absent_layout
    ));

    let (never_staged, spans) = identify(b"chunks this store never held")?;
    let layout = AdmittedLayout::from_spans(
        never_staged,
        registered_profile()?,
        spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let refusal =
        refusal_of(store.verify_admitted_layout(&layout, VerificationDepth::ChunkIdentity))?;
    assert!(matches!(
        refusal,
        VerificationRefusal::Missing {
            stage: VerificationDepth::ChunkIdentity,
            evidence: MissingEvidence::Chunk { index: 0, .. },
            ..
        }
    ));
    Ok(())
}

#[test]
fn a_wrong_target_passes_chunk_identity_and_fails_only_the_complete_blob()
-> Result<(), Box<dyn Error>> {
    let (store, _published) = published_store(SOURCE)?;
    let (real_target, spans) = identify(SOURCE)?;
    // Same length, different bytes: layout admission checks the length, and
    // only complete-blob verification can tell the two targets apart.
    let mut altered = SOURCE.to_vec();
    support::patch(&mut altered, 0, b"E")?;
    let (wrong_target, _) = identify(&altered)?;
    assert_ne!(wrong_target, real_target);
    let layout = AdmittedLayout::from_spans(
        wrong_target,
        registered_profile()?,
        spans,
        LayoutEntryLimit::MAXIMUM,
    )?;

    let shallow = store.verify_admitted_layout(&layout, VerificationDepth::ChunkIdentity)?;
    assert_eq!(shallow.depth(), VerificationDepth::ChunkIdentity);
    assert_eq!(shallow.target(), wrong_target);

    let refusal =
        refusal_of(store.verify_admitted_layout(&layout, VerificationDepth::CompleteBlobIdentity))?;
    assert!(matches!(
        refusal,
        VerificationRefusal::Corrupt {
            stage: VerificationDepth::CompleteBlobIdentity,
            evidence: CorruptionEvidence::BlobIdentity { expected, observed, .. },
            ..
        } if expected == wrong_target && observed == real_target
    ));
    Ok(())
}

#[test]
fn false_profile_boundaries_pass_chunk_identity_and_fail_only_the_complete_blob()
-> Result<(), Box<dyn Error>> {
    let mutation = mutation_cases()?
        .into_iter()
        .find(|candidate| candidate.case() == "profile-boundary-mismatch")
        .ok_or("profile-boundary mismatch fixture is absent")?;
    let encoded = mutation.mutated_record()?;
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    for bytes in [vec![0_u8; 262_143], vec![0_u8; 2]] {
        let mut source = Cursor::new(bytes);
        let _published = store
            .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
            .commit(&mut store)?;
    }
    let policy = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);
    let layout = AdmittedLayout::decode_record(&encoded, policy)?;

    let shallow = store.verify_admitted_layout(&layout, VerificationDepth::ChunkIdentity)?;
    assert_eq!(shallow.depth(), VerificationDepth::ChunkIdentity);
    assert_eq!(shallow.chunks_verified(), 2);

    let refusal =
        refusal_of(store.verify_admitted_layout(&layout, VerificationDepth::CompleteBlobIdentity))?;
    assert!(matches!(
        refusal,
        VerificationRefusal::Corrupt {
            stage: VerificationDepth::CompleteBlobIdentity,
            evidence: CorruptionEvidence::ProfileBoundary { index: 0, .. },
            ..
        }
    ));
    Ok(())
}

fn refusal_of(
    outcome: Result<VerificationReport, VerificationError>,
) -> Result<VerificationRefusal, Box<dyn Error>> {
    match outcome {
        Err(VerificationError::Refused(refusal)) => Ok(*refusal),
        Err(VerificationError::Operational(failure)) => {
            Err(format!("operational failure instead of a refusal: {failure:?}").into())
        }
        Ok(report) => Err(format!("report instead of a refusal: {report:?}").into()),
    }
}

fn published_store(bytes: &[u8]) -> Result<(ReferenceStore, PublishedBlob), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut source = Cursor::new(bytes);
    let published = store
        .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    Ok((store, published))
}

/// The one registered profile, taken from a staged layout so the test
/// depends on no profile constant.
fn registered_profile() -> Result<RegisteredStorageProfile, Box<dyn Error>> {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut source = Cursor::new(b"profile witness");
    let staged = store.stage(&mut source, LayoutEntryLimit::MAXIMUM)?;
    Ok(staged.layout().profile())
}

fn identify(bytes: &[u8]) -> Result<Identified, Box<dyn Error>> {
    let mut hasher = BlobHasher::new();
    hasher.update(bytes)?;
    let mut detector = FastCdc::new();
    let mut spans = Vec::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok((hasher.finish(), spans))
}
