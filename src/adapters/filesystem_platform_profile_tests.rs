//! Linux filesystem platform-profile laws.

use super::{
    LinuxDirectoryProperties, PROTOCOL_DIRECTORIES, admit_linux_child_properties,
    admit_linux_properties, linux_root_identity,
};
use rustix::fs::{NFS_SUPER_MAGIC, StatVfsMountFlags};

const EXT4_SUPER_MAGIC: rustix::fs::FsWord = 0x0000_ef53;
const EXT4_CASEFOLD_FLAG: u32 = 0x4000_0000;

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
fn every_protocol_child_must_share_the_root_filesystem_and_mount() {
    assert_eq!(PROTOCOL_DIRECTORIES, ["staging", "segments", "catalogs"]);
    let root = properties(8, 1, 41, 1);
    let mut casefolded = root;
    casefolded.inode_flags = EXT4_CASEFOLD_FLAG;
    let mut read_only = root;
    read_only.mount_flags = StatVfsMountFlags::RDONLY;
    let mut foreign_format = root;
    foreign_format.filesystem_type = NFS_SUPER_MAGIC;

    assert!(admit_linux_child_properties(root, root).is_ok());
    assert_unsupported(&admit_linux_child_properties(root, properties(8, 2, 41, 1)));
    assert_unsupported(&admit_linux_child_properties(root, properties(8, 1, 42, 1)));
    assert_unsupported(&admit_linux_child_properties(root, casefolded));
    assert_unsupported(&admit_linux_child_properties(root, read_only));
    assert_unsupported(&admit_linux_child_properties(root, foreign_format));
}

#[test]
fn root_identity_uses_linux_device_mount_and_inode_coordinates() {
    let identity = linux_root_identity(properties(8, 1, 41, 73));
    assert_eq!(identity.device(), rustix::fs::makedev(8, 1));
    assert_eq!(identity.mount(), 41);
    assert_eq!(identity.file(), 73);
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

const fn properties(
    device_major: u32,
    device_minor: u32,
    mount_id: u64,
    inode: u64,
) -> LinuxDirectoryProperties {
    LinuxDirectoryProperties {
        filesystem_type: EXT4_SUPER_MAGIC,
        mount_flags: StatVfsMountFlags::empty(),
        inode_flags: 0,
        device_major,
        device_minor,
        mount_id,
        inode,
    }
}
