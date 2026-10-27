//! This module owns bounded no-follow reads of retained-closure evidence.
//!
//! Catalog publication and catalog-head recovery finalization read
//! the current retention head, its manifest, and every root it selects through
//! this one reader before `HEAD` may change. The reader returns exact bytes;
//! the retained-closure gate decodes and binds every record itself.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::RetentionCurrentStateRefusal as Refusal;
use super::filesystem_retention_current::read_exact_optional;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::retention_error;
use super::filesystem_retention_storage::{require_no_retained_stage, require_pinned_directories};
use super::{filesystem_retention_namespace, head_decoder, root_header_decoder};
use crate::adapters::filesystem_exact_record;
use crate::adapters::retained_catalog_publication::RetainedEvidenceSource;
use crate::{RetentionHead, RetentionManifestEntry};

/// Pinned `retention`, `retention/roots`, and `retention/manifests` capabilities.
///
/// The owner holds writer authority and the store-root capability these
/// directories were admitted under. Every read follows no links, reads no more
/// than the record's fixed or grammar-bounded length plus one trailing-byte
/// probe, and mutates nothing.
pub(in crate::adapters) struct FilesystemRetainedEvidence {
    root: Dir,
    retention: Dir,
    roots: Dir,
    manifests: Dir,
}

impl FilesystemRetainedEvidence {
    /// Binds the admitted retention capabilities to the pinned store root.
    pub(in crate::adapters) const fn new(
        root: Dir,
        retention: Dir,
        roots: Dir,
        manifests: Dir,
    ) -> Self {
        Self {
            root,
            retention,
            roots,
            manifests,
        }
    }

    /// Reads the head after proving its absence is uncontradicted.
    ///
    /// The protocol names must still resolve to the pinned directories, no
    /// fixed retention stage may exist (one means retention recovery is
    /// required), and the retention namespace and both pools must admit. Only
    /// then is an absent `retention/HEAD` reported as `None`, and only while
    /// neither pool holds an artifact.
    pub(in crate::adapters) fn read_head(&self) -> io::Result<Option<Vec<u8>>> {
        require_pinned_directories(&self.root, &self.retention, &self.roots, &self.manifests)?;
        require_no_retained_stage(&self.retention)?;
        let census =
            filesystem_retention_namespace::admit(&self.retention, &self.roots, &self.manifests)?;
        let head = read_exact_optional(
            &self.retention,
            pool_name::HEAD,
            head_decoder::ENCODED_LENGTH,
        )?;
        match head {
            Some(bytes) => Ok(Some(bytes.into_vec())),
            None if census.is_empty() => Ok(None),
            None => Err(Refusal::HeadAbsentWithArtifacts.into_io()),
        }
    }

    pub(in crate::adapters) fn read_manifest(&self, head: &RetentionHead) -> io::Result<Vec<u8>> {
        let length = usize::try_from(head.manifest_length().get())
            .map_err(|_source| Refusal::RecordLengthOverflow.into_io())?;
        let name = pool_name::manifest(head.generation(), head.manifest_digest());
        read_required(&self.manifests, &name, length)
    }

    pub(in crate::adapters) fn read_root(
        &self,
        entry: RetentionManifestEntry,
    ) -> io::Result<Vec<u8>> {
        let namespace = self
            .roots
            .open_dir_nofollow(pool_name::namespace(entry.namespace()))?;
        let name = pool_name::root(entry.root_generation(), entry.root_digest());
        let metadata = namespace.symlink_metadata(&name)?;
        if !metadata.is_file() {
            return Err(Refusal::RecordKindOrLength.into_io());
        }
        let length = usize::try_from(metadata.len())
            .map_err(|_source| Refusal::RecordLengthOverflow.into_io())?;
        if length > root_header_decoder::MAXIMUM_ENCODED_LENGTH {
            return Err(Refusal::RecordKindOrLength.into_io());
        }
        read_required(&namespace, &name, length)
    }
}

impl RetainedEvidenceSource for &FilesystemRetainedEvidence {
    fn read_retention_head(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.read_head()
    }

    fn read_retention_manifest(&mut self, head: &RetentionHead) -> io::Result<Vec<u8>> {
        self.read_manifest(head)
    }

    fn read_retained_root(&mut self, entry: RetentionManifestEntry) -> io::Result<Vec<u8>> {
        self.read_root(entry)
    }
}

/// Reads one required immutable record; absence is the filesystem's `NotFound`.
fn read_required(directory: &Dir, name: &str, length: usize) -> io::Result<Vec<u8>> {
    filesystem_exact_record::read_exact_regular(directory, name, length)
        .map_err(|error| io::Error::from(retention_error(error)))
}
