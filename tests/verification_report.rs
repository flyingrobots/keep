//! Public verification-report laws over the reference view.
//! Size: Small. Oracle: explicit verification claims and preserved refusal evidence.
//! Delete only when the reference verification contract is removed or subsumed.

#[path = "layout_mutations/support.rs"]
mod layout_mutation_support;
mod support;

use std::error::Error;
use std::io::Cursor;

use keep::{
    AdmittedLayout, BlobHasher, BlobId, ChunkSpan, CorruptionEvidence, FastCdc, LayoutDecodePolicy,
    LayoutEntryLimit, MissingEvidence, PublishedBlob, ReferenceStore, ReferenceStoreCapacity,
    ReferenceVerificationContext, ReferenceVerificationEvidence, ReferenceVerificationSource,
    RegisteredStorageProfile, VerificationDepth, VerificationError, VerificationObservation,
    VerificationRefusal, VerificationReport, VerificationSource, VerificationSubject,
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
            VerificationSubject::Blob {
                identity: published.target(),
            },
            VerificationSubject::Layout {
                identity: published.layout_id(),
            },
        ] {
            let report = store.verify(subject, depth)?;
            assert_eq!(report.requested(), depth, "{subject:?} at {depth:?}");
            let [claim] = report.subjects() else {
                return Err("expected one subject claim".into());
            };
            assert_eq!(claim.subject(), subject);
            assert_eq!(claim.depth(), depth);
            let details = report
                .reference_details()
                .ok_or("reference evidence missing")?;
            assert_eq!(details.layout(), published.layout_id());
            assert_eq!(details.target(), published.target());
            assert_eq!(details.chunks_verified(), 1);
            assert_eq!(report.catalog(), None);
            assert_eq!(report.retention_head(), None);
        }
    }
    Ok(())
}

#[test]
fn unsupported_depths_refuse_with_the_exact_supported_set() -> Result<(), Box<dyn Error>> {
    let (store, published) = published_store(SOURCE)?;
    let subject = VerificationSubject::Blob {
        identity: published.target(),
    };
    for depth in [
        VerificationDepth::Framing,
        VerificationDepth::Checksum,
        VerificationDepth::CatalogReachability,
        VerificationDepth::RetentionClosure,
        VerificationDepth::SnapshotBinding,
    ] {
        let (refusal, context) = refusal_of(store.verify(subject, depth))?;
        assert_eq!(
            refusal,
            VerificationRefusal::Unsupported {
                subject,
                requested: depth,
                supported: &[
                    VerificationDepth::ChunkIdentity,
                    VerificationDepth::LayoutIdentity,
                    VerificationDepth::CompleteBlobIdentity,
                ],
            }
        );
        assert_eq!(context.subject(), subject);
        assert_eq!(context.stage(), depth);
        assert_eq!(
            context.evidence(),
            ReferenceVerificationEvidence::Unsupported
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

    let subject = VerificationSubject::Blob {
        identity: absent_blob,
    };
    let (refusal, context) = refusal_of(store.verify(subject, VerificationDepth::ChunkIdentity))?;
    assert_eq!(refusal, VerificationRefusal::Missing { subject });
    assert_eq!(context.stage(), VerificationDepth::ChunkIdentity);
    assert_eq!(
        context.evidence(),
        ReferenceVerificationEvidence::Missing(MissingEvidence::Blob(absent_blob))
    );

    let subject = VerificationSubject::Layout {
        identity: absent_layout,
    };
    let (refusal, context) =
        refusal_of(store.verify(subject, VerificationDepth::CompleteBlobIdentity))?;
    assert_eq!(refusal, VerificationRefusal::Missing { subject });
    assert_eq!(
        context.evidence(),
        ReferenceVerificationEvidence::Missing(MissingEvidence::Layout(absent_layout))
    );

    let (never_staged, spans) = identify(b"chunks this store never held")?;
    let layout = AdmittedLayout::from_spans(
        never_staged,
        registered_profile()?,
        spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let (refusal, context) =
        refusal_of(store.verify_admitted_layout(&layout, VerificationDepth::ChunkIdentity))?;
    let chunk = layout
        .entries()
        .first()
        .ok_or("layout has no chunk")?
        .chunk_id();
    assert_eq!(
        refusal,
        VerificationRefusal::Missing {
            subject: VerificationSubject::Chunk { identity: chunk }
        }
    );
    assert_eq!(context.stage(), VerificationDepth::ChunkIdentity);
    assert_eq!(
        context.evidence(),
        ReferenceVerificationEvidence::Missing(MissingEvidence::Chunk {
            layout: layout.encode_record()?.id(),
            index: 0,
            chunk,
        })
    );
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
    assert_eq!(shallow.requested(), VerificationDepth::ChunkIdentity);
    assert_eq!(
        shallow
            .reference_details()
            .ok_or("reference evidence missing")?
            .target(),
        wrong_target
    );

    let (refusal, context) =
        refusal_of(store.verify_admitted_layout(&layout, VerificationDepth::CompleteBlobIdentity))?;
    let layout_id = layout.encode_record()?.id();
    assert_eq!(
        refusal,
        VerificationRefusal::Corrupt {
            subject: VerificationSubject::Layout {
                identity: layout_id
            },
            expected: VerificationObservation::Blob(wrong_target),
            observed: VerificationObservation::Blob(real_target),
        }
    );
    assert_eq!(context.stage(), VerificationDepth::CompleteBlobIdentity);
    assert_eq!(
        context.evidence(),
        ReferenceVerificationEvidence::Corrupt(CorruptionEvidence::BlobIdentity {
            layout: layout_id,
            expected: wrong_target,
            observed: real_target,
        })
    );
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
    assert_eq!(shallow.requested(), VerificationDepth::ChunkIdentity);
    assert_eq!(
        shallow
            .reference_details()
            .ok_or("reference evidence missing")?
            .chunks_verified(),
        2
    );

    let (refusal, context) =
        refusal_of(store.verify_admitted_layout(&layout, VerificationDepth::CompleteBlobIdentity))?;
    assert_eq!(context.stage(), VerificationDepth::CompleteBlobIdentity);
    assert!(matches!(
        context.evidence(),
        ReferenceVerificationEvidence::Corrupt(CorruptionEvidence::ProfileBoundary {
            index: 0,
            ..
        })
    ));
    assert!(matches!(
        refusal,
        VerificationRefusal::Corrupt {
            expected: VerificationObservation::ProfileBoundary(_),
            observed: VerificationObservation::ProfileBoundary(_),
            ..
        }
    ));
    Ok(())
}

fn refusal_of(
    outcome: Result<VerificationReport, VerificationError>,
) -> Result<(VerificationRefusal, ReferenceVerificationContext), Box<dyn Error>> {
    match outcome {
        Err(VerificationError::Refused {
            refusal,
            source: Some(source),
        }) => match *source {
            VerificationSource::Reference(ReferenceVerificationSource::Refusal(context)) => {
                Ok((refusal, context))
            }
            other => Err(format!("unexpected verification source: {other:?}").into()),
        },
        Err(failure) => Err(format!("missing reference refusal context: {failure:?}").into()),
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
