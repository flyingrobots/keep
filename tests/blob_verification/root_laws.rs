//! Root-report evidence is not manufactured from canonical framing alone.

use super::{one_zero, support::require_error, with_records};
use keep::{
    AdmittedRetentionRoot, AdmittedSegmentRecord, CanonicalRetentionRoot,
    RegisteredRetentionProfile, RetentionAnchor, RetentionClosureLimits,
    RetentionClosureVerificationError as Closure, RetentionNamespace, RetentionPolicy,
    RetentionRoot, RootGeneration, VerificationDepth as Depth, VerificationError,
    VerificationRefusal as Refusal, VerificationSource, VerificationSubject as Subject,
};
use std::error::Error;

// Size: small. Oracle: one-zero realization and exact canonical root/catalog.
// Delete if closure reports are removed or a stronger boundary law subsumes it.
#[test]
fn a_root_report_requires_complete_closure_in_its_named_catalog() -> Result<(), Box<dyn Error>> {
    let layout = one_zero()?;
    let canonical = layout.encode_record()?;
    let root = root(RetentionClosureLimits::new(2, 2, 220, 509)?)?;
    let admitted = AdmittedRetentionRoot::decode(root.encoded())?;
    let records = [
        AdmittedSegmentRecord::for_chunk(&[0])?,
        AdmittedSegmentRecord::for_layout(&canonical)?,
    ];
    with_records(&records, |catalog| {
        for depth in [Depth::Framing, Depth::Checksum, Depth::RetentionClosure] {
            let report = admitted.verify(catalog, depth)?;
            assert_eq!(report.requested(), depth);
            assert_eq!(
                report.catalog(),
                Some((catalog.generation(), catalog.catalog_digest()))
            );
            let claims: Vec<_> = report
                .subjects()
                .iter()
                .map(|entry| (entry.subject(), entry.depth()))
                .collect();
            assert_eq!(
                claims,
                [(
                    Subject::RetentionRoot {
                        namespace: admitted.root().namespace().digest(),
                        generation: RootGeneration::new(1)?,
                        digest: root.digest()
                    },
                    depth
                )],
                "only this exact root may receive the requested evidence"
            );
        }
        Ok(())
    })
}

// Size: small. Oracle: canonical root bytes do not imply available members.
// Delete if root admission itself gains full catalog closure as a contract.
#[test]
fn canonical_root_bytes_do_not_certify_an_absent_layout() -> Result<(), Box<dyn Error>> {
    let root = root(RetentionClosureLimits::new(2, 2, 220, 509)?)?;
    let admitted = AdmittedRetentionRoot::decode(root.encoded())?;
    let layout = one_zero()?.encode_record()?.id();
    with_records(&[], |catalog| {
        let shallow = admitted.verify(catalog, Depth::Checksum)?;
        assert_eq!(
            shallow
                .subjects()
                .first()
                .ok_or("missing root claim")?
                .depth(),
            Depth::Checksum
        );
        let error = require_error(
            admitted.verify(catalog, Depth::RetentionClosure),
            "absent closure certified",
        )?;
        assert!(
            matches!(error, VerificationError::Refused { refusal: Refusal::Missing { subject: Subject::Layout { identity } }, .. } if identity == layout),
            "exact missing layout required: {error:?}"
        );
        Ok(())
    })
}

// Size: small. Oracle: an admitted resource cap is not evidence of corruption.
// Delete if resource failures are replaced by a stronger typed contract.
#[test]
fn a_closure_budget_failure_is_operational_with_its_original_limit() -> Result<(), Box<dyn Error>> {
    let layout = one_zero()?.encode_record()?;
    let root = root(RetentionClosureLimits::new(1, 2, 220, 509)?)?;
    let admitted = AdmittedRetentionRoot::decode(root.encoded())?;
    let records = [
        AdmittedSegmentRecord::for_chunk(&[0])?,
        AdmittedSegmentRecord::for_layout(&layout)?,
    ];
    with_records(&records, |catalog| {
        let error = require_error(
            admitted.verify(catalog, Depth::RetentionClosure),
            "resource limit ignored",
        )?;
        let VerificationError::Operational { source } = error else {
            return Err("resource refusal misclassified as content evidence".into());
        };
        assert!(
            matches!(
                *source,
                VerificationSource::Closure(Closure::LimitExceeded {
                    counter: keep::RetentionClosureCounter::Nodes,
                    maximum: 1,
                    observed: 2
                })
            ),
            "exact limit coordinates must survive"
        );
        Ok(())
    })
}

fn root(limits: RetentionClosureLimits) -> Result<CanonicalRetentionRoot, Box<dyn Error>> {
    let layout = one_zero()?.encode_record()?;
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(b"verification".as_slice())?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            limits,
        ),
        None,
        vec![RetentionAnchor::new(one_zero()?.target(), layout.id())],
    )?;
    Ok(CanonicalRetentionRoot::from_root(&root)?)
}

// Size: small. Oracle: root closure does not establish unrelated subject depths.
// Delete when this explicit supported-depth matrix is intentionally replaced.
#[test]
fn root_reports_refuse_unrelated_depths() -> Result<(), Box<dyn Error>> {
    let root = root(RetentionClosureLimits::new(2, 2, 220, 509)?)?;
    let admitted = AdmittedRetentionRoot::decode(root.encoded())?;
    with_records(&[], |catalog| {
        for requested in [
            Depth::ChunkIdentity,
            Depth::LayoutIdentity,
            Depth::CompleteBlobIdentity,
            Depth::CatalogReachability,
            Depth::SnapshotBinding,
        ] {
            let error = require_error(
                admitted.verify(catalog, requested),
                "unsupported root evidence certified",
            )?;
            assert!(
                matches!(error, VerificationError::Refused { refusal: Refusal::Unsupported { subject: Subject::RetentionRoot { namespace, generation, digest }, requested: actual, supported }, source: None } if namespace == admitted.root().namespace().digest() && generation == RootGeneration::new(1)? && digest == root.digest() && actual == requested && supported == [Depth::Framing, Depth::Checksum, Depth::RetentionClosure]),
                "exact root policy refusal required: {error:?}"
            );
        }
        Ok(())
    })
}
