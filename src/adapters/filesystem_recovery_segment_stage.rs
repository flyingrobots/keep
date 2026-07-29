//! This module owns a resumed writer-authorized filesystem segment stage.

use std::io::{self, Write};

use cap_std::fs::File;

use super::{FilesystemRecoveryStageDiscarder, SegmentStage};

/// Writable `current.seg` reopened from one exact reusable prefix.
///
/// The stage owns the pinned root, protocol namespaces, and exclusive writer
/// lock for its full lifetime. Dropping it preserves `current.seg` for a later
/// explicit recovery decision. It does not synchronize the staging directory,
/// publish immutable bytes, or select a catalog generation.
pub struct FilesystemRecoverySegmentStage {
    file: File,
    _authority: FilesystemRecoveryStageDiscarder,
}

impl FilesystemRecoverySegmentStage {
    pub(super) const fn new(file: File, authority: FilesystemRecoveryStageDiscarder) -> Self {
        Self {
            file,
            _authority: authority,
        }
    }
}

impl Write for FilesystemRecoverySegmentStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl SegmentStage for FilesystemRecoverySegmentStage {
    fn synchronize(&mut self) -> io::Result<()> {
        self.file.sync_all()
    }
}
