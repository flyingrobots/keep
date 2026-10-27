//! Laws: every interrupted initialization state classifies exactly and resumes to one store.

use std::fs;
use std::sync::mpsc;
use std::thread;

use super::initialization_test_fixture::{
    COMMITTED, TestResult, initialize, initialize_until, inventory, recorded_boundaries,
    require_refusal,
};
use super::{
    StoreInitializationBoundary as Boundary, StoreInitializationError as Error,
    StoreInitializationObserver, StoreInitializationPhase as Phase,
    StoreInitializationPrefix as Prefix, StoreInitializationRefusal as Refusal,
    initialization_observation,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{
    CanonicalStoreFormatMarker, FilesystemStoreAdmission, FilesystemStoreAdmissionError,
    WriterLockAcquireError,
};

/// The state a root holds after exactly `completed` phases, by the protocol table.
fn model(completed: &[Phase]) -> Option<Prefix> {
    completed.iter().try_fold(Prefix::Empty, |state, phase| {
        Some(match phase {
            Phase::OpenAndLockWriterFile => Prefix::Locked,
            Phase::WriteMarkerStage => Prefix::MarkerStaged,
            Phase::AdmitReaderFence => Prefix::ReaderFenced,
            Phase::AdmitDirectory(last) => Prefix::Directories { last: *last },
            Phase::LinkMarker => Prefix::MarkerLinked,
            Phase::RemoveMarkerStage => return None,
            _ => state,
        })
    })
}

#[test]
fn a_fresh_root_commits_to_exactly_the_initialized_namespace() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-fresh")?;

    let initialized = initialize(sandbox.path())?;

    assert_eq!(initialized.found(), Prefix::Empty);
    let expected = COMMITTED.iter().map(|path| (*path).to_owned()).collect();
    assert_eq!(inventory(sandbox.path())?, expected);
    let marker = CanonicalStoreFormatMarker::version_two();
    assert_eq!(fs::read(sandbox.path().join("FORMAT"))?, marker.encoded());
    for lock in ["writer.lock", "reader.lock"] {
        assert_eq!(fs::metadata(sandbox.path().join(lock))?.len(), 0, "{lock}");
    }
    Ok(())
}

/// Every boundary of the forward protocol leaves a state the classifier names
/// exactly as the protocol table does, and resuming from it commits the same
/// store; past the cleanup boundary the store is committed and refuses.
#[test]
fn every_interrupted_boundary_classifies_and_resumes_to_one_store() -> TestResult<()> {
    let reference = TestDirectory::create("initialization-boundaries")?;
    let boundaries = recorded_boundaries(reference.path())?;
    let committed = inventory(reference.path())?;
    for (stop, (phase, boundary)) in boundaries.iter().enumerate() {
        let sandbox = TestDirectory::create("initialization-interrupted")?;
        let interrupted = initialize_until(sandbox.path(), stop);
        assert!(
            matches!(interrupted, Err(Error::Io { phase: failed, .. }) if failed == *phase),
            "boundary {stop} ({phase:?} {boundary:?}) did not fail at its phase"
        );
        let completed: Vec<Phase> = boundaries
            .iter()
            .take(stop.saturating_add(1))
            .filter(|(_, seen)| *seen == Boundary::After)
            .map(|(seen, _)| *seen)
            .collect();
        let expected = model(&completed);
        let resumed = initialize(sandbox.path());
        match expected {
            Some(state) => {
                let found = resumed.map_err(|error| format!("{phase:?} {boundary:?}: {error}"))?;
                let found = found.found();
                assert_eq!(found, state, "{phase:?} {boundary:?}");
            }
            None => require_refusal(resumed, &Refusal::AlreadyInitialized)?,
        }
        assert_eq!(
            inventory(sandbox.path())?,
            committed,
            "{phase:?} {boundary:?}"
        );
    }
    Ok(())
}

#[test]
fn an_incomplete_marker_stage_is_discarded_and_rewritten() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-incomplete-stage")?;
    fs::write(sandbox.path().join("writer.lock"), b"")?;
    let marker = CanonicalStoreFormatMarker::version_two();
    let prefix = marker
        .encoded()
        .get(..48)
        .ok_or("marker shorter than 48 bytes")?;
    fs::write(sandbox.path().join("FORMAT.next"), prefix)?;

    let initialized = initialize(sandbox.path())?;

    assert_eq!(
        initialized.found(),
        Prefix::IncompleteMarkerStage { length: 48 }
    );
    assert_eq!(fs::read(sandbox.path().join("FORMAT"))?, marker.encoded());
    Ok(())
}

#[test]
fn a_failed_commit_synchronization_leaves_an_unadmitted_resumable_store() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-commit-sync-failure")?;
    let observer: StoreInitializationObserver = Box::new(|phase, boundary| {
        if phase == Phase::SynchronizeRootAfterMarker && boundary == Boundary::Before {
            Err(std::io::Error::other("injected synchronization failure"))
        } else {
            Ok(())
        }
    });

    let failed = FilesystemStoreAdmission::initialize_observed_for_repository_tasks(
        sandbox.path(),
        observer,
    );

    assert!(matches!(
        failed,
        Err(Error::Io {
            phase: Phase::SynchronizeRootAfterMarker,
            ..
        })
    ));
    let refused = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path());
    assert!(matches!(
        refused,
        Err(FilesystemStoreAdmissionError::Namespace { .. })
    ));
    assert_eq!(initialize(sandbox.path())?.found(), Prefix::MarkerLinked);
    drop(FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?);
    Ok(())
}

/// A second attempt during the first is refused by writer authority, and once
/// the first commits, a retry is refused as already initialized.
#[test]
fn concurrent_attempts_have_exactly_one_initializer() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-concurrent")?;
    let root = sandbox.path().to_path_buf();
    let (reached, paused) = mpsc::sync_channel(0);
    let (resume, resumed) = mpsc::sync_channel::<()>(0);
    let first = thread::spawn(move || {
        let observer: StoreInitializationObserver = Box::new(move |phase, boundary| {
            if phase == Phase::AdmitReaderFence && boundary == Boundary::Before {
                reached.send(()).map_err(std::io::Error::other)?;
                resumed.recv().map_err(std::io::Error::other)?;
            }
            Ok(())
        });
        FilesystemStoreAdmission::initialize_observed_for_repository_tasks(&root, observer)
            .map(|initialized| initialized.found())
    });
    paused.recv()?;

    let contended = initialize(sandbox.path());

    resume.send(())?;
    let first = first.join().map_err(|_| "first initializer panicked")??;
    assert_eq!(first, Prefix::Empty);
    assert!(matches!(
        contended,
        Err(Error::WriterLock {
            source: WriterLockAcquireError::Busy
        })
    ));
    require_refusal(initialize(sandbox.path()), &Refusal::AlreadyInitialized)
}

#[test]
fn classification_of_a_committed_root_is_a_refusal_not_a_prefix() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-classify-committed")?;
    drop(initialize(sandbox.path())?);
    let root = cap_std::fs::Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;

    let observed = initialization_observation::observe(&root);

    assert!(matches!(
        observed,
        Err(Error::Refused {
            refusal: Refusal::AlreadyInitialized
        })
    ));
    Ok(())
}
