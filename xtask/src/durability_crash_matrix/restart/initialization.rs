//! This module owns post-process-death verification of direct initialization.
//!
//! After the child dies at its coordinate, restart requires the exact entry
//! inventory and marker bytes that coordinate leaves and the exact admission
//! refusal or success, then runs the public initializer again as recovery and
//! requires the state it reports. It then reopens the store, publishes the
//! bundle as the first catalog through the writer, retains it, and
//! reads it back through the production durable reader.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use keep::{
    BlobId, CanonicalStoreFormatMarker, DurableStore, FilesystemNamespaceRefusal,
    FilesystemStoreAdmission, FilesystemStoreAdmissionError, ReaderAttemptLimit,
    StoreInitializationDirectory as Directory, StoreInitializationError,
    StoreInitializationPrefix as Prefix, StoreInitializationRefusal, execute_retention_publication,
};
use xtask::{
    DurabilityCrashCase, DurabilityCrashOccurrence, DurabilityCrashPoint as Point,
    DurabilityCrashPosition,
};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::crash_blob::CrashBlob;
use super::super::production_protocol::fixture::GoldenFixture;
use super::super::production_protocol::initialization::INCOMPLETE_STAGE_LENGTH;
use super::super::production_protocol::populated_store;
use super::super::production_protocol::retained_catalog::policy;
use super::super::production_protocol::retention::{preparation, reopened_authority};
use super::super::production_protocol::verification;
use super::inventory;

/// The protocol state one coordinate leaves behind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Empty,
    Locked,
    IncompleteStage,
    Staged,
    Fenced,
    Directories(usize),
    Linked,
    Committed,
}

pub(super) fn verify(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let state = state(case)?;
    require_inventory(store_root, state)?;
    require_marker_bytes(store_root, state)?;
    require_reopen(store_root, state)?;
    let resumed =
        match FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(store_root) {
            Ok(initialized) => Some(initialized.found()),
            Err(StoreInitializationError::Refused {
                refusal: StoreInitializationRefusal::AlreadyInitialized,
            }) => None,
            Err(source) => return Err(verification("recover interrupted initialization", source)),
        };
    let expected = resumes_from(state)?;
    if resumed != expected {
        return Err(DurabilityCrashMatrixError::InitializationPrefixMismatch {
            expected,
            observed: resumed,
        });
    }
    require_inventory(store_root, State::Committed)?;
    publish_and_read(store_root)
}

fn state(case: DurabilityCrashCase) -> Result<State, DurabilityCrashMatrixError> {
    use DurabilityCrashPosition::{After, Before, During};
    let position = case.position();
    Ok(match case.point() {
        Point::InitializeOpenAndLockWriterFile => match position {
            Before => State::Empty,
            During | After => State::Locked,
        },
        Point::InitializeWriteMarkerStage => match position {
            Before => State::Locked,
            During => State::IncompleteStage,
            After => State::Staged,
        },
        Point::InitializeSynchronizeMarkerStage
        | Point::InitializeSynchronizeRootAfterMarkerStage => State::Staged,
        Point::InitializeAdmitReaderFence => match position {
            Before => State::Staged,
            During | After => State::Fenced,
        },
        Point::InitializeAdmitDirectory => directory_state(case)?,
        Point::InitializeLinkMarker => match position {
            Before => State::Directories(Directory::ALL.len()),
            During | After => State::Linked,
        },
        Point::InitializeSynchronizeRootAfterMarker => State::Linked,
        Point::InitializeRemoveMarkerStage => match position {
            Before => State::Linked,
            During | After => State::Committed,
        },
        Point::InitializeSynchronizeRootAfterCleanup => State::Committed,
        Point::InitializeDiscardIncompleteMarkerStage => match position {
            Before => State::IncompleteStage,
            During | After => State::Locked,
        },
        point => return Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
    })
}

fn directory_state(case: DurabilityCrashCase) -> Result<State, DurabilityCrashMatrixError> {
    Ok(match case.position() {
        DurabilityCrashPosition::Before => State::Fenced,
        DurabilityCrashPosition::After => State::Directories(Directory::ALL.len()),
        DurabilityCrashPosition::During => {
            let occurrence = case.occurrence().map_or(0, DurabilityCrashOccurrence::get);
            let reached = usize::try_from(occurrence)
                .ok()
                .and_then(|ordinal| ordinal.checked_add(1))
                .ok_or(DurabilityCrashMatrixError::FixtureRange)?;
            State::Directories(reached)
        }
    })
}

fn resumes_from(state: State) -> Result<Option<Prefix>, DurabilityCrashMatrixError> {
    Ok(Some(match state {
        State::Empty => Prefix::Empty,
        State::Locked => Prefix::Locked,
        State::IncompleteStage => Prefix::IncompleteMarkerStage {
            length: INCOMPLETE_STAGE_LENGTH,
        },
        State::Staged => Prefix::MarkerStaged,
        State::Fenced => Prefix::ReaderFenced,
        State::Directories(count) => Prefix::Directories {
            last: *count
                .checked_sub(1)
                .and_then(|index| Directory::ALL.get(index))
                .ok_or(DurabilityCrashMatrixError::FixtureRange)?,
        },
        State::Linked => Prefix::MarkerLinked,
        State::Committed => return Ok(None),
    }))
}

