//! Device and mount separation laws, independent of the host operating system.
//!
//! The oracle is the platform contract's single-device/single-mount rule;
//! child inode identity is deliberately not required to equal the root inode.

use std::{error::Error, fs, io, path::Path};

use cap_std::fs::Dir;

use super::{admit_child_identity, open_profile};
use crate::adapters::filesystem_mount_refusal::FilesystemMountRefusal;
use crate::adapters::filesystem_root_identity::FilesystemRootIdentity;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::platform_filesystem::PlatformFilesystem;

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

/// Admits the root and refuses the first child, as a casefolded, read-only, or
/// foreign child would be refused by a native implementation.
struct RefusesChildren(std::cell::Cell<bool>);

impl PlatformFilesystem for RefusesChildren {
    fn open_root(&self, path: &Path) -> io::Result<Dir> {
        Dir::open_ambient_dir(path, cap_std::ambient_authority())
    }

    fn admit_directory(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        if self.0.replace(true) {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }
        self.root_identity(directory)
    }

    fn root_identity(&self, _directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        Ok(FilesystemRootIdentity::new(8, 41, 73))
    }

    fn repository_identity(&self, directory: &Dir) -> io::Result<FilesystemRootIdentity> {
        self.root_identity(directory)
    }
}

#[test]
fn every_present_protocol_child_must_satisfy_the_platform_profile() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("platform-child-refusal")?;
    fs::create_dir(sandbox.path().join("segments"))?;
    let platform = RefusesChildren(std::cell::Cell::new(false));
    let refusal = open_profile(&platform, sandbox.path(), &["segments"]);
    assert!(matches!(refusal, Err(source) if source.kind() == io::ErrorKind::Unsupported));
    Ok(())
}
