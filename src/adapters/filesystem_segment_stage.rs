//! Writer-authorized fixed-name filesystem segment stage.

use std::io::{self, Write};

use cap_std::fs::File;

use super::filesystem_catalog_artifact;
use super::filesystem_catalog_publisher::CURRENT_SEGMENT;
use super::{FilesystemCatalogPublisher, SegmentStage, SegmentStageCreateError};

/// An exclusively created empty `current.seg` staging file under writer authority.
///
/// Creation uses one atomic no-replacement filesystem operation. A successful
/// value therefore owns a new regular file positioned at byte zero. Dropping
/// the value closes the file but deliberately leaves its bytes and name for
/// explicit recovery.
///
/// The lifetime keeps the locked publisher borrowed until the writable file is
/// closed. This type does not synchronize the staging-directory entry, publish
/// the file, or establish that the surrounding filesystem satisfies Keep's
/// complete platform contract.
pub struct FilesystemSegmentStage<'publisher> {
    file: File,
    _publisher: &'publisher FilesystemCatalogPublisher,
}

impl<'publisher> FilesystemSegmentStage<'publisher> {
    pub(super) fn create(
        publisher: &'publisher FilesystemCatalogPublisher,
    ) -> Result<Self, SegmentStageCreateError> {
        let file =
            filesystem_catalog_artifact::create_exclusive(&publisher.staging, CURRENT_SEGMENT)
                .map_err(|source| SegmentStageCreateError::Create { source })?;
        Ok(Self {
            file,
            _publisher: publisher,
        })
    }
}

impl Write for FilesystemSegmentStage<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl SegmentStage for FilesystemSegmentStage<'_> {
    fn synchronize(&mut self) -> io::Result<()> {
        self.file.sync_all()
    }
}