fn expected_paths(state: State) -> BTreeSet<String> {
    let directories = match state {
        State::Directories(count) => count,
        State::Linked | State::Committed => Directory::ALL.len(),
        _ => 0,
    };
    let mut paths: BTreeSet<String> = Directory::ALL
        .iter()
        .take(directories)
        .map(|directory| directory.path().to_owned())
        .collect();
    let files: &[&str] = match state {
        State::Empty => &[],
        State::Locked => &["writer.lock"],
        State::IncompleteStage | State::Staged => &["writer.lock", "FORMAT.next"],
        State::Fenced | State::Directories(_) => &["writer.lock", "FORMAT.next", "reader.lock"],
        State::Linked => &["writer.lock", "FORMAT.next", "reader.lock", "FORMAT"],
        State::Committed => &["writer.lock", "reader.lock", "FORMAT"],
    };
    paths.extend(files.iter().map(|file| (*file).to_owned()));
    paths
}

fn require_inventory(store_root: &Path, state: State) -> Result<(), DurabilityCrashMatrixError> {
    let observed = inventory(store_root)?;
    let expected = expected_paths(state);
    if observed == expected {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::InventoryMismatch { expected, observed })
    }
}

/// Requires the stage and marker bytes, and one inode when both names exist.
fn require_marker_bytes(store_root: &Path, state: State) -> Result<(), DurabilityCrashMatrixError> {
    let marker = CanonicalStoreFormatMarker::version_two();
    let canonical = marker.encoded();
    let expected_stage = match state {
        State::IncompleteStage => canonical.get(..INCOMPLETE_STAGE_LENGTH),
        State::Staged | State::Fenced | State::Directories(_) | State::Linked => Some(canonical),
        _ => None,
    };
    if let Some(expected) = expected_stage {
        require_bytes(store_root, "FORMAT.next", expected)?;
    }
    if matches!(state, State::Linked | State::Committed) {
        require_bytes(store_root, "FORMAT", canonical)?;
    }
    if state == State::Linked {
        let stage_entry = metadata(store_root, "FORMAT.next")?;
        let marker_entry = metadata(store_root, "FORMAT")?;
        if (stage_entry.dev(), stage_entry.ino()) != (marker_entry.dev(), marker_entry.ino()) {
            return Err(mismatch("FORMAT and FORMAT.next are different inodes"));
        }
    }
    Ok(())
}

fn require_bytes(
    store_root: &Path,
    name: &'static str,
    expected: &[u8],
) -> Result<(), DurabilityCrashMatrixError> {
    let observed = fs::read(store_root.join(name))
        .map_err(|source| DurabilityCrashMatrixError::io("read initialization record", source))?;
    if observed == expected {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::artifact_bytes(
            name, expected, &observed,
        ))
    }
}

fn metadata(store_root: &Path, name: &str) -> Result<fs::Metadata, DurabilityCrashMatrixError> {
    fs::symlink_metadata(store_root.join(name))
        .map_err(|source| DurabilityCrashMatrixError::io("inspect initialization record", source))
}

/// Requires the exact admission outcome: refusal before commit, success after.
fn require_reopen(store_root: &Path, state: State) -> Result<(), DurabilityCrashMatrixError> {
    let reopened = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(store_root);
    let expected = match state {
        State::Empty => {
            return match reopened {
                Err(FilesystemStoreAdmissionError::WriterLock { .. }) => Ok(()),
                Ok(_) => Err(mismatch("an empty root reopened")),
                Err(other) => Err(mismatch(format!("empty root reopened as {other:?}"))),
            };
        }
        State::Committed => {
            return reopened
                .map(drop)
                .map_err(|source| verification("reopen committed store", source));
        }
        State::Locked => FilesystemNamespaceRefusal::MissingFormatMarker,
        _ => FilesystemNamespaceRefusal::InitializationIncomplete,
    };
    let observed = match &reopened {
        Err(FilesystemStoreAdmissionError::Namespace { source }) => source
            .get_ref()
            .and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>())
            .copied(),
        _ => None,
    };
    if observed == Some(expected) {
        Ok(())
    } else {
        Err(mismatch(format!(
            "uncommitted root reopened as {:?}, expected {expected:?}",
            reopened.err()
        )))
    }
}

/// Publishes and retains the bundle, then reconstructs its exact bytes.
fn publish_and_read(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let admission = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(store_root)
        .map_err(|source| verification("reopen recovered store", source))?;
    populated_store::publish_bundle_catalog(store_root, admission)?;
    populated_store::require_frozen_bundle(store_root)?;
    let mut authority = reopened_authority(store_root)?;
    let root = GoldenFixture::retention_root()?;
    let prepared = preparation(root.bytes())?;
    execute_retention_publication(&mut authority, &prepared)
        .map(drop)
        .map_err(|source| verification("retain recovered bundle", source))?;
    drop(authority);
    let store = DurableStore::open(store_root, policy()?, ReaderAttemptLimit::DEFAULT)
        .map_err(|source| verification("open recovered durable store", source))?;
    let bundle = CrashBlob::bundle()?;
    let mut output = Vec::new();
    store
        .reconstruct(bundle.target(), &mut output)
        .map(drop)
        .map_err(|source| verification("reconstruct recovered bundle", source))?;
    let observed = BlobId::hash_bytes(&output)
        .map_err(|source| verification("identify recovered bundle", source))?;
    if observed == bundle.target() && output == [0] {
        Ok(())
    } else {
        Err(mismatch("recovered bundle differs from its content"))
    }
}

fn mismatch(message: impl Into<String>) -> DurabilityCrashMatrixError {
    verification(
        "verify initialization crash recovery",
        io::Error::other(message.into()),
    )
}
