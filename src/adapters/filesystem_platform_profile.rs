//! This module owns fail-closed filesystem platform-profile admission.

use std::io;
use std::path::Path;

use cap_std::fs::Dir;

use super::filesystem_root_identity::FilesystemRootIdentity;

#[cfg(target_os = "linux")]
const PROTOCOL_DIRECTORIES: [&str; 3] = ["staging", "segments", "catalogs"];
/// Every version-two protocol directory, including nested immutable pools.
///
/// A migrated root receives writer authority only when each of these shares
/// the root's filesystem type, device, and mount identity and is not
/// casefolded or read-only.
#[cfg(target_os = "linux")]
const VERSION_TWO_PROTOCOL_DIRECTORIES: [&str; 9] = [
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

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct LinuxDirectoryProperties {
    filesystem_type: rustix::fs::FsWord,
    mount_flags: rustix::fs::StatVfsMountFlags,
    inode_flags: u32,
    device_major: u32,
    device_minor: u32,
    mount_id: u64,
}

#[cfg(target_os = "linux")]
pub(super) fn open(store_root: &Path) -> io::Result<Dir> {
    use std::fs::File;

    use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};

    let descriptor = openat2(
        CWD,
        store_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
    )?;
    let directory = Dir::from_std_file(File::from(descriptor));
    admit_linux_profile(&directory, &PROTOCOL_DIRECTORIES)?;
    Ok(directory)
}

/// Opens one version-two store root under the admitted Linux profile.
///
/// Identical to [`open`], but every version-two protocol directory that
/// exists must satisfy the same filesystem, mount, and inode-flag laws as the
/// root; absence is left to namespace admission.
#[cfg(target_os = "linux")]
pub(super) fn open_version_two(store_root: &Path) -> io::Result<Dir> {
    use std::fs::File;

    use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};

    let descriptor = openat2(
        CWD,
        store_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
    )?;
    let directory = Dir::from_std_file(File::from(descriptor));
    admit_linux_profile(&directory, &VERSION_TWO_PROTOCOL_DIRECTORIES)?;
    Ok(directory)
}

#[cfg(not(target_os = "linux"))]
pub(super) fn open(_store_root: &Path) -> io::Result<Dir> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem initialization currently requires the admitted Linux ext4 profile",
    ))
}

#[cfg(not(target_os = "linux"))]
pub(super) fn open_version_two(_store_root: &Path) -> io::Result<Dir> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "version-two reopen currently requires the admitted Linux ext4 profile",
    ))
}

#[cfg(target_os = "linux")]
fn admit_linux_profile(directory: &Dir, protocol_directories: &[&str]) -> io::Result<()> {
    let file = directory.try_clone()?.into_std_file();
    let root = linux_directory_properties(&file)?;
    admit_linux_properties(root.filesystem_type, root.mount_flags, root.inode_flags)?;
    for name in protocol_directories {
        let child = match open_protocol_directory(directory, name) {
            Ok(child) => child,
            Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(source),
        };
        let child = linux_directory_properties(&child.into_std_file())?;
        admit_linux_child_properties(root, child)?;
    }
    file.sync_all()
}

/// Opens a possibly nested protocol directory one no-follow component at a time.
#[cfg(target_os = "linux")]
fn open_protocol_directory(root: &Dir, name: &str) -> io::Result<Dir> {
    let mut components = name.split('/');
    let first = components
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty protocol name"))?;
    let mut current = super::sync_capable_directory::open(root, first)?;
    for component in components {
        current = super::sync_capable_directory::open(&current, component)?;
    }
    Ok(current)
}

#[cfg(target_os = "linux")]
fn linux_directory_properties(file: &std::fs::File) -> io::Result<LinuxDirectoryProperties> {
    use rustix::fs::{AtFlags, StatxFlags, fstatfs, fstatvfs, ioctl_getflags, statx};

    let filesystem = fstatfs(file)?;
    let mount = fstatvfs(file)?;
    let inode_flags = ioctl_getflags(file)?;
    let required = StatxFlags::BASIC_STATS | StatxFlags::MNT_ID;
    let status = statx(file, ".", AtFlags::empty(), required)?;
    let observed = StatxFlags::from_bits_retain(status.stx_mask);
    if !observed.contains(required) {
        return Err(unsupported_linux_profile());
    }
    Ok(LinuxDirectoryProperties {
        filesystem_type: filesystem.f_type,
        mount_flags: mount.f_flag,
        inode_flags: inode_flags.bits(),
        device_major: status.stx_dev_major,
        device_minor: status.stx_dev_minor,
        mount_id: status.stx_mnt_id,
    })
}

/// Whether an identity probe may proceed when `statx` omits `STATX_MNT_ID`.
///
/// Production admission requires the mount identity: without it a bind-mounted
/// or relocated root cannot be told apart from the original. The test and
/// repository-task bypass records an unreported mount identity as zero instead,
/// so the suite runs on kernels older than 5.8 while every production probe
/// stays strict.
#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MountIdentityPolicy {
    Required,
    #[cfg(any(test, feature = "repository-tasks"))]
    Lenient,
}

