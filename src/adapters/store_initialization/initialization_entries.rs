//! This module owns the idempotent creation of every entry initialization makes.
//!
//! Each operation either creates its entry or admits the exact entry a
//! previous attempt created, then synchronizes the parent directory. No
//! operation replaces, truncates, or follows a link.

use std::io::{self, Write};

use cap_fs_ext::DirExt;
use cap_std::fs::{Dir, File};

use super::StoreInitializationDirectory as Directory;
use super::initialization_namespace::{MARKER, MARKER_STAGE};
use crate::adapters::filesystem_exact_record::{
    self as exact_record, EntryIdentity, ExactRecordError,
};
use crate::adapters::filesystem_namespace_refusal::{
    self as namespace, FilesystemNamespaceRefusal,
};
use crate::adapters::{
    CanonicalStoreFormatMarker, filesystem_catalog_artifact, sync_capable_directory,
};

/// The open `FORMAT.next` stage, bound to the inode it was created or admitted as.
pub(super) struct MarkerStage {
    marker: CanonicalStoreFormatMarker,
    identity: EntryIdentity,
    file: File,
}

impl MarkerStage {
    /// Exclusively creates `FORMAT.next` and writes the canonical marker without synchronizing.
    pub(super) fn create(root: &Dir) -> io::Result<Self> {
        let marker = CanonicalStoreFormatMarker::version_two();
        let mut file = filesystem_catalog_artifact::create_exclusive(root, MARKER_STAGE)?;
        let identity = EntryIdentity::of_file(&file)?;
        file.write_all(marker.encoded())?;
        file.flush()?;
        Ok(Self {
            marker,
            identity,
            file,
        })
    }

    /// Reopens an existing `FORMAT.next` that holds exactly the canonical marker.
    pub(super) fn reopen(root: &Dir) -> io::Result<Self> {
        let marker = CanonicalStoreFormatMarker::version_two();
        let file = exact_record::open_regular(root, MARKER_STAGE)?;
        let identity = EntryIdentity::of_file(&file)?;
        let stage = Self {
            marker,
            identity,
            file,
        };
        stage.verify(root, MARKER_STAGE)?;
        Ok(stage)
    }

    /// Synchronizes the stage's bytes and reverifies its name, bytes, and inode.
    pub(super) fn synchronize(&self, root: &Dir) -> io::Result<()> {
        self.file.sync_all()?;
        self.verify(root, MARKER_STAGE)
    }

    /// Links the stage to `FORMAT` without replacement and verifies both names.
    ///
    /// An existing `FORMAT` is never replaced: it is admitted only if it is
    /// already this stage's inode with the canonical bytes.
    pub(super) fn link(&self, root: &Dir) -> io::Result<()> {
        self.verify(root, MARKER_STAGE)?;
        exact_record::link_without_replacement(root, MARKER_STAGE, root, MARKER)?;
        self.verify_linked(root)
    }

    /// Requires `FORMAT` and `FORMAT.next` to both name this stage's inode.
    pub(super) fn verify_linked(&self, root: &Dir) -> io::Result<()> {
        self.verify(root, MARKER_STAGE)?;
        self.verify(root, MARKER)
    }

    /// Removes `FORMAT.next`, leaving `FORMAT` as the only name of the marker.
    pub(super) fn remove(self, root: &Dir) -> io::Result<()> {
        self.verify_linked(root)?;
        root.remove_file(MARKER_STAGE)?;
        exact_record::require_absent(root, MARKER_STAGE).map_err(ExactRecordError::into_io)?;
        self.verify(root, MARKER)
    }

    fn verify(&self, root: &Dir, name: &str) -> io::Result<()> {
        exact_record::verify_named(root, name, self.marker.encoded(), self.identity)
            .map_err(ExactRecordError::into_io)
    }
}

/// Removes an incomplete `FORMAT.next` and synchronizes the root.
///
/// The caller classified the stage under writer authority as a strict prefix
/// of the canonical marker with nothing else created after it.
pub(super) fn discard_incomplete_stage(root: &Dir) -> io::Result<()> {
    root.remove_file(MARKER_STAGE)?;
    exact_record::require_absent(root, MARKER_STAGE).map_err(ExactRecordError::into_io)?;
    filesystem_catalog_artifact::synchronize_directory(root)
}

/// Creates or admits one protocol directory, then synchronizes its parent.
///
/// An existing directory is admitted only if it holds nothing but its
/// canonical children, so a resumed attempt never adopts foreign content.
pub(super) fn admit_directory(root: &Dir, directory: Directory) -> io::Result<()> {
    let parent = match directory.parent() {
        None => root.try_clone()?,
        Some(parent) => sync_capable_directory::open(root, parent.path())?,
    };
    match parent.create_dir(directory.name()) {
        Ok(()) => {}
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
        Err(source) => return Err(source),
    }
    namespace::require_directory(&parent, directory.name())?;
    let created = parent.open_dir_nofollow(directory.name())?;
    for entry in created.entries()? {
        let name = entry?.file_name();
        if !directory
            .children()
            .iter()
            .any(|child| name == child.name())
        {
            return Err(FilesystemNamespaceRefusal::UnexpectedEntry.into_io());
        }
    }
    filesystem_catalog_artifact::synchronize_directory(&parent)
}

/// Synchronizes the store root.
pub(super) fn synchronize_root(root: &Dir) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(root)
}
