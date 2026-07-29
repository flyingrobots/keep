//! This module owns concrete filesystem initialization operations.

use std::io;
use std::path::Path;

#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_initialization_namespace;
use super::filesystem_platform_profile;
use super::sync_capable_directory;
use super::{FilesystemWriterLock, StoreInitializationStorage};

const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";

pub(super) struct FilesystemInitializationStorage {
    directory: Dir,
    lock: Option<FilesystemWriterLock>,
}

impl FilesystemInitializationStorage {
    pub(super) fn admit(store_root: &Path) -> io::Result<Self> {
        let directory = filesystem_platform_profile::open(store_root)?;
        Ok(Self {
            directory,
            lock: None,
        })
    }

    #[cfg(test)]
    pub(super) fn admit_unchecked_for_tests(store_root: &Path) -> io::Result<Self> {
        let directory = Dir::open_ambient_dir(store_root, ambient_authority())?;
        Ok(Self {
            directory,
            lock: None,
        })
    }

    pub(super) fn into_lock(self) -> io::Result<FilesystemWriterLock> {
        self.lock.ok_or_else(|| {
            io::Error::other("initialization completed without retained writer authority")
        })
    }

    fn admit_directory(&self, name: &str) -> io::Result<()> {
        if self.lock.is_none() {
            return Err(io::Error::other(
                "initialization directory mutation requires writer authority",
            ));
        }
        match self.directory.create_dir(name) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(source),
        }
        let admitted = sync_capable_directory::open(&self.directory, name)?;
        drop(admitted);
        Ok(())
    }
}

impl StoreInitializationStorage for FilesystemInitializationStorage {
    fn admit_platform(&mut self) -> io::Result<()> {
        filesystem_initialization_namespace::admit(&self.directory)
    }

    fn open_and_lock_writer_file(&mut self) -> io::Result<()> {
        if self.lock.is_some() {
            return Err(io::Error::other("writer authority was already acquired"));
        }
        let lock = FilesystemWriterLock::initialize_in(self.directory.try_clone()?)
            .map_err(io::Error::other)?;
        self.lock = Some(lock);
        filesystem_initialization_namespace::admit(&self.directory)
    }

    fn admit_staging_directory(&mut self) -> io::Result<()> {
        self.admit_directory(STAGING_NAME)
    }

    fn admit_segment_pool_directory(&mut self) -> io::Result<()> {
        self.admit_directory(SEGMENTS_NAME)
    }

    fn admit_catalog_pool_directory(&mut self) -> io::Result<()> {
        self.admit_directory(CATALOGS_NAME)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        if self.lock.is_none() {
            return Err(io::Error::other(
                "root synchronization requires writer authority",
            ));
        }
        self.directory.try_clone()?.into_std_file().sync_all()
    }
}
