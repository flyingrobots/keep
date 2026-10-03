//! Missing, contradictory and unsupported blob evidence are distinct outcomes.

use super::{one_zero, support::require_error, with_records};
use keep::{
    AdmittedLayout, AdmittedSegmentRecord, BlobId, FastCdc, LayoutEntryLimit,
    RegisteredStorageProfile, RetentionClosureVerificationError as Closure, SegmentRecordIdentity,
    VerificationDepth as Depth, VerificationError, VerificationObservation as Observation,
    VerificationRefusal as Refusal, VerificationSource, VerificationSubject as Subject,
};
use std::error::Error;

// Size: small. Oracle: exact missing identity from the frozen layout.
// Delete when superseded by a stronger generated closure law.
#[test]
fn incomplete_blob_refuses_the_exact_missing_chunk() -> Result<(), Box<dyn Error>> {
    let layout = one_zero()?;
    let chunk = layout
        .entries()
        .first()
        .ok_or("fixture has no entry")?
        .chunk_id();
    let canonical = layout.encode_record()?;
    with_records(
        &[AdmittedSegmentRecord::for_layout(&canonical)?],
        |catalog| {
            for depth in [Depth::ChunkIdentity, Depth::CompleteBlobIdentity] {
                let error = require_error(
                    catalog.verify_blob(layout.target(), depth),
                    "incomplete blob was certified",
                )?;
                let VerificationError::Refused {
                    refusal,
                    source: Some(source),
                } = error
                else {
                    return Err("missing chunk lost its source".into());
                };
                assert_eq!(
                    refusal,
                    Refusal::Missing {
                        subject: Subject::Chunk { identity: chunk }
                    }
                );
                assert!(
                    matches!(*source, VerificationSource::Closure(Closure::MissingMember { identity: SegmentRecordIdentity::Chunk(observed) }) if observed == chunk),
                    "original exact missing-member cause must survive"
                );
            }
            Ok(())
        },
    )
}

// Size: small. Oracle: empty catalog contains no realization of a named blob.
// Delete when blob discovery changes its public absence contract.
#[test]
fn an_absent_blob_is_not_reported_as_corruption() -> Result<(), Box<dyn Error>> {
    let blob = BlobId::hash_bytes(&[0])?;
    with_records(&[], |catalog| {
        let error = require_error(
            catalog.verify_blob(blob, Depth::CompleteBlobIdentity),
            "absent blob certified",
        )?;
        assert!(
            matches!(error, VerificationError::Refused { refusal: Refusal::Missing { subject: Subject::Blob { identity } }, source: None } if identity == blob),
            "expected exact missing blob, got {error:?}"
        );
        Ok(())
    })
}

// Size: small. Oracle: one-zero chunk bytes cannot realize the one-one BlobId.
// Delete when a stronger generated reconstruction mismatch law subsumes this case.
#[test]
fn complete_blob_verification_preserves_expected_and_observed_identities()
-> Result<(), Box<dyn Error>> {
    let expected = BlobId::hash_bytes(&[1])?;
    let observed = BlobId::hash_bytes(&[0])?;
    let mut spans = Vec::new();
    let mut detector = FastCdc::new();
    detector.feed(&[0], |span| spans.push(span))?;
    spans.extend(detector.finish()?);
    let layout = AdmittedLayout::from_spans(
        expected,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let canonical = layout.encode_record()?;
    let records = [
        AdmittedSegmentRecord::for_chunk(&[0])?,
        AdmittedSegmentRecord::for_layout(&canonical)?,
    ];
    with_records(&records, |catalog| {
        let error = require_error(
            catalog.verify_blob(expected, Depth::CompleteBlobIdentity),
            "wrong logical identity certified",
        )?;
        let VerificationError::Refused {
            refusal,
            source: Some(source),
        } = error
        else {
            return Err("corruption lost its source".into());
        };
        assert_eq!(
            refusal,
            Refusal::Corrupt {
                subject: Subject::Blob { identity: expected },
                expected: Observation::Blob(expected),
                observed: Observation::Blob(observed)
            }
        );
        assert!(
            matches!(*source, VerificationSource::Closure(Closure::BlobIdentityMismatch { layout, expected: actual_expected, observed: actual_observed }) if layout == canonical.id() && actual_expected == expected && actual_observed == observed),
            "typed source must retain layout and both identities"
        );
        Ok(())
    })
}

// Size: small. Oracle: blob verification does not establish publication/retention.
// Delete when the supported subject/depth contract intentionally changes.
#[test]
fn blob_verification_refuses_unrelated_depths_before_discovery() -> Result<(), Box<dyn Error>> {
    let blob = BlobId::hash_bytes(&[0])?;
    with_records(&[], |catalog| {
        for requested in [
            Depth::CatalogReachability,
            Depth::RetentionClosure,
            Depth::SnapshotBinding,
        ] {
            let error = require_error(
                catalog.verify_blob(blob, requested),
                "unsupported depth certified",
            )?;
            assert!(
                matches!(error, VerificationError::Refused { refusal: Refusal::Unsupported { subject: Subject::Blob { identity }, requested: actual, supported }, source: None } if identity == blob && actual == requested && supported == [Depth::Framing, Depth::Checksum, Depth::ChunkIdentity, Depth::LayoutIdentity, Depth::CompleteBlobIdentity]),
                "exact policy refusal required: {error:?}"
            );
        }
        Ok(())
    })
}
