//! Exclusive fixed-name filesystem segment stage.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::{SegmentStage, SegmentStageCreateError};

const STAGE_NAME: &str = "current.seg";

/// An exclusively created empty `current.seg` staging file.
///
/// Creation uses one atomic no-replacement filesystem operation. A successful
/// value therefore owns a new regular file positioned at byte zero. Dropping
/// the value closes the file but deliberately leaves its bytes and name for
/// explicit recovery.
///
/// This type does not create the staging directory, synchronize its directory
/// entry, publish the file, or establish that the surrounding filesystem
/// satisfies Keep's complete platform contract.
pub struct FilesystemSegmentStage {
    file: File,
}

impl FilesystemSegmentStage {
    /// Exclusively creates `current.seg` beneath `staging_directory`.
    ///
    /// This operation performs blocking filesystem I/O and allocates the
    /// platform path buffer required to append the fixed stage name.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentStageCreateError`] without opening or truncating an
    /// existing filesystem entry.
    pub fn create(staging_directory: &Path) -> Result<Self, SegmentStageCreateError> {
        let path = staging_directory.join(STAGE_NAME);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|source| SegmentStageCreateError::Create { source })?;
        Ok(Self { file })
    }
}

impl Write for FilesystemSegmentStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl SegmentStage for FilesystemSegmentStage {
    fn synchronize(&mut self) -> io::Result<()> {
        self.file.sync_all()
    }
}
