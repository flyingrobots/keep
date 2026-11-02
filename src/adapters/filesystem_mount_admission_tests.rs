//! Device and mount separation laws, independent of the host operating system.
//!
//! The oracle is the platform contract's single-device/single-mount rule;
//! child inode identity is deliberately not required to equal the root inode.

use std::{error::Error, io};

use super::admit_child_identity;
use crate::adapters::filesystem_mount_refusal::FilesystemMountRefusal;
use crate::adapters::filesystem_root_identity::FilesystemRootIdentity;

#[test]
fn a_distinct_child_inode_on_the_same_mount_is_admitted() -> Result<(), Box<dyn Error>> {
    let root = FilesystemRootIdentity::new(8, 41, 73);
    let child = FilesystemRootIdentity::new(8, 41, 74);
    admit_child_identity(root, child)?;
    Ok(())
}

#[test]
fn a_child_on_another_device_is_refused() {
    let root = FilesystemRootIdentity::new(8, 41, 73);
    let child = FilesystemRootIdentity::new(9, 41, 74);
    assert_refusal(
        admit_child_identity(root, child),
        FilesystemMountRefusal::Device {
            expected: 8,
            observed: 9,
        },
    );
}

#[test]
fn a_child_on_another_mount_of_the_same_device_is_refused() {
    let root = FilesystemRootIdentity::new(8, 41, 73);
    let child = FilesystemRootIdentity::new(8, 42, 74);
    assert_refusal(
        admit_child_identity(root, child),
        FilesystemMountRefusal::Mount {
            expected: 41,
            observed: 42,
        },
    );
}

fn assert_refusal(result: io::Result<()>, expected: FilesystemMountRefusal) {
    assert!(matches!(result, Err(source)
        if source.kind() == io::ErrorKind::Unsupported
        && source.get_ref().and_then(|error| error.downcast_ref::<FilesystemMountRefusal>())
            == Some(&expected)));
}
