//! This module owns Linux descriptor observations and ext4 admission.

use std::{fs::File, io, path::Path};

use cap_std::fs::Dir;
use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};

use super::filesystem_root_identity::FilesystemRootIdentity;
use super::platform_filesystem::PlatformFilesystem;

pub(super) struct LinuxFilesystem;

impl PlatformFilesystem for LinuxFilesystem {
    fn open_root(&self, store_root: &Path) -> io::Result<Dir> {
        let descriptor = openat2(
            CWD,
            store_root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
        )?;
        Ok(Dir::from_std_file(File::from(descriptor)))
    }

    fn admit_directory(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        let file = directory.try_clone()?.into_std_file();
        let observed = linux_directory_properties(&file)?;
        admit_linux_properties(
            observed.filesystem_type,
            observed.mount_flags,
            observed.inode_flags,
        )?;
        Ok(linux_root_identity(
            observed.device_major,
            observed.device_minor,
            observed.mount_id,
            observed.inode,
        ))
    }

    fn root_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        let file = directory.try_clone()?.into_std_file();
        linux_file_identity(&file, MountIdentityPolicy::Required)
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    fn repository_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        let file = directory.try_clone()?.into_std_file();
        linux_file_identity(&file, MountIdentityPolicy::Lenient)
    }
}

#[derive(Clone, Copy)]
struct LinuxDirectoryProperties {
    filesystem_type: rustix::fs::FsWord,
    mount_flags: rustix::fs::StatVfsMountFlags,
    inode_flags: u32,
    device_major: u32,
    device_minor: u32,
    mount_id: u64,
    inode: u64,
}

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
        inode: status.stx_ino,
    })
}

/// Whether an identity probe may proceed when `statx` omits `STATX_MNT_ID`.
///
/// Production admission requires the mount identity: without it a bind-mounted
/// or relocated root cannot be told apart from the original. The test and
/// repository-task bypass records an unreported mount identity as zero instead,
/// so the suite runs on kernels older than 5.8 while every production probe
/// stays strict.
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

fn linux_root_identity(
    device_major: u32,
    device_minor: u32,
    mount_id: u64,
    inode: u64,
) -> FilesystemRootIdentity {
    let device = rustix::fs::makedev(device_major, device_minor);
    FilesystemRootIdentity::new(device, mount_id, inode)
}

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

fn unsupported_linux_profile() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "store namespace does not satisfy one local writable case-sensitive ext4 profile",
    )
}

#[cfg(test)]
#[path = "filesystem_platform_profile_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "filesystem_platform_profile_policy_tests.rs"]
mod policy_tests;
