//! Canonical but incorrectly selected roots cannot become verification evidence.

use super::filesystem_retention_pool_name as names;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, initial_root, open_authority, retention_witness,
    root_pool_path,
};
use crate::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot,
    FilesystemRetentionSnapshotError, ReaderAttemptLimit, RetentionHead, RetentionManifest,
    RetentionManifestEntry, RetentionManifestLength, RetentionNamespace,
    RetentionSelectedRootRefusal, SegmentReadPolicy, VerificationDepth, VerificationError,
    VerificationRefusal, VerificationSource, execute_retention_publication,
};
use std::error::Error;
use std::fs;

// Size: medium. Oracle: a manifest lookup by namespace cannot certify another
// namespace's canonical root, even when all selected bytes and digests agree.
// Delete if a stronger generated selection law retains this contradictory witness.
#[test]
fn a_canonical_foreign_root_cannot_certify_the_requested_namespace() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verification-foreign-root")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let original = root.root().namespace().digest();
    let namespace = RetentionNamespace::try_from(b"different-namespace".as_slice())?.digest();
    let manifest = RetentionManifest::new(
        preparation.liveness_generation(),
        None,
        vec![RetentionManifestEntry::new(
            namespace,
            root.root().generation(),
            root.digest(),
        )],
    )?;
    let manifest = CanonicalRetentionManifest::from_manifest(&manifest)?;
    let head = RetentionHead::new(
        preparation.liveness_generation(),
        RetentionManifestLength::new(u64::try_from(manifest.encoded().len())?)?,
        manifest.digest(),
        None,
    )?;
    let directory = sandbox
        .path()
        .join("retention/roots")
        .join(names::namespace(namespace));
    fs::create_dir(&directory)?;
    fs::write(
        directory.join(names::root(root.root().generation(), root.digest())),
        &bytes,
    )?;
    fs::write(
        sandbox
            .path()
            .join("retention/manifests")
            .join(names::manifest(head.generation(), head.manifest_digest())),
        manifest.encoded(),
    )?;
    fs::write(
        sandbox.path().join("retention/HEAD"),
        CanonicalRetentionHead::from_head(&head).encoded(),
    )?;
    let before = retention_witness(sandbox.path())?;
    let view = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let error = view
        .verify_retention(namespace, VerificationDepth::RetentionClosure)
        .err()
        .ok_or("foreign namespace was certified")?;
    let VerificationError::Refused {
        refusal: VerificationRefusal::Corrupt { .. },
        source: Some(source),
    } = error
    else {
        return Err("namespace contradiction lost its classification".into());
    };
    let VerificationSource::Retention(FilesystemRetentionSnapshotError::Root { source }) = *source
    else {
        return Err("namespace contradiction lost its original boundary".into());
    };
    assert_eq!(
        source
            .get_ref()
            .and_then(|source| source.downcast_ref::<RetentionSelectedRootRefusal>()),
        Some(&RetentionSelectedRootRefusal::Namespace {
            expected: namespace,
            observed: original
        }),
        "exact namespace coordinates must survive"
    );
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

// Size: medium. Oracle: canonical substitution still contradicts the manifest
// selection and must preserve exact expected/observed generation and digest.
// Delete if a stronger selected-root source-preservation law subsumes this case.
#[test]
fn a_selected_root_coordinate_refusal_keeps_its_typed_evidence() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verification-root-coordinate")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let expected = AdmittedRetentionRoot::decode(&bytes)?;
    let replacement = initial_root(b"coordinate-mismatch", &expected)?;
    let observed = AdmittedRetentionRoot::decode(replacement.encoded())?;
    fs::write(
        root_pool_path(sandbox.path(), &expected),
        replacement.encoded(),
    )?;
    let before = retention_witness(sandbox.path())?;
    let view =
        FilesystemRetentionSnapshot::load(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;
    let error = view
        .retained_root(expected.root().namespace().digest())
        .err()
        .ok_or("substituted root admitted")?;
    let FilesystemRetentionSnapshotError::Root { source } = error else {
        return Err("selected-root boundary changed".into());
    };
    assert_eq!(
        source
            .get_ref()
            .and_then(|source| source.downcast_ref::<RetentionSelectedRootRefusal>()),
        Some(&RetentionSelectedRootRefusal::Coordinate {
            expected_generation: expected.root().generation(),
            observed_generation: observed.root().generation(),
            expected_digest: expected.digest(),
            observed_digest: observed.digest()
        }),
        "a precise coordinate refusal must not be replaced with a message"
    );
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

fn policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    ))
}
