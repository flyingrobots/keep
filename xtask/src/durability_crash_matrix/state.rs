//! This module owns one crash child's retained filesystem state.

mod catalog;
pub(super) mod fixture;
mod head;
mod initialization;
mod recovery;
mod segment;

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::ops::Range;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use keep::FilesystemWriterLock;
use xtask::{DurabilityCrashCase, DurabilityCrashSequence};

use super::DurabilityCrashMatrixError;
use fixture::GoldenFixture;

const STORE_DIRECTORY: &str = "store";

pub(super) struct PreparedCrashState {
    state: StoreState,
}

impl PreparedCrashState {
    pub(super) fn await_process_death(
        self,
        stream: &mut UnixStream,
    ) -> Result<(), DurabilityCrashMatrixError> {
        let retained_root = &self.state.root;
        let mut release = [0_u8; 1];
        let result = stream
            .read_exact(&mut release)
            .map_err(|source| DurabilityCrashMatrixError::io("await process termination", source));
        let _ = retained_root;
        self.state.finish(result)
    }
}

pub(super) fn prepare(
    case: DurabilityCrashCase,
    case_root: &Path,
) -> Result<PreparedCrashState, DurabilityCrashMatrixError> {
    let mut state = StoreState::create(case_root)?;
    match case.point().sequence() {
        DurabilityCrashSequence::Segment => segment::prepare(&mut state, case)?,
        DurabilityCrashSequence::Catalog => catalog::prepare(&mut state, case)?,
        DurabilityCrashSequence::Head => head::prepare(&mut state, case)?,
        DurabilityCrashSequence::RecoveryDiscard => recovery::prepare(&mut state, case)?,
        DurabilityCrashSequence::Initialization => initialization::prepare(&mut state, case)?,
    }
    Ok(PreparedCrashState { state })
}

struct StoreState {
    root: PathBuf,
    active_file: Option<File>,
    writer_lock: Option<FilesystemWriterLock>,
}

impl StoreState {
    fn create(case_root: &Path) -> Result<Self, DurabilityCrashMatrixError> {
        let root = case_root.join(STORE_DIRECTORY);
        fs::create_dir(&root)
            .map_err(|source| DurabilityCrashMatrixError::io("create crash store root", source))?;
        Ok(Self {
            root,
            active_file: None,
            writer_lock: None,
        })
    }

    fn initialize(&mut self) -> Result<(), DurabilityCrashMatrixError> {
        self.create_writer_lock()?;
        self.create_directory("staging")?;
        self.create_directory("segments")?;
        self.create_directory("catalogs")?;
        self.acquire_writer_lock()
    }

    fn create_writer_lock(&self) -> Result<(), DurabilityCrashMatrixError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.root.join("writer.lock"))
            .map_err(|source| DurabilityCrashMatrixError::io("create writer lock", source))?;
        file.sync_all()
            .map_err(|source| DurabilityCrashMatrixError::io("synchronize writer lock", source))
    }

    fn acquire_writer_lock(&mut self) -> Result<(), DurabilityCrashMatrixError> {
        let lock = FilesystemWriterLock::try_acquire(&self.root)
            .map_err(DurabilityCrashMatrixError::WriterLock)?;
        self.writer_lock = Some(lock);
        Ok(())
    }

    fn create_directory(&self, relative: &str) -> Result<(), DurabilityCrashMatrixError> {
        fs::create_dir(self.root.join(relative))
            .map_err(|source| DurabilityCrashMatrixError::io("create protocol directory", source))
    }

    fn create_stage(&mut self, relative: &str) -> Result<(), DurabilityCrashMatrixError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(self.root.join(relative))
            .map_err(|source| DurabilityCrashMatrixError::io("create fixed stage", source))?;
        self.active_file = Some(file);
        Ok(())
    }

    fn write_range(
        &mut self,
        fixture: &GoldenFixture,
        range: Range<usize>,
    ) -> Result<(), DurabilityCrashMatrixError> {
        let bytes = fixture.range(range)?;
        self.active_file()?
            .write_all(bytes)
            .map_err(|source| DurabilityCrashMatrixError::io("write staged artifact", source))
    }

    fn flush(&mut self) -> Result<(), DurabilityCrashMatrixError> {
        self.active_file()?
            .flush()
            .map_err(|source| DurabilityCrashMatrixError::io("flush staged artifact", source))
    }

    fn synchronize_file(&self) -> Result<(), DurabilityCrashMatrixError> {
        self.active_file_ref()?
            .sync_all()
            .map_err(|source| DurabilityCrashMatrixError::io("synchronize staged artifact", source))
    }

    fn link(&self, source: &str, target: &str) -> Result<(), DurabilityCrashMatrixError> {
        fs::hard_link(self.root.join(source), self.root.join(target))
            .map_err(|source| DurabilityCrashMatrixError::io("link immutable artifact", source))
    }

    fn synchronize_directory(&self, relative: &str) -> Result<(), DurabilityCrashMatrixError> {
        File::open(self.root.join(relative))
            .and_then(|directory| directory.sync_all())
            .map_err(|source| {
                DurabilityCrashMatrixError::io("synchronize protocol directory", source)
            })
    }

    fn remove(&self, relative: &str) -> Result<(), DurabilityCrashMatrixError> {
        fs::remove_file(self.root.join(relative))
            .map_err(|source| DurabilityCrashMatrixError::io("remove protocol stage", source))
    }

    fn rename(&self, source: &str, target: &str) -> Result<(), DurabilityCrashMatrixError> {
        fs::rename(self.root.join(source), self.root.join(target))
            .map_err(|source| DurabilityCrashMatrixError::io("replace publication head", source))
    }

    fn write_immutable(
        &self,
        relative: &str,
        fixture: &GoldenFixture,
    ) -> Result<(), DurabilityCrashMatrixError> {
        let path = self.root.join(relative);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|source| {
                DurabilityCrashMatrixError::io("create immutable artifact", source)
            })?;
        file.write_all(fixture.bytes())
            .and_then(|()| file.sync_all())
            .map_err(|source| DurabilityCrashMatrixError::io("write immutable artifact", source))
    }

    fn active_file(&mut self) -> Result<&mut File, DurabilityCrashMatrixError> {
        self.active_file
            .as_mut()
            .ok_or(DurabilityCrashMatrixError::MissingActiveFile)
    }

    fn active_file_ref(&self) -> Result<&File, DurabilityCrashMatrixError> {
        self.active_file
            .as_ref()
            .ok_or(DurabilityCrashMatrixError::MissingActiveFile)
    }

    fn finish(
        self,
        result: Result<(), DurabilityCrashMatrixError>,
    ) -> Result<(), DurabilityCrashMatrixError> {
        drop(self.active_file);
        drop(self.writer_lock);
        drop(self.root);
        result
    }
}
