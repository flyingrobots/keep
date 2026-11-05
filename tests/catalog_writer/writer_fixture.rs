//! This module owns the writer suite's blobs and store witnesses.
//!
//! Store construction and successor publication belong to the shared
//! `tests/support` builder; this module fixes the suite's blobs and the
//! platform admission route. The library's interruption laws include this file
//! by path, so it names the crate as `keep` and reaches the builder through
//! `super`.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use keep::{AdmittedSegment, DurableStore, FilesystemCatalogPublisher, ReaderAttemptLimit};

use super::populated_store::{self, StoreAdmission};
pub(super) use super::populated_store::{TestResult, segment_policy};
pub(super) use super::successor_catalog::{
    Predecessor, admit_all, pool_segments, publish, republish,
};

/// The retained content every fixture store starts with.
pub(super) fn first_blob() -> Vec<u8> {
    b"keep version-two writer: first retained blob. ".repeat(3_000)
}

/// The content each law ingests through the writer.
pub(super) fn second_blob() -> Vec<u8> {
    b"KEEP V2 WRITER -- the second blob, ingested after the first catalog | ".repeat(2_500)
}

/// Opens the writer under the production platform profile.
pub(super) fn open_publisher(root: &Path) -> TestResult<FilesystemCatalogPublisher> {
    super::successor_catalog::open_publisher(root, StoreAdmission::PLATFORM)
}

/// Publishes a new generation-one retention root that anchors `content`.
pub(super) fn retain(root: &Path, namespace: &[u8], content: &[u8]) -> TestResult<()> {
    let _receipt = populated_store::retain(root, StoreAdmission::PLATFORM, namespace, &[content])?;
    Ok(())
}

/// Requires a fresh fenced view to reconstruct exactly `content`.
pub(super) fn assert_reads(root: &Path, content: &[u8]) -> TestResult<()> {
    let target = populated_store::identify(content)?.target;
    let store = DurableStore::open(
        root,
        populated_store::policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let mut output = Vec::new();
    let _receipt = store.reconstruct(target, &mut output)?;
    if output == content {
        Ok(())
    } else {
        Err("reconstructed bytes differ from the ingested content".into())
    }
}

/// Every store path, with the bytes of each regular file, in path order.
pub(super) type StoreWitness = Vec<(String, Option<Vec<u8>>)>;

/// Every path and regular-file byte string under `root`, in name order.
pub(super) fn witness(root: &Path) -> TestResult<StoreWitness> {
    let mut entries = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            let relative = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("non-UTF-8 store path")?
                .to_owned();
            if path.is_dir() {
                pending.push(path);
                entries.push((relative, None));
            } else {
                entries.push((relative, Some(fs::read(&path)?)));
            }
        }
    }
    entries.sort();
    Ok(entries)
}

/// Stages and seals `content` as `staging/current.seg` without publishing it.
///
/// Returns the exact sealed bytes. The closed stage remains for the caller.
pub(super) fn stage_only(
    publisher: &FilesystemCatalogPublisher,
    root: &Path,
    content: &[u8],
) -> TestResult<Vec<u8>> {
    let identified = populated_store::identify(content)?;
    let sealed = super::successor_catalog::stage_blob(publisher, content, &identified)?;
    drop(sealed);
    Ok(fs::read(root.join("staging/current.seg"))?)
}

/// The canonical segment-pool name of one admitted segment.
pub(super) fn segment_name(segment: &AdmittedSegment<'_>) -> TestResult<String> {
    let mut name = String::new();
    for byte in segment.digest().as_bytes() {
        write!(name, "{byte:02x}")?;
    }
    name.push_str(".seg");
    Ok(name)
}
