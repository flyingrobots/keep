//! This test module owns one migrated version-2 retention publication fixture.

use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionRoot,
    RetentionPublicationPreparation, RetentionTransitionDisposition,
};
use crate::LayoutEntryLimit;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    AdmittedCatalog, AdmittedSegment, CatalogSnapshot, ChecksummedCatalog,
    ChecksummedPublicationHead, FilesystemPlatformAdmission, FilesystemStoreMigrationAuthority,
    SegmentReadPolicy, SegmentRecordLimit,
};
use crate::{
    RetentionGenerationExpectation, RetentionNamespace, RetentionPolicy, RetentionRoot,
    RootGeneration, execute_store_migration, preflight_retention_transition,
    prepare_retention_publication,
};

/// Frozen canonical generation-one root.
pub(super) const ROOT_HEX: &str =
    include_str!("../../../conformance/segment-store/v2/one-anchor-root.hex");
/// Frozen canonical generation-one manifest.
pub(super) const MANIFEST_HEX: &str =
    include_str!("../../../conformance/segment-store/v2/one-root-manifest.hex");
/// Frozen canonical generation-one retention head.
pub(super) const HEAD_HEX: &str =
    include_str!("../../../conformance/segment-store/v2/one-root-head.hex");

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const CATALOG_HEAD_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-head.hex");

const SEGMENT_NAME: &str = "221f6745cd8a5221c9a87c3707593608479282b54a4a74d0e753fd76f70e8db2.seg";
const CATALOG_NAME: &str =
    "0000000000000001-0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811.cat";

/// Builds one migrated version-2 store and pins its retention authority.
///
/// The fixture publishes the exact bundle version-1 corpus, executes the
/// complete forward migration, releases writer authority, then reopens the
/// admitted root for retention publication.
pub(super) fn open_authority(
    name: &str,
) -> Result<(TestDirectory, FilesystemRetentionPublicationAuthority), Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    let admission =
        FilesystemPlatformAdmission::reopen_version_two_unchecked_for_tests(sandbox.path())?;
    let authority = FilesystemRetentionPublicationAuthority::open(admission)?;
    Ok((sandbox, authority))
}

/// Decodes one LF-terminated lowercase hexadecimal conformance fixture.
pub(super) fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

/// Runs one operation against the frozen bundle catalog snapshot.
pub(super) fn with_snapshot<T>(
    operation: impl FnOnce(&CatalogSnapshot<'_, '_, '_>) -> T,
) -> Result<T, Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let head_bytes = fixture(CATALOG_HEAD_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog: AdmittedCatalog<'_, '_> =
        ChecksummedCatalog::decode(&catalog_bytes)?.admit(&segments)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;
    Ok(operation(&snapshot))
}

/// Prepares the frozen generation-one root as an initial `Publish` transition.
pub(super) fn initial_preparation(
    root_bytes: &[u8],
) -> Result<RetentionPublicationPreparation<'_>, Box<dyn Error>> {
    let candidate = AdmittedRetentionRoot::decode(root_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;
    let preparation = prepare_retention_publication(preflight, None)?;
    assert_eq!(
        preparation.disposition(),
        RetentionTransitionDisposition::Publish
    );
    Ok(preparation)
}

/// Prepares `candidate_bytes` as the exact successor of `current` under
/// `current_manifest`.
pub(super) fn successor_preparation<'encoded>(
    current: &AdmittedRetentionRoot<'_>,
    current_manifest: &AdmittedRetentionManifest<'_>,
    candidate_bytes: &'encoded [u8],
) -> Result<RetentionPublicationPreparation<'encoded>, Box<dyn Error>> {
    let candidate = AdmittedRetentionRoot::decode(candidate_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Current(current.root().generation()),
            Some(current),
            candidate,
            snapshot,
        )
    })??;
    prepare_retention_publication(preflight, Some(current_manifest)).map_err(Into::into)
}

