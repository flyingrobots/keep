//! This module owns pinned migration namespace-directory admission.

use std::ffi::OsStr;
use std::io;

use cap_std::fs::Dir;

use crate::adapters::filesystem_exact_record::EntryIdentity;

use crate::adapters::{
    filesystem_catalog_artifact, filesystem_platform_profile, sync_capable_directory,
};

pub(super) struct PinnedMigrationDirectory {
    name: &'static str,
    identity: EntryIdentity,
    directory: Dir,
}

impl PinnedMigrationDirectory {
    pub(super) fn admit(parent: &Dir, name: &'static str) -> io::Result<Self> {
        let created = match parent.create_dir(name) {
            Ok(()) => true,
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => false,
            Err(source) => return Err(source),
        };
        let directory = sync_capable_directory::open(parent, name)?;
        require_same_filesystem(parent, &directory)?;
        let identity = EntryIdentity::from(&directory.dir_metadata()?);
        let pinned = Self {
            name,
            identity,
            directory,
        };
        pinned.verify(parent)?;
        if created {
            filesystem_catalog_artifact::synchronize_directory(parent)?;
        }
        Ok(pinned)
    }

    pub(super) fn verify(&self, parent: &Dir) -> io::Result<()> {
        let handle = EntryIdentity::from(&self.directory.dir_metadata()?);
        let current = sync_capable_directory::open(parent, self.name)?;
        require_same_filesystem(parent, &current)?;
        let current = EntryIdentity::from(&current.dir_metadata()?);
        let metadata = parent.symlink_metadata(self.name)?;
        if metadata.is_dir()
            && handle == self.identity
            && current == self.identity
            && EntryIdentity::from(&metadata) == handle
        {
            Ok(())
        } else {
            Err(ambiguous("migration directory changed identity"))
        }
    }

    pub(super) const fn directory(&self) -> &Dir {
        &self.directory
    }
}

pub(super) fn optional_directory(
    parent: &Dir,
    name: &'static str,
) -> io::Result<Option<PinnedMigrationDirectory>> {
    match parent.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(source),
        Ok(metadata) if metadata.is_dir() => {
            let directory = sync_capable_directory::open(parent, name)?;
            require_same_filesystem(parent, &directory)?;
            let pinned = PinnedMigrationDirectory {
                name,
                identity: EntryIdentity::from(&metadata),
                directory,
            };
            pinned.verify(parent)?;
            Ok(Some(pinned))
        }
        Ok(_) => Err(ambiguous("migration namespace entry has the wrong kind")),
    }
}

fn require_same_filesystem(parent: &Dir, child: &Dir) -> io::Result<()> {
    let parent = filesystem_platform_profile::root_identity(parent)?;
    let child = filesystem_platform_profile::root_identity(child)?;
    if parent.device() == child.device() && parent.mount() == child.mount() {
        Ok(())
    } else {
        Err(ambiguous(
            "migration namespace crossed the admitted filesystem or mount",
        ))
    }
}

pub(super) fn required_directory(
    parent: &Dir,
    name: &'static str,
) -> io::Result<PinnedMigrationDirectory> {
    optional_directory(parent, name)?
        .ok_or_else(|| ambiguous("required migration namespace directory was absent"))
}

pub(super) fn require_directory(parent: &Dir, name: &str) -> io::Result<()> {
    if parent.symlink_metadata(name)?.is_dir() {
        Ok(())
    } else {
        Err(ambiguous("required migration directory has the wrong kind"))
    }
}

pub(super) fn require_regular(parent: &Dir, name: &str, length: Option<usize>) -> io::Result<()> {
    let metadata = parent.symlink_metadata(name)?;
    let expected = length
        .map(u64::try_from)
        .transpose()
        .map_err(|_source| ambiguous("required migration file length exceeded u64"))?;
    if metadata.is_file() && expected.is_none_or(|expected| metadata.len() == expected) {
        Ok(())
    } else {
        Err(ambiguous(
            "required migration file has the wrong kind or length",
        ))
    }
}

pub(super) fn require_empty(directory: &Dir) -> io::Result<()> {
    let mut entries = directory.entries()?;
    if entries.next().transpose()?.is_none() {
        Ok(())
    } else {
        Err(ambiguous("new migration namespace was not empty"))
    }
}

pub(super) fn require_exact_membership(directory: &Dir, expected: &[&str]) -> io::Result<()> {
    if exact_membership(directory, expected)? {
        Ok(())
    } else {
        Err(ambiguous("migration namespace membership disagreed"))
    }
}

pub(super) fn exact_membership(directory: &Dir, expected: &[&str]) -> io::Result<bool> {
    let mut observed = Vec::with_capacity(expected.len());
    for entry in directory.entries()? {
        if observed.len() == expected.len() {
            return Ok(false);
        }
        observed.push(entry?.file_name());
    }
    Ok(observed.len() == expected.len()
        && observed.iter().all(|name| {
            expected
                .iter()
                .any(|candidate| name == OsStr::new(candidate))
        }))
}

pub(super) fn require_allowed_membership(directory: &Dir, allowed: &[&str]) -> io::Result<()> {
    let mut observed = 0_usize;
    for entry in directory.entries()? {
        observed = observed
            .checked_add(1)
            .ok_or_else(|| ambiguous("migration namespace count overflowed"))?;
        let name = entry?.file_name();
        if observed > allowed.len() || !allowed.iter().any(|candidate| name == *candidate) {
            return Err(ambiguous("migration namespace contains an unknown entry"));
        }
    }
    Ok(())
}

pub(super) fn ambiguous(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
