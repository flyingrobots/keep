//! This module owns fail-closed filesystem platform-profile admission.

use std::io;
use std::path::Path;

use cap_std::fs::Dir;

#[cfg(target_os = "linux")]
const PROTOCOL_DIRECTORIES: [&str; 3] = ["staging", "segments", "catalogs"];

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
    admit_linux_profile(&directory)?;
    Ok(directory)
}

#[cfg(not(target_os = "linux"))]
pub(super) fn open(_store_root: &Path) -> io::Result<Dir> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem initialization currently requires the admitted Linux ext4 profile",
    ))
}

#[cfg(target_os = "linux")]
fn admit_linux_profile(directory: &Dir) -> io::Result<()> {
    let file = directory.try_clone()?.into_std_file();
    let root = linux_directory_properties(&file)?;
    admit_linux_properties(root.filesystem_type, root.mount_flags, root.inode_flags)?;
    for name in PROTOCOL_DIRECTORIES {
        let child = match super::sync_capable_directory::open(directory, name) {
            Ok(child) => child,
            Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(source),
        };
        let child = linux_directory_properties(&child.into_std_file())?;
        admit_linux_child_properties(root, child)?;
    }
    file.sync_all()
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
mod tests {
    use super::{
        LinuxDirectoryProperties, PROTOCOL_DIRECTORIES, admit_linux_child_properties,
        admit_linux_properties,
    };

    use rustix::fs::{NFS_SUPER_MAGIC, StatVfsMountFlags};

    const EXT4_SUPER_MAGIC: rustix::fs::FsWord = 0x0000_ef53;
    const EXT4_CASEFOLD_FLAG: u32 = 0x4000_0000;

    #[test]
    fn only_writable_case_sensitive_ext4_is_admitted() {
        assert!(admit_linux_properties(EXT4_SUPER_MAGIC, StatVfsMountFlags::empty(), 0).is_ok());
        assert_unsupported(admit_linux_properties(
            EXT4_SUPER_MAGIC,
            StatVfsMountFlags::empty(),
            EXT4_CASEFOLD_FLAG,
        ));
        assert_unsupported(admit_linux_properties(
            EXT4_SUPER_MAGIC,
            StatVfsMountFlags::RDONLY,
            0,
        ));
        assert_unsupported(admit_linux_properties(
            NFS_SUPER_MAGIC,
            StatVfsMountFlags::empty(),
            0,
        ));
    }

    #[test]
    fn every_protocol_child_must_share_the_root_filesystem_and_mount() {
        assert_eq!(PROTOCOL_DIRECTORIES, ["staging", "segments", "catalogs"]);
        let root = properties(8, 1, 41);
        let mut casefolded = root;
        casefolded.inode_flags = EXT4_CASEFOLD_FLAG;
        let mut read_only = root;
        read_only.mount_flags = StatVfsMountFlags::RDONLY;
        let mut foreign_format = root;
        foreign_format.filesystem_type = NFS_SUPER_MAGIC;

        assert!(admit_linux_child_properties(root, root).is_ok());
        assert_unsupported(admit_linux_child_properties(root, properties(8, 2, 41)));
        assert_unsupported(admit_linux_child_properties(root, properties(8, 1, 42)));
        assert_unsupported(admit_linux_child_properties(root, casefolded));
        assert_unsupported(admit_linux_child_properties(root, read_only));
        assert_unsupported(admit_linux_child_properties(root, foreign_format));
    }

    fn assert_unsupported(result: std::io::Result<()>) {
        assert!(matches!(
            result,
            Err(ref error)
                if error.kind() == std::io::ErrorKind::Unsupported
                    && error.to_string()
                        == "store namespace does not satisfy one local writable case-sensitive ext4 profile"
        ));
    }

    const fn properties(
        device_major: u32,
        device_minor: u32,
        mount_id: u64,
    ) -> LinuxDirectoryProperties {
        LinuxDirectoryProperties {
            filesystem_type: EXT4_SUPER_MAGIC,
            mount_flags: StatVfsMountFlags::empty(),
            inode_flags: 0,
            device_major,
            device_minor,
            mount_id,
        }
    }
}
