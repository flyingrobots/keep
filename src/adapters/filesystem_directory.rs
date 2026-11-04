//! This module owns capability-relative access to exact protocol entry names.
//!
//! No Deref exposes unchecked pathname operations. Each operation accepts one
//! normal component; nested protocol directories are traversed with new pinned
//! handles. Native spelling observations precede even truncation and removal.

use std::{
    ffi::OsStr,
    io,
    path::{Component, Path},
};

use cap_fs_ext::DirExt;
use cap_std::fs::{Dir as RawDirectory, File, Metadata, OpenOptions, ReadDir};

use super::filesystem_entry_spelling;

#[derive(Debug)]
pub(super) struct FilesystemDirectory(RawDirectory);

impl FilesystemDirectory {
    pub(super) const fn from_raw(directory: RawDirectory) -> Self {
        Self(directory)
    }

    pub(super) const fn raw(&self) -> &RawDirectory {
        &self.0
    }

    pub(super) fn from_std_file(file: std::fs::File) -> Self {
        Self(RawDirectory::from_std_file(file))
    }

    pub(super) fn into_std_file(self) -> std::fs::File {
        self.0.into_std_file()
    }

    pub(super) fn open_ambient_dir(
        path: impl AsRef<Path>,
        authority: cap_std::AmbientAuthority,
    ) -> io::Result<Self> {
        RawDirectory::open_ambient_dir(path, authority).map(Self)
    }

    pub(super) fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    pub(super) fn dir_metadata(&self) -> io::Result<Metadata> {
        self.0.dir_metadata()
    }

    pub(super) fn entries(&self) -> io::Result<ReadDir> {
        self.0.entries()
    }

    pub(super) fn open_with(
        &self,
        path: impl AsRef<Path>,
        options: &OpenOptions,
    ) -> io::Result<File> {
        let name = self.require_exact(path.as_ref())?;
        self.0.open_with(name, options)
    }

    pub(super) fn symlink_metadata(&self, path: impl AsRef<Path>) -> io::Result<Metadata> {
        let name = self.require_exact(path.as_ref())?;
        self.0.symlink_metadata(name)
    }

    pub(super) fn open_dir_nofollow(&self, path: impl AsRef<Path>) -> io::Result<Self> {
        let name = self.require_exact(path.as_ref())?;
        self.0.open_dir_nofollow(name).map(Self)
    }

    pub(super) fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let name = self.require_exact(path.as_ref())?;
        self.0.create_dir(name)
    }

    pub(super) fn remove_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let name = self.require_exact(path.as_ref())?;
        self.0.remove_file(name)
    }

    pub(super) fn hard_link(
        &self,
        source: impl AsRef<Path>,
        destination: &Self,
        target: impl AsRef<Path>,
    ) -> io::Result<()> {
        let source = self.require_exact(source.as_ref())?;
        let target = destination.require_exact(target.as_ref())?;
        self.0.hard_link(source, &destination.0, target)
    }

    pub(super) fn rename(
        &self,
        source: impl AsRef<Path>,
        destination: &Self,
        target: impl AsRef<Path>,
    ) -> io::Result<()> {
        let source = self.require_exact(source.as_ref())?;
        let target = destination.require_exact(target.as_ref())?;
        self.0.rename(source, &destination.0, target)
    }

    fn require_exact<'a>(&self, path: &'a Path) -> io::Result<&'a OsStr> {
        // The directory itself has no entry to spell; synchronization reopens it.
        if path == Path::new(".") {
            return Ok(path.as_os_str());
        }
        let mut components = path.components();
        let name = match (components.next(), components.next()) {
            (Some(Component::Normal(name)), None) if path.as_os_str() == name => name,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "protocol entry must be one normal component",
                ));
            }
        };
        filesystem_entry_spelling::require_exact(&self.0, name)?;
        Ok(name)
    }
}

impl std::os::fd::AsFd for FilesystemDirectory {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.as_fd()
    }
}