/// Builds a generation-one root for another namespace from `template`'s policy.
pub(super) fn initial_root(
    namespace: &[u8],
    template: &AdmittedRetentionRoot<'_>,
) -> Result<CanonicalRetentionRoot, Box<dyn Error>> {
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(namespace)?,
        RootGeneration::INITIAL,
        RetentionPolicy::new(template.root().profile(), template.root().limits()),
        None,
        template.root().anchors().to_vec(),
    )?;
    CanonicalRetentionRoot::from_root(&root).map_err(Into::into)
}

/// Prepares `candidate_bytes` as a new namespace inserted into `current_manifest`.
pub(super) fn new_namespace_preparation<'encoded>(
    current_manifest: &AdmittedRetentionManifest<'_>,
    candidate_bytes: &'encoded [u8],
) -> Result<RetentionPublicationPreparation<'encoded>, Box<dyn Error>> {
    let candidate = AdmittedRetentionRoot::decode(candidate_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;
    prepare_retention_publication(preflight, Some(current_manifest)).map_err(Into::into)
}

/// Builds the exact semantic successor of one admitted root.
pub(super) fn successor_root(
    current: &AdmittedRetentionRoot<'_>,
) -> Result<CanonicalRetentionRoot, Box<dyn Error>> {
    let root = RetentionRoot::new(
        current.root().namespace().clone(),
        current.root().generation().successor()?,
        RetentionPolicy::new(current.root().profile(), current.root().limits()),
        Some(current.digest()),
        current.root().anchors().to_vec(),
    )?;
    CanonicalRetentionRoot::from_root(&root).map_err(Into::into)
}

pub(super) fn head_path(root: &Path) -> PathBuf {
    root.join("retention").join("HEAD")
}

pub(super) fn root_pool_path(root: &Path, candidate: &AdmittedRetentionRoot<'_>) -> PathBuf {
    root.join("retention")
        .join("roots")
        .join(hex(candidate.root().namespace().digest().as_bytes()))
        .join(format!(
            "{:016x}-{}.root",
            candidate.root().generation().get(),
            hex(candidate.digest().as_bytes())
        ))
}

pub(super) fn manifest_pool_path(
    root: &Path,
    preparation: &RetentionPublicationPreparation<'_>,
) -> PathBuf {
    root.join("retention").join("manifests").join(format!(
        "{:016x}-{}.manifest",
        preparation.liveness_generation().get(),
        hex(preparation.manifest_digest().as_bytes())
    ))
}

/// Every regular file beneath `retention`, with its exact bytes.
pub(super) fn retention_witness(root: &Path) -> io::Result<BTreeSet<(OsString, Vec<u8>)>> {
    let mut witness = BTreeSet::new();
    collect(&root.join("retention"), &mut witness)?;
    Ok(witness)
}

pub(super) const fn initial_generation() -> RootGeneration {
    RootGeneration::INITIAL
}

fn collect(directory: &Path, witness: &mut BTreeSet<(OsString, Vec<u8>)>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, witness)?;
        } else {
            witness.insert((path.into_os_string(), fs::read(entry.path())?));
        }
    }
    Ok(())
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().fold(String::new(), |mut rendered, byte| {
        let _ = write!(rendered, "{byte:02x}");
        rendered
    })
}

/// Builds one completely migrated version-2 store with writer authority released.
pub(super) fn migrated_store(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;
    write_version_one(&sandbox)?;
    let mut authority = FilesystemStoreMigrationAuthority::open(admission, maximum_policy())?;
    let intent = authority.observe_intent()?;
    let _receipt = execute_store_migration(&mut authority, &intent)?;
    drop(authority);
    Ok(sandbox)
}

fn write_version_one(sandbox: &TestDirectory) -> Result<(), Box<dyn Error>> {
    fs::write(
        sandbox.path().join("segments").join(SEGMENT_NAME),
        fixture(SEGMENT_HEX)?,
    )?;
    fs::write(
        sandbox.path().join("catalogs").join(CATALOG_NAME),
        fixture(CATALOG_HEX)?,
    )?;
    fs::write(sandbox.path().join("HEAD"), fixture(CATALOG_HEAD_HEX)?)?;
    Ok(())
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
