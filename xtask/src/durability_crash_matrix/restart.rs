//! This module owns independent post-process-death store verification.

mod expectation;
mod semantic;

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::DurabilityCrashMatrixError;
use super::production_protocol::fixture::GoldenFixture;
use expectation::ExpectedStoreState;
use xtask::DurabilityCrashCase;

pub(super) fn verify(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let expected = ExpectedStoreState::for_case(case)?;
    let observed_paths = inventory(store_root)?;
    if observed_paths != expected.paths() {
        return Err(DurabilityCrashMatrixError::InventoryMismatch {
            expected: expected.paths(),
            observed: observed_paths,
        });
    }
    let segment = GoldenFixture::segment()?;
    let catalog = GoldenFixture::catalog()?;
    let head = GoldenFixture::head()?;
    for (relative, bytes) in expected.artifacts() {
        let observed = fs::read(store_root.join(relative))
            .map_err(|source| DurabilityCrashMatrixError::io("read crash artifact", source))?;
        let expected_bytes = bytes.resolve(&segment, &catalog, &head)?;
        if observed != expected_bytes {
            return Err(DurabilityCrashMatrixError::artifact_bytes(
                relative,
                expected_bytes,
                &observed,
            ));
        }
    }
    if let Some((source, target)) = expected.hard_link() {
        verify_hard_link(store_root, source, target)?;
    }
    semantic::verify(store_root, &expected)?;
    Ok(())
}

fn inventory(store_root: &Path) -> Result<BTreeSet<String>, DurabilityCrashMatrixError> {
    let mut paths = BTreeSet::new();
    let mut pending = vec![PathBuf::new()];
    while let Some(relative_parent) = pending.pop() {
        let parent = store_root.join(&relative_parent);
        let entries = fs::read_dir(parent)
            .map_err(|source| DurabilityCrashMatrixError::io("inventory crash store", source))?;
        for entry in entries {
            let entry = entry.map_err(|source| {
                DurabilityCrashMatrixError::io("read crash-store entry", source)
            })?;
            let relative = relative_parent.join(entry.file_name());
            let text = relative
                .to_str()
                .ok_or(DurabilityCrashMatrixError::NonUnicodeStatePath)?
                .into();
            if !paths.insert(text) {
                let path = relative
                    .to_str()
                    .ok_or(DurabilityCrashMatrixError::NonUnicodeStatePath)?
                    .into();
                return Err(DurabilityCrashMatrixError::RepeatedInventoryPath { path });
            }
            let file_type = entry.file_type().map_err(|source| {
                DurabilityCrashMatrixError::io("inspect crash-store entry", source)
            })?;
            if file_type.is_dir() {
                pending.push(relative);
            }
        }
    }
    Ok(paths)
}

fn verify_hard_link(
    store_root: &Path,
    source: &'static str,
    target: &'static str,
) -> Result<(), DurabilityCrashMatrixError> {
    let source_metadata = fs::metadata(store_root.join(source))
        .map_err(|error| DurabilityCrashMatrixError::io("inspect crash source link", error))?;
    let target_metadata = fs::metadata(store_root.join(target))
        .map_err(|error| DurabilityCrashMatrixError::io("inspect crash target link", error))?;
    if source_metadata.dev() == target_metadata.dev()
        && source_metadata.ino() == target_metadata.ino()
    {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::HardLinkIdentityMismatch {
            source,
            target,
            source_device: source_metadata.dev(),
            source_inode: source_metadata.ino(),
            target_device: target_metadata.dev(),
            target_inode: target_metadata.ino(),
        })
    }
}
