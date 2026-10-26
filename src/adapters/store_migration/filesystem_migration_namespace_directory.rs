//! This module owns pinned migration namespace-directory admission.

use std::ffi::OsStr;
use std::io;

use cap_std::fs::Dir;

use super::FilesystemMigrationRefusal as Refusal;
use crate::adapters::filesystem_exact_record::EntryIdentity;
use crate::adapters::{FilesystemEntryKind, FilesystemOperationRefusal};

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
        require_kind(&metadata, FilesystemEntryKind::Directory)?;
        let entry = EntryIdentity::from(&metadata);
        [handle, current, entry]
            .into_iter()
            .find(|observed| *observed != self.identity)
            .map_or(Ok(()), |observed| {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    FilesystemOperationRefusal::IdentityChanged {
                        expected: self.identity.into(),
                        observed: observed.into(),
                    },
                ))
            })
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
        Ok(metadata) => Err(wrong_kind(&metadata, FilesystemEntryKind::Directory)),
    }
}

fn require_same_filesystem(parent: &Dir, child: &Dir) -> io::Result<()> {
    let parent = filesystem_platform_profile::root_identity(parent)?;
    let child = filesystem_platform_profile::root_identity(child)?;
    if parent.device() == child.device() && parent.mount() == child.mount() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            FilesystemOperationRefusal::MountBoundaryChanged {
                expected_device: parent.device(),
                observed_device: child.device(),
                expected_mount: parent.mount(),
                observed_mount: child.mount(),
            },
        ))
    }
}

pub(super) fn required_directory(
    parent: &Dir,
    name: &'static str,
) -> io::Result<PinnedMigrationDirectory> {
    optional_directory(parent, name)?.ok_or_else(|| ambiguous(Refusal::DirectoryAbsent { name }))
}

pub(super) fn require_directory(parent: &Dir, name: &str) -> io::Result<()> {
    require_kind(
        &parent.symlink_metadata(name)?,
        FilesystemEntryKind::Directory,
    )
}

pub(super) fn require_regular(parent: &Dir, name: &str, length: Option<usize>) -> io::Result<()> {
    let metadata = parent.symlink_metadata(name)?;
    let expected = length
        .map(|length| {
            u64::try_from(length)
                .map_err(|source| ambiguous(Refusal::FileLengthOverflow { length, source }))
        })
        .transpose()?;
    require_kind(&metadata, FilesystemEntryKind::File)?;
    match expected {
        Some(expected) if metadata.len() != expected => Err(ambiguous(Refusal::FileLength {
            expected,
            observed: metadata.len(),
        })),
        _ => Ok(()),
    }
}

pub(super) fn require_empty(directory: &Dir) -> io::Result<()> {
    let mut entries = directory.entries()?;
    if entries.next().transpose()?.is_none() {
        Ok(())
    } else {
        Err(ambiguous(Refusal::NamespaceNotEmpty))
    }
}

pub(super) fn require_exact_membership(directory: &Dir, expected: &[&str]) -> io::Result<()> {
    if exact_membership(directory, expected)? {
        Ok(())
    } else {
        Err(ambiguous(Refusal::MembershipDisagreed))
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
            .ok_or_else(|| ambiguous(Refusal::EntryCountOverflow))?;
        let name = entry?.file_name();
        if observed > allowed.len() || !allowed.iter().any(|candidate| name == *candidate) {
            return Err(ambiguous(Refusal::UnknownEntry));
        }
    }
    Ok(())
}

fn require_kind(metadata: &cap_std::fs::Metadata, expected: FilesystemEntryKind) -> io::Result<()> {
    if FilesystemEntryKind::observed(metadata.file_type()) == expected {
        Ok(())
    } else {
        Err(wrong_kind(metadata, expected))
    }
}

fn wrong_kind(metadata: &cap_std::fs::Metadata, expected: FilesystemEntryKind) -> io::Error {
    ambiguous(Refusal::EntryKind {
        expected,
        observed: FilesystemEntryKind::observed(metadata.file_type()),
    })
}

/// Carries a migration namespace refusal as the `InvalidData` error the port returns.
pub(super) fn ambiguous(refusal: Refusal) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, refusal)
}
