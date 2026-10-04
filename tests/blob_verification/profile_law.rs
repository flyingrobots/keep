//! Registered storage-profile evidence must be replayed before certification.

use super::{support::require_error, with_records};
use keep::{
    AdmittedLayout, AdmittedSegmentRecord, LayoutDecodePolicy, LayoutEntryLimit,
    RetentionClosureVerificationError as Closure, VerificationDepth, VerificationError,
    VerificationObservation, VerificationRefusal, VerificationSource,
};
use std::error::Error;

// Size: small. Oracle: retained canonical mutation corpus specifies a 262143
// boundary where FAST_CDC_64K_V1 must emit 262144 for this all-zero stream.
// Delete when replaced by a generated profile-conformance law with this witness.
#[test]
fn correct_blob_bytes_do_not_certify_false_profile_boundaries() -> Result<(), Box<dyn Error>> {
    let mutation = super::layout_mutation_support::mutation_cases()?
        .into_iter()
        .find(|candidate| candidate.case() == "profile-boundary-mismatch")
        .ok_or("profile witness absent")?;
    let encoded = mutation.mutated_record()?;
    let layout = AdmittedLayout::decode_record(
        &encoded,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
    )?;
    let canonical = layout.encode_record()?;
    let first = vec![0_u8; 262_143];
    let last = [0_u8; 2];
    let records = [
        AdmittedSegmentRecord::for_chunk(&first)?,
        AdmittedSegmentRecord::for_chunk(&last)?,
        AdmittedSegmentRecord::for_layout(&canonical)?,
    ];
    with_records(&records, |catalog| {
        let error = require_error(
            catalog.verify_blob(layout.target(), VerificationDepth::CompleteBlobIdentity),
            "false profile certified",
        )?;
        let VerificationError::Refused {
            refusal:
                VerificationRefusal::Corrupt {
                    expected: VerificationObservation::ProfileBoundary(Some(expected)),
                    observed: VerificationObservation::ProfileBoundary(Some(observed)),
                    ..
                },
            source: Some(source),
        } = error
        else {
            return Err("profile contradiction lost its classification or source".into());
        };
        assert_eq!(
            (expected.offset().get(), expected.length().get()),
            (0, 262_143)
        );
        assert_eq!(
            (observed.offset().get(), observed.length().get()),
            (0, 262_144)
        );
        assert!(
            matches!(*source, VerificationSource::Closure(Closure::ProfileBoundaryMismatch { layout, index: 0, expected: Some(source_expected), observed: Some(source_observed) }) if layout == canonical.id() && source_expected == expected && source_observed == observed),
            "typed original boundary coordinates must survive"
        );
        Ok(())
    })
}
