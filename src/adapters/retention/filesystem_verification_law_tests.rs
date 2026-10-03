//! Public verification outcomes over owned migrated filesystem witnesses.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, retention_witness, root_pool_path,
};
use crate::{
    AdmittedRetentionRoot, CatalogRestartByteLimit, CatalogRestartPolicy,
    FilesystemRetentionSnapshot, ReaderAttemptLimit, RetentionNamespace, SegmentReadPolicy,
    VerificationDepth as Depth, VerificationError, VerificationRefusal, VerificationSource,
    VerificationSubject, execute_retention_publication,
};
use std::error::Error;
use std::fs;

// Size: medium. Oracle: exact published corpus root/head and immutable bytes.
// Delete when superseded by a generated durable-report law retaining this case.
#[test]
fn published_root_reports_name_only_the_evidence_established_at_the_requested_depth()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verify-published-root")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let before = retention_witness(sandbox.path())?;
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let snapshot = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    for depth in [Depth::Framing, Depth::Checksum, Depth::RetentionClosure] {
        let report = snapshot.verify_retention(root.root().namespace().digest(), depth)?;
        assert_eq!(report.requested(), depth);
        assert_eq!(
            report.retention_head(),
            snapshot.retention_head().copied(),
            "publication provenance must survive verification"
        );
        assert_eq!(
            report.catalog(),
            (depth == Depth::RetentionClosure).then_some((
                snapshot.catalog().generation(),
                snapshot.catalog().catalog_digest()
            ))
        );
        let claims: Vec<_> = report
            .subjects()
            .iter()
            .map(|entry| (entry.subject(), entry.depth()))
            .collect();
        assert_eq!(
            claims,
            [(
                VerificationSubject::RetentionRoot {
                    namespace: root.root().namespace().digest(),
                    generation: root.root().generation(),
                    digest: root.digest()
                },
                depth
            )]
        );
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "verification must leave persisted evidence unchanged"
        );
    }
    Ok(())
}

// Size: medium. Oracle: the manifest contains no selected root for this namespace.
// Delete if an empty namespace acquires a different documented meaning.
#[test]
fn an_unretained_namespace_is_precisely_missing() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("verify-absent-namespace")?;
    drop(authority);
    let namespace = RetentionNamespace::try_from(b"absent".as_slice())?.digest();
    let before = retention_witness(sandbox.path())?;
    let snapshot = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let error = snapshot
        .verify_retention(namespace, Depth::RetentionClosure)
        .err()
        .ok_or("absent namespace certified")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Missing { subject: VerificationSubject::RetentionNamespace { namespace: actual } }, source: None } if actual == namespace),
        "expected exact absent namespace: {error:?}"
    );
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

// Size: medium. Oracle: a missing manifest-selected root is absence, not corruption.
// Delete when a stronger fault-injected selected-root law subsumes this case.
#[test]
fn a_missing_selected_root_preserves_its_original_filesystem_cause() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verify-missing-root")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    fs::remove_file(root_pool_path(sandbox.path(), &root))?;
    let before = retention_witness(sandbox.path())?;
    let snapshot = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let error = snapshot
        .verify_retention(root.root().namespace().digest(), Depth::RetentionClosure)
        .err()
        .ok_or("missing selected root certified")?;
    let VerificationError::Refused {
        refusal,
        source: Some(source),
    } = error
    else {
        return Err("missing selected root lost its classification or cause".into());
    };
    assert_eq!(
        refusal,
        VerificationRefusal::Missing {
            subject: VerificationSubject::RetentionNamespace {
                namespace: root.root().namespace().digest()
            }
        }
    );
    assert!(
        matches!(*source, VerificationSource::Retention(crate::FilesystemRetentionSnapshotError::Root { source }) if source.kind() == std::io::ErrorKind::NotFound)
    );
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

// Size: medium. Oracle: checksum delta from frozen selected bytes, with no repair.
// Delete if this exact corruption is covered by a stronger generated law.
#[test]
fn corrupted_root_verification_retains_exact_checksum_diagnostics() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verify-corrupt-root")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let offset = bytes.len().checked_sub(32).ok_or("no checksum")?;
    let expected: [u8; 32] = bytes.get(offset..).ok_or("no checksum")?.try_into()?;
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().ok_or("empty root")? ^= 1;
    let observed: [u8; 32] = corrupt.get(offset..).ok_or("no checksum")?.try_into()?;
    fs::write(root_pool_path(sandbox.path(), &root), &corrupt)?;
    let before = retention_witness(sandbox.path())?;
    let snapshot = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let error = snapshot
        .verify_retention(root.root().namespace().digest(), Depth::Checksum)
        .err()
        .ok_or("corrupt root certified")?;
    let VerificationError::Refused {
        refusal: VerificationRefusal::Corrupt { .. },
        source: Some(source),
    } = error
    else {
        return Err("corruption lost classification or source".into());
    };
    let VerificationSource::Retention(crate::FilesystemRetentionSnapshotError::Root { source }) =
        *source
    else {
        return Err("original boundary changed".into());
    };
    assert!(
        matches!(source.get_ref().and_then(|source| source.downcast_ref::<crate::RetentionRootDecodeError>()), Some(crate::RetentionRootDecodeError::ChecksumMismatch { expected: actual_expected, observed: actual_observed }) if *actual_expected == expected && *actual_observed == observed),
        "exact checksum coordinates must survive"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "corrupt evidence must remain unchanged"
    );
    Ok(())
}

fn policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    ))
}
