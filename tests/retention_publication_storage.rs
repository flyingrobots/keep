//! Retention publication storage-port laws.

#[path = "retention_publication_storage/recording_storage.rs"]
pub mod recording_storage;
mod support;

use std::error::Error;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    CanonicalRetentionManifest, ChecksummedRetentionHead, RetentionNamespaceAdmission,
    RetentionPublicationPhase, RetentionPublicationStorage,
};
use recording_storage::RecordingStorage;
use support::decode_hex;

const ROOT_HEX: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
const MANIFEST_HEX: &str = include_str!("../conformance/segment-store/v2/one-root-manifest.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v2/one-root-head.hex");

#[test]
fn storage_port_names_one_capability_per_publication_phase() -> Result<(), Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let manifest_bytes = fixture(MANIFEST_HEX)?;
    let head_bytes = fixture(HEAD_HEX)?;
    let root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let admitted_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let manifest = CanonicalRetentionManifest::from_manifest(admitted_manifest.manifest())?;
    let checksummed_head = ChecksummedRetentionHead::decode(&head_bytes)?;
    let head = CanonicalRetentionHead::from_head(checksummed_head.head());
    let mut storage = RecordingStorage::new();

    storage.write_root_stage(&root)?;
    storage.synchronize_root_stage()?;
    assert_eq!(
        storage.admit_root_namespace(&root)?,
        RetentionNamespaceAdmission::Created
    );
    storage.synchronize_roots_after_namespace()?;
    storage.link_root(&root)?;
    storage.synchronize_root_namespace(&root)?;
    storage.write_manifest_stage(&manifest)?;
    storage.synchronize_manifest_stage()?;
    storage.link_manifest(&manifest)?;
    storage.synchronize_manifest_pool()?;
    storage.write_head_stage(&head)?;
    storage.synchronize_head_stage()?;
    storage.replace_head()?;
    storage.synchronize_retention_namespace()?;
    storage.remove_root_stage()?;
    storage.remove_manifest_stage()?;
    storage.synchronize_cleanup()?;

    assert_eq!(storage.observed(), RetentionPublicationPhase::ALL);
    Ok(())
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
