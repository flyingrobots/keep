//! This module owns fail-closed filesystem platform-profile admission.

use std::io;
use std::path::Path;

use cap_std::fs::Dir;

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
    use rustix::fs::{fstatfs, fstatvfs, ioctl_getflags};

    let file = directory.try_clone()?.into_std_file();
    let filesystem = fstatfs(&file)?;
    let mount = fstatvfs(&file)?;
    let inode_flags = ioctl_getflags(&file)?;
    admit_linux_properties(filesystem.f_type, mount.f_flag, inode_flags.bits())?;
    file.sync_all()
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
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "store root does not satisfy the admitted local case-sensitive ext4 profile",
        ));
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::admit_linux_properties;

    use rustix::fs::{NFS_SUPER_MAGIC, StatVfsMountFlags};

    const EXT4_SUPER_MAGIC: rustix::fs::FsWord = 0x0000_ef53;
    const EXT4_CASEFOLD_FLAG: u32 = 0x4000_0000;

    #[test]
    fn only_writable_case_sensitive_ext4_is_admitted() {
        assert!(admit_linux_properties(EXT4_SUPER_MAGIC, StatVfsMountFlags::empty(), 0).is_ok());
        assert!(matches!(
            admit_linux_properties(
                EXT4_SUPER_MAGIC,
                StatVfsMountFlags::empty(),
                EXT4_CASEFOLD_FLAG,
            ),
            Err(ref error) if error.kind() == std::io::ErrorKind::Unsupported
        ));
        assert!(matches!(
            admit_linux_properties(EXT4_SUPER_MAGIC, StatVfsMountFlags::RDONLY, 0),
            Err(ref error) if error.kind() == std::io::ErrorKind::Unsupported
        ));
        assert!(matches!(
            admit_linux_properties(NFS_SUPER_MAGIC, StatVfsMountFlags::empty(), 0),
            Err(ref error) if error.kind() == std::io::ErrorKind::Unsupported
        ));
    }
}
