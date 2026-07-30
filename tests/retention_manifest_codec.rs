//! Public semantic and canonical-codec laws for retention manifests.

#[path = "retention_manifest_codec/refusal_laws.rs"]
mod refusal_laws;
mod support;

use std::io;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionManifest,
    LivenessGeneration, RetentionManifest, RetentionManifestEntry, RetentionManifestError,
};

pub(crate) const ONE_ANCHOR_ROOT: &str =
    include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
pub(crate) const ONE_ROOT_MANIFEST: &str =
    include_str!("../conformance/segment-store/v2/one-root-manifest.hex");
pub(crate) const ENTRY_SET_DIGEST_OFFSET: usize = 80;
pub(crate) const ENTRY_BODY_OFFSET: usize = 160;
pub(crate) const MANIFEST_DIGEST_OFFSET: usize = 232;
pub(crate) const CHECKSUM_OFFSET: usize = 264;

#[test]
fn one_root_manifest_has_one_semantic_and_canonical_representation()
-> Result<(), Box<dyn std::error::Error>> {
    let root_bytes = fixture_bytes(ONE_ANCHOR_ROOT)?;
    let root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let entry = RetentionManifestEntry::new(
        root.root().namespace().digest(),
        root.root().generation(),
        root.digest(),
    );
    let generation = LivenessGeneration::new(1)?;
    let manifest = RetentionManifest::new(generation, None, vec![entry])?;
    let canonical = CanonicalRetentionManifest::from_manifest(&manifest)?;
    let manifest_bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    assert_eq!(canonical.encoded(), manifest_bytes);

    let admitted = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    assert_eq!(admitted.encoded(), manifest_bytes);
    assert_eq!(admitted.manifest(), &manifest);
    assert_eq!(admitted.digest(), canonical.digest());
    assert_eq!(
        admitted.digest().as_bytes(),
        manifest_bytes
            .get(MANIFEST_DIGEST_OFFSET..MANIFEST_DIGEST_OFFSET + 32)
            .ok_or_else(|| io::Error::other("frozen manifest lacks its digest"))?
    );
    Ok(())
}

#[test]
fn manifest_history_and_namespace_set_are_admitted_canonically()
-> Result<(), Box<dyn std::error::Error>> {
    let entry = fixture_entry()?;
    let manifest_bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    let predecessor = AdmittedRetentionManifest::decode(&manifest_bytes)?.digest();
    assert!(matches!(
        RetentionManifest::new(LivenessGeneration::new(1)?, Some(predecessor), vec![entry]),
        Err(RetentionManifestError::InitialGenerationHasPredecessor { .. })
    ));
    assert!(matches!(
        RetentionManifest::new(LivenessGeneration::new(2)?, None, vec![entry]),
        Err(RetentionManifestError::MissingPredecessor { .. })
    ));
    assert!(matches!(
        RetentionManifest::new(
            LivenessGeneration::new(2)?,
            Some(predecessor),
            vec![entry, entry],
        ),
        Err(RetentionManifestError::DuplicateNamespace { .. })
    ));
    Ok(())
}

fn fixture_entry() -> Result<RetentionManifestEntry, Box<dyn std::error::Error>> {
    let root_bytes = fixture_bytes(ONE_ANCHOR_ROOT)?;
    let root = AdmittedRetentionRoot::decode(&root_bytes)?;
    Ok(RetentionManifestEntry::new(
        root.root().namespace().digest(),
        root.root().generation(),
        root.digest(),
    ))
}

pub(crate) fn fixture_bytes(fixture: &str) -> Result<Vec<u8>, io::Error> {
    let encoded = fixture
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("retention fixture lacks final newline"))?;
    support::decode_hex(encoded)
}
