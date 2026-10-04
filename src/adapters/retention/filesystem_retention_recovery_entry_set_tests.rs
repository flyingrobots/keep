//! These laws own unrelated namespace preservation in recovered successors.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, initial_root, manifest_pool_path,
    new_namespace_preparation, open_authority, retention_witness, successor_preparation,
    successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    CanonicalRetentionManifest, FilesystemRetentionRecoveryError, RetentionRecoveryRefusal,
};
use crate::{
    RetentionHead, RetentionManifest, RetentionManifestEntry, RetentionManifestLength,
    execute_retention_publication,
};

#[derive(Clone, Copy)]
enum Change {
    Drop,
    Alter,
}
#[derive(Clone, Copy)]
enum Point {
    Manifest,
    Head,
}

// Size: medium. Oracle: only the staged root's namespace may change.
// Delete only with this protocol or a stronger boundary-level replacement.
#[test]
fn recovery_refuses_dropped_namespaces_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Change::Drop, Point::Manifest)
}
#[test]
fn recovery_refuses_dropped_namespaces_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Change::Drop, Point::Head)
}
#[test]
fn recovery_refuses_altered_namespaces_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Change::Alter, Point::Manifest)
}
#[test]
fn recovery_refuses_altered_namespaces_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Change::Alter, Point::Head)
}

fn require_refusal(change: Change, point: Point) -> Result<(), Box<dyn Error>> {
    let label = match (change, point) {
        (Change::Drop, Point::Manifest) => "drop-manifest",
        (Change::Drop, Point::Head) => "drop-head",
        (Change::Alter, Point::Manifest) => "alter-manifest",
        (Change::Alter, Point::Head) => "alter-head",
    };
    let (sandbox, mut authority) = open_authority(&format!("recovery-entry-set-{label}"))?;
    let root = fixture(ROOT_HEX)?;
    let admitted_root = AdmittedRetentionRoot::decode(&root)?;
    let _first = execute_retention_publication(&mut authority, &initial_preparation(&root)?)?;
    let first = authority
        .observe_current()?
        .ok_or("missing initial state")?;
    let first_manifest = AdmittedRetentionManifest::decode(first.manifest_bytes())?;
    let other = initial_root(b"unrelated", &admitted_root)?;
    let addition = new_namespace_preparation(&first_manifest, other.encoded())?;
    let _second = execute_retention_publication(&mut authority, &addition)?;
    let current = authority
        .observe_current()?
        .ok_or("missing multi-namespace state")?;
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let successor = successor_root(&admitted_root)?;
    let preparation = successor_preparation(&admitted_root, &manifest, successor.encoded())?;
    let prefix = match point {
        Point::Manifest => 8,
        Point::Head => 13,
    };
    drive_publication(&mut authority, &preparation, prefix)?;
    let publication = preparation.publication().ok_or("missing publication")?;
    let staged = AdmittedRetentionManifest::decode(publication.manifest().encoded())?;
    let candidate = preparation.candidate().root().namespace().digest();
    let entries = changed_entries(staged.manifest().entries(), candidate, change)?;
    let changed = RetentionManifest::new(
        staged.manifest().generation(),
        staged.manifest().predecessor(),
        entries,
    )?;
    let encoded = CanonicalRetentionManifest::from_manifest(&changed)?;
    install_changed_stage(sandbox.path(), &preparation, &changed, &encoded, point)?;
    let before = retention_witness(sandbox.path())?;

    let result = authority.recover();

    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "unrelated namespace refusal must preserve every retained byte for {label}: {result:?}"
    );
    assert!(
        matches!(
            result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::ManifestNotSuccessor
            })
        ),
        "unrelated namespace change must refuse before recovery effects: {result:?}"
    );
    Ok(())
}

fn changed_entries(
    entries: &[RetentionManifestEntry],
    candidate: crate::RetentionNamespaceDigest,
    change: Change,
) -> Result<Vec<RetentionManifestEntry>, Box<dyn Error>> {
    let mut changed = Vec::new();
    for entry in entries {
        if entry.namespace() == candidate {
            changed.push(*entry);
        } else if matches!(change, Change::Alter) {
            changed.push(RetentionManifestEntry::new(
                entry.namespace(),
                entry.root_generation().successor()?,
                entry.root_digest(),
            ));
        }
    }
    Ok(changed)
}

fn install_changed_stage(
    root: &Path,
    preparation: &super::RetentionPublicationPreparation<'_>,
    manifest: &RetentionManifest,
    encoded: &CanonicalRetentionManifest,
    point: Point,
) -> Result<(), Box<dyn Error>> {
    let stage = root.join("retention/manifest.next");
    fs::write(&stage, encoded.encoded())?;
    if matches!(point, Point::Head) {
        fs::remove_file(manifest_pool_path(root, preparation))?;
        let name = pool_name::manifest(manifest.generation(), encoded.digest());
        fs::hard_link(&stage, root.join("retention/manifests").join(name))?;
        let head = RetentionHead::new(
            manifest.generation(),
            RetentionManifestLength::new(u64::try_from(encoded.encoded().len())?)?,
            encoded.digest(),
            manifest.predecessor(),
        )?;
        fs::write(
            root.join("retention/head.next"),
            CanonicalRetentionHead::from_head(&head).encoded(),
        )?;
    }
    Ok(())
}
