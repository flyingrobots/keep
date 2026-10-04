//! Medium filesystem laws: recovery admits all residue before its first mutation.
//! Oracle: exact public corruption causes and unchanged complete-store byte witnesses.
//! Delete only if compaction recovery is removed or stronger public laws subsume these.

#![cfg(target_os = "linux")]

#[allow(
    dead_code,
    reason = "shared fixture serves several independent runtime laws"
)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "shared fixture keeps sibling visibility at different module depths"
)]
#[path = "golden_file_worldline/durable_fixture.rs"]
mod durable_fixture;
#[allow(dead_code, reason = "shared sandbox also offers explicit teardown")]
#[expect(
    clippy::redundant_pub_crate,
    reason = "shared sandbox keeps sibling visibility at different module depths"
)]
#[path = "segment_filesystem_stage/sandbox.rs"]
mod durable_sandbox;

use keep::{CatalogRestartError, PublicationHeadDecodeError, recover_compaction};
type StoreWitness = BTreeMap<PathBuf, Vec<u8>>;
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn corrupt_later_head_preserves_earlier_discardable_stages() -> Result<(), Box<dyn Error>> {
    let store = durable_fixture::build("compaction-later-corrupt", &[b"retained"])?;
    fs::write(store.path().join("staging/current.seg"), [])?;
    fs::write(store.path().join("staging/current.cat"), [])?;
    let bad = corrupt_head(&fs::read(store.path().join("HEAD"))?)?;
    fs::write(store.path().join("head.next"), &bad)?;
    let before = witness(store.path())?;

    let error = recover_compaction(store.path(), durable_fixture::policy()?)
        .err()
        .ok_or("corrupt candidate admitted")?;
    assert_eq!(
        find_cause::<PublicationHeadDecodeError>(&error)?,
        &PublicationHeadDecodeError::InvalidMagic {
            observed: bad.get(..16).ok_or("magic missing")?.try_into()?,
        }
    );
    unchanged(store.path(), &before)?;
    Ok(())
}

#[test]
fn corrupt_current_head_is_not_reinterpreted_as_an_uninitialized_store()
-> Result<(), Box<dyn Error>> {
    let store = durable_fixture::build("compaction-current-corrupt", &[b"retained"])?;
    publish_successor(store.path())?;
    let head = fs::read(store.path().join("HEAD"))?;
    fs::write(store.path().join("head.next"), &head)?;
    let bad = corrupt_head(&head)?;
    fs::write(store.path().join("HEAD"), &bad)?;
    let before = witness(store.path())?;

    let error = recover_compaction(store.path(), durable_fixture::policy()?)
        .err()
        .ok_or("corrupt current head admitted")?;
    let current = find_cause::<CatalogRestartError>(&error)?;
    assert!(
        matches!(current, CatalogRestartError::Head { source }
        if *source == PublicationHeadDecodeError::InvalidMagic {
            observed: bad.get(..16).ok_or("magic missing")?.try_into()?,
        }),
        "current-head decoder cause must survive: {current:?}"
    );
    unchanged(store.path(), &before)?;
    Ok(())
}

fn publish_successor(root: &Path) -> Result<(), Box<dyn Error>> {
    use keep::{
        AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationExpectation,
        FilesystemCatalogPublisher, FilesystemCatalogSnapshot, FilesystemVersionTwoAdmission,
        SegmentPublication, publish_catalog_generation,
    };
    let policy = durable_fixture::policy()?;
    let current = FilesystemCatalogSnapshot::load(root, policy)?;
    let snapshot = current.snapshot()?;
    let bytes = fs::read_dir(root.join("segments"))?
        .map(|entry| fs::read(entry?.path()))
        .collect::<Result<Vec<_>, _>>()?;
    let segments = bytes
        .iter()
        .map(|bytes| AdmittedSegment::decode(bytes, keep::SegmentReadPolicy::MAXIMUM))
        .collect::<Result<Vec<_>, _>>()?;
    let successor = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(snapshot.catalog_digest()),
        &segments,
    )?;
    let mut publisher = FilesystemCatalogPublisher::open_version_two(
        FilesystemVersionTwoAdmission::reopen(root)?,
        policy,
    )?;
    let _receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::successor_of(&snapshot),
        SegmentPublication::none(),
        &successor,
        &segments,
    )?;
    Ok(())
}

fn unchanged(root: &Path, before: &StoreWitness) -> Result<(), Box<dyn Error>> {
    let after = witness(root)?;
    assert_eq!(
        after.keys().collect::<Vec<_>>(),
        before.keys().collect::<Vec<_>>(),
        "preflight refusal must preserve every earlier stage before recovery effects"
    );
    for (path, bytes) in before {
        assert_eq!(
            after.get(path),
            Some(bytes),
            "refusal changed evidence at {}",
            path.display()
        );
    }
    Ok(())
}

fn corrupt_head(head: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = head.to_vec();
    *bytes.first_mut().ok_or("head is empty")? ^= 1;
    Ok(bytes)
}

fn find_cause<'a, T: Error + 'static>(
    error: &'a (dyn Error + 'static),
) -> Result<&'a T, Box<dyn Error>> {
    let mut cause = error;
    loop {
        if let Some(found) = cause.downcast_ref::<T>() {
            return Ok(found);
        }
        cause = cause
            .source()
            .ok_or_else(|| format!("required typed cause is absent: {error:?}"))?;
    }
}

fn witness(root: &Path) -> Result<StoreWitness, Box<dyn Error>> {
    let mut paths = vec![root.to_path_buf()];
    let mut bytes = BTreeMap::new();
    while let Some(directory) = paths.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                paths.push(path);
            } else {
                bytes.insert(path.strip_prefix(root)?.to_path_buf(), fs::read(path)?);
            }
        }
    }
    assert!(
        !bytes.is_empty(),
        "complete store witness must not be vacuous"
    );
    Ok(bytes)
}
