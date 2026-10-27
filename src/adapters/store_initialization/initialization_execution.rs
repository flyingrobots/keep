//! This module owns the ordered execution of direct initialization from one observed prefix.
//!
//! Forward protocol, crash states, and recovery are specified in
//! `docs/formats/store/initialization.md`. Every phase runs
//! between two observer notifications, so the crash matrix kills the process
//! at an exact boundary of this code rather than of a reimplementation.

use std::io;

use cap_std::fs::Dir;

use super::initialization_entries::{self as entries, MarkerStage};
use super::initialization_reader_fence::admit_reader_fence;
use super::{
    StoreInitializationBoundary as Boundary, StoreInitializationDirectory as Directory,
    StoreInitializationError as Error, StoreInitializationPhase as Phase,
    StoreInitializationPrefix as Prefix, initialization_observation,
};
use crate::adapters::FilesystemWriterLock;

/// Observer notified before and after every phase; an error fails that phase.
pub(in crate::adapters) type PhaseObserver<'observer> =
    &'observer mut dyn FnMut(Phase, Boundary) -> io::Result<()>;

/// Classifies, locks, reclassifies, and completes initialization of `root`.
///
/// Returns the retained writer lock and the state the call found. The first
/// classification only prevents mutation of a refused root; the second, under
/// writer authority, selects the resumed phases.
///
/// # Errors
///
/// Returns the first refusal, lock failure, or phase failure. Effects already
/// made remain a recoverable prefix.
pub(in crate::adapters) fn initialize(
    root: &Dir,
    observer: PhaseObserver<'_>,
) -> Result<(FilesystemWriterLock, Prefix), Error> {
    let unlocked = initialization_observation::observe(root)?;
    let mut execution = Execution { root, observer };
    let lock = execution.lock()?;
    let locked = initialization_observation::observe(root)?;
    execution.resume(locked)?;
    let found = if unlocked == Prefix::Empty && locked == Prefix::Locked {
        Prefix::Empty
    } else {
        locked
    };
    Ok((lock, found))
}

struct Execution<'root, 'observer> {
    root: &'root Dir,
    observer: PhaseObserver<'observer>,
}

impl Execution<'_, '_> {
    fn phase<T>(
        &mut self,
        phase: Phase,
        operation: impl FnOnce(&Dir) -> io::Result<T>,
    ) -> Result<T, Error> {
        (self.observer)(phase, Boundary::Before).map_err(|source| Error::io(phase, source))?;
        let value = operation(self.root).map_err(|source| Error::io(phase, source))?;
        (self.observer)(phase, Boundary::After).map_err(|source| Error::io(phase, source))?;
        Ok(value)
    }

    fn lock(&mut self) -> Result<FilesystemWriterLock, Error> {
        let phase = Phase::OpenAndLockWriterFile;
        (self.observer)(phase, Boundary::Before).map_err(|source| Error::io(phase, source))?;
        let directory = self
            .root
            .try_clone()
            .map_err(|source| Error::io(phase, source))?;
        let lock = FilesystemWriterLock::initialize_in(directory)
            .map_err(|source| Error::WriterLock { source })?;
        entries::synchronize_root(self.root).map_err(|source| Error::io(phase, source))?;
        (self.observer)(phase, Boundary::After).map_err(|source| Error::io(phase, source))?;
        Ok(lock)
    }

    fn resume(&mut self, prefix: Prefix) -> Result<(), Error> {
        let stage = match prefix {
            Prefix::Empty | Prefix::Locked => {
                self.phase(Phase::WriteMarkerStage, MarkerStage::create)?
            }
            Prefix::IncompleteMarkerStage { .. } => {
                self.phase(
                    Phase::DiscardIncompleteMarkerStage,
                    entries::discard_incomplete_stage,
                )?;
                self.phase(Phase::WriteMarkerStage, MarkerStage::create)?
            }
            Prefix::MarkerStaged | Prefix::ReaderFenced | Prefix::Directories { .. } => {
                reopen(self.root)?
            }
            Prefix::MarkerLinked => {
                let stage = reopen(self.root)?;
                stage
                    .verify_linked(self.root)
                    .map_err(|source| Error::io(Phase::ObservePrefix, source))?;
                return self.commit(stage);
            }
        };
        self.build(&stage)?;
        self.phase(Phase::LinkMarker, |root| stage.link(root))?;
        self.commit(stage)
    }

    /// Makes the stage durable, then creates the fence and every directory.
    fn build(&mut self, stage: &MarkerStage) -> Result<(), Error> {
        self.phase(Phase::SynchronizeMarkerStage, |root| {
            stage.synchronize(root)
        })?;
        self.phase(
            Phase::SynchronizeRootAfterMarkerStage,
            entries::synchronize_root,
        )?;
        self.phase(Phase::AdmitReaderFence, admit_reader_fence)?;
        for directory in Directory::ALL {
            self.phase(Phase::AdmitDirectory(directory), |root| {
                entries::admit_directory(root, directory)
            })?;
        }
        Ok(())
    }

    /// Commits with the root synchronization after the link, then removes the stage.
    fn commit(&mut self, stage: MarkerStage) -> Result<(), Error> {
        self.phase(Phase::SynchronizeRootAfterMarker, entries::synchronize_root)?;
        self.phase(Phase::RemoveMarkerStage, |root| stage.remove(root))?;
        self.phase(
            Phase::SynchronizeRootAfterCleanup,
            entries::synchronize_root,
        )
    }
}

fn reopen(root: &Dir) -> Result<MarkerStage, Error> {
    MarkerStage::reopen(root).map_err(|source| Error::io(Phase::ObservePrefix, source))
}
