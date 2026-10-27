//! This module owns the catalog protocol stages shared by every writer authority.
//!
//! One implementation of the segment, catalog, and head transitions runs under
//! version-one publication authority and under version-two retained-closure
//! authority. Each authority owns its writer lock and decides which protocol
//! entry point may drive these stages; the stages themselves never acquire,
//! release, or widen authority.

use std::io;

use cap_std::fs::{Dir, File};

use super::filesystem_publisher_authority::FilesystemPublisherAuthority;
use super::{
    AdmittedSegment, CatalogRestartPolicy, ClosedSegment, FilesystemSegmentStage, SealedSegment,
    SegmentPublication, SegmentPublicationError, SegmentStageCreateError, sync_capable_directory,
};

/// Pinned catalog-protocol directories and the stages one writer retains.
///
/// The value pins the store root, `staging`, `segments`, and `catalogs`
/// capabilities and holds at most one writable catalog stage and one writable
/// `head.next`. Dropping it closes those handles but never publishes, removes,
/// truncates, or repairs protocol state. It does not hold writer authority;
/// the owning publisher does, and must drop this value first.
pub(super) struct FilesystemCatalogStages {
    pub(super) root: Dir,
    pub(super) staging: Dir,
    pub(super) segments: Dir,
    pub(super) catalogs: Dir,
    pub(super) policy: CatalogRestartPolicy,
    pub(super) authority: FilesystemPublisherAuthority,
    pub(super) catalog_stage: Option<File>,
    pub(super) head_stage: Option<File>,
}

impl FilesystemCatalogStages {
    /// Pins the protocol directories under `pinned_root` without following links.
    ///
    /// # Errors
    ///
    /// Returns the exact no-follow directory-open or inspection failure. A
    /// protocol entry that is not a directory returns
    /// [`io::ErrorKind::NotADirectory`].
    pub(super) fn open(pinned_root: &Dir, policy: CatalogRestartPolicy) -> io::Result<Self> {
        let root = sync_capable_directory::open(pinned_root, ".")?;
        let staging = sync_capable_directory::open(&root, "staging")?;
        let segments = sync_capable_directory::open(&root, "segments")?;
        let catalogs = sync_capable_directory::open(&root, "catalogs")?;
        Ok(Self {
            root,
            staging,
            segments,
            catalogs,
            policy,
            authority: FilesystemPublisherAuthority::new(),
            catalog_stage: None,
            head_stage: None,
        })
    }

    /// Exclusively creates `staging/current.seg` bound to these stages.
    pub(super) fn create_segment_stage(
        &self,
    ) -> Result<FilesystemSegmentStage<'_>, SegmentStageCreateError> {
        FilesystemSegmentStage::create(self)
    }

    /// Closes one sealed stage and binds it to `admitted` under this authority.
    pub(super) fn select_segment<'selection, 'records>(
        &self,
        sealed: SealedSegment<FilesystemSegmentStage<'_>>,
        admitted: &'selection AdmittedSegment<'records>,
    ) -> Result<SegmentPublication<'selection, 'records>, SegmentPublicationError> {
        let (stage, record_count, segment_length, digest) = sealed.into_parts();
        let authority = stage.close();
        if !self.authority.matches(&authority) {
            return Err(SegmentPublicationError::PublisherAuthority);
        }
        SegmentPublication::one_bound(
            ClosedSegment::admitted(record_count, segment_length, digest),
            admitted,
            authority,
        )
    }
}
