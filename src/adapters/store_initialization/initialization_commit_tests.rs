//! Laws: the commit point is one root synchronization between the marker link and stage removal.
//!
//! A real directory synchronization leaves nothing a test can observe, so these
//! laws substitute the root synchronizer. They witness that the protocol
//! requests the synchronization, in order, and what a failed one leaves. That
//! the production synchronizer reaches stable storage is not witnessed here.

use std::cell::RefCell;
use std::io;
use std::path::Path;

use cap_std::ambient_authority;

use super::initialization_entries;
use super::initialization_execution::initialize_with;
use super::initialization_test_fixture::{TestResult, initialize};
use super::{
    StoreInitializationBoundary as Boundary, StoreInitializationError as Error,
    StoreInitializationPhase as Phase, StoreInitializationPrefix as Prefix,
};
use crate::adapters::filesystem_directory::FilesystemDirectory as Dir;
use crate::adapters::filesystem_namespace_refusal::FilesystemNamespaceRefusal;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{FilesystemStoreAdmission, FilesystemStoreAdmissionError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Boundary(Phase, Boundary),
    /// One root synchronization and the marker names the root held when it ran.
    RootSynchronization {
        marker: bool,
        stage: bool,
    },
}

fn present(root: &Dir, name: &str) -> io::Result<bool> {
    match root.symlink_metadata(name) {
        Ok(_) => Ok(true),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(source),
    }
}

/// What one initialization returned and every boundary and root synchronization it ran.
type Observed = (Result<Prefix, Error>, Vec<Event>);

/// Initializes `path`, failing the `fail`-th root synchronization when given.
fn observed(path: &Path, fail: Option<usize>) -> TestResult<Observed> {
    let root = Dir::open_ambient_dir(path, ambient_authority())?;
    let events = RefCell::new(Vec::new());
    let mut observer = |phase: Phase, boundary: Boundary| -> io::Result<()> {
        events.borrow_mut().push(Event::Boundary(phase, boundary));
        Ok(())
    };
    let mut synchronize = |root: &Dir| -> io::Result<()> {
        let event = Event::RootSynchronization {
            marker: present(root, "FORMAT")?,
            stage: present(root, "FORMAT.next")?,
        };
        let index = events
            .borrow()
            .iter()
            .filter(|seen| matches!(seen, Event::RootSynchronization { .. }))
            .count();
        events.borrow_mut().push(event);
        if fail == Some(index) {
            Err(io::Error::other("injected root synchronization failure"))
        } else {
            initialization_entries::synchronize_root(root)
        }
    };
    let outcome =
        initialize_with(&root, &mut observer, &mut synchronize).map(|(_lock, found)| found);
    Ok((outcome, events.into_inner()))
}

const COMMIT: Event = Event::RootSynchronization {
    marker: true,
    stage: true,
};

#[test]
fn the_commit_synchronization_runs_after_the_link_and_before_stage_removal() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-commit-order")?;

    let (outcome, events) = observed(sandbox.path(), None)?;

    assert_eq!(outcome?, Prefix::Empty);
    let link = events
        .iter()
        .position(|event| *event == Event::Boundary(Phase::LinkMarker, Boundary::After))
        .ok_or("the marker was never linked")?;
    let commit: Vec<Event> = events.iter().skip(link).take(5).copied().collect();
    assert_eq!(
        commit,
        [
            Event::Boundary(Phase::LinkMarker, Boundary::After),
            Event::Boundary(Phase::SynchronizeRootAfterMarker, Boundary::Before),
            COMMIT,
            Event::Boundary(Phase::SynchronizeRootAfterMarker, Boundary::After),
            Event::Boundary(Phase::RemoveMarkerStage, Boundary::Before),
        ]
    );
    let synchronizations: Vec<Event> = events
        .iter()
        .filter(|event| matches!(event, Event::RootSynchronization { .. }))
        .copied()
        .collect();
    let root = |marker, stage| Event::RootSynchronization { marker, stage };
    assert_eq!(
        synchronizations,
        [
            root(false, false),
            root(false, true),
            COMMIT,
            root(true, false)
        ],
        "the lock phase and the three root-synchronization phases each request exactly one"
    );
    Ok(())
}

#[test]
fn a_failed_commit_synchronization_leaves_an_unadmitted_resumable_store() -> TestResult<()> {
    let sandbox = TestDirectory::create("initialization-commit-sync-failure")?;

    let (outcome, events) = observed(sandbox.path(), Some(2))?;

    assert!(matches!(
        outcome,
        Err(Error::Io {
            phase: Phase::SynchronizeRootAfterMarker,
            ..
        })
    ));
    assert_eq!(
        events.last(),
        Some(&COMMIT),
        "nothing may follow the failed commit synchronization"
    );
    require_incomplete(sandbox.path())?;
    assert_eq!(initialize(sandbox.path())?.found(), Prefix::MarkerLinked);
    drop(FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?);
    Ok(())
}

/// Requires admission to refuse `path` as an initialization that has not committed.
pub(super) fn require_incomplete(path: &Path) -> TestResult<()> {
    let Err(FilesystemStoreAdmissionError::Namespace { source }) =
        FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(path)
    else {
        return Err("an uncommitted initialization was not refused at the namespace".into());
    };
    let refusal = source
        .get_ref()
        .and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>());
    assert_eq!(
        refusal,
        Some(&FilesystemNamespaceRefusal::InitializationIncomplete)
    );
    Ok(())
}
