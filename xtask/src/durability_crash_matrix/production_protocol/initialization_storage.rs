//! This module owns crash injection around production store initialization.

use std::io;

use keep::{RepositoryInitializationStorage, StoreInitializationStorage};
use xtask::DurabilityCrashPoint;

use super::control::{CrashControl, DuringTiming};

pub(super) struct CrashInitializationStorage<'control> {
    inner: RepositoryInitializationStorage,
    control: &'control mut CrashControl,
}

impl<'control> CrashInitializationStorage<'control> {
    pub(super) const fn new(
        inner: RepositoryInitializationStorage,
        control: &'control mut CrashControl,
    ) -> Self {
        Self { inner, control }
    }
}

impl StoreInitializationStorage for CrashInitializationStorage<'_> {
    fn admit_platform(&mut self) -> io::Result<()> {
        self.inner.admit_platform()
    }

    fn open_and_lock_writer_file(&mut self) -> io::Result<()> {
        let point = DurabilityCrashPoint::OpenAndLockWriterFile;
        self.control.before(point, DuringTiming::After)?;
        self.inner.open_and_lock_writer_file()?;
        self.control.after(point, DuringTiming::After)
    }

    fn admit_staging_directory(&mut self) -> io::Result<()> {
        let point = DurabilityCrashPoint::CreateStagingDirectory;
        self.control.before(point, DuringTiming::After)?;
        self.inner.admit_staging_directory()?;
        self.control.after(point, DuringTiming::After)
    }

    fn admit_segment_pool_directory(&mut self) -> io::Result<()> {
        let point = DurabilityCrashPoint::CreateSegmentPoolDirectory;
        self.control.before(point, DuringTiming::After)?;
        self.inner.admit_segment_pool_directory()?;
        self.control.after(point, DuringTiming::After)
    }

    fn admit_catalog_pool_directory(&mut self) -> io::Result<()> {
        let point = DurabilityCrashPoint::CreateCatalogPoolDirectory;
        self.control.before(point, DuringTiming::After)?;
        self.inner.admit_catalog_pool_directory()?;
        self.control.after(point, DuringTiming::After)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        let point = DurabilityCrashPoint::SynchronizeRootAfterInitialization;
        self.control.before(point, DuringTiming::Before)?;
        self.inner.synchronize_root()?;
        self.control.after(point, DuringTiming::Before)
    }
}
