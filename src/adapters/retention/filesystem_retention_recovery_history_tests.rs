//! These laws own root-history refusal before recovered head publication.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, initial_root, manifest_pool_path,
    new_namespace_preparation, open_authority, retention_witness, root_pool_path,
    successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    CanonicalRetentionManifest, CanonicalRetentionRoot, FilesystemRetentionRecoveryError,
    RetentionPublicationPreparation, RetentionRecoveryRefusal,
};
use crate::{
    RetentionHead, RetentionManifest, RetentionManifestEntry, RetentionManifestLength,
    RetentionPolicy, RetentionRoot, execute_retention_publication,
};

// Size: medium. Oracle: a selected root admits only its exact next generation.
// Delete only with the protocol or a stronger runtime boundary replacement.
#[test]
fn recovered_head_refuses_a_skipped_root_generation() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("recovery-skipped-root-generation")?;
    let bytes = fixture(ROOT_HEX)?;
    let initial = AdmittedRetentionRoot::decode(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &initial_preparation(&bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("missing current state")?;
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let successor = successor_root(&initial)?;
    let preparation = successor_preparation(&initial, &manifest, successor.encoded())?;
    drive_publication(&mut authority, &preparation, 13)?;
    let skipped = RetentionRoot::new(
        initial.root().namespace().clone(),
        initial.root().generation().successor()?.successor()?,
        RetentionPolicy::new(initial.root().profile(), initial.root().limits()),
        Some(initial.digest()),
        initial.root().anchors().to_vec(),
    )?;
    let encoded = CanonicalRetentionRoot::from_root(&skipped)?;
    install_history(sandbox.path(), &preparation, &encoded)?;
    let before = retention_witness(sandbox.path())?;

    let result = authority.recover();

    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "skipped root generation must preserve every retained byte: {result:?}"
    );
    assert!(
        matches!(
            result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::RootNotSuccessor
            })
        ),
        "skipped root generation must refuse before head publication: {result:?}"
    );
    Ok(())
}

pub(super) fn install_history(
    root: &Path,
    preparation: &RetentionPublicationPreparation<'_>,
    encoded: &CanonicalRetentionRoot,
) -> Result<(), Box<dyn Error>> {
    let old = preparation.candidate();
    let candidate = AdmittedRetentionRoot::decode(encoded.encoded())?;
    let stage = root.join("retention/root.next");
    fs::write(&stage, encoded.encoded())?;
    fs::remove_file(root_pool_path(root, old))?;
    fs::hard_link(&stage, root_pool_path(root, &candidate))?;
    let publication = preparation.publication().ok_or("missing publication")?;
    let staged = AdmittedRetentionManifest::decode(publication.manifest().encoded())?;
    let entries = staged
        .manifest()
        .entries()
        .iter()
        .map(|entry| {
            if entry.namespace() == candidate.root().namespace().digest() {
                RetentionManifestEntry::new(
                    entry.namespace(),
                    candidate.root().generation(),
                    candidate.digest(),
                )
            } else {
                *entry
            }
        })
        .collect();
    let manifest = RetentionManifest::new(
        staged.manifest().generation(),
        staged.manifest().predecessor(),
        entries,
    )?;
    let canonical = CanonicalRetentionManifest::from_manifest(&manifest)?;
    let stage = root.join("retention/manifest.next");
    fs::write(&stage, canonical.encoded())?;
    fs::remove_file(manifest_pool_path(root, preparation))?;
    fs::hard_link(
        &stage,
        root.join("retention/manifests").join(pool_name::manifest(
            manifest.generation(),
            canonical.digest(),
        )),
    )?;
    let head = RetentionHead::new(
        manifest.generation(),
        RetentionManifestLength::new(u64::try_from(canonical.encoded().len())?)?,
        canonical.digest(),
        manifest.predecessor(),
    )?;
    fs::write(
        root.join("retention/head.next"),
        CanonicalRetentionHead::from_head(&head).encoded(),
    )?;
    Ok(())
}

// Size: medium. Oracle: the first publication starts at root generation one.
// Delete only with this protocol or a stronger runtime boundary replacement.
#[test]
fn recovered_initial_head_refuses_noninitial_root_history() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("recovery-noninitial-first-root")?;
    let bytes = fixture(ROOT_HEX)?;
    let initial = AdmittedRetentionRoot::decode(&bytes)?;
    let preparation = initial_preparation(&bytes)?;
    drive_publication(&mut authority, &preparation, 13)?;
    let noninitial = successor_root(&initial)?;
    install_history(sandbox.path(), &preparation, &noninitial)?;
    require_history_refusal(sandbox.path(), &mut authority)
}

// Size: medium. Oracle: a newly inserted namespace starts at root generation one.
// Delete only with this protocol or a stronger runtime boundary replacement.
#[test]
fn recovered_inserted_namespace_refuses_noninitial_root_history() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("recovery-noninitial-inserted-root")?;
    let bytes = fixture(ROOT_HEX)?;
    let template = AdmittedRetentionRoot::decode(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &initial_preparation(&bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("missing current state")?;
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let initial = initial_root(b"new-namespace", &template)?;
    let candidate = AdmittedRetentionRoot::decode(initial.encoded())?;
    let preparation = new_namespace_preparation(&manifest, initial.encoded())?;
    drive_publication(&mut authority, &preparation, 13)?;
    let noninitial = successor_root(&candidate)?;
    install_history(sandbox.path(), &preparation, &noninitial)?;
    require_history_refusal(sandbox.path(), &mut authority)
}

fn require_history_refusal(
    root: &Path,
    authority: &mut super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority,
) -> Result<(), Box<dyn Error>> {
    let before = retention_witness(root)?;
    let result = authority.recover();
    assert_eq!(
        retention_witness(root)?,
        before,
        "noninitial namespace history must preserve every retained byte: {result:?}"
    );
    assert!(
        matches!(
            result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::RootNotSuccessor
            })
        ),
        "noninitial namespace history must refuse before head publication: {result:?}"
    );
    Ok(())
}
