//! This module owns shared filesystem profile and protocol-directory admission.

use std::{io, path::Path};

use cap_std::fs::Dir;

use super::filesystem_mount_refusal::FilesystemMountRefusal;
use super::filesystem_root_identity::FilesystemRootIdentity;
use super::native_filesystem::NativeFilesystem;
use super::platform_filesystem::PlatformFilesystem;

pub(super) const PROTOCOL_DIRECTORIES: [&str; 3] = ["staging", "segments", "catalogs"];
pub(super) const VERSION_TWO_PROTOCOL_DIRECTORIES: [&str; 9] = [
    "staging",
    "segments",
    "catalogs",
    "retention",
    "retention/roots",
    "retention/manifests",
    "gc",
    "recovery",
    "recovery/dispositions",
];

pub(super) fn open(store_root: &Path) -> io::Result<Dir> {
    open_profile(&NativeFilesystem, store_root, &PROTOCOL_DIRECTORIES)
}

pub(super) fn open_version_two(store_root: &Path) -> io::Result<Dir> {
    open_profile(
        &NativeFilesystem,
        store_root,
        &VERSION_TWO_PROTOCOL_DIRECTORIES,
    )
}

#[cfg(feature = "repository-tasks")]
pub(super) fn admit_locked_root(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    admit_profile(&NativeFilesystem, directory, &PROTOCOL_DIRECTORIES)?;
    root_identity(directory)
}

pub(super) fn root_identity(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    NativeFilesystem.root_identity(directory)
}

#[cfg(any(test, feature = "repository-tasks"))]
pub(super) fn root_identity_lenient(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    NativeFilesystem.repository_identity(directory)
}

fn open_profile(
    platform: &impl PlatformFilesystem,
    store_root: &Path,
    protocol_directories: &[&str],
) -> io::Result<Dir> {
    let directory = platform.open_root(store_root)?;
    admit_profile(platform, &directory, protocol_directories)?;
    Ok(directory)
}

fn admit_profile(
    platform: &impl PlatformFilesystem,
    directory: &Dir,
    protocol_directories: &[&str],
) -> io::Result<()> {
    let root = platform.admit_directory(directory)?;
    for name in protocol_directories {
        let child = match open_protocol_directory(directory, name) {
            Ok(child) => child,
            Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(source),
        };
        let child = platform.admit_directory(&child)?;
        admit_child_identity(root, child)?;
    }
    directory.try_clone()?.into_std_file().sync_all()
}

fn open_protocol_directory(root: &Dir, name: &str) -> io::Result<Dir> {
    let mut components = name.split('/');
    let first = components
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty protocol name"))?;
    super::filesystem_namespace_refusal::require_directory(root, first)?;
    let mut current = super::sync_capable_directory::open(root, first)?;
    for component in components {
        super::filesystem_namespace_refusal::require_directory(&current, component)?;
        current = super::sync_capable_directory::open(&current, component)?;
    }
    Ok(current)
}

pub(super) fn admit_child_identity(
    root: FilesystemRootIdentity,
    child: FilesystemRootIdentity,
) -> io::Result<()> {
    if root.device() != child.device() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            FilesystemMountRefusal::Device {
                expected: root.device(),
                observed: child.device(),
            },
        ));
    }
    if root.mount() != child.mount() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            FilesystemMountRefusal::Mount {
                expected: root.mount(),
                observed: child.mount(),
            },
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "filesystem_mount_admission_tests.rs"]
mod tests;
