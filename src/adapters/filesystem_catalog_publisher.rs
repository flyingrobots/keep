//! This module owns writer-locked filesystem catalog publication state.

use std::io;

use cap_std::fs::{Dir, File};

use super::{
    CatalogRestartPolicy, FilesystemSegmentStage, FilesystemWriterLock, SegmentStageCreateError,
    sync_capable_directory,
};

pub(super) const CURRENT_SEGMENT: &str = "current.seg";
pub(super) const CURRENT_CATALOG: &str = "current.cat";
pub(super) const HEAD: &str = "HEAD";
pub(super) const NEXT_HEAD: &str = "head.next";

/// Exclusive filesystem authority for one catalog publication at a time.
///
/// The publisher owns the writer lock and pinned root, staging, segment-pool,
/// and catalog-pool directory capabilities until it is dropped. Dropping it
/// closes open stages and directory capabilities before releasing the writer
/// lock, but never publishes, removes, truncates, or repairs protocol state.
#[must_use]
pub struct FilesystemCatalogPublisher {
    pub(super) root: Dir,
    pub(super) staging: Dir,
    pub(super) segments: Dir,
    pub(super) catalogs: Dir,
    pub(super) policy: CatalogRestartPolicy,
    pub(super) catalog_stage: Option<File>,
    pub(super) head_stage: Option<File>,
    // Fields drop in declaration order. Writer authority must outlive every
    // directory capability and retained writable stage.
    pub(super) _lock: FilesystemWriterLock,
}

impl FilesystemCatalogPublisher {
    /// Pins the canonical publication directories under an acquired writer lock.
    ///
    /// # Errors
    ///
    /// Returns the exact root-clone, no-follow directory-open, or
    /// directory-inspection failure. A namespace entry that is not a directory
    /// returns [`io::ErrorKind::NotADirectory`]. A failure drops `lock` and
    /// therefore releases writer authority.
    pub fn open(lock: FilesystemWriterLock, policy: CatalogRestartPolicy) -> io::Result<Self> {
        let pinned_root = lock.clone_directory()?;
        let root = sync_capable_directory::open(&pinned_root, ".")?;
        let staging = sync_capable_directory::open(&root, "staging")?;
        let segments = sync_capable_directory::open(&root, "segments")?;
        let catalogs = sync_capable_directory::open(&root, "catalogs")?;
        Ok(Self {
            root,
            staging,
            segments,
            catalogs,
            policy,
            catalog_stage: None,
            head_stage: None,
            _lock: lock,
        })
    }

    /// Exclusively creates `staging/current.seg` under this writer authority.
    ///
    /// The returned stage borrows this publisher until the stage is dropped or
    /// consumed by [`crate::StagedSegment`] and explicitly closed after
    /// sealing. Creation performs blocking, capability-relative filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentStageCreateError`] without opening or truncating an
    /// existing filesystem entry.
    pub fn create_segment_stage(
        &self,
    ) -> Result<FilesystemSegmentStage<'_>, SegmentStageCreateError> {
        FilesystemSegmentStage::create(self)
    }
}
