//! Linux filesystem platform-profile laws.

use super::{admit_linux_properties, linux_directory_coordinate};
use crate::adapters::filesystem_platform_profile::PROTOCOL_DIRECTORIES;
use rustix::fs::{NFS_SUPER_MAGIC, StatVfsMountFlags};

const EXT4_SUPER_MAGIC: rustix::fs::FsWord = 0x0000_ef53;
const EXT4_CASEFOLD_FLAG: u32 = 0x4000_0000;

#[test]
fn the_read_only_mount_flag_constant_is_st_rdonly() {
    // The refusal laws use the same symbol as the predicate, so only this law
    // notices a dependency editing the constant. It does not cover how fstatvfs
    // passes kernel flags through, and no law uses a real read-only mount.
    assert_eq!(StatVfsMountFlags::RDONLY.bits(), 0x0001);
}

#[test]
fn only_writable_case_sensitive_ext4_is_admitted() {
    assert!(admit_linux_properties(EXT4_SUPER_MAGIC, StatVfsMountFlags::empty(), 0).is_ok());
    assert_unsupported(&admit_linux_properties(
        EXT4_SUPER_MAGIC,
        StatVfsMountFlags::empty(),
        EXT4_CASEFOLD_FLAG,
    ));
    assert_unsupported(&admit_linux_properties(
        EXT4_SUPER_MAGIC,
        StatVfsMountFlags::RDONLY,
        0,
    ));
    assert_unsupported(&admit_linux_properties(
        NFS_SUPER_MAGIC,
        StatVfsMountFlags::empty(),
        0,
    ));
}

#[test]
fn directory_coordinate_uses_linux_device_and_mount() {
    let identity = linux_directory_coordinate(8, 1, 41);
    assert_eq!(identity.device(), rustix::fs::makedev(8, 1));
    assert_eq!(identity.mount(), 41);
}

fn assert_unsupported(result: &std::io::Result<()>) {
    assert!(matches!(
        result,
        Err(error)
            if error.kind() == std::io::ErrorKind::Unsupported
                && error.to_string()
                    == "store namespace does not satisfy one local writable case-sensitive ext4 profile"
    ));
}

#[test]
fn admission_covers_every_protocol_directory() {
    assert_eq!(
        PROTOCOL_DIRECTORIES,
        [
            "staging",
            "segments",
            "catalogs",
            "retention",
            "retention/roots",
            "retention/manifests",
            "gc",
            "recovery",
            "recovery/dispositions",
        ]
    );
}
