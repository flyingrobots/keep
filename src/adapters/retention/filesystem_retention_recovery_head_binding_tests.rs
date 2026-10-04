//! These laws own staged head coordinates bound to the exact staged manifest.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, head_path, initial_preparation, open_authority,
    retention_witness, successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    ChecksummedRetentionHead, FilesystemRetentionPublicationAuthority,
    FilesystemRetentionRecoveryError, RetentionRecoveryRefusal,
};
use crate::{RetentionHead, RetentionManifestLength, execute_retention_publication};

#[derive(Clone, Copy)]
enum Binding {
    Length,
    Predecessor,
}

// Size: medium. Oracle: head length must equal its selected manifest's bytes.
// Delete only with the protocol or a stronger boundary-level replacement.
#[test]
fn recovery_refuses_a_canonical_head_with_the_wrong_manifest_length() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("recovery-head-manifest-length")?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 13)?;
    let publication = preparation.publication().ok_or("missing publication")?;
    let head = ChecksummedRetentionHead::decode(publication.head().encoded())?;
    let mismatched = RetentionHead::new(
        head.head().generation(),
        RetentionManifestLength::MINIMUM,
        head.head().manifest_digest(),
        head.head().predecessor(),
    )?;
    fs::write(
        sandbox.path().join("retention/head.next"),
        CanonicalRetentionHead::from_head(&mismatched).encoded(),
    )?;

    require_binding_refusal(&mut authority, sandbox.path(), Binding::Length)
}

// Size: medium. Oracle: staged head and manifest must name one predecessor.
// Delete only with the protocol or a stronger boundary-level replacement.
#[test]
fn recovery_refuses_a_canonical_head_with_another_manifest_predecessor()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("recovery-head-manifest-predecessor")?;
    let root = fixture(ROOT_HEX)?;
    let _published = execute_retention_publication(&mut authority, &initial_preparation(&root)?)?;
    let current = authority
        .observe_current()?
        .ok_or("missing current generation")?;
    let admitted_root = AdmittedRetentionRoot::decode(&root)?;
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let successor = successor_root(&admitted_root)?;
    let preparation = successor_preparation(&admitted_root, &manifest, successor.encoded())?;
    drive_publication(&mut authority, &preparation, 13)?;
    let publication = preparation
        .publication()
        .ok_or("missing successor publication")?;
    let head = ChecksummedRetentionHead::decode(publication.head().encoded())?;
    let mismatched = RetentionHead::new(
        head.head().generation(),
        head.head().manifest_length(),
        head.head().manifest_digest(),
        Some(publication.manifest().digest()),
    )?;
    fs::write(
        sandbox.path().join("retention/head.next"),
        CanonicalRetentionHead::from_head(&mismatched).encoded(),
    )?;

    require_binding_refusal(&mut authority, sandbox.path(), Binding::Predecessor)
}

fn require_binding_refusal(
    authority: &mut FilesystemRetentionPublicationAuthority,
    root: &Path,
    binding: Binding,
) -> Result<(), Box<dyn Error>> {
    let head_before = read_head(root)?;
    let before = retention_witness(root)?;

    let result = authority.recover();

    assert_eq!(
        read_head(root)?,
        head_before,
        "head/manifest binding refusal must preserve the published head: {result:?}"
    );
    let correct = matches!(
        (binding, &result),
        (
            Binding::Length,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::HeadStageNamesOtherManifest
            })
        ) | (
            Binding::Predecessor,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::HeadPredecessorMismatch
            })
        )
    );
    assert!(
        correct,
        "head/manifest binding must refuse before execution: {result:?}"
    );
    assert_eq!(
        retention_witness(root)?,
        before,
        "head/manifest binding refusal must preserve every retained byte"
    );
    Ok(())
}

fn read_head(root: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(head_path(root)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(source),
    }
}