/// Selects the recorded mount identity from what `statx` reported.
///
/// Returns `None` exactly when the policy requires a mount identity the kernel
/// did not report.
#[cfg(any(target_os = "linux", test))]
const fn admit_mount_identity(
    policy: MountIdentityPolicy,
    reported: bool,
    mount_id: u64,
) -> Option<u64> {
    match (policy, reported) {
        (_, true) => Some(mount_id),
        (MountIdentityPolicy::Required, false) => None,
        #[cfg(any(test, feature = "repository-tasks"))]
        (MountIdentityPolicy::Lenient, false) => Some(0),
    }
}

#[cfg(target_os = "linux")]
pub(super) fn root_identity(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    let file = directory.try_clone()?.into_std_file();
    linux_file_identity(&file, MountIdentityPolicy::Required)
}

/// Probes root identity for the test and repository-task admission bypass.
///
/// Identical to [`root_identity`] except that an unreported mount identity is
/// recorded as zero instead of refusing; production admission never uses it.
#[cfg(all(target_os = "linux", any(test, feature = "repository-tasks")))]
pub(super) fn root_identity_lenient(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    let file = directory.try_clone()?.into_std_file();
    linux_file_identity(&file, MountIdentityPolicy::Lenient)
}

#[cfg(target_os = "linux")]
fn linux_file_identity(
    file: &std::fs::File,
    policy: MountIdentityPolicy,
) -> io::Result<FilesystemRootIdentity> {
    use rustix::fs::{AtFlags, StatxFlags, statx};

    let requested = StatxFlags::BASIC_STATS | StatxFlags::MNT_ID;
    let status = statx(file, ".", AtFlags::empty(), requested)?;
    let observed = StatxFlags::from_bits_retain(status.stx_mask);
    if !observed.contains(StatxFlags::BASIC_STATS) {
        return Err(unsupported_linux_profile());
    }
    let reported = observed.contains(StatxFlags::MNT_ID);
    let mount_id = admit_mount_identity(policy, reported, status.stx_mnt_id)
        .ok_or_else(unsupported_linux_profile)?;
    Ok(linux_root_identity(
        status.stx_dev_major,
        status.stx_dev_minor,
        mount_id,
        status.stx_ino,
    ))
}

#[cfg(target_os = "linux")]
fn linux_root_identity(
    device_major: u32,
    device_minor: u32,
    mount_id: u64,
    inode: u64,
) -> FilesystemRootIdentity {
    let device = rustix::fs::makedev(device_major, device_minor);
    FilesystemRootIdentity::new(device, mount_id, inode)
}

#[cfg(all(not(target_os = "linux"), any(test, feature = "repository-tasks")))]
pub(super) fn root_identity(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    use cap_fs_ext::MetadataExt;

    let metadata = directory.dir_metadata()?;
    Ok(FilesystemRootIdentity::new(
        metadata.dev(),
        metadata.dev(),
        metadata.ino(),
    ))
}

#[cfg(all(not(target_os = "linux"), any(test, feature = "repository-tasks")))]
pub(super) fn root_identity_lenient(directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    root_identity(directory)
}

#[cfg(all(not(target_os = "linux"), not(any(test, feature = "repository-tasks"))))]
pub(super) fn root_identity(_directory: &Dir) -> io::Result<FilesystemRootIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem root identity currently requires the admitted Linux ext4 profile",
    ))
}

#[cfg(target_os = "linux")]
fn admit_linux_properties(
    filesystem_type: rustix::fs::FsWord,
    mount_flags: rustix::fs::StatVfsMountFlags,
    inode_flags: u32,
) -> io::Result<()> {
    // These values are the Linux UAPI ext4 superblock magic and per-directory
    // casefold inode flag. Keeping them local makes the admitted profile
    // visible at the exact decision boundary.
    const EXT4_SUPER_MAGIC: rustix::fs::FsWord = 0x0000_ef53;
    const EXT4_CASEFOLD_FLAG: u32 = 0x4000_0000;

    if filesystem_type != EXT4_SUPER_MAGIC
        || mount_flags.contains(rustix::fs::StatVfsMountFlags::RDONLY)
        || inode_flags & EXT4_CASEFOLD_FLAG != 0
    {
        return Err(unsupported_linux_profile());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn admit_linux_child_properties(
    root: LinuxDirectoryProperties,
    child: LinuxDirectoryProperties,
) -> io::Result<()> {
    admit_linux_properties(child.filesystem_type, child.mount_flags, child.inode_flags)?;
    if root.device_major != child.device_major
        || root.device_minor != child.device_minor
        || root.mount_id != child.mount_id
    {
        return Err(unsupported_linux_profile());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn unsupported_linux_profile() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "store namespace does not satisfy one local writable case-sensitive ext4 profile",
    )
}

#[cfg(all(test, target_os = "linux"))]
#[path = "filesystem_platform_profile_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "filesystem_platform_profile_policy_tests.rs"]
mod policy_tests;
