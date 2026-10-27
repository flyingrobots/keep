//! This module binds version-two writer authority to the retained-closure port.

use std::io;

use super::{
    FilesystemVersionTwoCatalogPublisher, FilesystemVersionTwoCatalogStages,
    RetainedCatalogPublicationStorage, RetainedClosureAdmission,
};
use crate::{RetentionHead, RetentionManifestEntry};

impl RetainedCatalogPublicationStorage for FilesystemVersionTwoCatalogPublisher {
    type Stages = FilesystemVersionTwoCatalogStages;

    fn read_retention_head(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.evidence.read_head()
    }

    fn read_retention_manifest(&mut self, head: &RetentionHead) -> io::Result<Vec<u8>> {
        self.evidence.read_manifest(head)
    }

    fn read_retained_root(&mut self, entry: RetentionManifestEntry) -> io::Result<Vec<u8>> {
        self.evidence.read_root(entry)
    }

    fn catalog_stages(&mut self, _admission: &RetainedClosureAdmission) -> &mut Self::Stages {
        &mut self.stages
    }
}
