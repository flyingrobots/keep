//! This test module owns sandboxes, witnesses, and interrupting observers for initialization laws.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{
    VersionTwoInitialization, VersionTwoInitializationBoundary as Boundary,
    VersionTwoInitializationError, VersionTwoInitializationObserver,
    VersionTwoInitializationPhase as Phase, VersionTwoInitializationRefusal,
};
use crate::adapters::FilesystemVersionTwoAdmission;

pub(super) type TestResult<T> = Result<T, Box<dyn Error>>;

/// Every path of a freshly committed store, relative to its root.
pub(super) const COMMITTED: [&str; 12] = [
    "FORMAT",
    "catalogs",
    "gc",
    "reader.lock",
    "recovery",
    "recovery/dispositions",
    "retention",
    "retention/manifests",
    "retention/roots",
    "segments",
    "staging",
    "writer.lock",
];

pub(super) fn initialize(
    root: &Path,
) -> Result<VersionTwoInitialization, VersionTwoInitializationError> {
    FilesystemVersionTwoAdmission::initialize_unchecked_for_repository_tasks(root)
}

/// Initializes with an observer that fails the `stop`-th boundary notification.
///
/// The failed notification stops the protocol exactly where process death at
/// that boundary would, with every earlier effect in place.
pub(super) fn initialize_until(
    root: &Path,
    stop: usize,
) -> Result<VersionTwoInitialization, VersionTwoInitializationError> {
    let mut seen = 0_usize;
    let observer: VersionTwoInitializationObserver = Box::new(move |_phase, _boundary| {
        let index = seen;
        seen = seen.saturating_add(1);
        if index == stop {
            Err(io::Error::other("interrupted by the initialization law"))
        } else {
            Ok(())
        }
    });
    FilesystemVersionTwoAdmission::initialize_observed_for_repository_tasks(root, observer)
}

/// Records every boundary notification of one complete initialization.
pub(super) fn recorded_boundaries(root: &Path) -> TestResult<Vec<(Phase, Boundary)>> {
    let record = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&record);
    let observer: VersionTwoInitializationObserver = Box::new(move |phase, boundary| {
        sink.lock()
            .map_err(|_| io::Error::other("boundary record poisoned"))?
            .push((phase, boundary));
        Ok(())
    });
    drop(FilesystemVersionTwoAdmission::initialize_observed_for_repository_tasks(root, observer)?);
    let boundaries = record
        .lock()
        .map_err(|_| "boundary record poisoned")?
        .clone();
    Ok(boundaries)
}

/// Every path below `root`, relative and sorted.
pub(super) fn inventory(root: &Path) -> TestResult<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            let relative = path.strip_prefix(root)?.to_str().ok_or("non-UTF-8 path")?;
            paths.insert(relative.to_owned());
            if fs::symlink_metadata(&path)?.is_dir() {
                pending.push(path);
            }
        }
    }
    Ok(paths)
}

/// One path and, for a regular file, its bytes.
pub(super) type WitnessEntry = (String, Option<Vec<u8>>);

/// Every path of the root with its bytes, for refusals that must not mutate.
pub(super) fn witness(root: &Path) -> TestResult<Vec<WitnessEntry>> {
    inventory(root)?
        .into_iter()
        .map(|relative| {
            let path = root.join(&relative);
            let bytes = if fs::symlink_metadata(&path)?.is_file() {
                Some(fs::read(&path)?)
            } else {
                None
            };
            Ok((relative, bytes))
        })
        .collect()
}

/// Requires `result` to be exactly `expected` as an initialization refusal.
pub(super) fn require_refusal(
    result: Result<VersionTwoInitialization, VersionTwoInitializationError>,
    expected: &VersionTwoInitializationRefusal,
) -> TestResult<()> {
    match result {
        Err(VersionTwoInitializationError::Refused { refusal }) if &refusal == expected => Ok(()),
        Err(other) => Err(format!("expected {expected:?}, got {other:?}").into()),
        Ok(_) => Err(format!("expected {expected:?}, but initialization admitted").into()),
    }
}
