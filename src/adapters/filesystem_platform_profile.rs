//! This module owns shared filesystem profile and protocol-directory admission.

use std::{io, path::Path};

use crate::adapters::filesystem_directory::FilesystemDirectory as Dir;

use super::filesystem_directory_coordinate::FilesystemDirectoryCoordinate;
use super::filesystem_mount_refusal::FilesystemMountRefusal;
use super::native_filesystem::NativeFilesystem;
use super::platform_filesystem::PlatformFilesystem;

/// Every protocol directory, including nested immutable pools.
///
/// A store root receives writer authority only when each of these shares
/// the root's filesystem type, device, and mount identity and satisfies the
/// admitted platform profile.
pub(super) const PROTOCOL_DIRECTORIES: [&str; 9] = [
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

/// Opens one store root under the admitted native profile.
///
/// The root is opened without following links, and every protocol directory
/// that exists must satisfy the same profile as the root and share its live
/// device and mount; absence is left to namespace admission.
pub(super) fn open(store_root: &Path) -> io::Result<Dir> {
    open_profile(&NativeFilesystem, store_root, &PROTOCOL_DIRECTORIES)
}

fn open_profile(
    platform: &impl PlatformFilesystem,
    store_root: &Path,
    protocol_directories: &[&str],
) -> io::Result<Dir> {
    let directory = Dir::from_raw(platform.open_root(store_root)?);
    admit_profile(platform, &directory, protocol_directories)?;
    Ok(directory)
}

fn admit_profile(
    platform: &impl PlatformFilesystem,
    directory: &Dir,
    protocol_directories: &[&str],
) -> io::Result<()> {
    let root = platform.admit_directory(directory.raw())?;
    for name in protocol_directories {
        let child = match open_protocol_directory(directory, name) {
            Ok(child) => child,
            Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(source),
        };
        let child = platform.admit_directory(child.raw())?;
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

fn admit_child_identity(
    root: FilesystemDirectoryCoordinate,
    child: FilesystemDirectoryCoordinate,
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
