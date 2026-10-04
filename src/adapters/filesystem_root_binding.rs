//! This module binds an optional caller locator to an already pinned store root.

use super::{FilesystemPlatformAdmissionError, StoreRootIdentityCoordinate};
use cap_fs_ext::MetadataExt;
use cap_std::fs::Dir;
use std::{io, path::Path};

/// Checks a locator without using it for subsequent proof or mutation.
/// This detects an observed mismatch; it does not isolate against arbitrary
/// concurrent namespace mutation. All later work uses the retained capability.
pub(super) fn require_locator(root: &Dir, path: &Path) -> io::Result<()> {
    let located = Dir::open_ambient_dir(path, cap_std::ambient_authority())?;
    let expected = root.dir_metadata()?;
    let observed = located.dir_metadata()?;
    for (coordinate, expected, observed) in [
        (
            StoreRootIdentityCoordinate::Device,
            expected.dev(),
            observed.dev(),
        ),
        (
            StoreRootIdentityCoordinate::File,
            expected.ino(),
            observed.ino(),
        ),
    ] {
        if expected != observed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                FilesystemPlatformAdmissionError::RootIdentityChanged {
                    coordinate,
                    expected,
                    observed,
                },
            ));
        }
    }
    Ok(())
}
